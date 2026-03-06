# Production Env Contract

This document defines the production-safe environment-variable contract using key names only.
It also separates current VM deployment reality from future Cloud Run cutover prep so placement is explicit.

## Architecture Boundary

- Active production path: API + worker + Postgres + Redis on the VM.
- Future prep-only path: API on Cloud Run, worker + Postgres + Redis still on the VM.
- Current `.github/workflows/cd.yml` deploys only the VM path.
- Future Cloud Run prep lives under `runtime/cloudrun/api/*` and is not consumed automatically.

## Placement Boundary

- Git-tracked examples:
  - `env/prod/*.example`
  - placeholders only, never real secrets
- VM repo env / on-host files:
  - `env/prod/runtime.env`
  - `env/prod/deploy.env`
  - `env/prod/caddy.env`
  - `env/prod/vm-secrets.env`
- Future Cloud Run plain env:
  - hostnames, routing values, Redis URL, and runtime guardrail values
- Future Google Secret Manager:
  - max `5` active secrets at all times
  - adding one requires removing another
  - default future set:
    - `DATABASE_URL`
    - `SESSION_SECRET`
    - `INTERNAL_IDENTITY_SECRET`
    - `MASTER_KEY`
    - `RESEND_API_KEY`
  - `REDIS_URL` stays plain env by default to preserve the 5-secret policy

## Runtime API/Worker Env (`env/prod/runtime.env*`)

Active use:
- current VM deployment reads this file directly

Future use:
- non-secret runtime keys can also feed Cloud Run
- current `runtime.env.example` remains VM-friendly
- future Cloud Run overrides live in `env/prod/cloudrun-api.env.example`

Key additions kept for future cutover:

| Key | Classification | Placement | Rationale |
|---|---|---|---|
| `API_DB_POOL_MAX_CONNECTIONS` | `optional` | VM repo env now; Cloud Run plain env later | API pool ceiling override for constrained backends |
| `WORKER_DB_POOL_MAX_CONNECTIONS` | `optional` | VM repo env | worker pool ceiling override |
| `DB_POOL_ACQUIRE_TIMEOUT_SECONDS` | `optional` | VM repo env now; Cloud Run plain env later | shared DB acquire-timeout guardrail |
| `DB_POOL_IDLE_TIMEOUT_SECONDS` | `optional` | VM repo env now; Cloud Run plain env later | shared DB idle-timeout guardrail |
| `DB_POOL_MAX_LIFETIME_SECONDS` | `optional` | VM repo env now; Cloud Run plain env later | shared DB connection lifetime cap |

Placement rules for sensitive runtime keys:

| Key | Active placement | Future Cloud Run placement | Notes |
|---|---|---|---|
| `DATABASE_URL` | VM repo env | GSM | credential-bearing |
| `SESSION_SECRET` | VM repo env | GSM | signing secret |
| `INTERNAL_IDENTITY_SECRET` | VM repo env | GSM | internal auth secret |
| `MASTER_KEY` | VM repo env | GSM | envelope key |
| `RESEND_API_KEY` | VM repo env | GSM | provider secret |
| `REDIS_URL` | VM repo env | Cloud Run plain env | stays outside GSM unless credentials force a tradeoff |

## Deploy Env (`env/prod/deploy.env.example`)

Reference-only mirror of the active VM deploy controls.
Canonical values are maintained in GitHub environment variables, not committed env files.

VM baseline expectation:
- active `/swapfile` at `2G`
- persisted `vm.swappiness=10`
- persisted `vm.vfs_cache_pressure=50`

VM secret file expectation:
- `VM_SECRET_ENV_FILE` points to an on-host `0600` env file
- deploy script creates this file when missing and persists `BOMINAL_DATABASE_URL`
- deploy script reads optional GHCR credentials from this file for private image pulls

