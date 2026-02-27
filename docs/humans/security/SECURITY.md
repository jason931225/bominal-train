# Security

Security controls and requirements for bominal.

## Current controls

## Authentication and sessions

- Password hashing: Argon2id (`api/app/core/security.py`)
- Auth mode is controlled by `AUTH_MODE`:
  - production requires `supabase` (Bearer JWT auth verified against Supabase JWKS)
  - `legacy` remains a development/backward-compatibility mode only
- Optional passkey auth is enabled via WebAuthn for session-auth flows:
  - authenticated passkey enrollment in account/signup flow
  - passkey login bootstrap issuing the same session cookie contract
- Session token stored as hash in DB (`sessions.token_hash`) for cookie mode
- Cookie flags:
  - `httponly=True`
  - `samesite="lax"`
  - `secure=True` in production only
- Remember-me lifetime: 90 days; default: 7 days
- Email-change security contract:
  - `PATCH /api/auth/account` never applies new email directly.
  - New email must be verified through a short-lived code/link before change takes effect.
  - Only after `/api/auth/account/email-change/confirm` succeeds is `users.email` updated.

## Authorization

- Role-based user model (`roles`, `users.role_id`)
- API access separation:
  - Public: unauthenticated auth bootstrap routes
  - Authenticated (production): Supabase Bearer
  - Authenticated (non-production): Supabase Bearer or session-cookie depending on `AUTH_MODE` (`legacy`/`supabase`)
  - Internal-only:
    - `X-Internal-Api-Key` must match `INTERNAL_API_KEY`, or
    - `X-Internal-Service-Token` must be a valid short-lived internal token signed by `INTERNAL_IDENTITY_SECRET`
  - Admin: local DB role (`admin`) required
- Supabase JWT role claims are not trusted as authorization source; API resolves local user by `sub` and enforces DB role.

## Secrets at rest

Secrets table uses envelope encryption:

- Random 256-bit DEK per record
- AES-256-GCM for payload encryption
- DEK wrapped by KEK (`MASTER_KEY`) via AES-256-GCM
- `kek_version` stored per secret for key-version awareness
- Active KEK source resolution order:
  - deploy-time `MASTER_KEY_OVERRIDE` (runtime-only env injected by deploy)
  - GCP Secret Manager (`GSM_MASTER_KEY_*` settings)
  - `MASTER_KEY` env fallback (disabled in production when GSM is enabled)

Used for:

- SRT credentials
- KTX credentials
- Payment card payload

Envelope decrypt behavior:

- Persisted secret decrypt paths must validate stored `kek_version` against active crypto settings.
- Envelope payload serialization uses JSON with stable separators and ASCII escaping; compatibility-sensitive changes require migration review.
- Key rotation execution path:
  - `MASTER_KEYS_BY_VERSION` supplies keyring entries for legacy versions.
  - `KEK_VERSION` defines the active write key.
  - `python -m app.admin_cli secret prepare-kek-rotation --new-version <N>` generates a new 32-byte KEK and env payload where `<N>` becomes the primary wrapping version while older versions remain available for unwrapping.
  - `python -m app.admin_cli secret rotate-kek --dry-run` validates rewrap viability.
  - `python -m app.admin_cli secret rotate-kek --yes` rewraps non-current secrets to active `kek_version`.
  - `python -m app.admin_cli secret rotate-kek-background --yes` executes batch rewrap in the background until no non-current `kek_version` rows remain.
  - `python -m app.admin_cli secret retire-kek --version <OLD> --rotation-completed-at <UTC_ISO> --yes` is the retirement readiness gate; it refuses retirement if any secrets still reference `<OLD>` or if `KEK_RETIREMENT_WINDOW_DAYS` has not elapsed since rewrap completion.
  - Rapid successive rotations are safe when keyring continuity is maintained:
    - new writes always use current `KEK_VERSION`;
    - decrypt keeps working as long as old versions remain in `MASTER_KEYS_BY_VERSION`;
    - rewrap converges to the currently active `KEK_VERSION` after rerun/background completion.
  - Production GSM policy:
    - `GSM_MASTER_KEY_VERSION` must be pinned (no `latest`)
    - `GSM_MASTER_KEY_ALLOW_ENV_FALLBACK=false`
    - missing GSM fetch is fail-closed for runtime startup

