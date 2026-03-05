# Rust Production Cutover Playbook

This playbook is the deterministic path to hard-cut traffic to Rust `api` + `worker` in production.

## Preconditions

- GitHub `production` environment exists.
- VM host is reachable via IAP SSH.
- VM has Docker available.
- VM has repo checkout at `/opt/bominal/repo` (or adjust paths consistently).

## 1) Set GitHub Production Variables

Required keys consumed by `.github/workflows/cd.yml`:

- `GCP_PROJECT_ID`
- `GCP_WORKLOAD_IDENTITY_PROVIDER`
- `GCP_SERVICE_ACCOUNT`
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
- `DEPLOY_API_SERVICE`
- `DEPLOY_WORKER_SERVICE`
- `DEPLOY_HEALTHCHECK_LIVE_URL`
- `DEPLOY_HEALTHCHECK_READY_URL`
- `POSTGRES_HOST`
- `POSTGRES_PORT`
- `POSTGRES_DB`
- `POSTGRES_USER`

Optional keys:

- `DEPLOY_COMPOSE_PROJECT_NAME`
- `DEPLOY_ROLLBACK_STATE_PATH`
- `DEPLOY_VM_BASELINE_SCRIPT`
- `DEPLOY_HEALTHCHECK_RETRIES`
- `DEPLOY_HEALTHCHECK_DELAY_SECONDS`

## 2) Prepare VM Runtime Env

Create and lock down runtime env file:

```bash
sudo install -d -m 0755 /opt/bominal/repo/env/prod
sudo touch /opt/bominal/repo/env/prod/runtime.env
sudo chown root:root /opt/bominal/repo/env/prod/runtime.env
sudo chmod 0600 /opt/bominal/repo/env/prod/runtime.env
```

Populate keys from `env/prod/runtime.env.example`:

- required auth/session keys: `SESSION_SECRET`, `INVITE_BASE_URL`
- required passkey keys: `PASSKEY_PROVIDER`, `WEBAUTHN_RP_ID`, `WEBAUTHN_RP_ORIGIN`, `WEBAUTHN_RP_NAME`
- required cryptographic keys: `MASTER_KEY`, `INTERNAL_IDENTITY_SECRET`
- required provider/email keys: `EMAIL_FROM_ADDRESS`, `RESEND_API_KEY`
- required database/cache keys: `DATABASE_URL`, `REDIS_URL`

## 3) Prepare VM Secret Env File

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

## 4) Validate On-Host Artifacts

Required files:

- `/opt/bominal/repo/runtime/compose.prod.yml`
- `/opt/bominal/repo/scripts/prod/deploy-runtime.sh`
- `/opt/bominal/repo/scripts/prod/apply-migrations.sh`
- `/opt/bominal/repo/scripts/prod/healthcheck-runtime.sh`
- `/opt/bominal/repo/scripts/prod/rollback-runtime.sh`
- `/opt/bominal/repo/runtime/migrations/*.sql`

## 5) Execute Cutover

- Trigger `CD` workflow on `main` with `deploy=true` (manual gate).
- Confirm sequence in logs:
  - image build/push + digest refs
  - remote deploy script
  - migration application
  - service restart
  - health checks (`/health`, `/ready`)

VM-local manual fallback (explicit):

```bash
cd /opt/bominal/repo
./scripts/prod-up.sh deploy --yes
```

## 6) Rollback Procedure

Automatic rollback runs when healthcheck step fails.

Manual rollback command:

```bash
cd /opt/bominal/repo
./scripts/prod-up.sh rollback --yes
```

## 7) Post-Cutover Verification

- API liveness: `curl -fsS http://127.0.0.1:8000/health`
- API readiness: `curl -fsS http://127.0.0.1:8000/ready`
- API service: `docker compose -f /opt/bominal/repo/runtime/compose.prod.yml ps api`
- Worker service: `docker compose -f /opt/bominal/repo/runtime/compose.prod.yml ps worker`

Day-2 operator checks:

```bash
cd /opt/bominal/repo
./scripts/prod-up.sh status
./scripts/prod-up.sh health
./scripts/prod-up.sh logs -f --since 30m --service api
```
