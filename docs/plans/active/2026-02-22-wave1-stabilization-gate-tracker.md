# Wave 1 Stabilization Gate Tracker (2026-02-22)

## Purpose

Provide a reviewer-friendly stabilization gate view for Wave 1 using objective repository evidence, with explicit remaining actions for anything not fully closed.

## Architecture Preface (for reviewers new to bominal)

`bominal` is a modular monorepo with:
- `web/` (Next.js App Router UI)
- `api/` (FastAPI backend)
- `worker` and `worker-restaurant` (arq background execution)
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
| W1-G08 | Fresh docs validation evidence exists for the current branch state | missing | Validators are defined (`infra/tests/test_docs_pointers.sh`, `infra/tests/test_intent_routing.sh`, `infra/tests/test_docs_consistency.sh`, `infra/tests/test_changelog.sh`, `infra/tests/test_execution_ledgers.sh`) | Run validators and attach command outputs to the Wave 1 review record |
| W1-G09 | Performance-wave verification is fresh (not only historical) | partial | Stage 9 implementation/changelog evidence exists (`CHANGELOG.md` entries), but timestamped re-validation proof is not bundled in a Wave 1 tracker artifact | Execute Stage 9 target verification commands and add dated evidence note under active plans |
| W1-G10 | Reviewer sign-off packet is complete for Wave 1 stabilization | partial | This tracker now centralizes status/evidence/actions | Add reviewer decision section (`approve`, `approve with conditions`, `hold`) after running validations |

## Reviewer Decision Stub

- Decision: `pending`
- Conditions:
  - Run and record docs validators (W1-G08).
  - Attach fresh Stage 9 verification evidence (W1-G09).
  - Update this tracker decision line after evidence review.
