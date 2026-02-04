# Security

Security controls and expectations for Bominal.

## Current controls

## Authentication and sessions

- Password hashing: Argon2id (`api/app/core/security.py`)
- Session auth with httpOnly cookies
- Session token stored as hash in DB (`sessions.token_hash`)
- Cookie flags:
  - `httponly=True`
  - `samesite="lax"`
  - `secure=True` in production only
- Remember-me lifetime: 90 days; default: 7 days

## Authorization

- Role-based user model (`roles`, `users.role_id`)
- API access separation:
  - Public: unauthenticated auth bootstrap routes
  - Authenticated: session-cookie required user routes
  - Internal-only: `X-Internal-Api-Key` header must match `INTERNAL_API_KEY`
  - Admin: session + `admin` role required

## Secrets at rest

Secrets table uses envelope encryption:

- Random 256-bit DEK per record
- AES-256-GCM for payload encryption
- DEK wrapped by KEK (`MASTER_KEY`) via AES-256-GCM
- `kek_version` stored per secret for key-version awareness

Used for:

- SRT credentials
- KTX credentials
- Payment card payload

## Payment data handling

- Payment card persisted encrypted in `secrets`
- CVV is not persisted to Postgres
- CVV is cached temporarily in Redis (encrypted payload) with TTL (`PAYMENT_CVV_TTL_SECONDS`)

## Logging and safe metadata

- Sensitive fields are redacted via `redact_sensitive`
- Task attempts/artifacts store safe metadata only (`meta_json_safe`, `data_json_safe`)

## Rate limiting

- Auth endpoints use in-memory limiter (single-process baseline)
- Train provider calls use Redis-backed token-bucket limits:
  - global host bucket
  - per-provider bucket
  - per-user/per-credential bucket

## Security requirements for contributors

1. Never add plaintext secrets to code, logs, tests, or fixtures.
2. Never print decrypted secrets in debug logs.
3. Any new sensitive field must be added to redaction logic.
4. Keep env secrets outside git (`infra/env/prod/*.env` is gitignored).
5. Preserve cookie security semantics unless intentionally changed and documented.

## Hardening backlog

Recommended next steps for production maturity:

- Replace in-memory auth limiter with Redis/distributed limiter.
- Add CSP and stricter security headers via reverse proxy.
- Add secret manager integration for `MASTER_KEY` and provider API keys.
- Implement key rotation workflow with `kek_version` migration path.
- Add audit logging for account settings and payment method changes.
- Add alerting for repeated auth/provider failures.

## Incident response minimums

- Rotate `MASTER_KEY` and provider credentials after compromise suspicion.
- Revoke active sessions (`sessions.revoked_at`) for impacted users.
- Review worker and API logs for abnormal provider/payment actions.
