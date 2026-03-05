# Runtime Refactor + Edge HTTP/3 Readiness Design

Date: 2026-03-04
Status: Proposed (one-shot draft from existing plan inputs)

## Problem Statement

`runtime/crates/api/src/web.rs`, `runtime/crates/api/src/http/admin.rs`, `runtime/crates/api/src/services/admin_service.rs`, `runtime/crates/api/src/http/internal_auth.rs`, `runtime/crates/api/src/services/provider_jobs_service.rs`, and `runtime/crates/worker/src/runtime/executor.rs` are oversized and hard to evolve safely. At the same time, CI/CD and artifact boundaries must stay aligned with `docs/MANUAL.md` and immutable constraints (`third_party/**` read-only, sensitive data controls, stable session-cookie contract).

HTTP/3 work is explicitly deferred and should remain edge-only unless canary evidence clears promotion gates.

## Design Goals

- Preserve route behavior and provider parity while reducing module complexity.
- Keep frontend artifacts generated-only and Docker-built.
- Normalize provider contracts so API/worker surfaces do not leak SRT wire types.
- Increase verification rigor (tests, security gates, coverage ratchet, smoke builds).
- Keep HTTP/3 as optional edge experiment with objective go/no-go criteria.

## Non-Goals

- No platform rewrite.
- No native HTTP/3 listener in `bominal-api`.
- No edits to `third_party/srtgo` or `third_party/catchtable`.
- No route-path redesign as part of the refactor.

## Considered Approaches

### Approach A: Strictly Sequential PRs
Single lane from baseline through docs.

- Advantage: simplest coordination.
- Risk: slowest throughput.

### Approach B: Phase-Gated + Controlled Parallel Lanes (Selected)
Shared prerequisites first, then lane-based execution with explicit ownership and merge order.

- Advantage: balanced speed and risk.
- Risk: requires stronger review discipline.

### Approach C: Big-Bang Branch
Do all splits/contracts in one branch.

- Advantage: fewer transitional shims.
- Risk: high review and rollback cost.

## Selected Architecture

### 1) Module Boundaries

- Split API web/auth/admin/internal-auth and services monoliths into focused module trees.
- Preserve route signatures via `mod.rs` re-exports and compatibility wrappers while moving code.
- Split worker executor into planner/dispatcher/state/retry/rate-limit modules with unit-level tests.

### 2) UI Boundary

- Keep current `bominal-ui-primitives` + `bominal-ui-patterns` and add a facade boundary (`bominal-ui`) for stable internal imports.
- Move dashboard/jobs/security rendering composition into reusable UI patterns.

### 3) Provider Boundary

- Add canonical provider contract/models/errors/retry/redaction modules in `runtime/crates/shared/src/providers`.
- Keep provider-specific wire models inside `providers/srt` and `providers/ktx` adapters only.

### 4) Delivery and Ops Boundary

- Keep generated assets untracked under `runtime/frontend/dist`.
- Keep Docker as build source of truth for runtime assets copied to `/app/frontend/dist`.
- Add mandatory CI gates for security, coverage ratchet, and container smoke checks.

### 5) HTTP/3 Boundary

- Edge-only canary with automatic fallback to HTTP/2/1.1.
- Promote only if KPI thresholds are met and no auth/session regressions appear.

## Testing Strategy

- TDD-first per refactor unit: add failing route/service/contract tests before each module split.
- Keep existing integration suite (`auth_page_test`, `dashboard_routes_test`, `admin_*`, `internal_api_auth_contract_test`) as behavior guardrails.
- Add worker module unit tests for state transitions, retry classification, and per-task rate-limit policy.
- Add CI freshness checks for generated internal API schema/SDK.

## Error Handling and Safety

- Fail closed on auth/internal-service boundaries.
- Centralize provider error classification + retryability.
- Preserve redaction policy and avoid sensitive payload logging.

## Rollout Plan

1. Baseline + branch policy lock
2. Artifact/Docker contract hardening
3. API/UI/admin/internal-auth/provider/worker refactor lanes
4. Internal schema + CI/security/perf gates
5. Docs/runbook alignment
6. Optional HTTP/3 edge canary after backlog completion

## Success Criteria

- No route regressions in existing API tests.
- No tracked `runtime/frontend/dist/**` artifacts.
- No oversized hotspot files remaining in target modules.
- CI enforces mandatory quality/security gates consistent with `docs/MANUAL.md` target state.
- HTTP/3 remains deferred unless canary evidence passes all promotion criteria.
