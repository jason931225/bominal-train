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
| `DATABASE_URL` | `secret-manager-only` | contains secret-bearing value or connection credential |
| `SUPABASE_URL` | `required` | core production runtime setting |
| `SUPABASE_PUBLISHABLE_KEY` | `public-safe` | browser-safe key for client auth bootstrap; must not contain secret material |
| `SUPABASE_JWKS_URL` | `optional` | defaults to `${SUPABASE_URL}/auth/v1/.well-known/jwks.json` when unset |
| `SUPABASE_JWT_ISSUER` | `required` | core production runtime setting |
| `SUPABASE_JWT_AUDIENCE` | `required` | core production runtime setting |
| `SUPABASE_JWKS_CACHE_SECONDS` | `optional` | non-core feature or tuning parameter |
| `SUPABASE_AUTH_WEBHOOK_SECRET` | `secret-manager-only` | secret used for webhook origin/auth validation |
| `REDIS_URL` | `secret-manager-only` | contains secret-bearing value or connection credential |
| `RUNTIME_QUEUE_KEY` | `required` | core runtime queue contract |
| `RUNTIME_QUEUE_DLQ_KEY` | `required` | core runtime dead-letter queue contract |
| `RUNTIME_LEASE_PREFIX` | `required` | core runtime lease key contract |
| `RUNTIME_RATE_LIMIT_PREFIX` | `required` | core runtime rate-limit key contract |
| `INTERNAL_IDENTITY_SECRET` | `secret-manager-only` | signing/verification secret for internal service identity token |
| `INTERNAL_IDENTITY_ISSUER` | `required` | core production runtime setting |
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

Host bootstrap baseline (non-env, mandatory on deploy VM):
- active `/swapfile` at `2G`,
- persisted `vm.swappiness=10`,
- persisted `vm.vfs_cache_pressure=50`.

Operational note:
- bake an idempotent guard for the above baseline into `DEPLOY_COMMAND` (prefix), and keep the same guard documented in `env/prod/deploy.env.example`.

| Key | Classification | Rationale |
|---|---|---|
| `AUTO_DEPLOY_MAIN` | `optional` | enables deploy on push to `main` when explicitly set to `true` |
| `GCP_PROJECT_ID` | `required` | required for workload identity and VM target project |
| `GCP_WORKLOAD_IDENTITY_PROVIDER` | `required` | required for OIDC federation |
| `GCP_SERVICE_ACCOUNT` | `required` | required for federated deploy identity |
| `DEPLOY_VM_NAME` | `required` | required VM target |
| `DEPLOY_VM_ZONE` | `required` | required VM zone |
| `DEPLOY_COMMAND` | `required` | fail-closed deploy contract |
| `DEPLOY_HEALTHCHECK_COMMAND` | `required` | fail-closed post-deploy verification contract |

## Caddy Env (`env/prod/caddy.env*`)

| Key | Classification | Rationale |
|---|---|---|
| `CADDY_ACME_EMAIL` | `required` | required for TLS site and ACME identity |
| `CADDY_SITE_ADDRESS` | `required` | required for TLS site and ACME identity |

## GitHub Actions Production Environment Vars (`.github/workflows/cd.yml`)

| Key | Classification | Rationale |
|---|---|---|
| `GCP_PROJECT_ID` | `required` | required for workload identity and VM target project |
| `GCP_WORKLOAD_IDENTITY_PROVIDER` | `required` | required for OIDC federation |
| `GCP_SERVICE_ACCOUNT` | `required` | required for federated deploy identity |
| `DEPLOY_VM_NAME` | `required` | required VM target |
| `DEPLOY_VM_ZONE` | `required` | required VM zone |
| `DEPLOY_COMMAND` | `required` | fail-closed deploy contract |
| `DEPLOY_HEALTHCHECK_COMMAND` | `required` | fail-closed post-deploy verification contract |
| `AUTO_DEPLOY_MAIN` | `optional` | enables deploy on push to `main` when explicitly set to `true` |
| `RUST_COVERAGE_MIN` | `optional` | configurable CI coverage floor |

## Mandatory Production Guards

- Keep all `must-be-false-in-prod` keys disabled.
- Keep all `secret-manager-only` values out of git and plaintext env files.
- Treat all `public-safe` keys as fully public data.
- Protect deploy variables in GitHub `production` environment and require reviewer approval.
