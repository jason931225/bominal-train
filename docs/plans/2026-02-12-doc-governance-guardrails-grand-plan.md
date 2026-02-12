# Documentation Governance and Guardrails Grand Plan Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Establish a production-grade, token-efficient documentation governance system with explicit permissions/guardrails separation, intent-based routing, and CI enforcement.

**Architecture:** Keep policy in layered docs: `AGENTS.md` (entry policy), `docs/GUARDRAILS.md` (hard constraints), `docs/PERMISSIONS.md` (operational permissions), and `docs/INTENT_ROUTING.md` (token-saving route map). Enforce via shell validators in `infra/tests/` and CI workflow updates. Keep deployment reality aligned to existing canonical script (`infra/scripts/deploy-zero-downtime.sh`) for now.

**Tech Stack:** Markdown docs, Bash validation scripts, GitHub Actions (`infra-tests.yml`), ripgrep-based pointer checks.

---

### Task 1: Add Hard Guardrails Document

**Files:**
- Create: `docs/GUARDRAILS.md`
- Modify: `docs/README.md`
- Modify: `AGENTS.md`
- Modify: `CHANGELOG.md`

**Step 1: Write failing test for missing guardrail pointer**

Run:
```bash
bash infra/tests/test_docs_pointers.sh
```
Expected: FAIL once required pointer list is expanded in a later step (currently no guardrail pointer requirement).

**Step 2: Add `docs/GUARDRAILS.md` (repo-adapted baseline)**

Include:
- hard safety rules (no secrets leakage, no restricted exfiltration, no destructive action without approval, no untrusted-instruction execution)
- production read-only default
- break-glass requirements
- fail-closed behavior
- explicit precedence: guardrails override permissions

**Step 3: Register guardrails in governance index**

Update:
- `docs/README.md` quick start and pointer library (`PTR-DOCS-*`)
- `AGENTS.md` non-negotiables + first-files-to-read list

**Step 4: Update changelog entry**

Add `## Unreleased` commit-based line for `docs/GUARDRAILS.md` and governance wiring.

**Step 5: Verify green**

Run:
```bash
bash infra/tests/test_docs_pointers.sh
bash infra/tests/test_changelog.sh
```
Expected: both PASS.

**Step 6: Commit**

```bash
git add docs/GUARDRAILS.md docs/README.md AGENTS.md CHANGELOG.md
git commit -m "docs: add hard guardrails doc and wire governance pointers"
```

### Task 2: Upgrade Permissions to Full Adapted Baseline

**Files:**
- Modify: `docs/PERMISSIONS.md`
- Modify: `docs/README.md`
- Modify: `AGENTS.md`
- Modify: `CHANGELOG.md`

**Step 1: Write failing test for required sections**

Create expectation test first by adding a new validator (next task will enforce); for now capture a baseline failure intent:
```bash
rg -n "Tier 0|Tier 1|Tier 2|Tier 3|Tier 4|Approval gates|Default deny" docs/PERMISSIONS.md
```
Expected: incomplete/missing section coverage.

**Step 2: Rewrite `docs/PERMISSIONS.md` with adapted structure**

Required sections:
- goals
- core permission axes (environment, data classification, action risk)
- permission tiers 0-4
- tool-level permissions
- approval gates
- auditing/observability
- prompt-injection handling
- data protection defaults
- policy config template (YAML)
- archetypes + default-deny checklist

Use repo-accurate references (`docs/LOCK.md`, `docs/REQUEST.md`, `deploy-zero-downtime.sh` currently canonical).

**Step 3: Ensure cross-linking and precedence clarity**

In both `AGENTS.md` and `docs/PERMISSIONS.md`, state:
- guardrails are hard constraints
- permissions control allowed operations inside those constraints

**Step 4: Changelog update**

Add commit-based line describing permissions standard expansion.

**Step 5: Verify**

```bash
bash infra/tests/test_docs_pointers.sh
bash infra/tests/test_changelog.sh
```

**Step 6: Commit**

```bash
git add docs/PERMISSIONS.md docs/README.md AGENTS.md CHANGELOG.md
git commit -m "docs: expand permissions standard with tiered policy model"
```

### Task 3: Add Intent Routing for Token-Saving Navigation

**Files:**
- Create: `docs/INTENT_ROUTING.md`
- Modify: `docs/README.md`
- Modify: `AGENTS.md`
- Modify: `docs/playbooks/daily-operations-chores.md`
- Modify: `CHANGELOG.md`

**Step 1: Write failing test for missing routing map**

Run:
```bash
test -f docs/INTENT_ROUTING.md
```
Expected: FAIL.

