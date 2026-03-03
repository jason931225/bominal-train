# bominal

`bominal` is a modular runtime platform centered on a Rust API + worker architecture.

## Current Code Layout

- `runtime/crates/api` - axum API and SSR shell.
- `runtime/crates/worker` - background task loops.
- `runtime/crates/shared` - shared config/contracts/integration helpers.
- `runtime/migrations` - SQL migrations.
- `runtime/frontend` - Tailwind build assets.
- `legacy/node-web` - legacy web app reference.
- `legacy/py-api` - retired runtime reference.
- `third_party/srtgo` - read-only train provider behavior reference.
- `third_party/catchtable` - read-only restaurant provider reference.

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
- PostgreSQL and Redis accessible via env vars

Bootstrap runtime assets:

```bash
npm --prefix runtime/frontend install
npm --prefix runtime/frontend run build:css
```

Build and test Rust workspace:

```bash
cd runtime
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

Run API:

```bash
cd runtime
cp env.example .env
cargo run -p bominal-api
```

Run worker:

```bash
cd runtime
cp env.example .env
cargo run -p bominal-worker
```

## Security and Safety Baseline

- Never commit or log secrets.
- Never persist raw cardholder data.
- Keep provider and internal API auth fail-closed.
- Preserve secure session-cookie semantics.

See `docs/MANUAL.md` for the complete policy baseline.
