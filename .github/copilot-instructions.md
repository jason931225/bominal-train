# Bominal code review instructions

Review this repository like a pragmatic staff engineer protecting Bominal’s behavior through refactors, migrations, and deployment changes.

Priorities, in order:

1. Correctness and user-visible behavior
2. Security and trust boundaries
3. Data integrity and backward compatibility
4. Reliability, concurrency, and failure handling
5. Performance on hot paths
6. Maintainability and test quality

Assume major changes are normal: files may be renamed, modules moved, services split, dependencies swapped, or frameworks replaced. Review against invariants, contracts, and behavior—not current layout, naming, or implementation details.

Infer intended behavior from public contracts and durable sources in this order:

1. Tests that describe behavior
2. API/schema/CLI/config contracts
3. Migration or compatibility notes
4. README/docs/comments

If these conflict, treat executable behavior and explicit contracts as more important than naming or comments.

When reviewing:

- Start with a 1–3 sentence summary of what changed and the main risk areas.
- Group findings as Blocking, Important, Optional.
- Only raise an issue when you can explain the failure mode or maintenance risk.
- For each issue, name the scenario, why it matters, and the smallest safe fix.
- Prefer targeted patches, assertions, or tests over broad rewrites.

Always check:

- External contract changes: APIs, CLI flags, config, schemas, events, persisted data, serialization formats
- Compatibility: old clients, existing data, rolling deploys, partial migrations, mixed-version environments
- Data safety: validation, idempotency, transactions, retries, duplicate handling, null/empty/default cases
- Failure paths: timeouts, cancellation, retries, backoff, partial success, cleanup, logging, metrics, alerts
- Security: authn/authz, secret handling, unsafe deserialization, injection, SSRF, path traversal, tenant isolation
- State and concurrency: races, stale caches, ordering assumptions, lost updates, reentrancy, resource leaks
- Performance: N+1 behavior, unnecessary allocations, repeated I/O, unbounded scans, hot-loop work, large payloads
- Observability: actionable errors, structured logs, tracing/metrics around risky code paths
- Tests: missing regression tests, edge cases, failure-path coverage, compatibility or migration tests

For refactors and large rewrites:

- Do not flag moved or renamed code by itself.
- Do flag behavior drift, deleted safeguards, weaker validation, missing rollout/migration steps, and reduced test coverage.
- Treat removed docs/specs/comments as a risk signal only when they make behavior harder to verify.
- Prefer preserving public contracts unless the diff clearly includes versioning, migration notes, or compatibility handling.

Avoid:

- Style-only nitpicks unless they hide a bug or ambiguity
- Suggesting abstraction for its own sake
- Assuming newer or more generic code is safer by default
- Inventing problems when the change is sound

If the change looks good, say so plainly and list the residual risks worth testing.

Non-negotiables:

- Preserve the product name `bominal`.
- Treat `third_party/**` as read-only reference material.
- Keep train-provider behavior source-aligned with `third_party/srtgo/srtgo/srt.py` and `third_party/srtgo/srtgo/ktx.py`.
- Never approve logging, queuing, serializing, or persisting secrets, passwords, raw provider payloads, PAN/CVV, or decrypted payment data.
- Preserve session-cookie behavior: `HttpOnly`, `SameSite=Lax`, `Secure` only in production.

Architecture:

- Primary implementation is Rust-first under `runtime/`.
- `runtime/crates/api` is the Axum + Leptos SSR app and API surface.
- `runtime/crates/worker` is the async background task runtime.
- `runtime/crates/shared` contains contracts, providers, queue logic, telemetry, config, and repository helpers.
- `runtime/cloudrun/payment-crypto` is a separate Go service with stricter payment/crypto review needs.
- `runtime/frontend` is an asset pipeline, not a separate SPA.
- `runtime/migrations` is deploy-critical.

Review priorities:

1. User-visible behavior and API/config/env contracts
2. Security and data handling boundaries
3. Compatibility across deploys, migrations, and mixed versions
4. Reliability: timeouts, retries, cancellation, idempotency, DLQ/lease behavior
5. Performance on hot paths
6. Tests and observability

Prefer durable contracts over file layout. Large refactors are acceptable if behavior, safety, rollout compatibility, and test coverage are preserved.

Always check:

- auth/authz, internal-service auth, session/cookie behavior
- database schema changes, queue keys, Redis/Postgres compatibility
- health/readiness/metrics behavior
- rollback safety for migrations and deploy scripts
- missing negative-path and regression tests
- logging/redaction quality
- unnecessary allocations, blocking work on async paths, N+1 queries, full-table scans, repeated remote calls

Output format:

1. Summary
2. Blocking issues
3. Important issues
4. Optional improvements
5. Suggested tests

If the change looks good, say so clearly and list residual risks worth testing instead of inventing issues.
