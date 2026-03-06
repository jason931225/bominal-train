# Rust Production Cutover Playbook

This playbook is the deterministic path to run the Rust API/SSR service on Cloud Run while keeping the Rust worker, Redis, and Postgres on the Compute Engine VM.

## Preconditions

- GitHub `production` environment exists.
- VM host is reachable via IAP SSH.
- VM has Docker available.
- VM has repo checkout at `/opt/bominal/repo` (or adjust paths consistently).

## 1) Set GitHub Production Variables

Required keys consumed by `.github/workflows/cd.yml`:

- `AUTO_DEPLOY_MAIN`
- `GCP_PROJECT_ID`
- `GCP_REGION`
- `GCP_WORKLOAD_IDENTITY_PROVIDER`
- `GCP_SERVICE_ACCOUNT`
- `ARTIFACT_REGISTRY_REPOSITORY`
- `CLOUDRUN_API_SERVICE`
- `CLOUDRUN_API_SERVICE_ACCOUNT`
- `CLOUDRUN_API_VPC_NETWORK`
- `CLOUDRUN_API_VPC_SUBNET`
- `USER_APP_HOST`
- `ADMIN_APP_HOST`
- `SESSION_COOKIE_DOMAIN`
- `INVITE_BASE_URL`
- `EMAIL_FROM_ADDRESS`
- `WEBAUTHN_RP_ID`
- `WEBAUTHN_RP_ORIGIN`
- `DEPLOY_VM_NAME`
- `DEPLOY_VM_ZONE`
- `DEPLOY_WORKDIR`
- `DEPLOY_SCRIPT_PATH`
- `DEPLOY_HEALTHCHECK_SCRIPT_PATH`
- `DEPLOY_ROLLBACK_SCRIPT_PATH`
- `VM_SECRET_ENV_FILE`
- `DEPLOY_RUNTIME_ENV_FILE`
- `DEPLOY_COMPOSE_FILE`
- `DEPLOY_MIGRATIONS_DIR`
- `DEPLOY_WORKER_SERVICE`
- `POSTGRES_HOST`
- `POSTGRES_PORT`
- `POSTGRES_DB`
- `POSTGRES_USER`

Optional keys:

- `CLOUDRUN_API_VPC_NETWORK_TAGS`
- `DEPLOY_COMPOSE_PROJECT_NAME`
- `DEPLOY_ROLLBACK_STATE_PATH`
- `DEPLOY_VM_BASELINE_SCRIPT`
- `DEPLOY_HEALTHCHECK_RETRIES`
- `DEPLOY_HEALTHCHECK_DELAY_SECONDS`

## 2) Prepare Cloud Run Secret Manager State

Create these Secret Manager secrets in the deploy project. The checked-in Cloud Run service definition expects these exact names:

- `DATABASE_URL`
- `REDIS_URL`
- `SESSION_SECRET`
- `INTERNAL_IDENTITY_SECRET`
- `MASTER_KEY`
- `RESEND_API_KEY`

## 3) Prepare VM Runtime Env

Create and lock down runtime env file:

```bash
sudo install -d -m 0755 /opt/bominal/repo/env/prod
sudo touch /opt/bominal/repo/env/prod/runtime.env
sudo chown root:root /opt/bominal/repo/env/prod/runtime.env
sudo chmod 0600 /opt/bominal/repo/env/prod/runtime.env
```

Populate keys from `env/prod/runtime.env.example` for the worker/VM runtime:

- required worker/database keys: `DATABASE_URL`, `WORKER_DB_POOL_MAX_CONNECTIONS`, `DB_POOL_*`
- required cache/runtime keys: `REDIS_URL`, `RUNTIME_QUEUE_*`, `RUNTIME_LEASE_PREFIX`, `RUNTIME_RATE_LIMIT_PREFIX`
- required auth/crypto/provider keys shared with worker flows: `INTERNAL_IDENTITY_SECRET`, `MASTER_KEY`, `EMAIL_FROM_ADDRESS`, `RESEND_API_KEY`
- recommended worker cadence keys: `WORKER_POLL_SECONDS=1`, `WORKER_RECONCILE_SECONDS=30`, `WORKER_WATCH_SECONDS=60`

## 4) Prepare VM Secret Env File

Create and lock down secret file expected by deploy scripts (recommended; deploy script can auto-create it if path is writable):

```bash
sudo install -d -m 0755 /opt/bominal/env/prod
sudo touch /opt/bominal/env/prod/vm-secrets.env
sudo chown root:root /opt/bominal/env/prod/vm-secrets.env
sudo chmod 0600 /opt/bominal/env/prod/vm-secrets.env
```

Set one database secret mode:

- `BOMINAL_DATABASE_URL=postgresql://...`
- or `BOMINAL_POSTGRES_PASSWORD=...`

## 5) Validate On-Host Artifacts

Required files:

- `/opt/bominal/repo/runtime/compose.prod.yml`
- `/opt/bominal/repo/runtime/cloudrun/api/service.yaml`
- `/opt/bominal/repo/scripts/prod/deploy-runtime.sh`
- `/opt/bominal/repo/scripts/prod/apply-migrations.sh`
- `/opt/bominal/repo/scripts/prod/healthcheck-runtime.sh`
- `/opt/bominal/repo/scripts/prod/rollback-runtime.sh`
- `/opt/bominal/repo/runtime/migrations/*.sql`

## 6) Execute Cutover

- Trigger `CD` workflow on `main` with `deploy=true`, or push to `main` with `AUTO_DEPLOY_MAIN=true`.
- Confirm sequence in logs:
  - image build/push + digest refs
  - API image copy from GHCR to Artifact Registry
  - Cloud Run service replace + public invoker binding
  - Cloud Run smoke checks (`/health`, `/ready`)
  - remote worker deploy script
  - migration application
  - worker restart
  - final worker + API health checks

VM-local manual fallback (explicit):

```bash
cd /opt/bominal/repo
export BOMINAL_HEALTHCHECK_LIVE_URL="$(gcloud run services describe bominal-api --region us-central1 --format='value(status.url)')/health"
export BOMINAL_HEALTHCHECK_READY_URL="$(gcloud run services describe bominal-api --region us-central1 --format='value(status.url)')/ready"
./scripts/prod-up.sh deploy --yes
```

## 7) Rollback Procedure

Automatic rollback behavior:

- Cloud Run traffic rolls back to the previous ready revision if the post-deploy Cloud Run smoke test fails.
- VM worker rollback runs if the final worker health-check step fails after the worker deploy.

Manual rollback command:

```bash
cd /opt/bominal/repo
export BOMINAL_HEALTHCHECK_LIVE_URL="$(gcloud run services describe bominal-api --region us-central1 --format='value(status.url)')/health"
export BOMINAL_HEALTHCHECK_READY_URL="$(gcloud run services describe bominal-api --region us-central1 --format='value(status.url)')/ready"
./scripts/prod-up.sh rollback --yes
```

## 8) Post-Cutover Verification

- API liveness: `curl -fsS "$(gcloud run services describe bominal-api --region us-central1 --format='value(status.url)')/health"`
- API readiness: `curl -fsS "$(gcloud run services describe bominal-api --region us-central1 --format='value(status.url)')/ready"`
- Worker service: `docker compose -f /opt/bominal/repo/runtime/compose.prod.yml ps worker`

Day-2 operator checks:

```bash
cd /opt/bominal/repo
./scripts/prod-up.sh status
./scripts/prod-up.sh health
./scripts/prod-up.sh logs -f --since 30m --service worker
```
