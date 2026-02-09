# Changelog

All notable changes to **bominal** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project aims to follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> Note: This repository did not previously publish tagged releases. The `0.1.0`
> section below is an initial, commit-derived snapshot of `main` as of
> 2026-02-08 (commit `f7e4118`).

## Unreleased

### Added

- `CHANGELOG.md` (Keep a Changelog format) to track notable changes.

### Changed

- `README.md` was tightened:
  - Clarify local vs production compose commands and deployment pointers.
  - Correct account update docs: `current_password` is required for changing
    `email` / `new_password`, but not for all profile fields.

## 0.1.0 - 2026-02-08

### Added

- Initial modular platform foundation: `web/` (Next.js), `api/` (FastAPI),
  `worker` (arq), `redis`, and `third_party/srtgo` (provider reference).
- CI image build + pull-based VM deploy scaffolding (GitHub Actions).
- Zero-downtime deployment workflow with health-check gating
  (`infra/scripts/deploy-zero-downtime.sh`, `infra/docker-compose.prod.yml`).
- Admin maintenance dashboard with user management.
- Operator tooling and scripts: `bominal-monitor`, `bominal-admin`,
  `predeploy-check.sh`, `quick-restart.sh`, and VM bootstrap helpers under
  `infra/scripts/`.
- Documentation pack under `docs/` (architecture, contributing, deployment,
  runbook, security controls).
- Terraform configuration under `infra/terraform/` for GCP CI/CD and an optional
  VM bootstrap path.
- Web UI improvements including a refreshed landing page/typography and build
  version display in navigation.

### Changed

- CI/CD deploy workflows were iterated toward script-driven, pull-based deploys
  and environment-driven configuration (region/project inputs, commit targeting).
- Production defaults and ops ergonomics were improved (for example: enabling
  Redis rate limiting in production; disabling Next.js telemetry in production
  builds).

### Fixed

- Worker: stop an arq crash loop.
- Worker: add graceful shutdown and task recovery.
- Worker: prevent deleted/paused tasks from being recovered or processed.
- Train credentials: auto-normalize phone numbers in credential input.
- Train: bootstrap KTX login flow and centralize time utilities.
- Deploy: multiple health check fixes (Python-based checks for slim images,
  127.0.0.1 health targets, tuned `start_period` values).
- Deploy: Caddy apex-domain redirect to `www`.
- Deploy: e2-micro deployment optimizations.
- Web/API integration: default client API base to same-origin.
- Admin UI: align task state labels.

### Security

- CI: mask generated SSH private keys in workflow logs.
- CI deploy: use OS Login and run deploy steps as the `bominal` user.

[Unreleased]: https://github.com/jason931225/bominal/compare/f7e4118...HEAD
[0.1.0]: https://github.com/jason931225/bominal/tree/f7e4118
