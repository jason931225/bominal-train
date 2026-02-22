# Changelog

All notable changes to **bominal** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project aims to follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> Note: This repository did not previously publish tagged releases. The `0.1.0`
> section below is an initial, commit-derived snapshot of `main` as of
> 2026-02-08 (commit `147cf92`).

## Unreleased

### Added

- [129d7dc] Added deep provider-transport regression coverage for allowlist helpers, retry/timeout classification, resilient operation profiles, Httpx redirect/host safety controls, and Curl fallback/close behaviors (`api/tests/test_provider_egress_transport.py`).
- [56565fa] Added direct route-level auth coverage for verification/reset token issuance, helper/template branches, session optional paths, and account update validation/error branches, plus expanded admin API integration coverage for stats/user management/train-ops edge flows (`api/tests/test_auth_route_units.py`, `api/tests/test_admin_ops.py`).
- [517b8a0] Added broad backend coverage suites for auth-service account scrub paths, identity auto-provision edge cases, rate-limit backends, API main/runtime guards, worker runtime lifecycle, provider factory/hybrid dispatch behavior, and email queue coverage (`api/tests/test_auth_service_units.py`, `api/tests/test_identity_service.py`, `api/tests/test_rate_limit_units.py`, `api/tests/test_main_units.py`, `api/tests/test_worker_runtime_units.py`, `api/tests/test_worker_train_module.py`, `api/tests/test_train_provider_factory.py`, `api/tests/test_train_provider_hybrid.py`, `api/tests/test_email_queue_units.py`).
- [4692ca1] Added focused backend unit suites for `admin_cli`, app-common health/error paths, DB session generator behavior, monitor utility/runtime formatting, and core time conversion parsing (`api/tests/test_admin_cli_units.py`, `api/tests/test_app_common_units.py`, `api/tests/test_db_session.py`, `api/tests/test_monitor_units.py`, `api/tests/test_time_utils.py`).
- [dee8ac8] Added high-coverage runtime hardening test suites for config/deps/wallet/redis/redaction/envelope/safe-metadata paths, plus split-runtime route/supabase auth regression assertions (`api/tests/test_core_hardening_units.py`, `api/tests/test_runtime_units.py`, `api/tests/test_api_runtime_split.py`, `api/tests/test_supabase_auth.py`).
- [dee8ac8] Added deploy-history record injection regression coverage to prove deployment records are parsed as data, not executed shell (`infra/tests/test_deploy_history_record_safety.sh`).
- [7adb5d6] Added top-nav task-attention notifications with per-task alerts, mark-as-read/clear flows, and clickable task routing, plus dev dummy-task alert population support (`web/components/top-nav-task-attention.tsx`, `web/components/top-nav.tsx`, `web/lib/train/dummy-task-cards.ts`, `web/components/train/train-dashboard.tsx`).
- [e8d3a90] Added Supabase JWT verification, local identity mapping by `sub`, Supabase-linked user schema fields, and artifact storage reference columns with migration support (`api/app/core/supabase_jwt.py`, `api/app/services/identity.py`, `api/app/db/models.py`, `api/alembic/versions/20260222_0009_supabase_identity_and_artifact_storage_refs.py`).
- [eb417fe] Added active PCI runtime policy codification and redaction/envelope hardening plan documents with canonical pointer registration (`docs/plans/active/2026-02-22-pci-runtime-policy-codification.md`, `docs/plans/2026-02-22-redaction-envelope-hardening.md`, `docs/plans/active/README.md`, `docs/README.md`).
- [ca58f68] Added SRT provider regression coverage for login-failure JSON signals, unpaid-cutoff expiry detection, old-PNR ticket parsing, standby wait-code routing, reserve-info no-data mapping, and expired-status manual-pay rejection (`api/tests/test_train_provider_crud.py`, `api/tests/test_train_tasks.py`).
- [8063176] Added PCI-focused security regression tests for redaction, envelope `kek_version` enforcement, safe metadata validation, queue payload safety, runtime Redis guards, logging redaction, and security config validation (`api/tests/test_crypto_redaction.py`, `api/tests/test_crypto_envelope.py`, `api/tests/test_safe_metadata.py`, `api/tests/test_queue_payload_safety.py`, `api/tests/test_runtime_security_checks.py`, `api/tests/test_logging_redaction.py`, `api/tests/test_security_config.py`).
- [8063176] Added continuous sensitive-log scanning utility and CI test gate for PAN/CVV/header leakage detection (`infra/scripts/scan_sensitive_logs.py`, `infra/tests/test_sensitive_log_scan.sh`, `.github/workflows/infra-tests.yml`).
- [65a179f] Added PCI DSS + OWASP ASVS remediation implementation plan and compliance mapping matrix (`docs/plans/2026-02-22-pci-dss-owasp-remediation.md`, `docs/security/compliance-matrix.md`).
- [45037ad] Added KTX wait-reserve regression coverage for waitlist-only selected train flows in worker execution (`api/tests/test_train_tasks.py`).
- [d5b6c96] Added queue re-enqueue regression coverage for stale ARQ result-key clearing, deferred enqueue dedup safety, and rejected enqueue signaling (`api/tests/test_train_queue.py`).
- [9dc1d9e] Added Wave 1 stabilization gate tracker and canonical pointer entries for reviewer-facing release-gate evidence (`docs/plans/active/2026-02-22-wave1-stabilization-gate-tracker.md`, `docs/plans/active/README.md`, `docs/README.md`).
- [9dc1d9e] Added provider egress resilience regression coverage for retry classification, bounded retries, and side-effect no-retry guarantees (`api/tests/test_provider_egress_transport.py`).
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

