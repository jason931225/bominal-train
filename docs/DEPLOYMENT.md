# Deployment

This project supports separated dev/prod compose stacks.

## Environments

- Dev compose: `infra/docker-compose.yml`
- Prod compose: `infra/docker-compose.prod.yml`
- CI/CD image-based deploy compose: `infra/docker-compose.deploy.yml`
- Dev env files: `infra/env/dev/*`
- Prod env files: `infra/env/prod/*.example` -> copy to real `.env` files

## Production bootstrap

1. Create env files:

```bash
cp infra/env/prod/postgres.env.example infra/env/prod/postgres.env
cp infra/env/prod/api.env.example infra/env/prod/api.env
cp infra/env/prod/web.env.example infra/env/prod/web.env
cp infra/env/prod/caddy.env.example infra/env/prod/caddy.env
```

2. Replace all `CHANGE_ME...` values.
   - Include a strong random value for `INTERNAL_API_KEY` in `infra/env/prod/api.env`.

3. Generate secure `MASTER_KEY`:

```bash
openssl rand -base64 32
```

4. Run predeploy checks:

```bash
bash infra/scripts/predeploy-check.sh
```

Optional manual pre-migration duplicate check (runs against current DB values):

```bash
docker compose -f infra/docker-compose.prod.yml run --rm api python scripts/check_duplicate_display_names.py
```

5. Deploy:

```bash
docker-compose -f infra/docker-compose.prod.yml up -d --build
```

The API startup command runs the duplicate `display_name` check before `alembic upgrade head`.

Image-based deploy (recommended for small VMs):

```bash
export BOMINAL_IMAGE_PREFIX=ghcr.io/your-org/bominal
export BOMINAL_IMAGE_TAG=latest
docker compose -f infra/docker-compose.deploy.yml pull
docker compose -f infra/docker-compose.deploy.yml up -d --remove-orphans
```

## GCP single-VM baseline (recommended first deployment)

Suggested approach:

- One Ubuntu VM running Docker/Compose
- Reverse proxy (Caddy or Nginx) terminates TLS
- Containers private on bridge network; expose only 80/443 publicly

Network guidance:

- VM tags: `bominal-web`, `bominal-ops`
- Public ingress: TCP `443` and `80` (for ACME challenge + HTTP->HTTPS redirect)
- SSH via IAP only (`35.235.240.0/20`)
- Do not open Postgres/Redis/API/Web internal ports publicly

## GitHub CI -> GCE (e2-micro)

This repo includes:

- Workflow: `.github/workflows/deploy-gce.yml`
- VM deploy script: `infra/scripts/deploy-gce.sh`
- Production image Dockerfiles:
  - `api/Dockerfile.prod`
  - `web/Dockerfile.prod`

### 1) One-time VM bootstrap

On the VM, install Docker/Compose and clone repo:

```bash
sudo apt-get update
sudo apt-get install -y docker.io docker-compose-plugin git
sudo usermod -aG docker "$USER"
sudo mkdir -p /opt
sudo chown -R "$USER":"$USER" /opt
git clone https://github.com/<owner>/<repo>.git /opt/bominal
cd /opt/bominal
git submodule update --init --recursive
```

If the repo is private, clone via SSH with a read-only GitHub deploy key installed on the VM:

```bash
git clone git@github.com:<owner>/<repo>.git /opt/bominal
```

(Recommended) Keep the GitHub deploy key **only** on the VM, and use a separate SSH key for CI -> VM access.

Create prod env files on the VM:

```bash
cp infra/env/prod/postgres.env.example infra/env/prod/postgres.env
cp infra/env/prod/api.env.example infra/env/prod/api.env
cp infra/env/prod/web.env.example infra/env/prod/web.env
cp infra/env/prod/caddy.env.example infra/env/prod/caddy.env
cp infra/env/prod/deploy.env.example infra/env/prod/deploy.env
```

Fill all production values (no `CHANGE_ME` placeholders).
Ensure `INTERNAL_API_KEY` is set in `infra/env/prod/api.env` before first deploy.
If you use `infra/scripts/deploy-gce.sh` manually, set `GHCR_USERNAME` and `GHCR_TOKEN` in `infra/env/prod/deploy.env` (or provide them as exported env vars).

### 2) GitHub secrets required

Set repository secrets:

- `GCE_HOST` - VM public IP or DNS
- `GCE_SSH_USER` - SSH user on VM
- `GCE_SSH_KEY` - private key for that user
- `GCE_DEPLOY_PATH` - repo path on VM (example: `/opt/bominal`)
- `GHCR_USERNAME` - GHCR user/org with read access
- `GHCR_TOKEN` - GHCR token (`read:packages`)

### 3) Deploy flow

On push to `main` (or manual run):

1. Runs API tests + web typecheck
2. Builds and pushes images to GHCR (`api`, `web`) tagged with commit SHA
3. SSHs to VM and runs `infra/scripts/deploy-gce.sh`
4. Pulls images and rolls containers with `infra/docker-compose.deploy.yml`

### 4) e2-micro notes

- Prefer image-based deploy (`docker-compose.deploy.yml`), not source builds on VM.
- Keep only required public ports open (usually `80/443`).
- Add swap on VM to reduce OOM risk under pressure.
- Run DB backups externally; do not rely on VM disk alone.

## Domain layout

Current Caddy setup is single-host path routing:

- `www.bominal.com` -> web (`3000`) for UI routes
- `www.bominal.com/api/*` -> api (`8000`)
- `www.bominal.com/health` and `/healthz` -> api health endpoints

Configure domain + ACME contact in `infra/env/prod/caddy.env`:

- `CADDY_SITE_ADDRESS=www.bominal.com`
- `CADDY_ACME_EMAIL=ops@bominal.com`

## Post-deploy smoke test

```bash
curl -sS http://localhost:8000/health
curl -sS -I http://localhost:3000
docker-compose -f infra/docker-compose.prod.yml ps
docker-compose -f infra/docker-compose.prod.yml logs --tail=150 caddy api worker web
```

## Rollback

If a new image is unhealthy:

1. Revert to previous commit/tag.
2. Rebuild/restart:

```bash
docker-compose -f infra/docker-compose.prod.yml up -d --build
```

3. Verify health endpoints and core routes again.

## Scaling path

Phase 1 (current): single VM.

Phase 2:

- Move Postgres to managed DB.
- Move Redis to managed Redis.
- Keep API/worker on Compute Engine or GKE.

Phase 3:

- Add load balancer/redundant app nodes.
- Formal CI/CD + migration gating + secret manager integration.
