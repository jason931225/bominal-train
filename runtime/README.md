# Runtime Workspace (Cutover Track)

This workspace is the parallel runtime implementation for Bominal cutover.

## Crates

- `crates/api`: axum API + Leptos SSR frontend serving.
- `crates/worker`: Tokio worker loops for queue/reconcile/watch/rotation controls.
- `crates/shared`: shared config, telemetry, HTTP client helpers, crypto/provider contracts.

## Tooling

From repo root, preferred first-run bootstrap:

```bash
./scripts/bootstrap-local.sh
```

From repo root, start local API + worker:

```bash
./scripts/dev-up.sh
```

`dev-up` also runs Tailwind CSS watch by default. Use `--no-css-watch` to disable it.
For Rust auto-restart on code edits, run `./scripts/dev-up.sh --rust-watch` (requires `cargo install cargo-watch`).
Use `--port 8001` if `:8000` is already in use.
Auth workspace page is served at `http://127.0.0.1:8000/auth`.
For passkey/password flows on this page, ensure `DATABASE_URL` and `REDIS_URL` are configured (handled by `./scripts/bootstrap-local.sh`).

Manual runtime checks:

```bash
cd runtime
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

## Frontend CSS Build

```bash
npm ci --prefix frontend
npm --prefix frontend run build:css
npm --prefix frontend run check:css:budget
```

`build:css` compiles Tailwind into `dist/tailwind.raw.css`, then runs Lightning CSS
to emit the optimized `dist/tailwind.css` artifact served by the runtime API.

## Images

```bash
docker build -f runtime/Dockerfile.api -t bominal/api:local runtime
docker build -f runtime/Dockerfile.worker -t bominal/worker:local runtime
```
