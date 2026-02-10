# Deployment

This project supports separated dev/prod compose stacks with zero-downtime deployment.

## Environments

- Dev compose: `infra/docker-compose.yml`
- Prod compose: `infra/docker-compose.prod.yml`
- Dev env files: `infra/env/dev/*`
- Prod env files: `infra/env/prod/*.example` -> copy to real `.env` files

---

## Zero-Downtime Deployment

### How It Works

The deployment uses Docker Compose health checks and the `--wait` flag:

1. **Build Phase** - New images are built while old containers keep running (no downtime)
2. **Deploy Phase** - Each service is deployed with `--wait`:
   - Docker starts the new container
   - Health check runs repeatedly until container is healthy
   - Only after new container is healthy does Docker stop the old one
3. **Verify Phase** - External health checks confirm the deployment

### Health Check Configuration

Each service has a health check in `docker-compose.prod.yml`:

| Service  | Health Check | Start Period |
|----------|--------------|--------------|
| postgres | `pg_isready` | 0s |
| redis    | `redis-cli ping` | 0s |
| api      | Python urllib (port 8000/health) | 30s |
| worker   | Python proc check for arq | 15s |
| web      | `wget --spider` (port 3000) | 60s |
| caddy    | `wget` (admin API port 2019) | 30s |

The `start_period` gives the container time to initialize before health checks begin counting failures.

> **Note**: Health checks use Python for api/worker (slim images lack curl/pgrep)
> and 127.0.0.1 instead of localhost (Alpine DNS issue).

### Deploy Commands

**Standard Zero-Downtime Deploy** (on VM):

```bash
# Deploy current main branch
sudo -u bominal /opt/bominal/repo/infra/scripts/deploy-zero-downtime.sh

# Deploy specific commit
sudo -u bominal /opt/bominal/repo/infra/scripts/deploy-zero-downtime.sh abc123f

# Check deployment status
sudo -u bominal /opt/bominal/repo/infra/scripts/deploy-zero-downtime.sh --status
```

**CI Deploy (Recommended: Pub/Sub, no SSH)**

GitHub Actions publishes a deploy request to Pub/Sub (authenticated via WIF).
The VM runs a pull-based deploy agent (systemd) that consumes the request and
runs `infra/scripts/deploy-zero-downtime.sh` locally.

CI-triggered deploys are **latest-only** (the message includes the triggering commit SHA for audit, but the VM deploys `:latest` images).

One-time GCP setup:

```bash
# Topic (publisher: GitHub Actions)
gcloud pubsub topics create bominal-deploy-requests --project bominal

# Subscription (consumer: VM)
gcloud pubsub subscriptions create bominal-deploy-requests-vm \
  --project bominal \
  --topic bominal-deploy-requests \
  --ack-deadline 600

# Allow GitHub Actions SA to publish
gcloud pubsub topics add-iam-policy-binding bominal-deploy-requests \
  --project bominal \
  --member="serviceAccount:github-actions@bominal.iam.gserviceaccount.com" \
  --role="roles/pubsub.publisher"

# Allow the VM service account to pull/ack (replace with the instance SA email)
gcloud compute instances describe bominal-deploy --zone us-central1-a \
  --format='value(serviceAccounts.email)'

gcloud pubsub subscriptions add-iam-policy-binding bominal-deploy-requests-vm \
  --project bominal \
  --member="serviceAccount:<VM_SERVICE_ACCOUNT_EMAIL>" \
  --role="roles/pubsub.subscriber"
```

VM install (one-time):

```bash
# Copy systemd unit
sudo cp /opt/bominal/repo/infra/systemd/bominal-deploy-agent.service /etc/systemd/system/

# Configure agent env
sudo mkdir -p /etc/bominal
sudo tee /etc/bominal/deploy-agent.env >/dev/null <<'EOF'
GCP_PROJECT_ID=bominal
GCP_REGION=us-central1
DEPLOY_SUBSCRIPTION=bominal-deploy-requests-vm
REPO_DIR=/opt/bominal/repo
DEPLOY_SCRIPT=/opt/bominal/repo/infra/scripts/deploy-zero-downtime.sh
EOF

sudo systemctl daemon-reload
sudo systemctl enable --now bominal-deploy-agent
sudo journalctl -u bominal-deploy-agent -f
```

**Remote Deploy** (from local machine):

