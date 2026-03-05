# Production Env Contract

This document defines the production-safe environment-variable contract using key names only (no secret values inspected).

## Classification

- `required`: must be set for production.
- `optional`: feature- or tuning-dependent.
- `secret-manager-only`: value must be injected from secret storage at runtime/CI; do not commit plaintext.
- `must-be-false-in-prod`: must be disabled in production; paired values must remain empty.
- `public-safe`: intentionally exposed to clients; must never carry secret material.

## Scope

- `env/prod/runtime.env` and `env/prod/runtime.env.example`
- `env/prod/deploy.env.example`
- `env/prod/caddy.env` and `env/prod/caddy.env.example`
- GitHub Actions production environment variables used by `.github/workflows/cd.yml`

## Runtime API/Worker Env (`env/prod/runtime.env*`)

| Key | Classification | Rationale |
|---|---|---|
| `APP_ENV` | `required` | core production runtime setting |
| `LOG_JSON` | `optional` | defaults internally when unset; explicit value recommended for operability |
| `FRONTEND_ASSETS_DIR` | `optional` | defaults internally when unset; explicit value recommended for deploy consistency |
| `SESSION_COOKIE_DOMAIN` | `optional` | enables shared session cookie scope across subdomains (for `www` + `ops`) |
| `STEP_UP_TTL_SECONDS` | `optional` | passkey step-up freshness window for sensitive admin mutations (defaults to 600s) |
| `PROVIDER_AUTH_PROBE_ENABLED` | `optional` | toggles live provider login probe on credential save; default `true` (set `false` to skip probe and persist credentials with `skipped` status) |
| `USER_APP_HOST` | `required` | canonical user-app host for route separation and redirects |
| `ADMIN_APP_HOST` | `required` | canonical admin-app host for strict admin route/API host enforcement |
| `UI_THEME_COOKIE_NAME` | `optional` | cookie key used for user theme preference (`system`, `light`, `dark`) |
| `STATION_CATALOG_JSON_PATH` | `required` | local path to committed station-catalog snapshot consumed by train-service runtime (`repo_only` source mode) |
| `STATION_CATALOG_SOURCE_MODE` | `required` | station catalog source-mode contract; production must be `repo_only` |
| `HTTP_REQUEST_TIMEOUT_SECONDS` | `optional` | API request timeout guardrail tuning (defaults to 30s) |
| `HTTP_REQUEST_BODY_LIMIT_BYTES` | `optional` | API max request body guardrail tuning (defaults to 2 MiB) |
| `HTTP_CONCURRENCY_LIMIT` | `optional` | API request concurrency guardrail tuning (defaults to 32) |
| `ADMIN_EMAILS` | `optional` | comma-separated admin allowlist for maintenance dashboard and metrics access |
| `INVITE_BASE_URL` | `required` | required for invite and auth-link generation |
| `SESSION_SECRET` | `secret-manager-only` | session signing secret; must never use default in production |
| `DATABASE_URL` | `secret-manager-only` | contains secret-bearing value or connection credential |
| `REDIS_URL` | `secret-manager-only` | contains secret-bearing value or connection credential |
| `RUNTIME_QUEUE_KEY` | `required` | core runtime queue contract |
| `RUNTIME_QUEUE_DLQ_KEY` | `required` | core runtime dead-letter queue contract |
| `RUNTIME_LEASE_PREFIX` | `required` | core runtime lease key contract |
| `RUNTIME_RATE_LIMIT_PREFIX` | `required` | core runtime rate-limit key contract |
| `INTERNAL_IDENTITY_SECRET` | `secret-manager-only` | signing/verification secret for internal service identity token |
| `INTERNAL_IDENTITY_ISSUER` | `required` | core production runtime setting |
| `PASSKEY_PROVIDER` | `required` | passkey strategy selector; production must be explicit |
| `WEBAUTHN_RP_ID` | `required` | passkey relying-party ID |
| `WEBAUTHN_RP_ORIGIN` | `required` | passkey relying-party origin |
| `WEBAUTHN_RP_NAME` | `required` | passkey relying-party display name |
| `WEBAUTHN_CHALLENGE_TTL_SECONDS` | `optional` | passkey challenge TTL tuning |
| `KEK_VERSION` | `required` | core production runtime setting |
| `MASTER_KEY` | `secret-manager-only` | contains secret-bearing value or connection credential |
| `MASTER_KEY_OVERRIDE` | `secret-manager-only` | optional override channel for envelope key injection |
| `EMAIL_FROM_ADDRESS` | `required` | core production runtime setting |
| `RESEND_API_KEY` | `secret-manager-only` | contains secret-bearing value or connection credential |
| `RESEND_BASE_URL` | `optional` | optional runtime tuning/override |
| `EVERVAULT_RELAY_BASE_URL` | `optional` | optional runtime tuning/override |
| `EVERVAULT_APP_ID` | `optional` | non-secret provider identifier used by runtime features |
| `WORKER_POLL_SECONDS` | `required` | core worker cadence setting |
| `WORKER_RECONCILE_SECONDS` | `required` | core worker cadence setting |
| `WORKER_WATCH_SECONDS` | `required` | core worker cadence setting |
| `KEY_ROTATION_SECONDS` | `required` | core worker cadence setting |

