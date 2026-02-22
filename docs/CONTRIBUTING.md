# Contributing

Thanks for contributing to Bominal.

## Prerequisites

- Python 3.12+
- Node.js 20+
- Postgres + Redis (local services)
- Git with submodule support
- Docker + Docker Compose (optional, for containerized simulation)

## Initial setup

From repo root:

```bash
git submodule update --init --recursive
./infra/scripts/local-setup.sh
./infra/scripts/local-run.sh
./infra/scripts/local-check.sh
```
If you prefer containerized local simulation, use:

```bash
docker compose -f infra/docker-compose.yml up --build
```

## Development conventions

## 1) General

- Keep the product name as `bominal`.
- Do not edit `third_party/srtgo` or `third_party/catchtable` (read-only references).
- Keep API handlers thin; move orchestration/business logic into service/worker layers.
- Favor additive migrations and backward-safe schema evolution.
- For restaurant/train provider onboarding, follow `docs/playbooks/restaurant-provider-adapter-workflow.md` and update `docs/provider-research/*` first.

## 2) Frontend (Next.js + TS + Tailwind)

- Reuse tokens/classes from `web/lib/ui.ts`.
- Keep user-facing train times in KST.
- Use Zod for client-side form validation.
- Prefer typed API contracts from `web/lib/types.ts`.
- Wordmark (`bominal`) uses `font-brand` + theme-aware color:
  - default: `text-blossom-800`
  - hover: `text-blossom-700`

## 3) Backend (FastAPI + SQLAlchemy)

- Keep endpoint schemas in `api/app/schemas` or module schemas.
- Use `AsyncSession` patterns consistent with existing routes/services.
- Do not expose secrets in responses or logs.
- Use `meta_json_safe` / `data_json_safe` for persisted provider metadata.

## 4) Security-sensitive changes

If touching auth, wallet, or provider credentials:

- verify cookie/security flags remain correct
- ensure redaction paths still cover new fields
- document behavior changes in `docs/SECURITY.md`

## Test and verification

Recommended one-command local verification (starts stack, waits for health, runs tests + typecheck):

```bash
./infra/scripts/local-check.sh
```

Optional cleanup after checks:

```bash
./infra/scripts/local-check.sh --down
```

Backend tests:

```bash
(cd api && ./.venv/bin/pytest -q)
```

Frontend type check:

```bash
(cd web && npx tsc --noEmit)
```

Recommended targeted smoke tests after major changes:

```bash
curl -sS http://localhost:8000/health
curl -sS -I http://localhost:3000
```

Note on local CORS:

- If you open the web app via `http://0.0.0.0:3000` or `http://127.0.0.1:3000`, your browser `Origin` will not be `http://localhost:3000`.
- Ensure `CORS_ORIGINS` includes the exact origin(s), otherwise auth requests may show “Could not reach bominal API.” due to CORS blocking.

## Migration workflow

Create a migration:

```bash
(cd api && ./.venv/bin/alembic revision -m "describe_change")
```

Apply migrations:

```bash
(cd api && ./.venv/bin/alembic upgrade head)
```

## Pull request checklist

- [ ] Feature works end-to-end locally
- [ ] Tests/typechecks pass
- [ ] No secrets added to code or git history
- [ ] Env changes reflected in `.env.example`/`infra/env/*`
- [ ] Docs updated (`README.md` and/or `docs/*.md`)
