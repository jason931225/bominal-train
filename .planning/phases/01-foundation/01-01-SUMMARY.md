---
phase: 01-foundation
plan: 01
status: complete
completed: 2026-03-27T15:29:03-04:00
requirements:
  - FND-01
  - FND-02
  - FND-03
files_modified:
  - Cargo.toml
  - crates/bominal-app/Cargo.toml
  - crates/bominal-server/Cargo.toml
---

# Phase 1 / Plan 01 Summary

Established the Leptos workspace foundation for the migration.

## What Changed

- Added workspace-scoped `leptos`, `leptos_meta`, `leptos_router`, `leptos_axum`, and `bominal-ui` dependencies in the root [Cargo.toml](/Users/jasonlee/projects/bominal-train/Cargo.toml).
- Declared the cargo-leptos multi-package project in [Cargo.toml](/Users/jasonlee/projects/bominal-train/Cargo.toml) with `bominal-server` as the bin package and `bominal-app` as the lib package.
- Rewrote [crates/bominal-app/Cargo.toml](/Users/jasonlee/projects/bominal-train/crates/bominal-app/Cargo.toml) around explicit `ssr` and `hydrate` features, islands mode, and `bominal-ui`.
- Added an `ssr` feature path in [crates/bominal-server/Cargo.toml](/Users/jasonlee/projects/bominal-train/crates/bominal-server/Cargo.toml) that pulls the app crate into the server build graph without switching the route layer yet.

## Outcome

- `crates/bominal-frontend` remains outside the active workspace build path.
- `bominal-app` now has the intended SSR/hydrate feature graph for the migration baseline.
- cargo-leptos can discover the repo as a workspace app/server pair instead of a CSR-only frontend crate.

## Verification

- `cargo check -p bominal-app --features ssr`
- `cargo check -p bominal-app --target wasm32-unknown-unknown --features hydrate`
- `cargo check -p bominal-server --features ssr`

All three commands passed on 2026-03-27.
