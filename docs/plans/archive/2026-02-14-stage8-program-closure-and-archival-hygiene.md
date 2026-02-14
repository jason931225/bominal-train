> Archive note (2026-02-14): Completed and moved from `docs/plans/active/` during Stage 8 closure.
> See `docs/plans/archive/2026-02-14-program-closure-report.md` for final status.

# Stage 8 Program Closure and Archival Hygiene Implementation Plan

Status: Completed in implementation (2026-02-14).

**Goal:** Close the completed restructure program with enforceable governance hygiene: archive completed plans, normalize execution ledgers, and add CI checks that prevent misleading template lock/request state.

**Architecture:** Treat Stage 8 as a closure-only governance stage. No product/runtime behavior changes. Convert completed active plans to historical archive records, add a current-state marker for `docs/plans/active/`, and enforce ledger-template safety through a dedicated infra test.

**Tech Stack:** Markdown docs governance, Bash validation scripts, GitHub Actions infra-tests.

---

### Task 1: Add closure-stage executable plan and archive report scaffold

**Files:**
- Create: `docs/plans/active/2026-02-14-stage8-program-closure-and-archival-hygiene.md`
- Create: `docs/plans/archive/2026-02-14-program-closure-report.md`

Steps:
1. Define closure scope, non-goals, and explicit acceptance criteria.
2. Record final completion evidence and verification set in closure report.

### Task 2: Normalize lock/request ledgers for safe templates

**Files:**
- Modify: `docs/LOCK.md`
- Modify: `docs/REQUEST.md`

Steps:
1. Add explicit `Current Entries` sections.
2. Convert template sample statuses from live states to non-live example values.
3. Keep template placeholders, but ensure they cannot be interpreted as active work.

### Task 3: Add CI guard for ledger-template safety

**Files:**
- Create: `infra/tests/test_execution_ledgers.sh`
- Modify: `.github/workflows/infra-tests.yml`
- Modify: `docs/playbooks/daily-operations-chores.md`

Steps:
1. Fail when lock/request template sections contain live statuses (`ACTIVE`/`OPEN`).
2. Fail when template placeholders appear in current-entry sections.
3. Add script to routine local and CI checks.

### Task 4: Archive completed plans and stabilize active-plan state

**Files:**
- Move completed plan docs from `docs/plans/active/` to `docs/plans/archive/`.
- Create: `docs/plans/active/README.md`
- Modify: `docs/plans/README.md`

Steps:
1. Move Stage 1-7 and umbrella/backlog trackers to archive.
2. Add archive note to moved plans.
3. Define that active directory may contain only `README.md` when no executable plans are open.

### Task 5: Update canonical pointers and intent routing

**Files:**
- Modify: `docs/README.md`
- Modify: `docs/INTENT_ROUTING.md`

Steps:
1. Repoint plan pointers to archived paths.
2. Add closure-report pointer.
3. Update plan-intent routing to use active README + closure report.

### Task 6: Close backend TODO archival hygiene gap

**Files:**
- Modify: `docs/todo/backend-production-readiness.md`

Steps:
1. Mark document as archived/closed with date.
2. Keep original task content as historical implementation record.

### Task 7: Verify and record changelog

Run:
- `bash infra/tests/test_execution_ledgers.sh`
- `bash infra/tests/test_docs_pointers.sh`
- `bash infra/tests/test_intent_routing.sh`
- `bash infra/tests/test_docs_consistency.sh`
- `bash infra/tests/test_changelog.sh`
- `python3 -m unittest discover -s infra/tests -p 'test_*.py'`

Acceptance Criteria:
- Completed Stage 1-7/umbrella/backlog plans are archived.
- `docs/plans/active/` clearly indicates no active executable plans.
- Lock/request template sections cannot be mistaken as live coordination entries.
- CI/local validations enforce the new ledger-safety contract.
- Changelog includes commit-based entries for Stage 8 closure/governance changes.

Assumptions and Defaults:
- Archival operations are documentation-governance only; no runtime behavior changes.
- Archived plans are immutable historical records except archive-note headers.
