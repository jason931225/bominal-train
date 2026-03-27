---
phase: 1
slug: foundation
status: ready
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-27
---

# Phase 1 — Validation Strategy

## Automated Verification Contract

| Scope | Command | Result |
|-------|---------|--------|
| App SSR build | `cargo check -p bominal-app --features ssr` | Required |
| App hydrate build | `cargo check -p bominal-app --target wasm32-unknown-unknown --features hydrate` | Required |
| Server SSR path | `cargo check -p bominal-server --features ssr` | Required |
| Integrated workspace build | `cargo leptos build` | Required |

## Artifact Checks

| Artifact | Expectation |
|----------|-------------|
| [Cargo.toml](/Users/jasonlee/projects/bominal-train/Cargo.toml) | cargo-leptos workspace metadata present |
| [crates/bominal-app/Cargo.toml](/Users/jasonlee/projects/bominal-train/crates/bominal-app/Cargo.toml) | `ssr` / `hydrate` features plus `bominal-ui` dependency |
| [crates/bominal-app/src/lib.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/lib.rs) | `hydrate_islands()` entrypoint |
| [crates/bominal-app/style/app.css](/Users/jasonlee/projects/bominal-train/crates/bominal-app/style/app.css) | app-owned CSS bridge derived from `bominal-ui` |
| [target/site/pkg](/Users/jasonlee/projects/bominal-train/target/site/pkg) | generated CSS, JS, and WASM bundle exists after `cargo leptos build` |

## Manual Verification

None required for Phase 1. This phase is compile-first infrastructure and is considered complete when the build graph and emitted artifacts are valid.
