# bominal

`bominal` is a modular runtime platform centered on a Rust API + worker architecture.

## Current Code Layout

- `runtime/crates/api` - axum API and SSR shell.
- `runtime/crates/worker` - background task loops.
- `runtime/crates/shared` - shared config/contracts/integration helpers.
- `runtime/migrations` - SQL migrations.
- `runtime/frontend` - Tailwind build assets.
- `third_party/srtgo` - read-only train provider behavior reference.
- `third_party/catchtable` - read-only restaurant provider reference.
- `docs/handoff` - preserved historical/reference handoff artifacts.

## Documentation

- Start: `docs/START_HERE.md`
- Canonical policy/ops manual: `docs/MANUAL.md`
- Docs index: `docs/README.md`
- Intent router: `docs/INTENT_ROUTING.md`
- Agent constraints: `AGENTS.md`
- Changelog: `CHANGELOG.md`

## Local Development

Prerequisites:
- Rust toolchain (edition 2024 capable)
- Node.js + npm (for Tailwind asset build)
- Docker (for local PostgreSQL + Redis)

One-command local bootstrap for first test:

```bash
./scripts/bootstrap-local.sh
```

What bootstrap does:
- Creates `env/local/runtime.env` (if missing) with localhost overrides.
- Starts local Postgres + Redis containers.
- Builds frontend CSS assets.
- Applies SQL files in `runtime/migrations/`.
- Runs `cargo test --workspace --locked`.

Optional toggles:

```bash
BOMINAL_RUN_TESTS=0 ./scripts/bootstrap-local.sh
BOMINAL_RUN_MIGRATIONS=0 ./scripts/bootstrap-local.sh
```

One-command local runtime bring-up (API + worker):

```bash
./scripts/dev-up.sh
```

Auth landing:

```bash
http://127.0.0.1:8000/
```

`/` includes:
- passkey-first sign-in action (when an eligible passkey factor/session context is available),
- password fallback sign-in,
- light/dark theme toggle (persisted per session via cookie-backed preference).

Compatibility alias:
- `/auth` permanently redirects to `/`.

Runtime route map:
- User app (`www.bominal.com`): `/`, `/dashboard`, `/dashboard/jobs`, `/dashboard/jobs/{job_id}`, `/dashboard/security`.
- Admin app (`ops.bominal.com`): `/admin/maintenance`, `/admin/users`, `/admin/runtime`, `/admin/observability`, `/admin/security`, `/admin/config`, `/admin/audit`.
- Observability contracts: `/health` (liveness), `/ready` (dependency readiness), `/admin/maintenance/metrics` (admin-only Prometheus text).
- Admin API highlights: `/api/admin/capabilities`, `/api/admin/runtime/jobs/stream` (SSE), `/api/admin/incidents`, `/api/admin/incidents/{incident_id}/status`, `/api/admin/observability/timeseries`.

Station catalog source-of-truth:
- Runtime uses committed snapshot files in `runtime/data/train/` (repo-driven, no production live-fetch path).
- Daily refresh automation runs via GitHub Actions workflow: `.github/workflows/station-catalog-refresh.yml`.
- Manual refresh command:

```bash
cd runtime
cargo run -p bominal-api --bin station_catalog_sync -- generate --snapshot data/train/station_catalog.v1.json --meta data/train/station_catalog.meta.json
cargo run -p bominal-api --bin station_catalog_sync -- validate --snapshot data/train/station_catalog.v1.json --meta data/train/station_catalog.meta.json
```

`dev-up` defaults to:
- running bootstrap with `BOMINAL_RUN_TESTS=0` (faster dev loop),
- applying migrations,
- starting both `bominal-api` and `bominal-worker`,
- running Tailwind CSS watch for local frontend edits,
- failing fast if `APP_PORT` is already occupied (to avoid partial startup thrash).

Useful flags:

```bash
./scripts/dev-up.sh --api-only
./scripts/dev-up.sh --worker-only
./scripts/dev-up.sh --no-bootstrap
./scripts/dev-up.sh --no-css-watch
./scripts/dev-up.sh --rust-watch
./scripts/dev-up.sh --takeover-watchers
./scripts/dev-up.sh --rust-watch --port 8001
```

Common startup failures:
- `error: another dev-up instance is already running`:

```bash
ps -p <pid> -o pid,command
kill <pid>
./scripts/dev-up.sh
```

If no `dev-up` process is actually running, clear a stale lock and retry:

```bash
rm -rf /tmp/bominal-dev-up.lock
./scripts/dev-up.sh
```

- `error: stale cargo-watch supervisor(s) detected for this repo`:

```bash
kill <pid>
./scripts/dev-up.sh
```

or auto-stop stale watchers:

```bash
./scripts/dev-up.sh --takeover-watchers
```

- `error: APP_PORT 8000 is already in use`:

```bash
lsof -nP -iTCP:8000 -sTCP:LISTEN
kill <pid>
./scripts/dev-up.sh
```