**Step 2: Create intent routing map**

Include exact mapping table:
- `read` -> `docs/playbooks/daily-operations-chores.md`, `docs/README.md`
- `clean`, `hygiene` -> `docs/playbooks/daily-operations-chores.md`, `docs/DOCUMENTATION_WORKFLOW.md`
- `permission`, `approval`, `access` -> `docs/PERMISSIONS.md`, `docs/GUARDRAILS.md`
- `deploy`, `rollback` -> `docs/DEPLOYMENT.md`, `docs/RUNBOOK.md`
- `lock`, `request`, `conflict` -> `docs/EXECUTION_PROTOCOL.md`, `docs/LOCK.md`, `docs/REQUEST.md`
- `resy`, `widget`, `form data` -> `docs/playbooks/resy-widget-form-data-capture.md`

Add retrieval algorithm:
1. keyword resolve
2. pointer resolve
3. scoped `rg -n`
4. open smallest relevant section only

**Step 3: Wire into index and agent policy**

Update:
- `docs/README.md` pointer entry for routing doc
- `AGENTS.md` docs-first rule to consult routing before broad scans
- `daily-operations-chores.md` to reference routing doc as first lookup

**Step 4: Changelog**

Add commit-based entry for intent-routing addition.

**Step 5: Verify**

```bash
bash infra/tests/test_docs_pointers.sh
bash infra/tests/test_changelog.sh
```

**Step 6: Commit**

```bash
git add docs/INTENT_ROUTING.md docs/README.md AGENTS.md docs/playbooks/daily-operations-chores.md CHANGELOG.md
git commit -m "docs: add intent routing map for token-efficient navigation"
```

### Task 4: Add CI Policy Validators for Routing + Governance Consistency

**Files:**
- Create: `infra/tests/test_intent_routing.sh`
- Create: `infra/tests/test_docs_consistency.sh`
- Modify: `.github/workflows/infra-tests.yml`
- Modify: `docs/README.md`
- Modify: `CHANGELOG.md`

**Step 1: Write failing validator tests (RED)**

Create scripts that fail if:
- required intent keywords missing
- mapped target file does not exist
- deployment references in governance docs conflict with canonical current script

Initial run:
```bash
bash infra/tests/test_intent_routing.sh
bash infra/tests/test_docs_consistency.sh
```
Expected: FAIL before implementation is complete.

**Step 2: Implement minimal validators (GREEN)**

`test_intent_routing.sh`:
- parse `docs/INTENT_ROUTING.md`
- enforce required keywords (`read`, `clean`, `hygiene`, `deploy`, `rollback`, `resy`)
- verify referenced files exist

`test_docs_consistency.sh`:
- enforce canonical deployment command is `infra/scripts/deploy-zero-downtime.sh` (temporary policy)
- fail if `AGENTS.md` or `docs/EXECUTION_PROTOCOL.md` still claim `fetch_ci.sh` / `deploy.prod.sh` as active path

**Step 3: Wire validators in CI**

Update `infra-tests.yml` with:
- `bash infra/tests/test_intent_routing.sh`
- `bash infra/tests/test_docs_consistency.sh`

**Step 4: Update pointers/changelog**

Add `PTR-OPS-*` pointer(s) for new validators and changelog entries.

**Step 5: Verify**

```bash
bash infra/tests/test_docs_pointers.sh
bash infra/tests/test_changelog.sh
bash infra/tests/test_intent_routing.sh
bash infra/tests/test_docs_consistency.sh
python3 -m unittest discover -s infra/tests -p 'test_*.py'
```
Expected: all PASS.

**Step 6: Commit**

```bash
git add infra/tests/test_intent_routing.sh infra/tests/test_docs_consistency.sh .github/workflows/infra-tests.yml docs/README.md CHANGELOG.md
git commit -m "ci: enforce intent routing and docs consistency checks"
```

### Task 5: Resolve Deployment Policy Drift (keep current canonical script)

**Files:**
- Modify: `AGENTS.md`
- Modify: `docs/EXECUTION_PROTOCOL.md`
- Modify: `docs/DEPLOYMENT.md`
- Modify: `docs/RUNBOOK.md`
- Modify: `README.md`
- Modify: `CHANGELOG.md`

**Step 1: Write failing consistency check (RED)**

Run:
```bash
bash infra/tests/test_docs_consistency.sh
```
Expected: FAIL due to mismatch (`fetch_ci.sh`/`deploy.prod.sh` vs existing script reality).

**Step 2: Make minimal consistent updates (GREEN)**