### Changed

- [517b8a0] Expanded email service coverage to exercise SMTP/Resend helper branches, delivery failure mapping, and queue enqueue/defer semantics while preserving runtime behavior (`api/tests/test_email_service.py`, `api/tests/test_email_queue_units.py`).
- [4692ca1] Fixed admin task prefix-match SQLAlchemy casting to use dialect-safe `String` casts instead of Python `str`, preventing CLI runtime failures on task cancel/unhide partial-ID lookups (`api/app/admin_cli.py`).
- [dee8ac8] Hardened deploy runtime record handling by replacing `source`-based rollback/status parsing with strict key/value decoding and updated rolling deploy targets for split API/worker services (`infra/scripts/deploy.sh`, `infra/tests/test_deploy_preflight.sh`).
- [dee8ac8] Updated local verification/runtime docs and tooling for current security posture, including local-check version wiring, hosted quota runbook guidance (Upstash non-CDE + Supabase), and web coverage dependency support (`infra/scripts/local-check.sh`, `docs/RUNBOOK.md`, `web/package.json`, `web/package-lock.json`).
- [dee8ac8] Renumbered the access-review migration to preserve linear Alembic history (`api/alembic/versions/20260222_0011_user_access_review_status.py`).
- [7adb5d6] Replaced provider-search retry backoff scheduling with a single stretched-exponential mean curve and mean-preserving gamma jitter, anchored at 24h/48h/72h and documented in architecture guidance (`api/app/modules/train/worker.py`, `api/tests/test_train_tasks.py`, `docs/ARCHITECTURE.md`).
- [7adb5d6] Refined train dashboard behavior and copy for payment-disabled flows, awaiting-payment activity handling, mobile-first schedule search interactions, and provider/train presentation updates across locale resources (`web/components/train/train-dashboard.tsx`, `web/components/train/train-task-detail.tsx`, `web/components/module-tile.tsx`, `web/lib/train/stations-i18n.ts`, `web/messages/en.json`, `web/messages/ko.json`, `api/app/modules/train/service.py`, `api/app/modules/train/schemas.py`, `api/app/modules/train/providers/srt_client.py`).
- [7adb5d6] Hardened local developer runtime checks by wiring compose/env utilities and local-check wrappers to current split-redis/local-service expectations and version/env propagation (`infra/scripts/lib/env_utils.sh`, `infra/scripts/local-check.sh`, `infra/scripts/local-run.sh`, `infra/docker-compose.yml`, `infra/env/dev/api.env`, `infra/env/dev/web.env`, `infra/env/prod/api.env.example`, `infra/env/prod/web.env.example`, `web/Dockerfile.dev`, `web/lib/server-auth.ts`).
- [4453b90] Switched queue/rate-limit/worker-monitor Redis clients to the non-CDE Redis endpoint, strengthened deploy preflight validation for auth/email/database env contracts, and documented required deployment placeholders for Supabase/Resend modes (`api/app/core/rate_limit.py`, `api/app/modules/train/queue.py`, `api/app/modules/restaurant/queue.py`, `api/app/services/email_queue.py`, `api/app/worker.py`, `api/app/worker_restaurant.py`, `api/app/monitor.py`, `infra/scripts/predeploy-check.sh`, `infra/tests/test_predeploy_check.sh`, `api/tests/test_queue_domains.py`, `api/tests/test_worker_settings.py`, `api/tests/test_notifications.py`, `infra/env/dev/api.env`, `infra/env/prod/api.env.example`, `README.md`, `docs/DEPLOYMENT.md`, `docs/RUNBOOK.md`).
- [69e5992] Clarified deployment docs and env templates so `deploy.env` is explicitly optional for helper workflows and not required by canonical `infra/scripts/deploy.sh`, and removed stale script naming from `deploy.env.example` comments (`README.md`, `docs/DEPLOYMENT.md`, `infra/env/prod/deploy.env.example`).
- [e19d5d0] Tightened production deployment placeholder handling by adding explicit `CHANGE_ME` markers for `GCP_PROJECT_ID`, Caddy hostname/email, and public web URL, and by extending predeploy checks to require deploy-critical keys across API/Postgres/Web/Caddy env files (`infra/env/prod/api.env.example`, `infra/env/prod/caddy.env.example`, `infra/env/prod/web.env.example`, `infra/scripts/predeploy-check.sh`, `infra/tests/test_predeploy_check.sh`, `README.md`, `docs/DEPLOYMENT.md`).
- [10be95e] Split Redis configuration into non-CDE and CDE endpoints, blocked Upstash-hosted Redis for CDE CVV cache routing, switched wallet CVV paths and production Redis persistence guard to CDE Redis, and documented the operational/security contract updates (`api/app/core/config.py`, `api/app/core/redis.py`, `api/app/services/wallet.py`, `api/app/main.py`, `api/tests/test_security_config.py`, `api/tests/test_wallet.py`, `api/tests/test_auth_flow.py`, `docs/SECURITY.md`, `docs/ARCHITECTURE.md`, `docs/RUNBOOK.md`, `README.md`, `infra/env/dev/api.env`, `infra/env/prod/api.env.example`).
- [e787265] Hardened Supabase identity mapping to avoid auto-provision collisions by not mutating existing local email from JWT claims and by dropping conflicting display names to `null` (`api/app/services/identity.py`, `api/tests/test_supabase_auth.py`).
- [e8d3a90] Added configurable auth modes (`legacy`/`supabase`/`dual`) with fail-closed Bearer precedence in dual mode, Supabase env contracts, and auth/security documentation updates (`api/app/core/config.py`, `api/app/http/deps.py`, `infra/env/dev/api.env`, `infra/env/prod/api.env.example`, `docs/ARCHITECTURE.md`, `docs/SECURITY.md`, `README.md`).
- [bdb126d] Stabilized client-side auth navigation by removing `router.push()+router.refresh()` race patterns and enforcing same-origin browser API calls (with Next `/api` proxy rewrites) so `SameSite=Lax` session cookies are not rejected in cross-site fetch contexts (`web/components/login-form.tsx`, `web/components/logout-button.tsx`, `web/components/account/account-settings-panel.tsx`, `web/components/auth/password-reset-confirm-form.tsx`, `web/lib/api-base.ts`, `web/next.config.mjs`, `infra/env/dev/web.env`, `infra/env/prod/web.env.example`, `README.md`, `docs/DEPLOYMENT.md`).
- [ca58f68] Aligned SRT provider behavior with source-reference semantics for per-passenger reserve payload fields, standby eligibility (`rsvWaitPsbCd` contains `9`), reservation not-found/no-data mapping, and unpaid payment-cutoff expiry status propagation (`api/app/modules/train/providers/srt_client.py`, `api/app/modules/train/ticket_sync.py`, `api/app/modules/train/service.py`, `api/app/modules/train/worker.py`, `api/app/modules/train/schemas.py`).
- [ca58f68] Documented SRT expiration/not-found operational signals and provider contract updates in architecture/runbook docs (`docs/ARCHITECTURE.md`, `docs/RUNBOOK.md`).
- [4aa5281] Enforced CDE runtime controls with hardened redaction patterns, logging-boundary redaction, `kek_version`-enforced decrypt boundaries, bounded CVV TTL policy settings, provider egress allowlist/redirect SSRF guards, and safe metadata enforcement in worker/service persistence paths (`api/app/core/crypto/redaction.py`, `api/app/core/logging.py`, `api/app/core/crypto/envelope.py`, `api/app/core/crypto/secrets_store.py`, `api/app/core/config.py`, `api/app/services/wallet.py`, `api/app/modules/train/providers/transport.py`, `api/app/core/crypto/safe_metadata.py`, `api/app/modules/train/worker.py`, `api/app/modules/train/service.py`, `api/app/modules/restaurant/worker.py`, `api/app/modules/train/queue.py`, `api/app/modules/restaurant/queue.py`, `api/app/main.py`, `infra/env/dev/api.env`, `infra/env/prod/api.env.example`).
- [65a179f] Codified PCI relay isolation, CDE boundaries, redaction and Redis CVV enforcement, provider payload safety, and egress security requirements across canonical docs (`docs/SECURITY.md`, `docs/GUARDRAILS.md`, `docs/PERMISSIONS.md`, `docs/ARCHITECTURE.md`, `docs/RUNBOOK.md`, `README.md`, `docs/README.md`).
- [c982a4f] Updated Wave 1 stabilization gate tracker with dated verification evidence, gate-status closure for W1-G08/W1-G09/W1-G10, and reviewer decision conditions (`docs/plans/active/2026-02-22-wave1-stabilization-gate-tracker.md`).
- [9dc1d9e] Debounced authenticated session activity writes with configurable interval to reduce DB write amplification while preserving session expiry/revocation checks on each request (`api/app/http/deps.py`, `api/app/services/auth.py`, `api/app/core/config.py`, `api/tests/test_auth_flow.py`).
- [9dc1d9e] Introduced fail-safe provider transport resilience primitives (operation-aware timeouts, bounded transient retries, non-retryable fail-closed behavior) and wired live/hybrid provider clients to surface structured retryable outcomes (`api/app/modules/train/providers/transport.py`, `api/app/modules/train/providers/factory.py`, `api/app/modules/train/providers/hybrid.py`).
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

