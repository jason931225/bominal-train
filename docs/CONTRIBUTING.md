# Contributing

Thanks for contributing to Bominal.

## Prerequisites

- Docker + Docker Compose (v1 or v2)
- Git with submodule support

## Initial setup

From repo root:

```bash
git submodule update --init --recursive
docker-compose -f infra/docker-compose.yml up --build
```

If your environment uses Compose v2 plugin, use `docker compose` instead of `docker-compose`.

## Development conventions

## 1) General

- Keep the product name as `bominal`.
- Do not edit `third_party/srtgo` (read-only reference).
- Keep API handlers thin; move orchestration/business logic into service/worker layers.
- Favor additive migrations and backward-safe schema evolution.

## 2) Frontend (Next.js + TS + Tailwind)

- Reuse tokens/classes from `web/lib/ui.ts`.
- Keep user-facing train times in KST.
- Use Zod for client-side form validation.
- Prefer typed API contracts from `web/lib/types.ts`.

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

Backend tests:

```bash
docker-compose -f infra/docker-compose.yml exec api pytest -q
```

Frontend type check:

```bash
docker-compose -f infra/docker-compose.yml exec web npx tsc --noEmit
```

Recommended targeted smoke tests after major changes:

```bash
curl -sS http://localhost:8000/health
curl -sS -I http://localhost:3000
```

## Migration workflow

Create a migration:

```bash
docker-compose -f infra/docker-compose.yml exec api alembic revision -m "describe_change"
```

Apply migrations:

```bash
docker-compose -f infra/docker-compose.yml exec api alembic upgrade head
```

## Pull request checklist

- [ ] Feature works end-to-end locally
- [ ] Tests/typechecks pass
- [ ] No secrets added to code or git history
- [ ] Env changes reflected in `.env.example`/`infra/env/*`
- [ ] Docs updated (`README.md` and/or `docs/*.md`)