```bash
gcloud compute ssh bominal-deploy --zone=us-central1-a --tunnel-through-iap \
  --command="cd /opt/bominal/repo && sudo -u bominal infra/scripts/deploy-zero-downtime.sh"
```

---

## Rollback Procedures

### Automatic Rollback

The deployment script tracks versions in `/opt/bominal/deployments/`:

```bash
# Rollback to previous deployment
sudo -u bominal /opt/bominal/repo/infra/scripts/deploy-zero-downtime.sh --rollback
```

### Manual Rollback

If the script fails, manually rollback:

```bash
cd /opt/bominal/repo

# Find the previous good commit
cat /opt/bominal/deployments/previous
# Or: git log --oneline -10

# Checkout and redeploy
sudo -u bominal git checkout <commit>
sudo docker compose -f infra/docker-compose.prod.yml build api web
sudo docker compose -f infra/docker-compose.prod.yml up -d --wait
```

### Database Migration Rollback

If a migration was applied and needs reverting:

```bash
# 1. Check current migration state
sudo docker compose -f infra/docker-compose.prod.yml exec api alembic current

# 2. Downgrade to specific revision
sudo docker compose -f infra/docker-compose.prod.yml exec api alembic downgrade <revision>

# 3. Checkout old code and redeploy
sudo -u bominal git checkout <commit>
sudo docker compose -f infra/docker-compose.prod.yml build api
sudo docker compose -f infra/docker-compose.prod.yml up -d --wait api worker
```

### Version Tracking

The deployment system maintains:

| File | Purpose |
|------|---------|
| `/opt/bominal/deployments/current` | Currently deployed commit hash |
| `/opt/bominal/deployments/previous` | Previous deployment (for rollback) |
| `/opt/bominal/deployments/<timestamp>` | Historical deployment records |

Each `<timestamp>` record includes the deployed commit and image references. Newer
records also include `api_digest` / `web_digest` (immutable `name@sha256:...`)
which the rollback path prefers for deterministic rollbacks (even if `:latest`
has moved).

Only the last 10 deployment records are kept.

---

## Production Bootstrap (GCE e2-micro)

### 1) Create env files

```bash
cp infra/env/prod/postgres.env.example infra/env/prod/postgres.env
cp infra/env/prod/api.env.example infra/env/prod/api.env
cp infra/env/prod/web.env.example infra/env/prod/web.env
cp infra/env/prod/caddy.env.example infra/env/prod/caddy.env
```

### 2) Configure secrets

Replace all `CHANGE_ME...` values, including `INTERNAL_API_KEY` in `api.env`.

Generate secure `MASTER_KEY`:

```bash
openssl rand -base64 32
```

### 3) Run predeploy checks

```bash
bash infra/scripts/predeploy-check.sh
```

Optional manual pre-migration duplicate check:

```bash
docker compose -f infra/docker-compose.prod.yml run --rm api python scripts/check_duplicate_display_names.py
```

### 4) Initial Deploy

```bash
bash infra/scripts/deploy-zero-downtime.sh
```

The API startup runs duplicate `display_name` check before `alembic upgrade head`.

## GCE e2-micro (Free Tier) Setup

The e2-micro has 1GB RAM and 0.25 vCPU. Key optimizations:

- **Swap**: 1GB swap file created by `vm-docker-bootstrap.sh`
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
bash infra/scripts/deploy-zero-downtime.sh
```

This will:
1. Pull latest code
2. Build all container images (takes ~5-10 min on e2-micro)
3. Start services with health check verification
4. Wait until all containers are healthy before completing

### 4) Verify

```bash
curl -sS https://www.bominal.com/health
docker compose -f infra/docker-compose.prod.yml logs --tail=50
```

---

## Monitoring Deployment Health

### Container Health

```bash
# Check all container health statuses
docker ps --format "table {{.Names}}\t{{.Status}}"

# Watch health in real-time
watch -n2 'docker ps --format "table {{.Names}}\t{{.Status}}" | grep bominal'
```

### Service Health Endpoints

```bash
# API health
curl -s http://127.0.0.1:8000/health

# Web via Caddy
curl -sI http://127.0.0.1/