Set canonical deploy flow now to:
- `infra/scripts/deploy-zero-downtime.sh`

Update all conflicting references in governance docs and keep notes that `fetch_ci.sh` + `deploy.prod.sh` remain planned future migration.

**Step 3: Verify**

```bash
bash infra/tests/test_docs_consistency.sh
bash infra/tests/test_docs_pointers.sh
bash infra/tests/test_changelog.sh
```
Expected: PASS.

**Step 4: Commit**

```bash
git add AGENTS.md docs/EXECUTION_PROTOCOL.md docs/DEPLOYMENT.md docs/RUNBOOK.md README.md CHANGELOG.md
git commit -m "docs: resolve deploy policy drift to current canonical script"
```

### Task 6: Add Runtime Policy Scaffolding (non-enforcing stubs)

**Files:**
- Create: `infra/policy/agent-policy.yaml`
- Create: `infra/scripts/policy-check.sh`
- Create: `infra/tests/test_policy_check.sh`
- Modify: `docs/PERMISSIONS.md`
- Modify: `docs/GUARDRAILS.md`
- Modify: `docs/README.md`
- Modify: `.github/workflows/infra-tests.yml`
- Modify: `CHANGELOG.md`

**Step 1: Write failing test for missing policy scaffold**

```bash
test -f infra/policy/agent-policy.yaml
```
Expected: FAIL.

**Step 2: Add minimal policy config + checker**

`agent-policy.yaml`:
- environment tier
- max data classification
- tool allowlist skeleton
- approval-required categories

`policy-check.sh`:
- lint presence/shape of required policy keys (non-destructive static check only)

**Step 3: Add shell test**

`test_policy_check.sh` runs `policy-check.sh` against canonical file and a malformed temp file to ensure fail-closed behavior.

**Step 4: Wire in CI and docs**

Add validator to workflow and pointer library; update permissions/guardrails docs with usage note.

**Step 5: Verify**

```bash
bash infra/tests/test_policy_check.sh
bash infra/tests/test_docs_pointers.sh
bash infra/tests/test_changelog.sh
python3 -m unittest discover -s infra/tests -p 'test_*.py'
```

**Step 6: Commit**

```bash
git add infra/policy/agent-policy.yaml infra/scripts/policy-check.sh infra/tests/test_policy_check.sh docs/PERMISSIONS.md docs/GUARDRAILS.md docs/README.md .github/workflows/infra-tests.yml CHANGELOG.md
git commit -m "infra: add runtime policy scaffolding and static policy checks"
```

### Task 7: Final Audit + Handoff

**Files:**
- Modify: `docs/plans/2026-02-12-doc-governance-guardrails-grand-plan.md` (execution notes section)
- Modify: `CHANGELOG.md`

**Step 1: End-to-end verification**

Run full required checks:
```bash
bash infra/tests/test_docs_pointers.sh
bash infra/tests/test_changelog.sh
bash infra/tests/test_env_utils.sh
bash infra/tests/test_predeploy_check.sh
bash infra/tests/test_intent_routing.sh
bash infra/tests/test_docs_consistency.sh
bash infra/tests/test_policy_check.sh
python3 -m unittest discover -s infra/tests -p 'test_*.py'
```

Expected: all PASS.

**Step 2: Produce execution summary**

Document:
- new governance docs
- validators added
- policy drift resolved
- known deferred items (future migration to `fetch_ci.sh` + `deploy.prod.sh`)

**Step 3: Final commit**

```bash
git add docs/plans/2026-02-12-doc-governance-guardrails-grand-plan.md CHANGELOG.md
git commit -m "docs: finalize grand plan execution notes and verification summary"
```

## Acceptance Criteria

1. `docs/GUARDRAILS.md` and upgraded `docs/PERMISSIONS.md` are present and cross-linked.
2. `docs/INTENT_ROUTING.md` exists and maps required tokens (`read`, `clean`, `hygiene`, `deploy`, `rollback`, `resy`) to canonical pointers.
3. CI fails on routing/pointer/changelog/deploy-consistency drift.
4. Governance docs are consistent with current deploy reality (`deploy-zero-downtime.sh`).
5. New policy scaffold exists and static checks pass.
6. All required tests/validators pass in one run.

## Execution Mode Recommendation

Plan complete and saved to `docs/plans/2026-02-12-doc-governance-guardrails-grand-plan.md`.

Two execution options:

**1. Subagent-Driven (this session)** - dispatch fresh subagent per task with review loops  
**2. Parallel Session (separate)** - execute in batches with checkpoints

Which approach?