## Deploy Env (`env/prod/deploy.env.example`)

Reference-only mirror of the GitHub `production` environment deploy controls.
Canonical values must be maintained in GitHub environment variables, not committed env files.

VM baseline expectation (performed by deploy script or one-time host prep):
- active `/swapfile` at `2G`,
- persisted `vm.swappiness=10`,
- persisted `vm.vfs_cache_pressure=50`.

VM secret file expectation:
- `VM_SECRET_ENV_FILE` points to an on-host `0600` env file.
- Deploy script creates this file when missing and persists `BOMINAL_DATABASE_URL`.
- Deploy script reads optional GHCR credentials from this file for private image pulls.
- File should include one of:
  - `BOMINAL_DATABASE_URL=...`
  - `BOMINAL_POSTGRES_PASSWORD=...`
  - `GHCR_USERNAME=...` (required when GHCR images are private)
  - `GHCR_TOKEN=...` (required when GHCR images are private)

| Key | Classification | Rationale |
|---|---|---|
| `GCP_PROJECT_ID` | `required` | required for workload identity and VM target project |
| `GCP_WORKLOAD_IDENTITY_PROVIDER` | `required` | required for OIDC federation |
| `GCP_SERVICE_ACCOUNT` | `required` | required for federated deploy identity |
| `DEPLOY_VM_NAME` | `required` | required VM target |
| `DEPLOY_VM_ZONE` | `required` | required VM zone |
| `DEPLOY_WORKDIR` | `required` | required working directory for remote execution |
| `DEPLOY_SCRIPT_PATH` | `required` | fail-closed deploy script entrypoint |
| `DEPLOY_HEALTHCHECK_SCRIPT_PATH` | `required` | fail-closed post-deploy verification entrypoint |
| `DEPLOY_ROLLBACK_SCRIPT_PATH` | `required` | rollback entrypoint used when healthcheck fails |
| `VM_SECRET_ENV_FILE` | `required` | on-host secret env path for database credential material |
| `DEPLOY_RUNTIME_ENV_FILE` | `required` | runtime env file updated by deploy script |
| `DEPLOY_COMPOSE_FILE` | `required` | compose file used for pull/up and health verification |
| `DEPLOY_MIGRATIONS_DIR` | `required` | migration directory consumed during deploy before service restart |
| `DEPLOY_API_SERVICE` | `required` | compose service identifier for API |
| `DEPLOY_WORKER_SERVICE` | `required` | compose service identifier for worker |
| `DEPLOY_HEALTHCHECK_LIVE_URL` | `required` | live endpoint checked after deploy |
| `DEPLOY_HEALTHCHECK_READY_URL` | `required` | ready endpoint checked after deploy |
| `POSTGRES_HOST` | `required` | required deploy-time Postgres host contract for self-hosted DB |
| `POSTGRES_PORT` | `required` | required deploy-time Postgres port contract for self-hosted DB |
| `POSTGRES_DB` | `required` | required deploy-time Postgres database name contract |
| `POSTGRES_USER` | `required` | required deploy-time Postgres username contract |
| `DEPLOY_COMPOSE_PROJECT_NAME` | `optional` | compose project scoping when non-default naming is used |
| `DEPLOY_ROLLBACK_STATE_PATH` | `optional` | explicit path for rollback state persistence |
| `DEPLOY_VM_BASELINE_SCRIPT` | `optional` | override path for host baseline guard script |
| `DEPLOY_HEALTHCHECK_RETRIES` | `optional` | retry budget for healthcheck script |
| `DEPLOY_HEALTHCHECK_DELAY_SECONDS` | `optional` | delay between healthcheck retries |

