# PCI DSS + OWASP ASVS Compliance Matrix

## Scope

- Platform baseline: OWASP ASVS L2.
- Payment/CDE/relay paths: OWASP ASVS L3 controls.
- Payment controls aligned to PCI DSS 4.0.1 (cardholder data protection, key management, logging, and network boundary controls).

## Control mapping

| Control | Repo enforcement | Verification |
|---|---|---|
| Session cookie security (`HttpOnly`, `SameSite=Lax`, `Secure` in prod) | `api/app/services/auth.py`, `api/app/http/deps.py`, `docs/SECURITY.md` | `api/tests/test_auth_flow.py` |
| Password hashing + session token hashing | `api/app/core/security.py`, `api/app/services/auth.py` | `api/tests/test_auth_flow.py` |
| Envelope encryption AES-256-GCM + per-record DEK + `kek_version` | `api/app/core/crypto/envelope.py`, `api/app/core/crypto/secrets_store.py` | `api/tests/test_crypto_envelope.py` |
| CVV only in Redis bounded TTL | `api/app/services/wallet.py`, `api/app/core/config.py`, `infra/env/*/api.env*` | `api/tests/test_wallet.py`, `api/tests/test_security_config.py` |
| Redaction boundary enforcement | `api/app/core/crypto/redaction.py`, `api/app/core/logging.py`, `api/app/main.py` | `api/tests/test_crypto_redaction.py`, `api/tests/test_logging_redaction.py` |
| Safe metadata only (`meta_json_safe`, `data_json_safe`) | `api/app/core/crypto/safe_metadata.py`, `api/app/modules/train/*.py`, `api/app/modules/restaurant/worker.py` | `api/tests/test_safe_metadata.py` |
| Queue payload safety | `api/app/modules/train/queue.py`, `api/app/modules/restaurant/queue.py` | `api/tests/test_queue_payload_safety.py` |
| Provider egress allowlist + SSRF controls | `api/app/modules/train/providers/transport.py`, `api/app/core/config.py` | `api/tests/test_provider_egress_transport.py` |
| Runtime Redis persistence guard in production | `api/app/main.py` | `api/tests/test_runtime_security_checks.py` |
| Continuous sensitive log scanning (PAN/CVV/token markers) | `infra/scripts/scan_sensitive_logs.py`, CI workflow | `infra/tests/test_sensitive_log_scan.sh` |

## Release gate

Deployment must be blocked when CRITICAL controls fail, including:

- raw PAN/CVV in logs/queues/artifacts
- CVV persistence outside Redis TTL cache
- disabled TLS verification on provider payment egress
- missing/enforcement-bypassed `kek_version` at decrypt boundaries
- non-allowlisted provider egress host usage
