# Security Policy

## Principles

- fail closed on uncertainty
- least privilege by default
- minimize sensitive data lifetime
- never expose secrets in outputs/logs/docs

## Auth and Authorization

- preserve session cookie contract: HttpOnly + SameSite=Lax + Secure only in production
- local DB roles remain authorization source of truth

## Secrets and Crypto

- envelope encryption with `kek_version` enforcement
- key rotation requires explicit procedure and verification

## Payment and CDE

- PAN/CVV must not persist outside approved CDE boundaries
- CVV must not be collected, accepted, cached, or persisted by bominal
- Legacy CVV cache keys (if any) must be purged via admin controls
- no raw provider payment payload logging/persistence
- payment egress must be allowlisted with TLS verification enabled

## Queue and Logging Safety

- queue payloads must contain safe identifiers only
- redact sensitive fields at logging and persistence boundaries
