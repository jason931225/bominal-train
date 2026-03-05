---
applyTo: "runtime/crates/**,runtime/Cargo.toml,runtime/Cargo.lock,runtime/Dockerfile.api,runtime/Dockerfile.worker,runtime/compose.prod.yml,runtime/data/train/**"
---

Review these changes as part of the Rust runtime platform.

Focus on:

- Axum + Leptos SSR contract stability: route behavior, auth, cookies, error envelopes, health/readiness/metrics, and admin/SSE behavior when touched.
- Tokio correctness: no blocking work on hot async paths; preserve timeouts, cancellation, backpressure, bounded fan-out, and graceful shutdown.
- SQLx/Postgres safety: schema/query compatibility, transaction boundaries, idempotency, null/default handling, and mixed-version deploy safety.
- Redis/queue behavior: lease TTLs, retry/DLQ semantics, duplicate delivery, stale lock recovery, and key-prefix compatibility.
- Shared/provider contracts: avoid drift in queue payloads, telemetry fields, provider request/response handling, and station-catalog behavior.
- Performance: flag N+1 queries, full-table scans, repeated remote fetches, unnecessary cloning/serialization, and sync CPU work in async code.
- Tests: ask for contract and negative-path tests in touched crates, not only unit tests.

For station-catalog-related changes, preserve the committed snapshot model and validation flow; do not introduce a production live-fetch dependency.
