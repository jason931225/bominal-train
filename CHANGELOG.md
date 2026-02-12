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

### Changed

- [0d84ae8] Enforced changelog requirements in governance docs (`AGENTS.md`, `docs/EXECUTION_PROTOCOL.md`).
- [0d84ae8] Added `CHANGELOG.md` to canonical pointers in `docs/README.md` and required-pointer validation.
- [0d84ae8] Extended daily chores playbook with token-saving search/navigation operations (`rg --files`, scoped `rg -n`, pointer-first reads).
- [0d84ae8] Aligned governance docs to current canonical deploy script (`infra/scripts/deploy-zero-downtime.sh`) and removed active references to `fetch_ci.sh`/`deploy.prod.sh`.
- [0d84ae8] Standardized compose command examples to `docker compose` in high-traffic docs and added `infra/tests/test_docs_consistency.sh`.

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
  (`infra/scripts/deploy-zero-downtime.sh`, `infra/docker-compose.prod.yml`).
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
