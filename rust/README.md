# Rust Workspace (Cutover Track)

This workspace is the parallel Rust implementation for Bominal cutover.

## Crates

- `crates/api`: axum API + Leptos SSR frontend serving.
- `crates/worker`: Tokio worker loops for queue/reconcile/watch/rotation controls.
- `crates/shared`: shared config, Supabase helpers, telemetry, HTTP client helpers.

## Tooling

```bash
cd rust
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
docker build -f rust/Dockerfile.api -t bominal/rust-api:local rust
docker build -f rust/Dockerfile.worker -t bominal/rust-worker:local rust
```
