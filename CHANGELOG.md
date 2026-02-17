# Changelog

All notable changes to **bominal** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project aims to follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> Note: This repository did not previously publish tagged releases. The `0.1.0`
> section below is an initial, commit-derived snapshot of `main` as of
> 2026-02-08 (commit `147cf92`).

## Unreleased

### Added

- [0d84ae8] Added commit-based changelog governance and CI validation (`infra/tests/test_changelog.sh`).
- [0d84ae8] Added standardized documentation workflow and playbook system (`docs/DOCUMENTATION_WORKFLOW.md`, `docs/playbooks/*`).
- [0d84ae8] Added daily operations/chores playbook for routine low-latency execution (`docs/playbooks/daily-operations-chores.md`).
- [0d84ae8] Added `docs/PERMISSIONS.md` and integrated permission protocol into docs/governance pointers.
- [0d84ae8] Added new module/feature workflow playbook (`docs/playbooks/new-module-feature-workflow.md`).
- [0d84ae8] Added `docs/GUARDRAILS.md` as hard constraints separate from permission policy.
- [0d84ae8] Added `docs/INTENT_ROUTING.md` and CI validator `infra/tests/test_intent_routing.sh` for keyword-to-pointer routing.
- [0d84ae8] Consolidated backend markdown TODO into `docs/todo/backend-production-readiness.md`.
- [5039127] Added deprecation inventory and guarded reference check (`docs/deprecations/2026-02-14-inventory.md`, `infra/tests/test_deprecation_references.sh`).
- [1d61909] Added explicit queue-domain constants and restaurant queue producer contract (`api/app/core/queue_domains.py`, `api/app/modules/restaurant/queue.py`).
- [1842ca3] Added module capability metadata contract for staged module exposure (`api/app/http/routes/modules.py`, `api/tests/test_modules_api.py`).
- [5dc90c6] Added restaurant policy scaffold helpers for auth fallback and payment lease behavior (`api/app/modules/restaurant/policy.py`, `api/app/modules/restaurant/lease.py`, `api/app/modules/restaurant/types.py`).
- [8199f85] Added canonical deprecation workflow policy and machine registry (`docs/DEPRECATION_WORKFLOW.md`, `docs/deprecations/registry.json`).
- [da12731] Added registry-driven deprecation guard and policy validation tests (`infra/scripts/deprecation_guard.py`, `infra/tests/test_deprecation_policy.sh`).
- [69fd34e] Added Stage 8 closure artifacts and active-plan state marker (`docs/plans/archive/2026-02-14-program-closure-report.md`, `docs/plans/archive/2026-02-14-stage8-program-closure-and-archival-hygiene.md`, `docs/plans/active/README.md`).
- [69fd34e] Added ledger-template safety validator (`infra/tests/test_execution_ledgers.sh`).
- [f84421d] Added Stage 9 active performance execution plan and pointer registration (`docs/plans/active/2026-02-14-stage9-performance-optimization.md`, `docs/README.md`).
- [eee7868] Added repeatable train task list latency benchmark script with p50/p95 reporting and validation checks (`infra/scripts/benchmark-train-task-list.sh`, `infra/tests/test_benchmark_train_task_list.sh`).
- [eee7868] Added high-risk local DB reset workflow with optional fresh-schema rebuild and sign-in credential preservation (`infra/scripts/reset-local-db.sh`, `infra/tests/test_reset_local_db.sh`).
- [1c118c4] Added hybrid benchmark gate tooling for train task-list latency comparisons (`infra/scripts/benchmark-train-task-list-compare.sh`, `infra/scripts/benchmark-threshold-check.sh`, `infra/benchmarks/train-task-list-baseline.json`, `infra/tests/test_benchmark_train_task_list_compare.sh`).
- [1c118c4] Added web unit-test harness and polling behavior coverage for train dashboard task refresh logic (`web/vitest.config.ts`, `web/test/setup.ts`, `web/components/train/__tests__/train-dashboard.polling.test.tsx`, `web/package.json`).
- [0d817df] Added restaurant provider adapter scaffold package and canonical contract tests (`api/app/modules/restaurant/providers/*`, `api/tests/test_restaurant_provider_contracts.py`).
- [0d817df] Added provider-research documentation pack for OpenTable/Resy endpoint contracts and DB-safe schema mapping (`docs/provider-research/*`).
- [0d817df] Added reusable restaurant/provider onboarding playbooks and active execution plan artifacts (`docs/playbooks/restaurant-provider-adapter-workflow.md`, `docs/playbooks/provider-adapter-contract-template.md`, `docs/plans/active/2026-02-17-restaurant-provider-foundations.md`).
- [13bdb1d] Added OpenTable stage-1 adapter coverage for refresh/profile/cancel and configurable auth/search/create contract paths, with dedicated adapter tests (`api/app/modules/restaurant/providers/opentable_adapter.py`, `api/tests/test_restaurant_provider_opentable.py`).
- [32cc570] Added concrete OpenTable OTP auth contract defaults (`/dapi/authentication/sendotpfromsignin`, `/dapi/authentication/signinwithotp`) with request-shape normalization and expanded adapter tests (`api/app/modules/restaurant/providers/opentable_adapter.py`, `api/tests/test_restaurant_provider_opentable.py`).
- [0058173] Added frozen OpenTable search/create contract wiring with normalized variable schema and settings-backed operation/hash controls (`api/app/modules/restaurant/providers/opentable_adapter.py`, `api/app/core/config.py`, `api/app/modules/restaurant/providers/factory.py`, `api/tests/test_restaurant_provider_opentable.py`).

