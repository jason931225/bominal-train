# Independent Code Best-Practice Review Plan

Date: 2026-02-24  
Status: Active  
Owner: Engineering

## Objective

Run a full code-quality and maintainability review using language/framework/infra best practices independent of bominal-specific policy decisions.

This review is explicitly separate from project-specific constraints so that we can detect baseline engineering debt and future-proofing gaps.

## Scope

- `api/` (FastAPI, SQLAlchemy, worker runtime, queue interactions)
- `web/` (Next.js App Router, state/data flows, rendering and hydration safety)
- `infra/` (build/deploy scripts, compose/runtime defaults, operational safety)
- test architecture quality (signal strength, mutation resistance for critical logic)

Out of scope:
- Product roadmap decisions
- Third-party provider behavior changes
- `third_party/**` source modifications

## Review Principles

1. Prioritize correctness and safety over style-only changes.
2. Prefer measurable outcomes (latency, reliability, failure containment, test signal).
3. Keep recommendations actionable with migration path and rollback notes.
4. Separate critical blockers from opportunistic cleanup.

## Workstream Plan

### Phase 1: Baseline Inventory

1. Capture static baseline:
   - module boundaries and ownership
   - dependency graph and runtime versions
   - high-risk surfaces (auth, crypto, queue/worker, deploy)
2. Capture runtime baseline:
   - startup, health, and steady-state resource profile
   - top hot paths and current bottlenecks
3. Capture test baseline:
   - per-domain coverage profile
   - weak tests (execution-only / low-signal assertions)

Deliverable:
- Baseline snapshot with explicit risk map.

### Phase 2: Best-Practice Gap Assessment

Assess against ecosystem standards (independent of local policy):

1. Backend:
   - FastAPI request lifecycle and async correctness
   - DB session/transaction boundaries
   - background worker idempotency and retry correctness
2. Frontend:
   - render/data-fetch boundaries
   - state consistency and UX failure handling
   - accessibility and mobile resilience
3. Infra:
   - immutable deploy practices
   - health/rollback guarantees
   - image/runtime minimalism and security posture
4. Test quality:
   - mutation-resistant assertions on critical invariants
   - regression coverage for highest-risk paths

Deliverable:
- Gap list with severity (`P0/P1/P2`) and rationale.

### Phase 3: Remediation Roadmap

1. Convert findings into sequenced implementation tracks:
   - Track A: correctness/safety blockers
   - Track B: performance/reliability
   - Track C: maintainability/refactor debt
2. For each track include:
   - target state
   - concrete tasks
   - test and rollout requirements
   - rollback strategy

Deliverable:
- Executable roadmap with owner and ordering.

### Phase 4: Validation and Ratchet

1. Add or update tests for each critical fix.
2. Introduce ratchet metrics:
   - reliability SLO-oriented checks
   - warning/deprecation burn-down
   - coverage-quality trend (not blanket 100%)
3. Re-run baseline to verify improvement.

Deliverable:
- Before/after validation report.

## Exit Criteria

1. All P0 blockers resolved or explicitly waived with owner + date.
2. Critical paths have high-signal regression tests.
3. Deploy and rollback paths validated with objective evidence.
4. Follow-up backlog is prioritized and time-bounded.

## Initial Candidate Focus Areas

1. Authentication and account-recovery hardening.
2. Worker/provider reliability and retry safety.
3. UI state consistency for event-driven updates.
4. Deploy script resilience and operational ergonomics.
5. Dependency and warning lifecycle management.
