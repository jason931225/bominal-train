# AGENTS.md

Guidance for automated/code agents working in this repository.

## Mission

Build and operate **bominal**, a modular product platform with:

- `web/` (Next.js App Router + TypeScript + Tailwind)
- `api/` (FastAPI + Postgres + Alembic)
- `worker` (arq background jobs)
- `redis` (queue + train-provider rate limiter)
- `third_party/srtgo` (read-only train provider reference)
- `third_party/catchtable` (read-only restaurant provider reference)

## Non-negotiables

1. Preserve product name `bominal` in UI, config, and docs.
2. Treat `third_party/srtgo` and `third_party/catchtable` as read-only.
3. Keep provider integrations source-aligned with `srtgo/srt.py` and `srtgo/ktx.py`.
4. Never log sensitive data (passwords, tokens, card data, CVV, full provider payloads with secrets).
5. Preserve session cookie behavior: `HttpOnly`, `SameSite=Lax`, and `Secure` only in production.
6. Follow governance precedence:
   1) `docs/agents/GUARDRAILS.md`
   2) `docs/agents/PERMISSIONS.md`
   3) `docs/agents/EXECUTION_PROTOCOL.md`
   4) `docs/governance/**`
   5) `docs/humans/**`
7. Keep `CHANGELOG.md` commit-based (Keep a Changelog categories under `## Unreleased`).
8. Follow `docs/governance/DOCUMENTATION_POLICY.md` for docs workflow.
9. For complex/repeatable tasks, use or create a playbook under `docs/playbooks/`.
10. Treat `third_party/**` docs as reference-only, never canonical policy.

## Required First Reads (Docs-First Gate)

Before planning or coding:

1. `docs/START_HERE.md`
2. `docs/README.md`
3. `docs/agents/README.md`
4. `docs/agents/GUARDRAILS.md`
5. `docs/agents/PERMISSIONS.md`
6. `docs/agents/EXECUTION_PROTOCOL.md`
7. `docs/INTENT_ROUTING.md`

Then read task-specific canonical docs from `docs/governance/**` and `docs/humans/**`.

## Docs-Last Gate (Mandatory)

Before completion/PR/merge:

1. Re-read relevant changed docs.
2. Ensure docs match final behavior.
3. Verify against execution protocol and active plan requirements.
4. Update `CHANGELOG.md` for notable in-scope changes.

## Pointer Convention (Mandatory)

Use `docs/README.md` as the canonical pointer library.
Any new canonical document or plan used for implementation must be added there before completion.

## Repo Map

- `web/app/` page routes
- `web/components/` UI and feature components
- `web/lib/` shared helpers and tokens
- `api/app/http/routes/` HTTP routes
- `api/app/modules/train/` train domain
- `api/app/core/crypto/` envelope encryption and redaction
- `api/app/db/` SQLAlchemy models/session
- `api/alembic/versions/` migrations
- `infra/` compose files, env files, scripts

## Local Workflow

```bash
git submodule update --init --recursive
docker compose -f infra/docker-compose.yml up --build
```

Focused verification:

```bash
docker compose -f infra/docker-compose.yml exec api pytest -q
docker compose -f infra/docker-compose.yml exec web npx tsc --noEmit
```

## Production Deployment (Important)

Use canonical deploy flow:

```bash
sudo -u bominal /opt/bominal/repo/infra/scripts/deploy.sh
```

Do not use `docker compose down` in production.
Do not bypass post-deploy health verification.
See `docs/humans/operations/DEPLOYMENT.md` and `docs/governance/DEPRECATION_POLICY.md`.