### Changed

- [0d84ae8] Enforced changelog requirements in governance docs (`AGENTS.md`, `docs/EXECUTION_PROTOCOL.md`).
- [0d84ae8] Added `CHANGELOG.md` to canonical pointers in `docs/README.md` and required-pointer validation.
- [0d84ae8] Extended daily chores playbook with token-saving search/navigation operations (`rg --files`, scoped `rg -n`, pointer-first reads).
- [0d84ae8] Aligned governance docs to current canonical deploy script (`infra/scripts/deploy.sh`) and removed active references to `fetch_ci.sh`/`deploy.prod.sh`.
- [0d84ae8] Standardized compose command examples to `docker compose` in high-traffic docs and added `infra/tests/test_docs_consistency.sh`.
- [83e6d6c] Split worker entrypoints into train and restaurant runtime settings, and wired `worker-restaurant` service in dev/prod compose plus deploy/restart helpers.
- [d9901c4] Hardened deploy runtime with script-level lock, running-stack detection, strict preflight resource gate, and smoke-failure auto-rollback controls.
- [f5645f4] Canonized plan governance with active/archive lifecycle structure and stage-level status tracking under `docs/plans/active/`.
- [bbc1f8f] Renamed canonical deploy entrypoint to `infra/scripts/deploy.sh` and aligned governance/docs/script references.
- [c71f46b] Updated architecture docs for queue-domain contracts, module capabilities, and restaurant policy scaffold.
- [d2dabfa] Centralized compose detection/file-resolution helpers in `infra/scripts/lib/env_utils.sh` and aligned wrapper scripts.
- [a04acce] Applied additional idiomatic shell improvements to wrapper scripts and shared env helper usage.
- [10ce9af] Renamed deploy regression shell tests to match `deploy.sh` naming and updated Stage 5 plan/status references.
- [da12731] Enforced blocking deprecation checks in CI and deploy-preflight flow (`.github/workflows/infra-tests.yml`, `.github/workflows/deploy.yml`, `infra/scripts/predeploy-check.sh`).
- [8199f85] Standardized docs/governance routing and pointer coverage for deprecation lifecycle handling (`docs/README.md`, `docs/INTENT_ROUTING.md`, `infra/tests/test_docs_pointers.sh`).
- [69fd34e] Archived completed restructure stage plans from `docs/plans/active/` to `docs/plans/archive/` and synchronized plan routing/pointers for closure state.
- [69fd34e] Normalized lock/request ledgers to separate live entries from non-live templates and enforced the rule in infra CI workflow (`docs/LOCK.md`, `docs/REQUEST.md`, `.github/workflows/infra-tests.yml`).
- [f84421d] Optimized train task list performance with bounded `limit` query support, latest-row summary selection, composite list indexes, and bounded active/completed dashboard polling (`api/app/modules/train/service.py`, `api/app/modules/train/router.py`, `api/alembic/versions/20260214_0008_train_task_perf_indexes.py`, `web/components/train/train-dashboard.tsx`).
- [0b58ef2] Reduced task-list tail latency by using PostgreSQL `DISTINCT ON` latest-attempt/artifact summary paths with non-Postgres ranking fallback compatibility (`api/app/modules/train/service.py`, `api/alembic/versions/20260215_0009_task_list_tail_latency_indexes.py`).
- [b386052] Reduced frontend train dashboard polling/load by refreshing completed tasks on periodic or forced triggers and skipping unchanged task-list state commits (`web/components/train/train-dashboard.tsx`).
- [758f0f5] Documented benchmark/reset shell script operations and guardrails in runbook procedures (`docs/RUNBOOK.md`).
- [721600a] Extended infra CI workflow to execute benchmark/reset shell script validation tests (`.github/workflows/infra-tests.yml`, `infra/tests/test_benchmark_train_task_list.sh`, `infra/tests/test_reset_local_db.sh`).
- [6b60051] Documented Stage10 backend performance completion status in architecture and active performance plan (`docs/ARCHITECTURE.md`, `docs/plans/active/2026-02-14-stage9-performance-optimization.md`).
- [1c118c4] Expanded infra CI validation to run web unit tests and benchmark compare script checks for perf-sensitive path changes (`.github/workflows/infra-tests.yml`).
- [1c118c4] Added Stage12 perf hardening execution notes and runbook gate command references (`docs/plans/active/2026-02-14-stage9-performance-optimization.md`, `docs/RUNBOOK.md`, `docs/ARCHITECTURE.md`).
- [d1b8c61] Registered CatchTable reference files as read-only provider endpoint sources in agent and contributor guidance (`AGENTS.md`, `README.md`, `docs/CONTRIBUTING.md`, `docs/ARCHITECTURE.md`).
- [0d817df] Expanded architecture/readme/contributing guidance and canonical pointer routing for provider-adapter implementation workflow (`docs/ARCHITECTURE.md`, `README.md`, `docs/CONTRIBUTING.md`, `docs/README.md`, `docs/INTENT_ROUTING.md`, `docs/playbooks/README.md`).
- [13bdb1d] Extended restaurant provider configuration defaults and documentation status to support configurable OpenTable OTP endpoint paths and stage-1 endpoint inventory updates (`api/app/core/config.py`, `api/app/modules/restaurant/providers/factory.py`, `api/tests/test_restaurant_policy_config.py`, `docs/provider-research/opentable-endpoints.md`, `docs/provider-research/restaurant-provider-endpoint-inventory.md`, `docs/plans/active/2026-02-17-restaurant-provider-foundations.md`).
- [32cc570] Updated OpenTable provider docs/plan status to mark OTP auth endpoints as confirmed contracts and narrowed remaining capture gaps to search/create and response-schema freeze (`docs/provider-research/opentable-endpoints.md`, `docs/provider-research/restaurant-provider-endpoint-inventory.md`, `docs/plans/active/2026-02-17-restaurant-provider-foundations.md`, `docs/ARCHITECTURE.md`).
- [0058173] Updated OpenTable provider docs/plan inventory and prod env template to codify non-metadata-driven search/create contracts and remaining persisted-hash capture gaps (`docs/provider-research/opentable-endpoints.md`, `docs/provider-research/restaurant-provider-endpoint-inventory.md`, `docs/plans/active/2026-02-17-restaurant-provider-foundations.md`, `docs/ARCHITECTURE.md`, `infra/env/prod/api.env.example`).
- [f97c336] Froze OpenTable `search.availability` (`RestaurantsAvailability`) and two-step `reservation.create` (`BookDetailsStandardSlotLock` + `/dapi/booking/make-reservation`) contracts in adapter/config/tests, and updated provider/architecture schema docs with captured payload mappings (`api/app/modules/restaurant/providers/opentable_adapter.py`, `api/app/core/config.py`, `api/tests/test_restaurant_provider_opentable.py`, `api/tests/test_restaurant_policy_config.py`, `docs/provider-research/opentable-endpoints.md`, `docs/provider-research/restaurant-provider-endpoint-inventory.md`, `docs/provider-research/restaurant-db-schema-mapping.md`, `docs/ARCHITECTURE.md`, `infra/env/prod/api.env.example`).

