# Resy Endpoint Contract Notes

Redacted provider-contract notes for Resy integration. This document excludes raw credentials, session cookies, and payment secrets.

## Endpoint summary

| Endpoint | Method | Canonical operation | Current status |
|---|---|---|---|
| `/4/auth/password` | `OPTIONS` | browser preflight | observed |
| `/4/auth/password` | `POST` | `auth.start` + `auth.complete` (password flow) | observed |

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

Required for full feature but not yet contract-frozen:

- session refresh endpoint (`auth.refresh`)
- profile endpoint (`profile.get`)
- availability endpoint(s) (`search.availability`)
- lock/hold endpoint(s) (`reservation.create` pre-step)
- reservation-create endpoint (`reservation.create`)
- cancellation endpoint (`reservation.cancel`)
- logout endpoint

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

## Existing workflow alignment

`docs/playbooks/resy-widget-form-data-capture.md` already defines deterministic capture flow for:

1. auth bootstrap
2. availability
3. lock/hold
4. guarded book attempt before payment completion

This endpoint document extends that workflow by freezing the canonical adapter mapping and redaction requirements.

## Required follow-up captures for full Resy adapter

1. Profile endpoint used to confirm authenticated identity.
2. Availability endpoint request and slot response schema.
3. Lock/hold and reservation-create endpoints with idempotency expectations.
4. Cancellation endpoint with confirmation identifiers.
5. Session refresh/logout endpoints for long-running worker flows.

## Safe logging checklist for Resy adapters

- redact `Authorization` header values
- redact credential form values
- do not persist provider session cookies/tokens
- persist only safe operation metadata and normalized reservation references