## Payment data handling

- Payment card persisted encrypted in `secrets`
- CVV is not accepted by wallet APIs
- CVV is never cached or persisted by bominal
- Redis routing is split by purpose:
  - `REDIS_URL_NON_CDE`: queueing, rate limiting, non-sensitive cache
  - `REDIS_URL_CDE`: CDE runtime-sensitive flows
  - `REDIS_URL_CDE` falls back to `REDIS_URL` when unset

## PCI Relay Worker Isolation Policy

### Scope

This policy applies to any component that:

- Receives raw cardholder data (PAN, expiry)
- Transmits cardholder data to third-party providers
- Decrypts stored card payload for provider submission

This policy defines bominal's Cardholder Data Environment (CDE) boundary.

### 1. Cardholder Data Environment (CDE) Definition

The CDE includes:

- API code paths that decrypt wallet secrets
- Any worker that constructs provider payment requests
- Envelope decryption logic for card payload
- Any runtime memory holding decrypted PAN

The CDE explicitly does not include:

- Web layer
- Task metadata storage (`*_safe` fields)
- Queue payloads
- Artifacts
- Logs

No raw cardholder data may enter non-CDE systems.

### 2. Ephemeral Relay Worker Requirements (Mandatory)

Any worker performing payment submission must satisfy all of the following.

Statelessness:

- No card data persisted to Postgres
- No card data serialized into ARQ jobs
- No card data written to disk
- No card data written to artifacts or `task_attempt` rows
- No card data cached beyond the required execution window

Memory lifetime:

- Decrypt card payload only immediately before provider submission
- Clear references after submission (explicit variable overwrite where language allows)
- No reuse of decrypted card payload across retries

Logging prohibition:

- Never log request bodies containing card data
- Never log provider request/response raw payloads
- Disable HTTP client debug logging
- Global redaction middleware must sanitize:
  - PAN patterns (13-19 digit sequences)
  - CVV
  - expiry fields
  - authorization headers
  - cookies
  - session tokens

Violation of this section is CRITICAL.

### 3. Legacy CVV Cache Cleanup

- CVV caching is deprecated and disabled for wallet updates.
- Legacy keys may still exist from older releases and must be purged.
- Run `python -m app.admin_cli secret purge-payment-cvv --yes` to delete current/legacy CVV keys.

CVV must never:

- Appear in Postgres
- Appear in queue payloads
- Appear in artifacts
- Appear in logs

### 4. Queue Safety Contract

ARQ job payloads must never contain:

- PAN
- CVV
- Expiry
- Raw provider request JSON
- Raw provider response JSON
- Session tokens
- Decrypted secrets

Only the following may be queued:

- `task_id`
- provider identifiers
- safe metadata
- reference IDs
- idempotency keys

Workers must retrieve sensitive data at runtime via secure lookup.

### 5. Provider Payload Safety

Allowed to persist:

- `meta_json_safe`
- `data_json_safe`
- masked identifiers (for example, last 4 digits only)
- provider reservation IDs
- provider status codes

Never allowed to persist:

- full card numbers
- CVV
- full raw provider request payload
- raw provider response containing sensitive fields

If provider response includes sensitive fields:

- sanitize before persistence
- store only necessary safe metadata

### 6. Network Egress Controls

Workers handling payment must:

- Connect only to approved provider domains
- Use TLS 1.2+ with certificate verification enabled
- Disable proxy inheritance unless explicitly required
- Enforce connect/read/total timeouts

SSRF risk mitigation:

- No dynamic hostnames from user input
- Provider base URLs must come from a configuration allowlist

### 7. Observability Constraints

