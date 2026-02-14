# Stage 6 Safe Deprecation Cleanup Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

Status: Completed in implementation (2026-02-14).

**Goal:** Remove deprecated runtime artifacts safely, with explicit inventory, compatibility window, and verification that no callers depend on deprecated paths.

**Architecture:** Use an inventory-first process. Mark deprecations with owner, dependency scan, and removal criteria. Add compatibility notices where needed before deleting artifacts. Validate by searching for references and running infra/docs checks.

**Tech Stack:** Bash, ripgrep, docs governance scripts.

---

### Task 1: Create deprecation inventory and ownership record

**Files:**
- Create: `docs/deprecations/2026-02-14-inventory.md`
- Modify: `docs/README.md`

**Step 1: Build inventory from repo scan**

Run: `rg --files | rg "deprecated|legacy|old|backup"`
Expected: includes at least `infra/docker-compose.deploy.yml.deprecated`.

**Step 2: Write inventory doc (failing check by absence first)**

Document for each candidate:
- path
- reason deprecated
- active replacement
- known callers
- removal gate criteria

**Step 3: Register inventory pointer**

Add pointer entry to `docs/README.md` under docs or plan section.

**Step 4: Commit**

```bash
git add docs/deprecations/2026-02-14-inventory.md docs/README.md
git commit -m "docs: add deprecation inventory and ownership metadata"
```

---

### Task 2: Add compatibility warnings before deletion

**Files:**
- Modify: `README.md`
- Modify: `docs/DEPLOYMENT.md`
- Modify: `docs/RUNBOOK.md`

**Step 1: Write failing expectation checks (grep-based)**

Run: `rg -n "docker-compose\.deploy\.yml\.deprecated" README.md docs/DEPLOYMENT.md docs/RUNBOOK.md`
Expected: no mention before changes (RED for docs requirement).

**Step 2: Implement compatibility notices**

Add short transition note:
- artifact is deprecated
- canonical replacement path
- planned removal condition/date

**Step 3: Verify notices exist**

Run: `rg -n "deprecated" README.md docs/DEPLOYMENT.md docs/RUNBOOK.md`
Expected: matching entries present.

**Step 4: Commit**

```bash
git add README.md docs/DEPLOYMENT.md docs/RUNBOOK.md
git commit -m "docs: add compatibility notices for deprecated deploy artifacts"
```

---

### Task 3: Remove deprecated artifact after dependency scan

**Files:**
- Delete: `infra/docker-compose.deploy.yml.deprecated`
- Modify: `infra/tests/test_docs_consistency.sh` (if needed)
- Modify: `docs/deprecations/2026-02-14-inventory.md`

**Step 1: Write failing dependency guard test**

Create/extend `infra/tests/test_deprecation_references.sh` to fail if deprecated artifact is still referenced as active.

**Step 2: Run RED check**

Run: `bash infra/tests/test_deprecation_references.sh`
Expected: FAIL before cleanup is complete.

**Step 3: Remove artifact and update inventory state**

- Delete deprecated compose file.
- Mark inventory entry as removed with commit/date.

**Step 4: Run GREEN checks**

Run:
- `bash infra/tests/test_deprecation_references.sh`
- `rg -n "docker-compose\.deploy\.yml\.deprecated" -S`

Expected: PASS and no active references.

**Step 5: Commit**

```bash
git add infra/tests/test_deprecation_references.sh docs/deprecations/2026-02-14-inventory.md
git rm infra/docker-compose.deploy.yml.deprecated
git commit -m "chore(infra): remove deprecated compose artifact after guarded scan"
```

---

### Task 4: Final verification and closure

**Step 1: Run infra + docs checks**

Run:
- `bash infra/tests/test_docs_pointers.sh`
- `bash infra/tests/test_changelog.sh`
- `bash infra/tests/test_intent_routing.sh`
- `bash infra/tests/test_docs_consistency.sh`

Expected: all PASS.

**Step 2: Update deprecation inventory status**

Mark each item as:
- open
- compatibility window
- removed

**Step 3: Commit closure note**

```bash
git add docs/deprecations/2026-02-14-inventory.md
git commit -m "docs: close stage-6 deprecation cleanup status"
```

## Acceptance Criteria

- All deprecated artifacts are inventoried with owner and replacement path.
- No removed artifact has unresolved active callers.
- Transition notices are documented before deletion.
- Cleanup is reproducible and enforced by tests/guards.

## Assumptions and Defaults

- Deprecation cleanup is conservative: inventory + compatibility note before deletion.
- If any unknown caller remains, artifact stays in compatibility window and is not deleted.
