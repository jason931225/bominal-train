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

Auth page:

```bash
http://127.0.0.1:8000/auth
```

`/auth` includes:
- passkey-first sign-in action (when an eligible passkey factor/session context is available),
- password fallback sign-in,
- optional passkey registration during/after signup.

`dev-up` defaults to:
- running bootstrap with `BOMINAL_RUN_TESTS=0` (faster dev loop),
- applying migrations,
- starting both `bominal-api` and `bominal-worker`,
- running Tailwind CSS watch for local frontend edits.

Useful flags:

```bash
./scripts/dev-up.sh --api-only
./scripts/dev-up.sh --worker-only
./scripts/dev-up.sh --no-bootstrap
./scripts/dev-up.sh --no-css-watch
./scripts/dev-up.sh --rust-watch
./scripts/dev-up.sh --rust-watch --port 8001
```

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
cargo run -p bominal-api
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
  --set DEPLOY_API_SERVICE=api \
  --set DEPLOY_WORKER_SERVICE=worker \
  --set DEPLOY_HEALTHCHECK_LIVE_URL=http://127.0.0.1:8000/health/live \
  --set DEPLOY_HEALTHCHECK_READY_URL=http://127.0.0.1:8000/health/ready \
  --set POSTGRES_HOST=127.0.0.1 \
  --set POSTGRES_PORT=5432 \
  --set POSTGRES_DB=bominal \
  --set POSTGRES_USER=bominal
```

Outputs (from `env/prod/*.example` templates; depends on `--only` selection):
- `env/prod/runtime.env`
- `env/prod/caddy.env`
- `env/prod/deploy.env`

## Security and Safety Baseline

- Never commit or log secrets.
- Never persist raw cardholder data.
- Keep provider and internal API auth fail-closed.
- Preserve secure session-cookie semantics.

See `docs/MANUAL.md` for the complete policy baseline.
