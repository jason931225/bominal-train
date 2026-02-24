# Contributing

Thanks for contributing to Bominal.

## Prerequisites

- Python 3.14.3+
- Node.js 24+
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
- Use workflow naming conventions for GitHub Actions:
  - CI workflows: `.github/workflows/ci-*.yml` and `name: CI - ...`
  - CD workflows: `.github/workflows/cd-*.yml` and `name: CD - ...`
- For restaurant/train provider onboarding, follow `docs/playbooks/restaurant-provider-adapter-workflow.md` and update `docs/provider-research/*` first.

## 2) Frontend (Next.js + TS + Tailwind)

- Reuse tokens/classes from `web/lib/ui.ts`.
- Keep user-facing train times in KST.
- Use Zod for client-side form validation.
- Prefer typed API contracts from `web/lib/types.ts`.
- Keep linting non-interactive in CI/background terminals:
  - use `web/eslint.config.mjs` with `next/core-web-vitals`;
  - do not reintroduce `next lint` interactive setup prompts in automation scripts.
- Next.js 16 migration notes:
  - use `proxy.ts` (not `middleware.ts`);
  - `headers()` / `cookies()` APIs are async in server components/helpers (`await headers()`, `await cookies()`).
- Container dependency split:
  - `web/Dockerfile.dev` is Chromium-free for faster default dev/test/build loops;
  - Playwright E2E runs in dedicated `web/Dockerfile.e2e` / `web-e2e` compose profile.
- Tailwind 4 migration notes:
  - PostCSS plugin is `@tailwindcss/postcss` (not `tailwindcss` in `postcss.config.js`);
  - global CSS uses `@import "tailwindcss";` with `@config "../tailwind.config.ts";`.
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

Pre-stage gate (mandatory):

- Follow `Docs > Plan > Test` ordering before staging commits.
- Enforce high-quality tests for critical areas before staging.
- Verify test relevance before staging:
  - each staged behavior change must have directly relevant test coverage.
- Prefer behavior/invariant assertions; avoid assertion spam tied to internals.
- Resolve warnings in touched scope before staging:
  - runtime warnings
  - deprecation warnings
  - avoidable tooling warnings
  If a warning cannot be safely resolved in-scope, document rationale and open a tracked follow-up.

Recommended one-command local verification (starts stack, waits for health, runs tests + typecheck):

```bash
./infra/scripts/local-check.sh
```

- `local-check.sh` now hard-fails before test execution if `worker` is not running the expected worker settings process.

Optional cleanup after checks:

```bash
./infra/scripts/local-check.sh --down
```

Backend tests:

```bash
(cd api && ./.venv/bin/pytest -q)
```

- Coverage thresholds are enforced by repository config and CI gates; treat them as minimums, not as a substitute for meaningful test quality.

Frontend type check:

```bash
(cd web && npx tsc --noEmit)
```

Frontend unit tests with coverage gate:

```bash
(cd web && npm run test:ci)
```

- Coverage thresholds are enforced by repository config and CI gates; prioritize meaningful behavior verification over blanket threshold chasing.

Coverage ignore policy (mandatory):

- New `/* c8 ignore */`, `# pragma: no cover`, or equivalent coverage exclusions require:
  - inline rationale at the annotation site, and
  - explicit justification in review notes for why deterministic test coverage is not feasible in-scope.
- Do not add coverage-ignore annotations for convenience.

Assertiveness and mutation gates (risk-based):

```bash
python3 infra/scripts/check_assertive_tests.py api/tests
node infra/scripts/check_assertive_tests_web.mjs web
bash infra/scripts/run_mutation_api.sh
npm run --prefix web test:mutation
```

- Run these gates for critical-path changes (auth/session, permissions, crypto/redaction, payment/CDE, deploy safety).
- For low-risk non-critical changes, these gates are recommended but not mandatory.
- Keep assertions meaningful: avoid vacuous checks (`assert True`, `expect(true).toBe(true)`).
- Use mutation checks where they provide signal on security- or correctness-critical invariants.

Frontend E2E tests (containerized, Chromium-enabled profile):

```bash
docker compose -f infra/docker-compose.yml --profile e2e run --rm --build web-e2e
```

Web dependency lockfile hygiene (mandatory when `web/package.json` changes):

```bash
(cd web && npm install)
```

- Commit `web/package-lock.json` together with `web/package.json`.
- CI runs `npm ci --prefix web`; out-of-sync lockfiles will fail before tests.

NPM warning handling (mandatory):

- Do not ignore `npm warn deprecated` output.
- Any npm warning encountered during `npm install`/`npm ci` must be handled before staging by one of:
  - direct remediation (upgrade/remove the dependency chain causing the warning), or
  - documented upstream block with:
    - exact warning text,
    - source package chain (`npm ls <package> --all`),
    - owner,
    - target removal version/date.
- Suppressing warnings without remediation or explicit tracked ownership is not allowed.

Current tracked npm deprecation warnings:

- none (must remain empty in green state).
- if any warning appears again, add it here with chain/owner/target-removal before staging.

Future-proof dependency policy:

- Prefer latest stable package releases when updating runtime/tooling dependencies.
- If latest introduces behavior breaks, pin the newest compatible version and record:
  - incompatibility summary,
  - blocked package/version,
  - owner and revisit target date.
- Deprecation-triggered overhauls are allowed and expected:
  - replace deprecated dependencies with maintained alternatives when direct upgrade is not sufficient.

Current compatibility holds (must be re-evaluated during dependency modernization):

- `redis-py` library hold:
  - Python client `redis` is pinned to `5.3.1` because `arq==0.27.0` requires `redis<6`.
  - This is not a Redis server-version hold; runtime Redis server remains modern (`redis:8.6-alpine` in local stack).
- `vitest` baseline note (not a hold):
  - stack is migrated to `vitest` / `@vitest/coverage-v8` `4.x`.
  - coverage floors are rebaselined to Vitest 4 accounting and must be ratcheted upward in follow-up test-improvement PRs.
- `eslint` hold:
  - current stable baseline is `eslint` `9.x`; `eslint@10` is currently incompatible with `eslint-config-next@16.1.6` in this stack.
  - exit criteria: upgrade when Next/eslint-config-next support for ESLint 10 is validated in this repo.
- security overrides:
  - `package.json` overrides enforce patched transitive versions (`esbuild`, `minimatch`, `glob`) so `npm audit` stays green while major-hold packages remain.

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
