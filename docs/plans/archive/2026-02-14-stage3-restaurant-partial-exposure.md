> Archive note (2026-02-14): Completed and moved from `docs/plans/active/` during Stage 8 closure.
> See `docs/plans/archive/2026-02-14-program-closure-report.md` for final status.

# Stage 3 Restaurant Partial Exposure Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Expose restaurant module capabilities safely through `/api/modules` so UI and clients can differentiate implemented operations from coming-soon placeholders.

**Architecture:** Extend the module contract with machine-readable capability flags. Keep restaurant module in controlled rollout mode (`coming_soon=true`) while explicitly advertising only safe, implemented operations. Use capability-first rendering in web dashboard without enabling unavailable actions.

**Tech Stack:** FastAPI, Pydantic, Next.js App Router, TypeScript, pytest.

---

### Task 1: Extend module API schema with capabilities

**Files:**
- Modify: `api/app/schemas/module.py`
- Modify: `api/app/http/routes/modules.py`
- Test: `api/tests/test_modules_api.py`

**Step 1: Write failing API contract tests**

```python
async def test_modules_response_includes_capabilities(client):
    ...

async def test_restaurant_capabilities_are_safe_subset(client):
    ...
```

**Step 2: Run tests to verify RED**

Run: `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_modules_api.py`
Expected: FAIL because `capabilities` is missing from `ModuleOut`.

**Step 3: Implement minimal schema + route changes**

- Add fields to `ModuleOut`:
  - `enabled: bool`
  - `capabilities: list[str]`
- Return explicit capability lists from `/api/modules` for all modules.

**Step 4: Run tests to verify GREEN**

Run: `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_modules_api.py`
Expected: PASS.

**Step 5: Commit**

```bash
git add api/app/schemas/module.py api/app/http/routes/modules.py api/tests/test_modules_api.py
git commit -m "feat(modules): expose capability flags in module list API"
```

---

### Task 2: Centralize restaurant capability policy

**Files:**
- Create: `api/app/modules/restaurant/capabilities.py`
- Modify: `api/app/http/routes/modules.py`
- Test: `api/tests/test_modules_api.py`

**Step 1: Write failing policy tests**

```python
def test_restaurant_capability_registry_exposes_only_implemented_ops():
    ...
```

**Step 2: Run tests to verify RED**

Run: `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_modules_api.py -k capability_registry`
Expected: FAIL because capability registry file/constants do not exist.

**Step 3: Implement minimal capability registry**

Define constants in `capabilities.py`:
- `RESTAURANT_CAPABILITIES_EXPOSED`
- `RESTAURANT_CAPABILITIES_COMING_SOON`

Use the exposed list in `/api/modules` response.

**Step 4: Run tests to verify GREEN**

Run: `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_modules_api.py -k capability_registry`
Expected: PASS.

**Step 5: Commit**

```bash
git add api/app/modules/restaurant/capabilities.py api/app/http/routes/modules.py api/tests/test_modules_api.py
git commit -m "feat(restaurant): centralize capability exposure policy"
```

---

### Task 3: Update web module typing and dashboard rendering

**Files:**
- Modify: `web/lib/types.ts`
- Modify: `web/app/dashboard/page.tsx`
- Modify: `web/components/module-tile.tsx`
- Test: `web` typecheck

**Step 1: Write failing frontend expectations (type-level + render behavior)**

Implement a lightweight assertion test or compile-time fixture to require:
- `BominalModule.enabled`
- `BominalModule.capabilities`

If no test harness exists, start from typecheck RED by introducing usage first.

**Step 2: Run typecheck to verify RED**

Run: `docker compose -f infra/docker-compose.yml run --rm web npx tsc --noEmit`
Expected: FAIL until types and components are updated.

**Step 3: Implement minimal frontend adaptation**

- Extend `BominalModule` type with `enabled` and `capabilities`.
- Preserve current tile behavior for `coming_soon` modules.
- Render capability badges from server response (no clickable actions for unavailable features).

**Step 4: Run typecheck to verify GREEN**

Run: `docker compose -f infra/docker-compose.yml run --rm web npx tsc --noEmit`
Expected: PASS.

**Step 5: Commit**

```bash
git add web/lib/types.ts web/app/dashboard/page.tsx web/components/module-tile.tsx
git commit -m "feat(web): render module capabilities from modules API"
```

---

### Task 4: Documentation + end-to-end verification

**Files:**
- Modify: `docs/humans/engineering/ARCHITECTURE.md`
- Modify: `README.md`

**Step 1: Document capability contract**
- Add `/api/modules` response field documentation.
- Clarify that restaurant remains controlled exposure with explicit capability flags.

**Step 2: Run API + web verification**

Run:
- `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_modules_api.py`
- `docker compose -f infra/docker-compose.yml run --rm web npx tsc --noEmit`

Expected: both PASS.

**Step 3: Run baseline validation**

Run:
- `bash infra/tests/test_docs_pointers.sh`
- `bash infra/tests/test_changelog.sh`
- `bash infra/tests/test_intent_routing.sh`
- `bash infra/tests/test_docs_consistency.sh`

Expected: all PASS.

**Step 4: Commit**

```bash
git add docs/humans/engineering/ARCHITECTURE.md README.md
git commit -m "docs: define module capability exposure contract"
```

## Acceptance Criteria

- `/api/modules` returns `enabled` and `capabilities` for each module.
- Restaurant capabilities are explicitly scoped to implemented operations only.
- Web dashboard renders capability metadata without exposing unavailable actions.
- Docs describe the capability contract and rollout intent.

## Assumptions and Defaults

- Capability identifiers are stable strings owned by backend.
- `coming_soon` remains `true` for restaurant until Stage 4 policy and execution contracts are complete.
