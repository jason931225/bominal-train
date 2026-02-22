# Resy Endpoint Contract Notes

Redacted provider-contract notes for Resy integration. This document excludes raw credentials, session cookies, and payment secrets.

## Endpoint summary

| Endpoint | Method | Canonical operation | Current status |
|---|---|---|---|
| `/4/auth/password` | `OPTIONS` | browser preflight | observed |
| `/4/auth/password` | `POST` | `auth.start` + `auth.complete` (password flow) | observed |
| `/2/user` | `GET` | `profile.get` | reference-confirmed (third_party) |
| `/4/find` | `GET` | `search.availability` | reference-confirmed (third_party) |
| `/3/details` | `POST` | `reservation.create` (pre-step token) | reference-confirmed (third_party) |
| `/3/book` | `POST` | `reservation.create` (commit) | reference-confirmed (third_party) |
| `/3/cancel` | `POST` | `reservation.cancel` | reference-confirmed (third_party) |
| `/3/venues` | `GET` | search discovery support | reference-confirmed (third_party) |
| `/3/auth/refresh` | `POST` | `auth.refresh` | implemented (adapter), live-freeze pending |
| `/3/auth/logout` | `POST` | logout (supporting) | implemented (adapter), live-freeze pending |

## Third-party reference cross-check (2026-02-17)

Reviewed source set:

- `third_party/resy/references/api-docs.md`
- `third_party/resy/scripts/utils.py`
- `third_party/resy/scripts/auth.py`
- `third_party/resy/scripts/availability.py`
- `third_party/resy/scripts/book.py`
- `third_party/resy/scripts/cancel.py`
- `third_party/resy/scripts/search.py`

Cross-check conclusions:

1. Provider flow alignment is consistent with prior capture notes:
   - auth (`/4/auth/password`) -> profile/session probe (`/2/user`) -> availability (`/4/find`) -> details token (`/3/details`) -> commit (`/3/book`) -> cancel (`/3/cancel`).
2. Request encoding expectations are stable:
   - `/4/find`: query params
   - `/3/details`, `/3/book`, `/3/cancel`: `application/x-www-form-urlencoded`
3. `reservation.create` is explicitly two-step:
   - `/3/details` returns `book_token.value`
   - `/3/book` consumes `book_token` and optional payment metadata
4. `source_id` is expected on `/3/book` and defaults to `resy.com` in reference scripts.
5. `Idempotency-Key` is optional but recommended for `/3/book` retry safety.

Important confidence note:

- `third_party/resy` is read-only reference material, not canonical policy.
- Contract freeze remains pending until replay against current live captures in this repo workflow.

## Adapter implementation status (2026-02-17)

Implemented in `api/app/modules/restaurant/providers/resy_adapter.py`:

- `auth.start` uses `POST /4/auth/password` password flow contract.
- `auth.start` fails fast when password or API key config is missing.
- `auth.start` enforces body-level failure handling on HTTP 200 (`errors` or `success=false`).
- `auth.start` emits normalized safe output:
  - `requires_otp=false`
  - `password_flow_complete=true`
  - `provider_account_ref` (string when present)
  - `challenge_token` containing only `password_flow_complete` and `provider_account_ref`
- `auth.complete` is challenge-token based for password flow and does not execute a second network call.
- stage-2 endpoint paths are now config-driven:
  - `profile.get`: `GET /2/user`
  - `search.availability`: `GET /4/find`
  - `reservation.create`: `POST /3/details` then `POST /3/book`
  - `reservation.cancel`: `POST /3/cancel` with fallback retry using `resy_token` when first cancel attempt fails
- stage-3 auth/session paths are now config-driven:
  - `auth.refresh`: `POST /3/auth/refresh`
  - `logout` (supporting helper): `POST /3/auth/logout`
- stage-2 safe output normalization now includes:
  - profile safe identity summary (`provider_account_ref`, name/email, reservation/payment-method counts)
  - normalized slot list from `/4/find` (`config.token` -> canonical slot token/id)
  - create step metadata (`payment_required`, `resy_token_present`, `display_date`, `display_time`)
  - cancel fallback flag (`used_resy_token_fallback`)

Required for full feature but not yet contract-frozen:

- profile/search/create/cancel edge-case semantics under live captures:
  - payment-required / special-policy venues
  - cancel fallback requirements across reservation variants
- auth refresh response edge-case semantics (token rotation / throttling envelopes)
- logout endpoint response contract across account states

## Observed authentication contract

### `OPTIONS /4/auth/password`

Purpose: browser CORS preflight for password login path.

Observed request headers include:

- `Access-Control-Request-Method: POST`
- `Access-Control-Request-Headers: authorization,cache-control,x-origin`
- `Origin: https://resy.com`

Adapter guidance:

- backend adapter calls should use direct server-side HTTP flow and do not require browser preflight semantics.

### `POST /4/auth/password`

Canonical operation:

- `auth.start` for adapter lifecycle
- `auth.complete` in same call for password-mode provider

Observed request characteristics:

- content type: `application/x-www-form-urlencoded`
- required headers:
  - `Authorization: ResyAPI api_key="<provider-key>"`
  - `X-Origin: https://resy.com`
- body fields:
  - `email`
  - `password`

Redacted request example:

```txt
POST /4/auth/password
Content-Type: application/x-www-form-urlencoded
Authorization: ResyAPI api_key="<provider-key>"
X-Origin: https://resy.com

email=<email>&password=<password>
```

Observed response expectations:

- success returns authenticated-session context (exact token schema not persisted in docs)
- failure returns auth error envelope

Implementation guidance:

- never log or persist `email`/`password` raw values
- store only safe auth result metadata (success/failure, retryability, provider code)
- normalize account reference values to string for canonical provider-account identity keys

## Reference-derived operation contracts (not yet frozen)

### `GET /2/user` (`profile.get`)

Reference usage:

- used by `third_party/resy/scripts/auth.py` and `third_party/resy/scripts/list_reservations.py`
- acts as authenticated-session probe and user profile/reservations source

Expected safe extraction targets:

- `id`, `email`, `first_name`, `last_name`
- reservations list entries for `reservation_id`, `venue`, `scheduled_date`, `scheduled_time`

### `GET /4/find` (`search.availability`)

Reference request shape:

- query params:
  - `venue_id`
  - `day` (YYYY-MM-DD)
  - `party_size`
  - optional `lat`, `long`

Reference response anchors:

- `results.venues[].slots[].config.token` (config token for details step)
- `results.venues[].slots[].date.start` (slot datetime)

### `POST /3/details` (`reservation.create` pre-step)

Reference request shape (form encoded):

- `commit=1`
- `config_id=<slot token>`
- `day`
- `party_size`
- optional `notes`

Reference response anchors:

- `book_token.value`
- optional payment/hold details (for safe metadata only)

### `POST /3/book` (`reservation.create` commit)

Reference request shape (form encoded):

- `book_token=<value from /3/details>`
- `source_id=<provider source id>` (adapter default: `resy.com-venue-details`)
- optional `struct_payment_method=<json-string>`
- optional `Idempotency-Key` header for retry safety

Reference response anchors:

- reservation identifiers (`reservation_id` and/or `resy_token`)
- display metadata (`display_date`, `display_time`)

### `POST /3/cancel` (`reservation.cancel`)

Reference request shape (form encoded):

- required: `reservation_id`
- compatibility fallback from prior notes: include `resy_token` when provider requires token-paired cancellation

Reference response anchors:

- `status` (for example `cancelled`)

## Existing workflow alignment

`docs/playbooks/resy-widget-form-data-capture.md` already defines deterministic capture flow for:

1. auth bootstrap
2. availability
3. lock/hold
4. guarded book attempt before payment completion

This endpoint document extends that workflow by freezing the canonical adapter mapping and redaction requirements.

## Required follow-up captures for full Resy adapter

1. Freeze live `/3/auth/refresh` request/response contract for long-running auth maintenance.
2. Freeze live `/3/auth/logout` request/response contract for explicit session teardown.
3. Capture payment-required/policy-heavy create variants and confirm safe field mapping under non-standard venue requirements.
4. Capture cancel fallback coverage across reservation variants to validate when `resy_token` is mandatory.

## Safe logging checklist for Resy adapters

- redact `Authorization` header values
- redact credential form values
- do not persist provider session cookies/tokens
- persist only safe operation metadata and normalized reservation references