| Key | Classification | Placement | Rationale |
|---|---|---|---|
| `AUTO_DEPLOY_MAIN` | `optional` | GitHub production env | enables deploy on push to `main` when explicitly `true` |
| `GCP_PROJECT_ID` | `required` | GitHub production env | required for workload identity and VM target project |
| `GCP_WORKLOAD_IDENTITY_PROVIDER` | `required` | GitHub production env | required for OIDC federation |
| `GCP_SERVICE_ACCOUNT` | `required` | GitHub production env | required for federated deploy identity |
| `DEPLOY_VM_NAME` | `required` | GitHub production env | required VM target |
| `DEPLOY_VM_ZONE` | `required` | GitHub production env | required VM zone |
| `DEPLOY_WORKDIR` | `required` | GitHub production env | required working directory for remote execution |
| `DEPLOY_SCRIPT_PATH` | `required` | GitHub production env | fail-closed deploy script entrypoint |
| `DEPLOY_HEALTHCHECK_SCRIPT_PATH` | `required` | GitHub production env | fail-closed post-deploy verification entrypoint |
| `DEPLOY_ROLLBACK_SCRIPT_PATH` | `required` | GitHub production env | rollback entrypoint used when healthcheck fails |
| `VM_SECRET_ENV_FILE` | `required` | GitHub production env | on-host secret env path for DB credential material |
| `DEPLOY_RUNTIME_ENV_FILE` | `required` | GitHub production env | runtime env file updated by deploy script |
| `DEPLOY_COMPOSE_FILE` | `required` | GitHub production env | compose file used for pull/up and health verification |
| `DEPLOY_MIGRATIONS_DIR` | `required` | GitHub production env | migration directory consumed during deploy before service restart |
| `DEPLOY_API_SERVICE` | `required` | GitHub production env | compose service identifier for API |
| `DEPLOY_WORKER_SERVICE` | `required` | GitHub production env | compose service identifier for worker |
| `DEPLOY_HEALTHCHECK_LIVE_URL` | `required` | GitHub production env | live endpoint checked after deploy |
| `DEPLOY_HEALTHCHECK_READY_URL` | `required` | GitHub production env | ready endpoint checked after deploy |
| `POSTGRES_HOST` | `required` | GitHub production env | deploy-time Postgres host contract |
| `POSTGRES_PORT` | `required` | GitHub production env | deploy-time Postgres port contract |
| `POSTGRES_DB` | `required` | GitHub production env | deploy-time Postgres DB name contract |
| `POSTGRES_USER` | `required` | GitHub production env | deploy-time Postgres username contract |

## Future Cloud Run Bootstrap Env (`env/prod/cloudrun-api.env.example`)

Active use:
- none; this is future cutover prep only

Bootstrap flow:
- generate `env/prod/cloudrun-api.env` with `./scripts/bootstrap-prod.sh --only cloudrun-api`
- render a service manifest with `./runtime/cloudrun/api/bootstrap.sh --env-file env/prod/cloudrun-api.env`

Plain env values that belong in `cloudrun-api.env`:
- `USER_APP_HOST`
- `ADMIN_APP_HOST`
- `SESSION_COOKIE_DOMAIN`
- `INVITE_BASE_URL`
- `EMAIL_FROM_ADDRESS`
- `WEBAUTHN_RP_ID`
- `WEBAUTHN_RP_ORIGIN`
- `REDIS_URL`
- `HTTP_REQUEST_TIMEOUT_SECONDS`
- `HTTP_REQUEST_BODY_LIMIT_BYTES`
- `HTTP_CONCURRENCY_LIMIT`
- `PASSWORD_HASH_CONCURRENCY`
- `API_DB_POOL_MAX_CONNECTIONS`
- `DB_POOL_ACQUIRE_TIMEOUT_SECONDS`
- `DB_POOL_IDLE_TIMEOUT_SECONDS`
- `DB_POOL_MAX_LIFETIME_SECONDS`

GSM secret names that belong in `cloudrun-api.env`:
- `GSM_DATABASE_URL_SECRET`
- `GSM_SESSION_SECRET_SECRET`
- `GSM_INTERNAL_IDENTITY_SECRET_SECRET`
- `GSM_MASTER_KEY_SECRET`
- `GSM_RESEND_API_KEY_SECRET`

## Caddy Env (`env/prod/caddy.env*`)

| Key | Classification | Placement | Rationale |
|---|---|---|---|
| `CADDY_ACME_EMAIL` | `required` | VM repo env | required for TLS site and ACME identity |
| `CADDY_SITE_ADDRESS` | `required` | VM repo env | required for TLS site and ACME identity |

## GitHub Actions Production Environment Vars (`.github/workflows/cd.yml`)

`cd.yml` currently consumes only the active VM deploy keys listed in `env/prod/deploy.env.example`.
It does not read future Cloud Run bootstrap variables.

## Mandatory Production Guards

- Keep all git-tracked env examples placeholder-only.
- Keep active VM secrets out of git and logs.
- Keep future Cloud Run within the `5`-secret GSM policy.
- If `REDIS_URL` ever needs GSM because it starts carrying credentials, remove another GSM secret first.
- Protect deploy variables in GitHub `production` environment and require reviewer approval.
