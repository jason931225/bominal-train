# Restaurant Provider to DB Schema Mapping

Safe storage mapping for provider automation events. This map is for implementation consistency and migration planning.

## Storage principles

- Persist only minimal data required for retry, observability, and user-facing status.
- Persist safe metadata only (`meta_json_safe`, `data_json_safe`).
- Never persist plaintext credentials, cookies, tokens, OTPs, or CVV.
- Reuse existing `tasks`, `task_attempts`, `artifacts`, and `secrets` structures unless additive schema is justified.

## Canonical identifier names

Use these stable identifier keys in safe metadata:

- `provider_account_ref`
- `provider_session_ref`
- `provider_restaurant_ref`
- `provider_slot_ref`
- `provider_reservation_ref`
- `provider_confirmation_ref`
- `provider_security_token_ref` (masked/truncated only)

## Operation-to-storage mapping

| Canonical operation | Primary table | Required safe fields |
|---|---|---|
| `auth.start` | `task_attempts.meta_json_safe` | `provider`, `operation`, `delivery_channel`, `challenge_present`, `provider_account_ref` |
| `auth.complete` | `task_attempts.meta_json_safe` | `provider`, `operation`, `challenge_verified`, `provider_session_ref` |
| `auth.refresh` | `task_attempts.meta_json_safe` | `provider`, `operation`, `refresh_source`, `refresh_result` |
| `profile.get` | `artifacts.data_json_safe` (optional) | `provider`, `profile_snapshot_safe` |
| `search.availability` | `task_attempts.meta_json_safe` | `provider`, `restaurant_id`, `party_size`, `slot_count`, `provider_slot_refs` |
| `reservation.create` | `artifacts.data_json_safe` | `provider`, `provider_slot_ref`, `slot_lock_ref`, `provider_reservation_ref`, `provider_confirmation_ref`, `reservation_state`, `confirmation_enrichment_safe`, `policy_safe` |
| `reservation.cancel` | `task_attempts.meta_json_safe` and/or `artifacts.data_json_safe` | `provider`, `provider_confirmation_ref`, `cancel_status`, `provider_request_id` |

## Existing bominal table alignment

### `tasks`

- `module` must be `restaurant`.
- `spec_json` stores execution intent (safe inputs only).
- Add normalized keys in `spec_json` for cross-provider parity:
  - `provider`
  - `provider_account_ref`
  - `restaurant_id`
  - `party_size`
  - `date_time_local`

### `task_attempts`

- `action` should map to canonical operation segment:
  - `AUTH`, `SEARCH`, `RESERVE`, `CANCEL`, `PAY`
- `error_code` should use canonical codes where possible (`auth_failed`, `provider_rate_limited`, `reservation_unavailable`, etc.).
- `meta_json_safe` stores endpoint-level diagnostics without raw payload leakage.

### `artifacts`

- Use for durable reservation/cancellation evidence.
- `kind` examples:
  - `restaurant_reservation`
  - `restaurant_cancellation`
- `data_json_safe` stores user-facing confirmation-safe data only.
- For OpenTable two-step create flow, persist only safe reservation identifiers:
  - `provider_slot_ref` (`slotHash`)
  - `slot_lock_ref` (`slotLockId`)
  - `provider_reservation_ref` (`reservationId`)
  - `provider_confirmation_ref` (`confirmationNumber`)
  - optional `confirmation_enrichment_safe` subset from booking-confirmation query
  - `policy_safe` booleans only (for example `confirm_points`, `marketing_opt_in_restaurant`, `restaurant_policy_acknowledged`)
  - never persist raw `slotAvailabilityToken` or `securityToken` in safe artifacts

### `secrets`

- Store encrypted credentials only (for supported provider auth modes).
- Keep provider-specific credentials namespaced by secret kind.
- Session tokens/cookies should not be persisted as durable secrets without explicit policy approval.

## Proposed canonical error codes

- `auth_required`
- `auth_challenge_failed`
- `auth_refresh_failed`
- `provider_rate_limited`
- `provider_unavailable`
- `availability_empty`
- `reservation_conflict`
- `reservation_not_found`
- `reservation_cancel_failed`
- `not_implemented`

## Migration trigger guidance

Additive schema/migration work is justified only when one of these is true:

1. `tasks/spec_json` can no longer hold stable query/filter fields safely.
2. Analytics/reporting requires indexed provider-reservation identifiers.
3. Operational SLOs require indexed attempt metadata beyond existing columns.

Until then, keep schema lean and enforce contract discipline at adapter layer.
