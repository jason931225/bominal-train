# Deployment

This project supports separated dev/prod compose stacks.

## Environments

- Dev compose: `infra/docker-compose.yml`
- Prod compose: `infra/docker-compose.prod.yml`
- Dev env files: `infra/env/dev/*`
- Prod env files: `infra/env/prod/*.example` -> copy to real `.env` files

## Production bootstrap

1. Create env files:

```bash
cp infra/env/prod/postgres.env.example infra/env/prod/postgres.env
cp infra/env/prod/api.env.example infra/env/prod/api.env
cp infra/env/prod/web.env.example infra/env/prod/web.env
```

2. Replace all `CHANGE_ME...` values.

3. Generate secure `MASTER_KEY`:

```bash
openssl rand -base64 32
```

4. Run predeploy checks:

```bash
bash infra/scripts/predeploy-check.sh
```

5. Deploy:

```bash
docker-compose -f infra/docker-compose.prod.yml up -d --build
```

## GCP single-VM baseline (recommended first deployment)

Suggested approach:

- One Ubuntu VM running Docker/Compose
- Reverse proxy (Caddy or Nginx) terminates TLS
- Containers private on bridge network; expose only 80/443 publicly

Network guidance:

- VM tags: `bominal-web`, `bominal-ops`
- Public ingress: TCP `443` (and optionally `80` for redirect)
- SSH via IAP only (`35.235.240.0/20`)
- Do not open Postgres/Redis/API internal ports publicly

## Domain layout

You can start with one domain:

- `app.bominal.com` -> reverse proxy -> web (`3000`) and api (`8000`) by path or host routing

Or split hosts:

- `app.bominal.com` (web)
- `api.bominal.com` (api)

## Post-deploy smoke test

```bash
curl -sS http://localhost:8000/health
curl -sS -I http://localhost:3000
docker-compose -f infra/docker-compose.prod.yml ps
docker-compose -f infra/docker-compose.prod.yml logs --tail=150 api worker web
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

