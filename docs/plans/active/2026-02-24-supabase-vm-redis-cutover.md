# 2026-02-24 Supabase + VM Redis Cutover

## Goal

Complete production cutover to:

- Supabase Postgres (all app tables)
- Supabase Auth (API identity source)
- VM-hosted Redis (no Upstash dependency)

While payment remains hard-disabled until separate PCI re-enable work.

## Scope

In scope:

1. Payment hard-disable enforcement (backend + API capability exposure).
2. Supabase-first production config and deployment guards.
3. VM Redis runtime alignment.
4. Polling persistence reduction to state-transition-only writes.
5. Documentation + changelog parity with implemented behavior.

Out of scope:

1. Payment re-enable / PCI runtime activation.
2. Upstash migration for any path.
3. Historical production-data import (fresh start approved).

## Architecture Decisions

1. Production DB moves to Supabase Postgres via `DATABASE_URL` / `SYNC_DATABASE_URL`.
2. Production auth mode is `AUTH_MODE=supabase`.
3. Production Redis remains VM-managed and used for:
   - ARQ queues
   - rate limiting
   - worker heartbeat
   - lease keys
4. Payment is globally disabled via `PAYMENT_ENABLED=false`:
   - `/api/wallet/*` blocked
   - `/api/train/tasks/{id}/pay` blocked
   - train `auto_pay` coerced to false
   - `wallet.payment_card` capability hidden from `/api/modules`

## Implementation Plan

### Phase 1 — Save plan and enforce payment-off gates

Files:

- `api/app/core/config.py`
- `api/app/http/deps.py`
- `api/app/http/routes/wallet.py`
- `api/app/modules/train/router.py`
- `api/app/modules/train/service.py`
- `api/app/modules/train/worker.py`
- `api/app/main.py`
- `api/app/main_gateway.py`
- `api/app/http/routes/modules.py`
- `infra/env/dev/api.env`
- `infra/env/prod/api.env.example`

Changes:

1. Introduce and validate `PAYMENT_ENABLED`.
2. Add dependency gate for payment-only routes.
3. Avoid mounting wallet router when payment is disabled.
4. Ensure train runtime never executes payment flow while disabled.
5. Keep CDE/Upstash restriction active only when payment-enabled.

### Phase 2 — Supabase + VM Redis deploy readiness

Files:

- `infra/scripts/predeploy-check.sh`
- `infra/docker-compose.prod.yml`
- `infra/scripts/deploy.sh`
- `infra/scripts/quick-restart.sh`
- `README.md`
- `docs/DEPLOYMENT.md`
- `docs/RUNBOOK.md`
- `docs/ARCHITECTURE.md`
- `docs/SECURITY.md`

Changes:

1. Remove stale production-local-postgres assumptions.
2. Enforce prod env contracts for Supabase endpoints + credentials.
3. Ensure deploy/startup scripts no longer require local postgres service in prod.
4. Document VM Redis as canonical Redis runtime.

### Phase 3 — Polling persistence/storage reduction

Files:

- `api/app/modules/train/worker.py`
- `api/app/modules/train/service.py`
- `web/components/train/train-dashboard.tsx`
- `web/components/train/train-task-detail.tsx`

Changes:

1. Persist only meaningful state/failure/success events.
2. Eliminate retry-noise persistence from tight polling loops.
3. Keep UI focused on state changes; avoid poll-progress noise.

### Phase 4 — Tests and CI verification

Files:

- `api/tests/test_security_config.py`
- `api/tests/test_modules_api.py`
- `api/tests/test_api_access_control.py`
- `api/tests/test_train_tasks.py`
- `api/tests/test_train_service_payment_units.py`
- `infra/tests/test_predeploy_check.sh`

Add/update tests:

1. Payment-disabled route gating (`403`).
2. Module capability omission when payment disabled.
3. Supabase-required predeploy env checks.
4. Polling persistence behavior regression.

## Verification Commands

Run after implementation:

```bash
cd api && ./.venv/bin/pytest -q
```

```bash
bash infra/tests/test_predeploy_check.sh
bash infra/tests/test_deploy_changed_image_rollout.sh
bash infra/tests/test_deploy_running_container_detection.sh
```

```bash
python3 infra/scripts/check_assertive_tests.py api/tests
node infra/scripts/check_assertive_tests_web.mjs web
```

## Acceptance Criteria

1. Production can run with Supabase DB + Supabase auth + VM Redis only.
2. Payment routes/features are server-blocked while disabled.
3. Queue/rate-limit/worker runtime remains stable with VM Redis.
4. Polling does not generate high-frequency persistence noise.
5. Docs and changelog reflect final runtime behavior.
