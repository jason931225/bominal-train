# Backlog Status Report (2026-02-14)

## Scope

This report covers active work status across:
- `docs/todo/backend-production-readiness.md`
- `docs/plans/active/2026-02-11-bominal-grand-restructure-plan.md`
- archived plans under `docs/plans/archive/`

## Executive Status

- Backend production-readiness hardening: **implemented in code**.
- Pub/Sub deploy agent rollout plan: **implemented and archived**.
- Admin ops + local checks + train retry observability: **implemented and archived**.
- Grand restructure stages 2-7: **implemented in code/docs with stage-level verification**.

## Detailed Status

### A) Backend production-readiness backlog (`docs/todo/backend-production-readiness.md`)

Status: **Completed in implementation, pending archival hygiene**.

Implemented evidence:
- Worker shutdown cancellation hardening:
  - `api/app/worker.py`
  - `api/tests/test_worker_shutdown_recovery.py`
  - `api/tests/test_worker_heartbeat.py`
- Auth uniqueness race handling (`IntegrityError` -> deterministic `409`):
  - `api/app/http/routes/auth.py`
  - `api/tests/test_auth_flow.py`
- Proxy-aware auth rate-limit keying:
  - `api/app/http/deps.py`
  - `api/tests/test_api_access_control.py`
- Changelog references:
  - `CHANGELOG.md` entries `[220d2c6]`, `[b05ca4b]`, `[b231d4c]`

### B) Grand restructure Stage 2: worker split + queue contracts

Status: **Completed in implementation**.

Implemented:
- Split worker entrypoints exist:
  - `api/app/worker_train.py`
  - `api/app/worker_restaurant.py`
- Compose wiring includes restaurant worker:
  - `infra/docker-compose.yml`
  - `infra/docker-compose.prod.yml`
- Shared queue-domain constants and explicit queue names:
  - `api/app/core/queue_domains.py`
  - `api/app/worker.py`
  - `api/app/worker_restaurant.py`
  - `api/app/modules/train/queue.py`
  - `api/app/services/email_queue.py`
  - `api/app/modules/restaurant/queue.py`
- Contract-level regression tests for queue/worker domain isolation:
  - `api/tests/test_queue_domains.py`
  - `api/tests/test_worker_settings.py`

### C) Grand restructure Stage 3: restaurant partial exposure

Status: **Completed in implementation**.

Implemented:
- Extended `/api/modules` contract with capability metadata:
  - `api/app/schemas/module.py`
  - `api/app/http/routes/modules.py`
- Restaurant capability policy registry:
  - `api/app/modules/restaurant/capabilities.py`
- API regression coverage for module capability output:
  - `api/tests/test_modules_api.py`
- Web contract + rendering updates:
  - `web/lib/types.ts`
  - `web/app/dashboard/page.tsx`
  - `web/components/module-tile.tsx`

### D) Grand restructure Stage 4: restaurant policy enforcement

Status: **Completed in implementation**.

Implemented:
- Restaurant policy helpers/types:
  - `api/app/modules/restaurant/types.py`
  - `api/app/modules/restaurant/policy.py`
- Redis payment lease helpers:
  - `api/app/modules/restaurant/lease.py`
- Restaurant worker policy scaffold integration:
  - `api/app/modules/restaurant/worker.py`
- Configuration surface for policy tuning:
  - `api/app/core/config.py`
  - `infra/env/dev/api.env`
  - `infra/env/prod/api.env.example`
- Regression coverage:
  - `api/tests/test_restaurant_policy.py`
  - `api/tests/test_restaurant_worker_policy_flow.py`
  - `api/tests/test_restaurant_policy_config.py`

### E) Grand restructure Stage 5: infra deploy hardening

Status: **Completed in implementation**.

Implemented:
- Rollback paths and digest-aware history tracking in `infra/scripts/deploy.sh`.
- Pub/Sub deploy agent pulls and executes deploy script.
- Script-level deploy lock contract in deploy script (`flock` with deterministic fallback lock).
- Running-container detection with explicit first-deploy bootstrap path vs rolling-update path.
- Resource/swap preflight gate integrated before pull/deploy mutation.
- Smoke failure auto-rollback trigger path controlled by `AUTO_ROLLBACK_ON_SMOKE_FAILURE`.
- Stage 5 regression tests:
  - `infra/tests/test_deploy_zero_downtime_lock.sh`
  - `infra/tests/test_deploy_zero_downtime_running_container_detection.sh`
  - `infra/tests/test_deploy_zero_downtime_preflight.sh`
  - `infra/tests/test_deploy_zero_downtime_smoke_rollback.sh`

### F) Grand restructure Stage 6: safe deprecation cleanup

Status: **Completed in implementation**.

Implemented:
- Deprecation inventory with owner/replacement/removal gates:
  - `docs/deprecations/2026-02-14-inventory.md`
- Compatibility notices for deprecated deploy artifact:
  - `README.md`
  - `docs/DEPLOYMENT.md`
  - `docs/RUNBOOK.md`
- Deprecated artifact removal:
  - removed `infra/docker-compose.deploy.yml.deprecated`
- Guarded regression check for active references:
  - `infra/tests/test_deprecation_references.sh`

### G) Grand restructure Stage 7: docs canonization

Status: **Completed in implementation**.

Implemented:
- Canonical policy docs and validators are in place:
  - `docs/README.md`
  - `docs/EXECUTION_PROTOCOL.md`
  - `docs/PERMISSIONS.md`
  - `docs/GUARDRAILS.md`
  - `docs/INTENT_ROUTING.md`
  - `infra/tests/test_docs_pointers.sh`
  - `infra/tests/test_docs_consistency.sh`
- Active/archive planning lifecycle canonized:
  - `docs/plans/README.md`
  - `docs/plans/active/*`
  - `docs/plans/archive/*`
- Executable stage plans for stages 2-7 exist and are status-tracked under `docs/plans/active/`.

## Active Execution Plans (Created in this workstream)

- `docs/plans/active/2026-02-14-stage2-worker-split-queue-contracts.md`
- `docs/plans/active/2026-02-14-stage3-restaurant-partial-exposure.md`
- `docs/plans/active/2026-02-14-stage4-restaurant-policy-enforcement.md`
- `docs/plans/active/2026-02-14-stage5-infra-deploy-hardening.md`
- `docs/plans/active/2026-02-14-stage6-safe-deprecation-cleanup.md`
- `docs/plans/active/2026-02-14-stage7-docs-canonization.md`

## Assumptions

- “Implemented” means present in codebase and covered by at least targeted tests.
- “Not started” means no production-facing contract implementation beyond placeholders/scaffolding.
- This report is an execution snapshot; update it whenever stage status changes.