### Removed

- [f3cbeda] Removed completed governance plan document (`docs/plans/2026-02-12-doc-governance-guardrails-grand-plan.md`).
- [5039127] Removed deprecated deploy compose artifact (`infra/docker-compose.deploy.yml.deprecated`) after guarded dependency scan.

### Fixed

- [ac078a8] Fixed changelog gate reliability by switching `infra/tests/test_changelog.sh` to temp-file streaming for Unreleased parsing, eliminating long-shell-substitution hangs on large changelog sections.
- [3235c90] Prevented Vitest CI runs from loading Playwright e2e specs by excluding `e2e/**` and `playwright.config.ts` from the unit-test runner (`web/vitest.config.ts`).
- [8938629] Restored wait-reserve-aware seat-class fallback so waitlist-capable KTX/SRT schedules can proceed when direct seat availability is false (`api/app/modules/train/worker.py`).
- [59ff9b6] Prevented deferred polling self-dedup lockout and stale deterministic enqueue result reuse by splitting deferred/immediate ARQ enqueue semantics (`api/app/modules/train/queue.py`).
- [15bf7fc] Prevented rollback pointer swap on rollback-deploy failure by making rollback deploy/pull failures propagate and adding canary-stage regression coverage (`infra/scripts/deploy.sh`, `infra/tests/test_deploy_canary_stages.sh`).
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
