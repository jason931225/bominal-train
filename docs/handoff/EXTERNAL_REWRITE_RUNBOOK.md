# External Rewrite Runbook

## Objective

Rebuild provider adapters for functional parity with current third-party behavior while keeping `srtgo` canonical for SRT overlap.

## Execution Sequence

1. Read `tools/srtgo-capture/HANDOFF_EXTERNAL_REWRITE.md`.
2. Implement `critical` endpoints first from `PROVIDER_FIELD_MAP.json`.
3. Respect auth-scope labels before adding endpoint probes.
4. Add `high_signal` endpoints that provide stateless/context-gated value.
5. Keep `mapped` and `unknown` behind feature flags or deferred backlog.
6. Validate flows: login, search, reserve, list/detail, cancel, logout.
7. Validate optional payment flow only with safe test credentials/environment.
8. Reconcile divergences using `srtgo` as canonical tie-breaker unless explicitly overridden.

## Acceptance Checks

- Endpoint request/response field behavior matches canonical parser expectations.
- Redaction contract is preserved in any capture/debug tooling.
- Public stateless shortlist remains callable without login session data.
- Context-gated endpoints correctly enforce NetFunnel/session prerequisites.
- Unknown endpoints are explicitly tracked; no forced assumptions.