or run on another port:

```bash
./scripts/dev-up.sh --port 8001
```

- `Blocking waiting for file lock on package cache/artifact directory`:
  - expected occasionally when API and worker start in parallel,
  - only actionable if followed by a process exit/error (for example port conflict).

`--rust-watch` requires:

```bash
cargo install cargo-watch
```

Manual fallback (if you do not want the bootstrap script):

```bash
npm ci --prefix runtime/frontend
npm --prefix runtime/frontend run build:css
```

Then build and test Rust workspace:

```bash
cd runtime
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

Run API:

```bash
cd runtime
set -a
source ../env/dev/runtime.env
[ -f ../env/local/runtime.env ] && source ../env/local/runtime.env
set +a
cargo run -p bominal-api --bin bominal-api
```

Run worker:

```bash
cd runtime
set -a
source ../env/dev/runtime.env
[ -f ../env/local/runtime.env ] && source ../env/local/runtime.env
set +a
cargo run -p bominal-worker
```

Env setup notes:
- Use `env/dev/runtime.env` as the canonical development baseline.
- Put machine-local secret overrides in `env/local/runtime.env` (gitignored).

## Production Bootstrap

Generate production env files from templates with either prompts (operator mode) or fail-closed CLI mode (CI/operator automation):

```bash
./scripts/bootstrap-prod.sh --interactive
```

```bash
./scripts/bootstrap-prod.sh --non-interactive --force \
  --only deploy \
  --set GCP_PROJECT_ID=your-project \
  --set GCP_WORKLOAD_IDENTITY_PROVIDER=projects/123456789/locations/global/workloadIdentityPools/pool/providers/provider \
  --set GCP_SERVICE_ACCOUNT=bominal-deploy@your-project.iam.gserviceaccount.com \
  --set DEPLOY_VM_NAME=bominal-deploy \
  --set DEPLOY_VM_ZONE=us-central1-a \
  --set DEPLOY_WORKDIR=/opt/bominal/repo \
  --set DEPLOY_SCRIPT_PATH=/opt/bominal/repo/scripts/prod/deploy-runtime.sh \
  --set DEPLOY_HEALTHCHECK_SCRIPT_PATH=/opt/bominal/repo/scripts/prod/healthcheck-runtime.sh \
  --set DEPLOY_ROLLBACK_SCRIPT_PATH=/opt/bominal/repo/scripts/prod/rollback-runtime.sh \
  --set VM_SECRET_ENV_FILE=/opt/bominal/env/prod/vm-secrets.env \
  --set DEPLOY_RUNTIME_ENV_FILE=/opt/bominal/repo/env/prod/runtime.env \
  --set DEPLOY_COMPOSE_FILE=/opt/bominal/repo/runtime/compose.prod.yml \
  --set DEPLOY_MIGRATIONS_DIR=/opt/bominal/repo/runtime/migrations \
  --set DEPLOY_API_SERVICE=api \
  --set DEPLOY_WORKER_SERVICE=worker \
  --set DEPLOY_HEALTHCHECK_LIVE_URL=http://127.0.0.1:8000/health \
  --set DEPLOY_HEALTHCHECK_READY_URL=http://127.0.0.1:8000/ready \
  --set POSTGRES_HOST=127.0.0.1 \
  --set POSTGRES_PORT=5432 \
  --set POSTGRES_DB=bominal \
  --set POSTGRES_USER=bominal
```

Outputs (from `env/prod/*.example` templates; depends on `--only` selection):
- `env/prod/runtime.env`
- `env/prod/caddy.env`
- `env/prod/deploy.env`
- `env/prod/vm-secrets.env` (auto-created if missing; includes `BOMINAL_DATABASE_URL` and GHCR credential placeholders for private pulls)

## Production Operations (VM)

Use `scripts/prod-up.sh` as the canonical operator entrypoint on the VM:

```bash
cd /opt/bominal/repo
./scripts/prod-up.sh start
./scripts/prod-up.sh status
./scripts/prod-up.sh health
./scripts/prod-up.sh logs -f --since 30m --service api
```

Deploy/rollback are explicit and guarded:

```bash
cd /opt/bominal/repo
./scripts/prod-up.sh deploy --yes
./scripts/prod-up.sh rollback --yes
```

`deploy` reuses `scripts/prod/deploy-runtime.sh` and still requires deploy inputs (`BOMINAL_API_IMAGE`, `BOMINAL_WORKER_IMAGE`, `BOMINAL_POSTGRES_*`) to be set by CI/CD or operator exports.

## Security and Safety Baseline

- Never commit or log secrets.
- Never persist raw cardholder data.
- Keep provider and internal API auth fail-closed.
- Preserve secure session-cookie semantics.
- Keep admin routes and APIs served from `ops.bominal.com`; user app routes stay on `www.bominal.com`.
- Sensitive admin mutations require recent passkey step-up, typed confirmation target, and a mandatory reason.

See `docs/MANUAL.md` for the complete policy baseline.