For payment-related flows, logs may include:

- `task_id`
- provider name
- attempt number
- status category (2xx/4xx/5xx)
- execution time

Logs must never include:

- request bodies
- response bodies
- decrypted secrets
- headers containing credentials
- cookies

Exception handlers must pass through redaction before emission.

### 8. Retry and Idempotency Controls

- Payment retries must not reuse persisted card payload
- Payment idempotency must be based on:
  - provider reservation ID
  - stored artifact references

If retry occurs:

- re-fetch encrypted secret
- decrypt only in memory

### 9. Crash and Dump Safety

Production runtime must:

- Disable core dumps
- Avoid writing stack traces containing request payloads
- Ensure exception messages do not include decrypted payload

### 10. Security Severity Classification

The following are automatically CRITICAL:

- Plaintext PAN persistence anywhere
- Any CVV collection/caching/persistence in runtime paths
- Logging of provider payment payload
- Queue serialization of card data
- Reintroduction of CVV cache write paths
- Envelope encryption bypass
- TLS verification disabled

These conditions must block deployment.

### 11. Verification Requirements (Mandatory Before Deploy)

Before any production release affecting payment flows:

Unit tests must verify:

- Wallet rejects legacy `cvv` payloads
- Queue payload schema excludes card fields
- Redaction function masks PAN patterns

Log scans must confirm:

- No payment payload is logged during integration tests

Manual review must verify:

- Redis config (persistence mode)
- HTTP client config (TLS verification enabled)

### 12. Relationship to Guardrails

This policy extends:

- `docs/agents/GUARDRAILS.md` (hard constraints)
- `docs/agents/PERMISSIONS.md` (approval boundaries)

Guardrails override implementation shortcuts. If this policy conflicts with feature velocity, security prevails.

## Logging and safe metadata

- Sensitive fields are redacted via `redact_sensitive`
- Task attempts/artifacts store safe metadata only (`meta_json_safe`, `data_json_safe`)

## PCI relay worker isolation policy

This policy applies to any component that receives raw PAN/expiry, decrypts payment payloads, or submits card data to external providers.

### CDE boundary

The Cardholder Data Environment (CDE) includes:

- API code paths that decrypt wallet secrets
- Workers that construct provider payment requests
- Envelope decryption logic and runtime memory that holds decrypted PAN

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
  - MUST re-fetch encrypted secret on retry; MUST NOT reuse persisted plaintext payload.
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

### Legacy CVV key cleanup policy

- CVV inputs are rejected by wallet APIs and are not cached.
- CDE Redis endpoint (`REDIS_URL_CDE` or fallback `REDIS_URL`) MUST NOT be Upstash-hosted.
- Legacy CVV keys from prior releases MUST be removed with:
  - `python -m app.admin_cli secret purge-payment-cvv --yes`
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
- when egress gateways are configured:
  - `TRAIN_PROVIDER_EGRESS_PROXY_URL` MUST target internal `egress-train`
  - if `RESTAURANT_MODULE_ENABLED=true`, `RESTAURANT_PROVIDER_EGRESS_PROXY_URL` MUST target internal `egress-restaurant`
  - egress gateways MUST deny unmatched paths and methods (fail closed)
  - egress gateways MUST not expose administrative control surfaces

### Crash/dump safety

Production runtime MUST:

- disable core dumps for payment execution paths
- avoid stack traces that include decrypted payloads
- avoid exception messages that include request/response bodies with sensitive fields

### Severity classification and deploy gate

The following are automatically CRITICAL and block deploy:

- plaintext PAN persistence anywhere
- any CVV collection/caching/persistence
- logging of provider payment payloads
- queue serialization of card data
- reintroduction of CVV cache write paths
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
7. CI security gates must remain enabled:
   - repository secret scan (gitleaks),
   - Python dependency vulnerability scan (`pip-audit`),
   - Web dependency vulnerability scan (`npm audit --omit=dev`),
   - frontend unit-test coverage gate.

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
