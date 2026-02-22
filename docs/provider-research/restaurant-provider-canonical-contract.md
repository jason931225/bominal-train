# Restaurant Provider Canonical Contract

This document defines the disciplined adapter language shared across restaurant providers, aligned with the train-provider architecture style.

## Provider identity rules

- Use uppercase canonical provider keys:
  - `RESY`
  - `OPENTABLE`
  - `CATCHTABLE` (reference source currently read-only)
- Normalize inbound user/provider labels before factory selection.
- Reject unknown providers at factory boundary.

## Canonical operation IDs

The adapter surface is fixed to these operation IDs:

1. `auth.start`
2. `auth.complete`
3. `auth.refresh`
4. `profile.get`
5. `search.availability`
6. `reservation.create`
7. `reservation.cancel`

These map directly to `api/app/modules/restaurant/providers/constants.py` and must remain stable unless explicitly versioned.

## Adapter contract location

- `api/app/modules/restaurant/providers/base.py` defines:
  - `RestaurantProviderOutcome`
  - `RestaurantSearchSlot`
  - `RestaurantProviderClient` protocol
- `api/app/modules/restaurant/providers/factory.py` defines canonical provider resolution.

## Outcome contract (all operations)

Every adapter method returns `RestaurantProviderOutcome`:

- `ok: bool`
- `retryable: bool`
- `error_code: str | None`
- `error_message_safe: str | None`
- `data: dict[str, Any]`

Rules:

- `error_message_safe` must be user-safe and free of secrets/PII.
- `data` must contain safe subsets only; raw provider secrets are prohibited.
- retry behavior is encoded explicitly through `retryable` + `error_code`.

## Authentication flow contract

### `auth.start`

Inputs:
- `account_identifier` (`email` or phone, provider-specific)
- optional `password` (required for password providers like Resy)
- optional `delivery_channel` (`email`, `sms`)

Output expectations:
- challenge metadata (if OTP flow)
- safe next-step hints only
- for OTP providers: `auth.start` must expose normalized challenge metadata only and fail when challenge reference is absent on success HTTP status

### `auth.complete`

Inputs:
- `account_identifier`
- `challenge_token` (provider-issued challenge handle when applicable)
- `otp_code` (if OTP flow)

Output expectations:
- success/failure and safe profile/session hints
- no raw token or cookie persistence in outcome payload
- body-level provider errors (for example `success=false` with HTTP 200) must be treated as operation failure

### `auth.refresh`

Inputs:
- `account_ref` (stable local reference)

Output expectations:
- keepalive/refresh success signal
- provider-specific heartbeat metadata may be recorded in safe form

## Reservation flow contract

### `search.availability`

Inputs:
- `account_ref`
- `restaurant_id`
- `party_size`
- `date_time_local`
- optional provider metadata (market, timezone hints, etc.)

Output expectations:
- normalized slot list (`RestaurantSearchSlot` or safe equivalent)

### `reservation.create`

Inputs:
- `account_ref`
- selected normalized slot
- optional metadata

Output expectations:
- provider confirmation ID, provider reservation reference, safe trace metadata, and optional non-blocking confirmation enrichment payload

### `reservation.cancel`

Inputs:
- `account_ref`
- `restaurant_id`
- `confirmation_number`
- optional `security_token` (provider-specific)
- optional metadata

Output expectations:
- cancellation status and safe provider confirmation

## Worker-policy alignment

- Auth fallback policy remains in `api/app/modules/restaurant/policy.py`:
  - `refresh -> bootstrap -> fail`
- Payment concurrency lock remains provider/account/restaurant scoped:
  - `build_payment_lease_key(provider, account_ref, restaurant_id)`
- Non-committing phases do not require payment lease:
  - `search`, `availability`, `quote`, `hold`

## Security requirements

- Never persist raw cookies/tokens/OTP/passwords in attempts, artifacts, or docs.
- Use `meta_json_safe` and `data_json_safe` for persisted provider metadata.
- Treat anti-bot endpoints as observation-only until explicit implementation approval.
