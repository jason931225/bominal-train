# AGENTS.md

Guidance for AI/code agents working in this repository.

## Mission

Build and operate **bominal**, a modular product platform with:

- `web/` (Next.js App Router + TypeScript + Tailwind)
- `api/` (FastAPI + Postgres + Alembic)
- `worker` (arq background jobs)
- `redis` (queue + train-provider rate limiter)
- `third_party/srtgo` (read-only train provider reference)

## Non-negotiables

1. Preserve product name as `bominal` in UI, config, and docs.
2. Treat `third_party/srtgo` as read-only. Never patch or reformat it.
3. Keep provider integrations source-aligned with `srtgo/srt.py` and `srtgo/ktx.py`.
4. Never log sensitive data (passwords, tokens, card data, CVV, full provider payloads with secrets).
5. Keep session auth cookie behavior:
   - httpOnly
   - SameSite=Lax
   - Secure only in production

## First files to read

1. `docs/README.md`
2. `docs/ARCHITECTURE.md`
3. `docs/CONTRIBUTING.md`
4. `docs/DEPLOYMENT.md` (for production deploys and rollbacks)
5. `docs/RUNBOOK.md`
6. `README.md`

## Repo map

- `web/app/` page routes
- `web/components/` UI and feature components
- `web/lib/` shared client/server helpers and design tokens
- `api/app/api/routes/` HTTP routes
- `api/app/modules/train/` train domain (router, service, worker, providers)
- `api/app/core/crypto/` envelope encryption and redaction
- `api/app/db/` SQLAlchemy models/session
- `api/alembic/versions/` DB migrations
- `infra/` compose files, env files, predeploy checks

## Local workflow

```bash
git submodule update --init --recursive
docker-compose -f infra/docker-compose.yml up --build
```

Use these for focused verification:

```bash
docker-compose -f infra/docker-compose.yml exec api pytest -q
docker-compose -f infra/docker-compose.yml exec web npx tsc --noEmit
```

## Change strategy

1. Keep API handlers thin; heavy train logic belongs in service/worker.
2. Prefer additive migrations; do not mutate old migrations.
3. Reuse `web/lib/ui.ts` classes for consistent UI.
4. Keep train times in KST in user-facing UI.
5. Use explicit safe metadata fields (`*_safe`) when storing provider response details.

## Definition of done for feature work

- Build compiles (web typecheck, backend imports cleanly).
- Relevant backend tests pass.
- Docker compose stack starts cleanly.
- No broken auth/session flow regressions.
- No unresolved placeholders in production env templates.
- Docs updated in `docs/` when behavior or operations change.

## Production Deployment (IMPORTANT)

**Always use zero-downtime deployment** for production:

```bash
# On VM (via SSH)
sudo -u bominal /opt/bominal/repo/infra/scripts/deploy-zero-downtime.sh

# Remote deploy
gcloud compute ssh bominal-deploy --zone=us-central1-a --tunnel-through-iap \
  --command="cd /opt/bominal/repo && sudo -u bominal infra/scripts/deploy-zero-downtime.sh"
```

**Rollback if needed**:

```bash
sudo -u bominal /opt/bominal/repo/infra/scripts/deploy-zero-downtime.sh --rollback
```

**Never**:
- Use `docker compose down` in production (causes downtime)
- Modify `/opt/bominal/deployments/*` version tracking files
- Skip health check verification after deploy

**Always**:
- Use `--wait` flag with `docker compose up -d`
- Verify health after deploy: `curl https://www.bominal.com/health`
- Keep rollback info handy before deploying

See `docs/DEPLOYMENT.md` for full procedures.

