# Provider Adapter Contract Template

Use this template when introducing a new provider for train or restaurant automation.

## 1) Provider identity

- Canonical provider key:
- User-facing provider name:
- Module: `train` or `restaurant`

## 2) Canonical operation coverage

For each operation, document endpoint, request shape, response shape, retry policy, and safe persistence fields.

| Operation ID | Endpoint(s) | Request shape | Response shape | Retry policy | Safe persisted fields |
|---|---|---|---|---|---|
| `auth.start` |  |  |  |  |  |
| `auth.complete` |  |  |  |  |  |
| `auth.refresh` |  |  |  |  |  |
| `profile.get` |  |  |  |  |  |
| `search.availability` |  |  |  |  |  |
| `reservation.create` |  |  |  |  |  |
| `reservation.cancel` |  |  |  |  |  |

## 3) Auth lifecycle notes

- Primary auth mode:
- Challenge/OTP behavior:
- Refresh/heartbeat behavior:
- Logout behavior:

## 4) Error normalization

Map provider errors to canonical error codes.

| Provider code/message | Canonical error code | Retryable | Notes |
|---|---|---|---|
|  |  |  |  |

## 5) Security and redaction

- Sensitive request fields:
- Sensitive response fields:
- Logging redaction rules:
- Persistence exclusions:

## 6) Adapter implementation plan

- Contract test file:
- Adapter implementation file:
- Factory registration file:
- Worker/policy integration points:

## 7) Verification checklist

- [ ] RED tests written first.
- [ ] Adapter methods satisfy canonical protocol.
- [ ] Factory rejects unknown providers.
- [ ] No sensitive fields leak into logs/artifacts.
- [ ] Docs pointers and changelog updated.
