# Stage 5 Infra Deploy Hardening Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

Status: Completed in implementation (2026-02-14).

**Goal:** Harden `infra/scripts/deploy-zero-downtime.sh` with explicit deploy lock, running-container detection, resource/swap preflight, and deterministic smoke+rollback behavior.

**Architecture:** Keep `deploy-zero-downtime.sh` as canonical deploy entrypoint. Add preflight and locking before any image pull/deploy mutation. Ensure post-deploy smoke checks have a clear failure path that triggers rollback (configurable) and preserves traceability in deployment history.

**Tech Stack:** Bash, Docker Compose, system utilities (`flock`, `free`, `swapon`), shell tests in `infra/tests`.

---

### Task 1: Add deploy lock contract to deploy script

**Files:**
- Modify: `infra/scripts/deploy-zero-downtime.sh`
- Create: `infra/tests/test_deploy_zero_downtime_lock.sh`

**Step 1: Write failing lock tests**

Test expectations:
- second concurrent invocation exits with clear lock message and non-zero code
- first invocation retains lock until completion

**Step 2: Run tests to verify RED**

Run: `bash infra/tests/test_deploy_zero_downtime_lock.sh`
Expected: FAIL because script does not enforce script-level lock.

**Step 3: Implement minimal lock behavior**

- Add lock file (for example `/tmp/bominal-deploy.lock`) and `flock -n` acquisition near script start.
- Print deterministic message on lock contention.

**Step 4: Run tests to verify GREEN**

Run: `bash infra/tests/test_deploy_zero_downtime_lock.sh`
Expected: PASS.

**Step 5: Commit**

```bash
git add infra/scripts/deploy-zero-downtime.sh infra/tests/test_deploy_zero_downtime_lock.sh
git commit -m "infra(deploy): enforce single-run deploy lock"
```

---

### Task 2: Add running-container detection and first-deploy path

**Files:**
- Modify: `infra/scripts/deploy-zero-downtime.sh`
- Create: `infra/tests/test_deploy_zero_downtime_running_container_detection.sh`

**Step 1: Write failing detection tests**

Test expectations:
- first deploy with no running containers follows bootstrap-safe path
- subsequent deploy with running stack follows rolling-update path

**Step 2: Run tests to verify RED**

Run: `bash infra/tests/test_deploy_zero_downtime_running_container_detection.sh`
Expected: FAIL before detection branch exists.

**Step 3: Implement minimal detection logic**

- Detect running stack via `docker compose ... ps` or container filters.
- Use explicit branch for first deploy vs update deploy.

**Step 4: Run tests to verify GREEN**

Run: `bash infra/tests/test_deploy_zero_downtime_running_container_detection.sh`
Expected: PASS.

**Step 5: Commit**

```bash
git add infra/scripts/deploy-zero-downtime.sh infra/tests/test_deploy_zero_downtime_running_container_detection.sh
git commit -m "infra(deploy): add running-container detection path"
```

---

### Task 3: Add resource/swap preflight gate

**Files:**
- Modify: `infra/scripts/deploy-zero-downtime.sh`
- Modify: `infra/scripts/predeploy-check.sh`
- Create: `infra/tests/test_deploy_zero_downtime_preflight.sh`

**Step 1: Write failing preflight tests**

Test expectations:
- deploy exits before pull/deploy when memory/swap thresholds are below minimum
- deploy proceeds when thresholds pass

**Step 2: Run tests to verify RED**

Run: `bash infra/tests/test_deploy_zero_downtime_preflight.sh`
Expected: FAIL before preflight gate is integrated.

**Step 3: Implement minimal preflight integration**

- Add preflight function in deploy script (or invoke `predeploy-check.sh --skip-smoke-tests` in strict mode).
- Enforce minimum memory+swap gate on e2-micro profile.

**Step 4: Run tests to verify GREEN**

Run: `bash infra/tests/test_deploy_zero_downtime_preflight.sh`
Expected: PASS.

**Step 5: Commit**

```bash
git add infra/scripts/deploy-zero-downtime.sh infra/scripts/predeploy-check.sh infra/tests/test_deploy_zero_downtime_preflight.sh
git commit -m "infra(deploy): gate deploy on resource and swap preflight"
```

---

### Task 4: Add post-deploy smoke + rollback trigger path

**Files:**
- Modify: `infra/scripts/deploy-zero-downtime.sh`
- Create: `infra/tests/test_deploy_zero_downtime_smoke_rollback.sh`
- Modify: `docs/DEPLOYMENT.md`
- Modify: `docs/RUNBOOK.md`

**Step 1: Write failing smoke/rollback tests**

Test expectations:
- smoke failure triggers rollback routine when auto-rollback is enabled
- smoke success does not trigger rollback

**Step 2: Run tests to verify RED**

Run: `bash infra/tests/test_deploy_zero_downtime_smoke_rollback.sh`
Expected: FAIL before rollback trigger branch is explicit.

**Step 3: Implement minimal smoke rollback behavior**

- Add `AUTO_ROLLBACK_ON_SMOKE_FAILURE` (default `true` in prod context).
- On failed smoke verification: run rollback, return non-zero, emit deterministic logs.

**Step 4: Run tests to verify GREEN**

Run: `bash infra/tests/test_deploy_zero_downtime_smoke_rollback.sh`
Expected: PASS.

**Step 5: Commit**

```bash
git add infra/scripts/deploy-zero-downtime.sh infra/tests/test_deploy_zero_downtime_smoke_rollback.sh docs/DEPLOYMENT.md docs/RUNBOOK.md
git commit -m "infra/deploy: add smoke-failure rollback trigger and docs"
```

---

### Task 5: Full infra verification gate

**Step 1: Run deploy-script test suite**

Run:
- `bash infra/tests/test_deploy_zero_downtime_lock.sh`
- `bash infra/tests/test_deploy_zero_downtime_running_container_detection.sh`
- `bash infra/tests/test_deploy_zero_downtime_preflight.sh`
- `bash infra/tests/test_deploy_zero_downtime_smoke_rollback.sh`

Expected: all PASS.

**Step 2: Run baseline infra validators**

Run:
- `bash infra/tests/test_env_utils.sh`
- `bash infra/tests/test_predeploy_check.sh`
- `bash infra/tests/test_docs_consistency.sh`

Expected: all PASS.

**Step 3: Run docs governance checks**

Run:
- `bash infra/tests/test_docs_pointers.sh`
- `bash infra/tests/test_changelog.sh`
- `bash infra/tests/test_intent_routing.sh`

Expected: all PASS.

## Acceptance Criteria

- Deploy script itself enforces single-run lock.
- Script has explicit first-deploy vs running-stack behavior.
- Resource/swap preflight blocks unsafe deploy attempts.
- Smoke failure path can trigger deterministic rollback.
- Deployment docs and runbook match implemented behavior exactly.

## Assumptions and Defaults

- Canonical deploy entrypoint remains `infra/scripts/deploy-zero-downtime.sh`.
- Production profile uses cautious defaults (`AUTO_ROLLBACK_ON_SMOKE_FAILURE=true`).
