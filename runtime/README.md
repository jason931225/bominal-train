# Runtime Workspace (Cutover Track)

This workspace is the parallel runtime implementation for Bominal cutover.

## Crates

- `crates/api`: axum API + Leptos SSR frontend serving.
- `crates/worker`: Tokio worker loops for queue/reconcile/watch/rotation controls.
- `crates/shared`: shared config, Supabase helpers, telemetry, HTTP client helpers.

## Tooling

```bash
cd runtime
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

## Tailwind Build

```bash
npm --prefix frontend install
npm --prefix frontend run build:css
```

## Images

```bash
docker build -f runtime/Dockerfile.api -t bominal/api:local runtime
docker build -f runtime/Dockerfile.worker -t bominal/worker:local runtime
```