### Removed

- [f3cbeda] Removed completed governance plan document (`docs/plans/2026-02-12-doc-governance-guardrails-grand-plan.md`).
- [5039127] Removed deprecated deploy compose artifact (`infra/docker-compose.deploy.yml.deprecated`) after guarded dependency scan.

### Fixed

- [220d2c6] Ensured worker shutdown recovers in-flight tasks even when heartbeat cancellation raises `CancelledError`, with regression coverage in `api/tests/test_worker_shutdown_recovery.py`.
- [b05ca4b] Converted commit-time auth uniqueness races to deterministic `409` conflicts in register/account update flows (`api/app/http/routes/auth.py`).
- [b231d4c] Made auth rate-limit client IP extraction proxy-aware (`cf-connecting-ip` / `x-forwarded-for`) in `api/app/http/deps.py`.
- [adb5da8] Stabilized deploy lock regression test to avoid startup race under fallback lock mode.

## 0.1.0 - 2026-02-08

### Added

- Initial modular platform foundation: `web/` (Next.js), `api/` (FastAPI),
  `worker` (arq), `redis`, and `third_party/srtgo` (provider reference).
- Auth + session cookie flows, with account deletion behavior that retains task
  rows for a removal window.
- Train module APIs and worker orchestration:
  - provider credential management (SRT/KTX)
  - station list + schedule search
  - background task creation and state machine
  - attempt timeline and ticket artifacts
