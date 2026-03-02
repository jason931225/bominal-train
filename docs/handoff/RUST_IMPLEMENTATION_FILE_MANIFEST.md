# Rust Implementation File Manifest

Purpose: authoritative inventory of files introduced for the Rust migration.

Rules:
- Every new Rust migration file must be listed here in the same commit where it is created.
- Paths must be repo-relative and unique.
- `status` tracks migration lifecycle for each file:
  - `active`: used by current Rust implementation.
  - `draft`: temporary/in-progress artifact.
  - `remove_candidate`: expected deletion after consolidation.
- Legacy files are not listed here; this manifest is Rust-cutover scope only.

| path | status | notes |
|---|---|---|
| docs/plans/active/2026-03-01-rust-leptos-ssr-cutover.md | active | executable migration plan |
| docs/handoff/RUST_IMPLEMENTATION_FILE_MANIFEST.md | active | this manifest |
| rust/Cargo.toml | active | Rust workspace root with pinned dependency contract |
| rust/Cargo.lock | active | resolved dependency lock for reproducible builds |
| rust/README.md | active | workspace usage and build commands |
| rust/env.example | active | env contract baseline for Rust API/worker |
| rust/migrations/202603010001_bootstrap.sql | draft | initial sqlx migration scaffold |
| rust/crates/shared/Cargo.toml | active | shared crate manifest |
| rust/crates/shared/src/lib.rs | active | shared module export root |
| rust/crates/shared/src/config.rs | active | Supabase/Redis/Evervault/Resend/runtime schedule config model |
| rust/crates/shared/src/http_client.rs | active | reqwest builder with `curl-transport` feature gate |
| rust/crates/shared/src/supabase.rs | active | JWKS/JWT header validation scaffold |
| rust/crates/shared/src/telemetry.rs | active | JSON/plain tracing bootstrap |
| rust/crates/api/Cargo.toml | active | axum + leptos SSR API binary manifest |
| rust/crates/api/src/main.rs | active | API routes, health checks, Supabase webhook endpoint, static serving |
| rust/crates/api/src/web.rs | active | Leptos SSR homepage component/render function |
| rust/crates/worker/Cargo.toml | active | worker binary manifest |
| rust/crates/worker/src/main.rs | active | poll/reconcile/watch/rotation loops with graceful shutdown |
| rust/frontend/package.json | active | Tailwind CSS build scripts |
| rust/frontend/package-lock.json | active | locked npm dependency graph for Tailwind tooling |
| rust/frontend/tailwind.config.js | active | Tailwind content/theme config for Rust sources |
| rust/frontend/styles/tailwind.css | active | Tailwind source stylesheet |
| rust/frontend/dist/tailwind.css | active | generated stylesheet served by Rust API |
