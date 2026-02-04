# Deployment

This project supports separated dev/prod compose stacks.

## Environments

- Dev compose: `infra/docker-compose.yml`
- Prod compose: `infra/docker-compose.prod.yml`
- Dev env files: `infra/env/dev/*`
- Prod env files: `infra/env/prod/*.example` -> copy to real `.env` files

## Production bootstrap (GCE e2-micro)

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
bash infra/scripts/deploy.sh
```

Or manually:

```bash
docker compose -f infra/docker-compose.prod.yml build --pull
docker compose -f infra/docker-compose.prod.yml up -d --remove-orphans
```

The API startup command runs the duplicate `display_name` check before `alembic upgrade head`.

## GCE e2-micro (Free Tier) Setup

The e2-micro has 1GB RAM and 0.25 vCPU. Key optimizations:

- **Swap**: 1GB swap file is created by `vm-docker-bootstrap.sh`
- **Memory limits**: All containers have memory caps to prevent OOM
- **Pre-built images**: `Dockerfile.prod` builds during `docker compose build`, not at runtime

### 1) One-time VM bootstrap

Run the bootstrap script as root:

```bash
sudo bash infra/scripts/vm-docker-bootstrap.sh
```

This installs Docker, creates a `bominal` user, and adds 1GB swap.

### 2) Clone repo and configure

```bash
sudo -u bominal -i
cd /opt/bominal
git clone https://github.com/<owner>/<repo>.git repo
cd repo
git submodule update --init --recursive
```

Create prod env files:

```bash
for f in infra/env/prod/*.env.example; do cp "$f" "${f%.example}"; done
```

Edit each file and replace all `CHANGE_ME` values:
- `infra/env/prod/postgres.env` - database credentials
- `infra/env/prod/api.env` - MASTER_KEY, INTERNAL_API_KEY, DATABASE_URL
- `infra/env/prod/web.env` - NEXT_PUBLIC_API_BASE_URL
- `infra/env/prod/caddy.env` - CADDY_SITE_ADDRESS, CADDY_ACME_EMAIL

Generate secrets:

```bash
# MASTER_KEY (for encryption)
openssl rand -base64 32

# INTERNAL_API_KEY
openssl rand -hex 32
```

### 3) Deploy

```bash
bash infra/scripts/deploy.sh
```

This will:
1. Pull latest code
2. Build all container images (takes ~5-10 min on e2-micro)
3. Start services with memory limits
4. Wait for health checks

### 4) Verify

```bash
curl -sS https://www.bominal.com/health
docker compose -f infra/docker-compose.prod.yml logs --tail=50
```

## Network guidance

- One Ubuntu VM running Docker/Compose
- Caddy terminates TLS via ACME (Let's Encrypt)
- Containers private on bridge network; expose only 80/443 publicly

Firewall rules:

- VM tags: `bominal-web`, `bominal-ops`
- Public ingress: TCP `443` and `80` (for ACME challenge + HTTP->HTTPS redirect)
- SSH via IAP only (`35.235.240.0/20`)
- Do not open Postgres/Redis/API/Web internal ports publicly

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