- Admin maintenance dashboard, including user management and system stats.
- Admin-only OpenAPI documentation endpoints (`/api/docs`, `/api/openapi.json`).
- Local developer scripts:
  - `infra/scripts/local-setup.sh`
  - `infra/scripts/local-run.sh`
  - `infra/scripts/local-check.sh`
- CI image build + pull-based VM deploy scaffolding (GitHub Actions).
- Zero-downtime deployment workflow with health-check gating
  (`infra/scripts/deploy.sh`, `infra/docker-compose.prod.yml`).
- Operator tooling and scripts: `bominal-monitor`, `bominal-admin`,
  `predeploy-check.sh`, `quick-restart.sh`, and VM bootstrap helpers under
  `infra/scripts/`.
- Documentation pack under `docs/` (architecture, contributing, deployment,
  runbook, security controls).
- Terraform under `infra/terraform/` for GCP CI/CD and an optional VM bootstrap
  path.
- Web UI improvements including a refreshed landing page/typography and build
  version display in navigation.

### Changed

- Train task UX/observability:
  - task summaries include last attempt metadata
  - expose `next_run_at` for polling tasks
  - "Retry now" endpoint + dashboard hints
- Train worker behavior:
  - idempotent task enqueue per task (prevents duplicate jobs across restarts)
  - Redis token-bucket rate limiter for outbound provider calls
- Deployment and CI were iterated toward script-driven, pull-based deploys and
  environment-driven configuration (for example: region/project inputs, commit
  targeting, OS Login, and running deploys as the `bominal` user).
- Production defaults and ops ergonomics were improved (for example: enabling
  Redis rate limiting in production; disabling Next.js telemetry in production;
  tuned resource limits/health checks for small VMs).

### Fixed

- Worker: stop an arq crash loop; add graceful shutdown and task recovery; avoid
  recovering deleted/paused tasks.
- Train credentials: auto-normalize phone numbers in credential input.
- Train: bootstrap KTX login flow and centralize time utilities.
- Deploy: multiple health check fixes (Python-based checks for slim images,
  127.0.0.1 health targets, tuned `start_period` values, Caddy health checks).
- Deploy: Caddy apex-domain redirect to `www`.
- Deploy: e2-micro deployment optimizations.
- Web/API integration: default client API base to same-origin.
- Admin UI: align task state labels.
- API health: Redis check uses the shared core Redis client.

### Security

- CI: mask generated SSH private keys in workflow logs.
- CI deploy: use OS Login and run deploy steps as the `bominal` user.

[Unreleased]: https://github.com/jason931225/bominal/compare/147cf92...HEAD
[0.1.0]: https://github.com/jason931225/bominal/tree/147cf92
