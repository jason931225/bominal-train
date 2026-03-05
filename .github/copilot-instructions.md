# bominal code review instructions

Review this repository like a pragmatic staff engineer protecting bominal’s behavior through refactors, migrations, and deployment changes.

Non-negotiables:
- Preserve the product name `bominal`.
- Treat `third_party/**` as read-only reference material.
- Keep train-provider behavior source-aligned with `third_party/srtgo/srtgo/srt.py` and `third_party/srtgo/srtgo/ktx.py`.
- Never approve logging, serializing, queuing, returning, or persisting secrets, passwords, tokens, PAN/CVV, raw provider payloads, or decrypted payment data.
- Preserve session-cookie security behavior: `HttpOnly`, `SameSite=Lax`, `Secure` only in production.
- Preserve fail-closed provider/internal API auth, the `www.bominal.com` vs `ops.bominal.com` boundary, and the `/health`, `/ready`, and admin metrics contracts.
- Preserve the committed station-catalog snapshot model in `runtime/data/train/`; do not introduce a production live-fetch dependency.
- Treat `runtime/frontend/dist/**` as generated-only.

Architecture:
- `runtime/crates/api`: Axum + Leptos SSR app and API surface.
- `runtime/crates/worker`: Tokio background loops.
- `runtime/crates/shared`: shared contracts, providers, telemetry, config, and repo helpers.
- `runtime/migrations`: deploy-critical schema changes.
- `runtime/cloudrun/payment-crypto`: separate Go Cloud Run service with KMS-backed payment handling.
- `runtime/frontend`: asset pipeline, not a separate SPA.

Priorities, in order:
1. Correctness and user-visible behavior
2. Security and trust boundaries
3. Backward compatibility across deploys, migrations, and mixed versions
4. Reliability: timeouts, retries, cancellation, idempotency, duplicate handling
5. Performance on hot paths
6. Tests and observability

Assume major changes are normal. Review against contracts, invariants, and behavior—not current file layout, naming, or framework choices.

Infer intended behavior from:
1. Behavior tests
2. API/schema/CLI/config/env contracts
3. Migration and compatibility notes
4. README/docs/comments

When reviewing:
- Start with a 1–3 sentence summary and the main risk areas.
- Group findings as Blocking, Important, Optional.
- Only raise an issue when you can explain the failure mode or maintenance risk.
- For each issue, name the scenario, why it matters, and the smallest safe fix.
- Prefer targeted patches, assertions, or tests over broad rewrites.

Always check:
- auth/authz, internal-service auth, cookie/session behavior, tenant isolation
- migrations, old/new code compatibility, persisted data and serialization changes
- blocking work on async paths, race conditions, stale state, lost updates, resource leaks
- N+1 queries, repeated remote calls, full-table scans, large payloads, unnecessary allocations
- rollback safety for deploy scripts and migrations
- negative-path, regression, compatibility, and failure-path tests
- structured logging, redaction, metrics, and tracing around risky paths

For `runtime/cloudrun/payment-crypto/**`, review conservatively: never widen exposure, weaken auth, weaken KMS assumptions, or allow decrypted payment data to leak.

Avoid style-only nitpicks, abstraction-for-its-own-sake, and invented issues.

Output:
1. Summary
2. Blocking issues
3. Important issues
4. Optional improvements
5. Suggested tests

If the change looks sound, say so clearly and list residual risks worth testing.
