---
applyTo: "runtime/cloudrun/payment-crypto/**"
---

This path is the Go Cloud Run payment-crypto service. Review conservatively.

Treat as a high-sensitivity path:

- Never widen public exposure, skip internal-service auth, or weaken TLS/KMS requirements.
- Preserve request validation, redaction, and endpoint/status-code contracts for store/decrypt/execute flows.
- Keep KMS AAD/context/versioning stable across encrypt/decrypt and DB read/write paths.
- Ensure DB behavior is idempotent and consistent for upsert + active lookup flows.
- Prefer explicit context propagation and bounded I/O; flag missing timeouts, repeated heavyweight setup, or unbounded memory growth.
- Reject changes that log, persist, or return decrypted payment data outside the intended contract.
- Require negative-path tests for auth failure, invalid payloads, KMS errors, DB errors, and compatibility with existing stored records.
