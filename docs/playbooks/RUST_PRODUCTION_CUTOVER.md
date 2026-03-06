# Rust Production Runtime and Future Cloud Run Cutover

This playbook separates the active VM deployment path from the future Cloud Run API cutover path.
Current production remains VM-first.

## Current Active Production Path

- API + worker + Postgres + Redis run on the VM.
- `.github/workflows/cd.yml` targets only this path.
- Operator entrypoint remains `./scripts/prod-up.sh`.

## Current VM Bootstrap

Generate the active VM env files:

```bash
./scripts/bootstrap-prod.sh --interactive
```

Current active outputs:
- `env/prod/runtime.env`
- `env/prod/caddy.env`
- `env/prod/deploy.env`
- `env/prod/vm-secrets.env`

Runtime and deploy steps remain unchanged:

```bash
cd /opt/bominal/repo
./scripts/prod-up.sh deploy --yes
./scripts/prod-up.sh rollback --yes
```

## Future Cloud Run API Cutover Prep

Goal:
- move only API/SSR to Cloud Run
- keep worker + Postgres + Redis on the VM
- make cutover an env switch plus a bootstrap/render step

### 1) Generate the Future Cloud Run Env

```bash
./scripts/bootstrap-prod.sh --only cloudrun-api --interactive
```

This writes:
- `env/prod/cloudrun-api.env`

### 2) Respect the Secret Boundary

Future Cloud Run GSM secrets are capped at `5` total:
- `DATABASE_URL`
- `SESSION_SECRET`
- `INTERNAL_IDENTITY_SECRET`
- `MASTER_KEY`
- `RESEND_API_KEY`

Rules:
- adding one GSM secret requires removing another
- `REDIS_URL` stays plain Cloud Run env by default
- active VM runtime secrets remain on the VM and out of git

### 3) Render the Cloud Run Service YAML

```bash
./runtime/cloudrun/api/bootstrap.sh --env-file env/prod/cloudrun-api.env
```

This renders:
- `runtime/cloudrun/api/rendered/<service>.yaml`

The script prints the next `gcloud run services replace ...` command, but does not deploy anything.

### 4) Worker/VM Alignment for Eventual Cutover

When cutover time arrives, apply the smaller runtime guardrails intentionally:
- `API_DB_POOL_MAX_CONNECTIONS`
- `WORKER_DB_POOL_MAX_CONNECTIONS`
- `DB_POOL_ACQUIRE_TIMEOUT_SECONDS`
- `DB_POOL_IDLE_TIMEOUT_SECONDS`
- `DB_POOL_MAX_LIFETIME_SECONDS`
- `WORKER_POLL_SECONDS`
- `WORKER_WATCH_SECONDS`

Those knobs exist now so the cutover does not require code changes.

## Verification Checklist

Current VM path:
- `./scripts/prod-up.sh health`
- `docker compose -f /opt/bominal/repo/runtime/compose.prod.yml ps`

Future Cloud Run prep path:
- `./scripts/bootstrap-prod.sh --only cloudrun-api --dry-run`
- `./runtime/cloudrun/api/bootstrap.sh --env-file env/prod/cloudrun-api.env --dry-run`
