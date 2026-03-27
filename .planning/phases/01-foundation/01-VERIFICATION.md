---
phase: 01-foundation
verified: 2026-03-27T15:29:03-04:00
status: passed
score: 7/7 must-haves verified
---

# Phase 1: Foundation Verification Report

**Phase Goal:** Leptos 0.8 foundation compiles in both SSR and hydrate modes with cargo-leptos orchestrating the build and `bominal-ui` wired as the shared UI source.

## Goal Achievement

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `bominal-frontend` is no longer treated as the active workspace build target | VERIFIED | [Cargo.toml](/Users/jasonlee/projects/bominal-train/Cargo.toml) workspace members include `bominal-app` but not `bominal-frontend` |
| 2 | Root workspace metadata declares a cargo-leptos app/server pair | VERIFIED | [Cargo.toml](/Users/jasonlee/projects/bominal-train/Cargo.toml) contains `[[workspace.metadata.leptos]]` with `bin-package = "bominal-server"` and `lib-package = "bominal-app"` |
| 3 | `bominal-app` exposes explicit SSR and hydrate features using islands mode | VERIFIED | [crates/bominal-app/Cargo.toml](/Users/jasonlee/projects/bominal-train/crates/bominal-app/Cargo.toml) defines `ssr`, `hydrate`, and `leptos` with `features = ["islands"]` |
| 4 | `bominal-app` depends on `bominal-ui` as the canonical shared UI source | VERIFIED | [crates/bominal-app/Cargo.toml](/Users/jasonlee/projects/bominal-train/crates/bominal-app/Cargo.toml) includes `bominal-ui` with `leptos`, `train`, and `auth` features |
| 5 | The app crate exports an islands-ready hydrate entrypoint | VERIFIED | [crates/bominal-app/src/lib.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/lib.rs) exports `hydrate()` and calls `leptos::mount::hydrate_islands()` |
| 6 | The active CSS input is owned by the app crate and derived from `bominal-ui` | VERIFIED | [crates/bominal-app/style/app.css](/Users/jasonlee/projects/bominal-train/crates/bominal-app/style/app.css) is the configured style file and documents its `bominal-ui` derivation |
| 7 | The integrated cargo-leptos build path succeeds end-to-end | VERIFIED | `cargo leptos build` passed and emitted [target/site/pkg](/Users/jasonlee/projects/bominal-train/target/site/pkg) with `bominal-app.css`, JS, and WASM assets |

## Automated Checks

| Command | Result |
|---------|--------|
| `cargo check -p bominal-app --features ssr` | PASSED |
| `cargo check -p bominal-app --target wasm32-unknown-unknown --features hydrate` | PASSED |
| `cargo check -p bominal-server --features ssr` | PASSED |
| `cargo leptos build` | PASSED |

## Notes

- `cargo leptos build` emitted a Tailwind v4 warning that JavaScript config files are legacy, but the build completed successfully with the current bridge configuration.
- `cargo check -p bominal-server --features ssr` surfaced pre-existing `bominal-service` warnings in `crates/bominal-service/src/providers/ktx/client.rs`; they did not block Phase 1 verification.

## Verdict

Phase 1 is complete. The repo now has a working Leptos SSR/hydrate build foundation that later routing, infrastructure, and page-port phases can build on.