## Caddy Env (`env/prod/caddy.env*`)

| Key | Classification | Rationale |
|---|---|---|
| `CADDY_ACME_EMAIL` | `required` | required for TLS site and ACME identity |
| `CADDY_SITE_ADDRESS` | `required` | required for TLS site and ACME identity |

## GitHub Actions Production Environment Vars (`.github/workflows/cd.yml`)

`cd.yml` consumes these keys from the protected `production` environment.

| Key | Classification | Rationale |
|---|---|---|
| `GCP_PROJECT_ID` | `required` | required for workload identity and VM target project |
| `GCP_WORKLOAD_IDENTITY_PROVIDER` | `required` | required for OIDC federation |
| `GCP_SERVICE_ACCOUNT` | `required` | required for federated deploy identity |
| `DEPLOY_VM_NAME` | `required` | required VM target |
| `DEPLOY_VM_ZONE` | `required` | required VM zone |
| `DEPLOY_WORKDIR` | `required` | required working directory for remote execution |
| `DEPLOY_SCRIPT_PATH` | `required` | fail-closed deploy script entrypoint |
| `DEPLOY_HEALTHCHECK_SCRIPT_PATH` | `required` | fail-closed post-deploy verification entrypoint |
| `DEPLOY_ROLLBACK_SCRIPT_PATH` | `required` | rollback entrypoint used when healthcheck fails |
| `VM_SECRET_ENV_FILE` | `required` | on-host secret env path for database credential material |
| `DEPLOY_RUNTIME_ENV_FILE` | `required` | runtime env file updated by deploy script |
| `DEPLOY_COMPOSE_FILE` | `required` | compose file used for pull/up and health verification |
| `DEPLOY_MIGRATIONS_DIR` | `required` | migration directory consumed during deploy before service restart |
| `DEPLOY_API_SERVICE` | `required` | compose service identifier for API |
| `DEPLOY_WORKER_SERVICE` | `required` | compose service identifier for worker |
| `DEPLOY_HEALTHCHECK_LIVE_URL` | `required` | live endpoint checked after deploy |
| `DEPLOY_HEALTHCHECK_READY_URL` | `required` | ready endpoint checked after deploy |
| `POSTGRES_HOST` | `required` | required deploy-time Postgres host contract for self-hosted DB |
| `POSTGRES_PORT` | `required` | required deploy-time Postgres port contract for self-hosted DB |
| `POSTGRES_DB` | `required` | required deploy-time Postgres database name contract |
| `POSTGRES_USER` | `required` | required deploy-time Postgres username contract |
| `DEPLOY_COMPOSE_PROJECT_NAME` | `optional` | compose project scoping when non-default naming is used |
| `DEPLOY_ROLLBACK_STATE_PATH` | `optional` | explicit path for rollback state persistence |
| `DEPLOY_VM_BASELINE_SCRIPT` | `optional` | override path for host baseline guard script |
| `DEPLOY_HEALTHCHECK_RETRIES` | `optional` | retry budget for healthcheck script |
| `DEPLOY_HEALTHCHECK_DELAY_SECONDS` | `optional` | delay between healthcheck retries |

## GitHub Actions Production Environment Secrets (`.github/workflows/cd.yml`)

No custom production-environment secrets are read directly by `cd.yml`.
Database credential material and optional GHCR pull credentials are read on-VM via `VM_SECRET_ENV_FILE`.

## Mandatory Production Guards

- Keep all `must-be-false-in-prod` keys disabled.
- Keep all `secret-manager-only` values out of git and plaintext env files.
- Treat all `public-safe` keys as fully public data.
- Protect deploy variables in GitHub `production` environment and require reviewer approval.
