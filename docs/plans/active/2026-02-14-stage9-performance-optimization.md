# Stage 9 Performance Optimization Plan (Backend -> Frontend)

## Goal

Reduce end-user latency and server load for train task workflows with backend-first optimization, followed by frontend query/poll efficiency improvements.

## User Constraints (Confirmed)

- Backend performance first, frontend second.
- Structural overhaul is allowed.
- Database risk acceptance: high (`accept all risk`).
- Preserve user sign-in credentials when possible:
  - `users.email`
  - `users.display_name`
  - `users.password_hash`
- Session/runtime data does not need to be preserved.

## Execution Status (2026-02-15)

- Phase 1 (baseline + RED tests): completed.
- Phase 2 backend optimization: completed.
  - Initial bounded list + latest-row summary/index improvements shipped in Stage9 commits.
  - Stage10 backend follow-up shipped PostgreSQL `DISTINCT ON` latest-row paths with non-Postgres fallback compatibility in `api/app/modules/train/service.py`.
  - Stage10 additive index migration applied: `api/alembic/versions/20260215_0009_task_list_tail_latency_indexes.py`.
- Phase 3 frontend optimization: completed.
  - Train dashboard polling now refreshes active tasks every cycle while completed tasks refresh periodically or on forced triggers (initial load/visibility/action mutations).
  - Frontend task list updates are key-compared before state commit to reduce unnecessary rerenders.
- Phase 4 reset workflow: completed (`infra/scripts/reset-local-db.sh` + tests/docs).
- Stage 12 comprehensive hardening: completed.
  - Added backend tie-order regression tests for latest-attempt/latest-ticket summary selection determinism.
  - Added frontend polling behavior unit-test suite with Vitest + Testing Library (`web/components/train/__tests__/train-dashboard.polling.test.tsx`).
  - Added benchmark compare + hybrid-threshold gate scripts and shell validation coverage (`infra/scripts/benchmark-train-task-list-compare.sh`, `infra/scripts/benchmark-threshold-check.sh`, `infra/tests/test_benchmark_train_task_list_compare.sh`).
  - Wired web unit tests and benchmark-compare script validation into `.github/workflows/ci-infra-quality-gates.yml`.

## Scope

- Backend:
  - `api/app/modules/train/service.py`
  - `api/app/modules/train/router.py`
  - `api/app/modules/train/schemas.py`
  - `api/alembic/versions/*` (additive migration only)
  - `api/tests/test_train_tasks.py`
- Frontend:
  - `web/components/train/train-dashboard.tsx`
  - `web/lib/types.ts`
- Infra/ops (optional but in-scope):
  - `infra/scripts/*` (local performance reset helpers if needed)

## Phase 1: Baseline + RED Tests

1. Add failing backend test coverage for task-list pagination limits:
- `GET /api/train/tasks?status=<...>&limit=<n>` returns at most `n`.
- invalid limit values fail validation.

2. Add failing frontend-oriented backend contract test:
- list endpoint remains backward compatible while supporting separate active/completed fetching.

## Phase 2: Backend Implementation (Priority)

1. Add list pagination contract:
- Extend `list_tasks` with `limit` support.
- Wire `limit` query param in train router.

2. Optimize latest-attempt/ticket summary queries:
- Replace current full-scan-per-task-id approach with single-query latest-row extraction (window function strategy).

3. Add DB indexes for list/query hot paths:
- tasks composite index for filtered ordering.
- latest-attempt lookup index.
- latest-ticket lookup index.

4. Verification:
- Run targeted backend tests.
- Run broader backend suite relevant to train/auth.

## Phase 3: Frontend Implementation (After Backend Green)

1. Replace single `status=all` polling request with two targeted calls:
- active tasks request (small limit).
- completed tasks request (small limit, optional refresh).

2. Preserve UX behavior:
- same task cards/actions/messages.
- no regression in refresh behavior or auth-expiry handling.

3. Verification:
- web typecheck (`npx tsc --noEmit`).
- smoke-check dashboard behavior assumptions by static inspection + contract tests.

## Phase 4: Optional Data-Reset Workflow (Risk Accepted)

If backend profile remains constrained after query/index improvements:
1. Add documented local reset path with optional preservation of `users.email`, `users.display_name`, `users.password_hash`.
2. Keep this local/dev oriented by default; production run requires explicit operator confirmation and rollback notes.

## Rollback

- Code rollback: revert Stage 9 commits.
- DB rollback: downgrade Stage 9 migration revision.
- Frontend rollback: revert dashboard polling split to previous single-endpoint behavior.

## Completion Criteria

1. Backend list endpoint supports bounded result sets and optimized latest-row selection.
2. Frontend polling no longer requests unbounded mixed task payloads.
3. Targeted tests pass and typecheck is green.
4. Docs and changelog updated with commit-based entries.
