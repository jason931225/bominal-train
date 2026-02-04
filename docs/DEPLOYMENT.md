# Deployment

Bominal deployment is split into two practical stages:

- local production simulation (`build` on your machine)
- VM production deployment (`pull && up -d` on VM)

## Recommended VM baseline (free-tier friendly)

- GCP zone: `us-central1-a`
- Machine type: `e2-micro`
- Disk: `pd-standard`, 10GB
- OS image: `debian-12` (minimal/default Debian)
- VM tags: `bominal-web`, `bominal-ops`

## Stage A: local production simulation

```bash
cp infra/env/prod/postgres.env.example infra/env/prod/postgres.env
cp infra/env/prod/api.env.example infra/env/prod/api.env
cp infra/env/prod/web.env.example infra/env/prod/web.env
bash infra/scripts/predeploy-check.sh
docker compose -f infra/docker-compose.prod.yml up -d --build
```

## Stage B: VM production deploy (CI images + compose pull/up)

### 1) Build images in CI

Workflow file:

- `.github/workflows/build-images.yml`

Produced images:

- `ghcr.io/<owner>/bominal-api:<tag>`
- `ghcr.io/<owner>/bominal-web:<tag>`

Worker uses the same API image.

### 2) Fresh VM bootstrap (Debian 12)

After copying repo to VM:

```bash
sudo /opt/bominal/repo/infra/scripts/vm-docker-bootstrap.sh
```

### 3) Configure prod envs

```bash
cp infra/env/prod/postgres.env.example infra/env/prod/postgres.env
cp infra/env/prod/api.env.example infra/env/prod/api.env
cp infra/env/prod/web.env.example infra/env/prod/web.env
```

Replace all `CHANGE_ME...` values, especially:

- `MASTER_KEY`
- DB password fields in `api.env` and `postgres.env`

### 4) Deploy images

```bash
sudo -u bominal /opt/bominal/repo/infra/scripts/vm-docker-deploy.sh latest
```

Or deploy a specific CI tag:

```bash
sudo -u bominal /opt/bominal/repo/infra/scripts/vm-docker-deploy.sh sha-<git_sha>
```

### 5) Verify

```bash
curl -sS https://www.bominal.com/health
curl -sS -I https://www.bominal.com/login
docker compose -f infra/docker-compose.deploy.yml ps
```

## DNS and Cloudflare

- `A @ -> <VM_PUBLIC_IP>`
- `CNAME www -> @`

Start DNS-only while validating; move to proxied once health checks and login are stable.

## Rollback

Redeploy a previous immutable image tag:

```bash
sudo -u bominal /opt/bominal/repo/infra/scripts/vm-docker-deploy.sh sha-<older_sha>
```