# Full external check
curl -sI https://www.bominal.com/
```

### Logs During Deploy

```bash
# Follow logs during deployment
sudo docker compose -f infra/docker-compose.prod.yml logs -f --tail=50 api web
```

---

## Troubleshooting

### Deploy Hangs on Health Check

If deployment hangs waiting for a container to become healthy:

1. Check container logs:
   ```bash
   docker logs bominal-api --tail=100
   ```

2. Check what's failing:
   ```bash
   docker inspect bominal-api --format='{{json .State.Health}}'
   ```

3. Force restart if needed:
   ```bash
   docker compose -f infra/docker-compose.prod.yml restart api
   ```

### Container Starts but Unhealthy

Common causes:
- **API**: Database migration failed, missing env vars
- **Web**: Build failed, missing env vars
- **Worker**: Redis connection failed

Check logs:
```bash
docker logs bominal-api 2>&1 | tail -50
docker logs bominal-web 2>&1 | tail -50
```

### Rollback Not Working

If deployment history is corrupted:

```bash
# Find recent commits
cd /opt/bominal/repo
git log --oneline -20

# Manually set version pointers
echo "abc123f" > /opt/bominal/deployments/current
echo "def456a" > /opt/bominal/deployments/previous

# Then retry rollback
sudo -u bominal infra/scripts/deploy-zero-downtime.sh --rollback
```

Malformed historical records (e.g. bad files under `/opt/bominal/deployments/<timestamp>`)
should not break `--status` or deploy/rollback runs; the script will warn and skip
records it can’t read.

---

## Network Guidance

- One Debian VM running Docker/Compose
- Caddy terminates TLS via ACME (Let's Encrypt)
- Containers private on bridge network; expose only 80/443 publicly

Firewall rules:

- VM tags: `bominal-web`, `bominal-ops`
- Public ingress: TCP `443` and `80` (for ACME challenge + HTTP->HTTPS redirect)
- SSH via IAP only (`35.235.240.0/20`)
- Do not open Postgres/Redis/API/Web internal ports publicly

## Domain Layout

Current Caddy setup is single-host path routing:

- `www.bominal.com` -> web (`3000`) for UI routes
- `www.bominal.com/api/*` -> api (`8000`)
- `www.bominal.com/health` and `/healthz` -> api health endpoints
- `bominal.com` -> redirects to `www.bominal.com`

Configure domain + ACME contact in `infra/env/prod/caddy.env`:

- `CADDY_SITE_ADDRESS=www.bominal.com`
- `CADDY_ACME_EMAIL=ops@bominal.com`

---

## For AI Agents

> **IMPORTANT**: Read this section before making deployment changes.

### Deployment Rules

1. **Always use the zero-downtime script** for production deploys:
   ```bash
   sudo -u bominal /opt/bominal/repo/infra/scripts/deploy-zero-downtime.sh
   ```

2. **Never modify these files** without explicit user approval:
   - `infra/scripts/deploy-zero-downtime.sh`
   - `infra/docker-compose.prod.yml` (health checks section)
   - `/opt/bominal/deployments/*` (version tracking)

3. **Preserve version history** - the deployment system tracks:
   - `/opt/bominal/deployments/current` - current commit
   - `/opt/bominal/deployments/previous` - for rollback

4. **Test changes locally first** when possible:
   ```bash
   docker compose -f infra/docker-compose.yml up --build
   ```

5. **Always verify after deploy**:
   ```bash
   curl -s https://www.bominal.com/health
   docker ps --format "table {{.Names}}\t{{.Status}}" | grep bominal
   ```

### Rollback Procedure for Agents

If a deployment causes issues:

```bash
# Automatic rollback (preferred)
sudo -u bominal /opt/bominal/repo/infra/scripts/deploy-zero-downtime.sh --rollback

# Manual rollback if automatic fails
cd /opt/bominal/repo
cat /opt/bominal/deployments/previous  # Get previous commit
sudo -u bominal git checkout <commit>
sudo docker compose -f infra/docker-compose.prod.yml build api web
sudo docker compose -f infra/docker-compose.prod.yml up -d --wait
```

### What Triggers Downtime

These actions **will cause downtime**:
- `docker compose down` (stops all containers)
- `docker compose restart` without health checks
- Breaking migrations without rollback plan

These actions are **zero-downtime**:
- `deploy-zero-downtime.sh` (uses health checks)
- `docker compose up -d --wait` (waits for healthy)
- Code changes without schema changes

---

## Scaling Path

Phase 1 (current): single VM with zero-downtime deploys.

Phase 2:
- Move Postgres to managed DB (Cloud SQL)
- Move Redis to managed Redis (Memorystore)
- Keep API/worker on Compute Engine

Phase 3:
- Add load balancer with multiple app nodes
- Blue-green or canary deployments
- CI/CD with migration gating + secret manager integration
