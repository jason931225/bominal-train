# Wave 1 Stabilization Gate Tracker (2026-02-22)

## Purpose

Provide a reviewer-friendly stabilization gate view for Wave 1 using objective repository evidence, with explicit remaining actions for anything not fully closed.

## Architecture Preface (for reviewers new to bominal)

`bominal` is a modular monorepo with:
- `web/` (Next.js App Router UI)
- `api/` (FastAPI backend)
- `worker` (arq background execution)
- `postgres` + `redis` infrastructure

Primary user-facing capability is Train task automation. Restaurant and Calendar remain staged behind controlled exposure.

## Current Feature Status Snapshot

- Train module: active and user-facing (`README.md`, `docs/ARCHITECTURE.md`).
- Restaurant module: policy/runtime scaffold present, still controlled exposure (`docs/ARCHITECTURE.md`, `/api/modules` contract notes).
- Calendar module: coming soon (`README.md`, `docs/ARCHITECTURE.md`).
- Restructure program Stages 1-8: archived as completed (`docs/plans/archive/2026-02-14-program-closure-report.md`).
- Stage 9 performance plan: active execution artifact (`docs/plans/active/2026-02-14-stage9-performance-optimization.md`).

## Impact of Proposed Changes

This tracker is docs-only and does not change runtime behavior. It reduces reviewer onboarding time and makes remaining stabilization risk explicit before further delivery/merge decisions.

## Gate Status Legend

- `done`: objective repo evidence exists and gate intent is satisfied.
- `partial`: evidence exists but is incomplete, stale, or missing closure proof.
- `missing`: no objective closure evidence yet.

## Wave 1 Stabilization Gates

| Gate ID | Gate | Status | Objective evidence available in repo | Concrete remaining action |
|---|---|---|---|---|
| W1-G01 | Architecture baseline is documented for all runtime tiers | done | `docs/ARCHITECTURE.md` defines web/api/workers + infra topology | Re-check only when runtime topology changes |
| W1-G02 | Current module exposure is reviewer-visible (Train active, Restaurant/Calendar staged) | done | `docs/ARCHITECTURE.md`, `README.md`, module contract notes in docs | Re-verify at next module exposure change |
| W1-G03 | Prior restructure work (Stages 1-8) has closure evidence | done | `docs/plans/archive/2026-02-14-program-closure-report.md`, `docs/plans/archive/2026-02-14-backlog-status-report.md` | Keep archive immutable; add new closure report for future waves |
| W1-G04 | Active Wave work has an executable plan with bounded scope | partial | `docs/plans/active/2026-02-14-stage9-performance-optimization.md` exists with phases/acceptance criteria | Add or link explicit Stage 9 closure artifact once final verification is complete |
| W1-G05 | Program-level lock/request hygiene is closed | done | `docs/LOCK.md` entries are `RELEASED`; `docs/REQUEST.md` says no open cross-scope requests | Maintain clean ledgers during new active sessions |
| W1-G06 | Docs governance policy is canonical and cross-linked | done | `docs/EXECUTION_PROTOCOL.md`, `docs/DOCUMENTATION_WORKFLOW.md`, `docs/PERMISSIONS.md`, `docs/GUARDRAILS.md`, `docs/INTENT_ROUTING.md` | Re-run docs consistency checks whenever governance docs change |
| W1-G07 | Active Wave 1 tracker is discoverable via canonical pointers | done | This tracker in `docs/plans/active/` plus pointer registration in `docs/README.md` | Keep pointer entry updated if file is renamed or archived |
| W1-G08 | Fresh docs validation evidence exists for the current branch state | done | 2026-02-22 UTC evidence run captured in this tracker (`test_docs_pointers`, `test_execution_ledgers`, `test_changelog`, `test_intent_routing`, `test_docs_consistency`) | Re-run the validator bundle on every branch that changes docs/governance artifacts |
| W1-G09 | Performance-wave verification is fresh (not only historical) | done | 2026-02-22 UTC Stage 9 freshness run captured in this tracker (`pytest` provider egress + auth flow, web `tsc --noEmit`) | Re-run the Stage 9 freshness suite if API/Web contract behavior changes |
| W1-G10 | Reviewer sign-off packet is complete for Wave 1 stabilization | done | Gate table, dated evidence, and explicit decision state are now in this tracker | Carry decision conditions forward into the next wave intake checklist |

## Verification Evidence (2026-02-22 UTC)

Execution window: 2026-02-22T00:39:00Z to 2026-02-22T00:42:00Z.

### Docs/governance validators

- `bash infra/tests/test_docs_pointers.sh` -> `OK: Canonical pointer library is valid (42 pointers).`
- `bash infra/tests/test_execution_ledgers.sh` -> `OK: execution ledgers are structurally valid and template-safe.`
- `bash infra/tests/test_changelog.sh` -> `OK: CHANGELOG.md structure and commit-based Unreleased entries are valid.`
- `bash infra/tests/test_intent_routing.sh` -> `OK: intent routing is valid.`
- `bash infra/tests/test_docs_consistency.sh` -> `OK: docs consistency checks passed.`
- `bash infra/tests/test_deprecation_policy.sh` -> `OK: deprecation policy validation checks passed.`
- `bash infra/tests/test_deprecation_references.sh` -> `OK: deprecation reference scan checks passed.`

### Stage 9 freshness checks

- `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_provider_egress_transport.py` -> `5 passed in 0.03s`
- `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_auth_flow.py` -> `19 passed, 17 warnings in 5.69s`
- `docker compose -f infra/docker-compose.yml run --rm web npx tsc --noEmit` -> exit code `0`
- Note: host-side `cd web && npx tsc --noEmit` required network access to npm registry in this environment and was not used for final gate evidence.

## Reviewer Decision Stub

- Decision: `approve with conditions`
- Conditions:
  - Add/link an explicit Stage 9 closure artifact before archiving the active Stage 9 plan (`W1-G04` currently `partial`).
  - If API/Web behavior changes after this evidence window, rerun the Stage 9 freshness checks and append updated timestamps.
