# Security

Security controls and requirements for bominal.

## Current controls

## Authentication and sessions

- Password hashing: Argon2id (`api/app/core/security.py`)
- Auth mode is controlled by `AUTH_MODE`:
  - `legacy`: session-cookie auth
  - `supabase`: Bearer JWT auth verified against Supabase JWKS
  - `dual`: Bearer JWT first, cookie fallback
- Session token stored as hash in DB (`sessions.token_hash`) for cookie mode
- Cookie flags:
  - `httponly=True`
  - `samesite="lax"`
  - `secure=True` in production only
- Remember-me lifetime: 90 days; default: 7 days

## Authorization

- Role-based user model (`roles`, `users.role_id`)
- API access separation:
  - Public: unauthenticated auth bootstrap routes
  - Authenticated: Supabase Bearer or session-cookie depending on `AUTH_MODE`
  - Internal-only: `X-Internal-Api-Key` header must match `INTERNAL_API_KEY`
  - Admin: local DB role (`admin`) required
- Supabase JWT role claims are not trusted as authorization source; API resolves local user by `sub` and enforces DB role.

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
- Redis routing is split by purpose:
  - `REDIS_URL_NON_CDE`: queueing, rate limiting, non-sensitive cache
  - `REDIS_URL_CDE`: CDE-only CVV cache
  - `REDIS_URL_CDE` falls back to `REDIS_URL` when unset

## Logging and safe metadata

- Sensitive fields are redacted via `redact_sensitive`
- Task attempts/artifacts store safe metadata only (`meta_json_safe`, `data_json_safe`)

## PCI relay worker isolation policy

This policy applies to any component that receives raw PAN/CVV, decrypts payment payloads, or submits card data to external providers.

### CDE boundary

The Cardholder Data Environment (CDE) includes:

- API code paths that decrypt wallet secrets
- Workers that construct provider payment requests
- Redis keys storing encrypted CVV
- Envelope decryption logic and runtime memory that holds decrypted PAN/CVV

The CDE explicitly excludes:

- Web layer
- Queue payloads
- `meta_json_safe` / `data_json_safe` records
- Artifacts that are not explicitly safe metadata
- Logs and observability payload bodies

No raw cardholder data may enter non-CDE systems.

### Ephemeral relay worker requirements (mandatory)

Any worker performing payment submission MUST satisfy all requirements below:

- Stateless runtime:
  - MUST NOT persist card data to Postgres.
  - MUST NOT serialize card data into ARQ jobs.
  - MUST NOT write card data to disk, artifacts, or task attempts.
- Memory lifetime:
  - MUST decrypt card payload only immediately before provider submission.
  - MUST drop decrypted references after submission (best-effort zeroization in language/runtime limits).
  - MUST re-fetch encrypted secret + CVV on retry; MUST NOT reuse persisted plaintext payload.
- Logging prohibition:
  - MUST NOT log request or response bodies containing card data.
  - MUST disable HTTP client debug payload logging.
  - MUST apply redaction middleware to PAN/CVV/token/header/cookie patterns.

Violation of this section is CRITICAL.

### Redaction enforcement architecture

- `redact_sensitive()` is mandatory at every logging and persistence boundary where untrusted/provider payloads can flow.
- Global exception handlers MUST redact emitted context.
- Structured logging formatters MUST redact message, exception text, and `extra` fields before emission.
- Provider traces written to `meta_json_safe` / `data_json_safe` MUST pass through redaction first.

### Redis CVV policy

- CVV may exist only in Redis and only as encrypted payload.
- `PAYMENT_CVV_TTL_SECONDS` MUST be bounded by:
  - `PAYMENT_CVV_TTL_MIN_SECONDS`
  - `PAYMENT_CVV_TTL_MAX_SECONDS`
- TTL MUST be set on write and MUST NOT be extended indefinitely.
- CDE Redis endpoint (`REDIS_URL_CDE` or fallback `REDIS_URL`) MUST NOT be Upstash-hosted.
- Production Redis for CDE workloads MUST NOT persist CVV-bearing keys to disk (`AOF`/`RDB` disabled).
- CVV-bearing keys MUST be excluded from backups.
- CVV MUST NEVER appear in Postgres, queue payloads, artifacts, or logs.

### Queue safety contract

ARQ payloads MUST NOT contain:

- PAN/CVV/expiry
- decrypted secrets
- raw provider request or response payloads
- session tokens or authorization headers

Only safe identifiers may be queued (`task_id`, provider IDs, reference IDs, idempotency keys, safe metadata references).

### Provider payload safety contract

Persistable provider fields are limited to safe metadata:

- `meta_json_safe`
- `data_json_safe`
- masked identifiers (for example: last 4 digits)
- provider reservation IDs
- provider status/error codes

Never persist:

- full card number
- CVV
- full raw provider request JSON
- raw provider response JSON containing sensitive fields

### Network egress and SSRF controls

Workers handling payment MUST:

- connect only to configured provider domain allowlist (`PAYMENT_PROVIDER_ALLOWED_HOSTS`)
- enforce TLS certificate verification
- disable proxy inheritance by default (`PAYMENT_TRANSPORT_TRUST_ENV=false` unless explicitly approved)
- enforce bounded connect/read/total timeouts
- reject user-input hostnames and dynamic outbound host routing

### Crash/dump safety

Production runtime MUST:

- disable core dumps for payment execution paths
- avoid stack traces that include decrypted payloads
- avoid exception messages that include request/response bodies with sensitive fields

### Severity classification and deploy gate

The following are automatically CRITICAL and block deploy:

- plaintext PAN persistence anywhere
- CVV persistence outside Redis TTL cache
- logging of provider payment payloads
- queue serialization of card data
- missing CVV TTL on Redis writes
- envelope encryption bypass
- TLS verification disabled on provider payment egress

## Rate limiting

- Auth endpoints support both in-memory and Redis-backed limiting:
  - Set `RATE_LIMIT_USE_REDIS=1` in production for distributed rate limiting
  - Uses sliding window algorithm with Redis sorted sets
- Train provider calls use Redis-backed token-bucket limits:
  - global host bucket
  - per-provider bucket
  - per-user/per-credential bucket

## Security requirements for contributors

1. Never add plaintext secrets to code, logs, tests, fixtures, docs, or changelog entries.
2. Never print decrypted secrets in debug logs.
3. Any new sensitive field must be added to redaction logic and corresponding tests.
4. Keep env secrets outside git (`infra/env/prod/*.env` is gitignored).
5. Preserve cookie security semantics unless intentionally changed and documented.
6. For payment/CDE/relay changes, run PCI/ASVS security tests before completion.

## Hardening backlog

Recommended next steps for production maturity:

- Add CSP and stricter security headers via reverse proxy.
- Add secret manager integration for `MASTER_KEY` and provider API keys.
- Complete key rotation workflow using `kek_version` multi-key migration path.
- Add audit logging for account settings and payment method changes.
- Add alerting for repeated auth/provider failures.

## Incident response minimums

- Rotate `MASTER_KEY` and provider credentials after compromise suspicion.
- Revoke active sessions (`sessions.revoked_at`) for impacted users.
- Review worker and API logs for abnormal provider/payment actions.
