> Archive note (2026-02-14): Completed and moved from `docs/plans/active/` during Stage 8 closure.
> See `docs/plans/archive/2026-02-14-program-closure-report.md` for final status.

# Stage 7 Docs Canonization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

Status: Completed in implementation (2026-02-14).

**Goal:** Finalize canonical docs governance for restructure work: active plan index, archive hygiene, pointer completeness, and policy consistency across AGENTS + docs.

**Architecture:** Treat `docs/README.md` as the canonical pointer registry and enforce active/archive separation for plans. Keep policy constraints synchronized across `AGENTS.md`, `docs/agents/EXECUTION_PROTOCOL.md`, and operational docs through validator-driven updates.

**Tech Stack:** Markdown docs, shell validators in `infra/tests`.

---

### Task 1: Normalize active/archive plan structure and pointer coverage

**Files:**
- Modify: `docs/README.md`
- Modify: `docs/INTENT_ROUTING.md`
- Modify: `docs/plans/archive/2026-02-14-backlog-status-report.md`

**Step 1: Write failing pointer checks**

Run: `bash infra/tests/test_docs_pointers.sh`
Expected: FAIL if active plans are missing pointer entries.

**Step 2: Implement pointer updates**

- Add pointers for each active stage plan.
- Add pointer for backlog status report.
- Keep archived plans out of active pointer section.

**Step 3: Re-run pointer checks**

Run: `bash infra/tests/test_docs_pointers.sh`
Expected: PASS.

**Step 4: Commit**

```bash
git add docs/README.md docs/INTENT_ROUTING.md docs/plans/archive/2026-02-14-backlog-status-report.md
git commit -m "docs: normalize active plan pointers and intent routing"
```

---

### Task 2: Align protocol wording across governance docs

**Files:**
- Modify: `AGENTS.md`
- Modify: `docs/agents/EXECUTION_PROTOCOL.md`
- Modify: `docs/governance/DOCUMENTATION_POLICY.md`
- Modify: `docs/agents/PERMISSIONS.md`
- Modify: `docs/agents/GUARDRAILS.md`

**Step 1: Write failing consistency checks**

Run: `bash infra/tests/test_docs_consistency.sh`
Expected: FAIL if protocol wording drifts from canonical deploy/policy rules after updates.

**Step 2: Implement wording alignment**

Ensure all docs consistently state:
- canonical deploy script path
- lock/request lifecycle requirements
- docs-first/docs-last gates
- changelog policy language

**Step 3: Re-run consistency checks**

Run: `bash infra/tests/test_docs_consistency.sh`
Expected: PASS.

**Step 4: Commit**

```bash
git add AGENTS.md docs/agents/EXECUTION_PROTOCOL.md docs/governance/DOCUMENTATION_POLICY.md docs/agents/PERMISSIONS.md docs/agents/GUARDRAILS.md
git commit -m "docs: align governance protocol language across canonical docs"
```

---

### Task 3: Add explicit archive policy for plans

**Files:**
- Create: `docs/plans/README.md`
- Modify: `docs/README.md`
- Test: pointer validator

**Step 1: Write failing discoverability check (manual/grep)**

Run: `rg -n "active|archive" docs/plans/README.md docs/README.md`
Expected: missing archive policy before file creation.

**Step 2: Implement plan lifecycle policy**

Define in `docs/plans/README.md`:
- when to create active plans
- when to archive plans
- archive entry requirements (status, scope, evidence)
- no transcript/draft fragments in canonical plans

**Step 3: Re-run pointer checks**

Run: `bash infra/tests/test_docs_pointers.sh`
Expected: PASS.

**Step 4: Commit**

```bash
git add docs/plans/README.md docs/README.md
git commit -m "docs(plans): add active/archive lifecycle policy"
```

---

### Task 4: Final docs verification gate

**Step 1: Run all docs validators**

Run:
- `bash infra/tests/test_docs_pointers.sh`
- `bash infra/tests/test_changelog.sh`
- `bash infra/tests/test_intent_routing.sh`
- `bash infra/tests/test_docs_consistency.sh`

Expected: all PASS.

**Step 2: Run infra validator baseline**

Run:
- `bash infra/tests/test_env_utils.sh`
- `bash infra/tests/test_predeploy_check.sh`

Expected: PASS.

**Step 3: Record closure in backlog status report**

Update `docs/plans/archive/2026-02-14-backlog-status-report.md` stage statuses from partial to complete for docs canonization.

**Step 4: Commit**

```bash
git add docs/plans/archive/2026-02-14-backlog-status-report.md
git commit -m "docs: close stage-7 canonization status"
```

## Acceptance Criteria

- Active plans and archived plans are clearly separated and documented.
- Pointer library fully resolves active execution plans.
- Governance docs state consistent canonical policy.
- Validators enforce canonized docs state.

## Assumptions and Defaults

- Archived plans are retained for traceability but are not executable artifacts.
- Active plans must be decision-complete and implementation-ready.
