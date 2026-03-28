---
phase: 04-auth-pages
plan: 03
status: complete
completed: 2026-03-27T17:12:20-04:00
requirements:
  - AUTH-01
  - AUTH-05
---

# Phase 4 / Plan 03 Summary

Added the minimum passkey WASM client and browser bridge needed for the auth landing and add-passkey pages to trigger live WebAuthn ceremonies.

## What Changed

- Added [crates/bominal-app/src/api/passkey.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/api/passkey.rs) with the start/finish fetch flow for passkey login and passkey registration against the existing `/api/auth/passkey/...` endpoints.
- Wired the `/auth` and `/auth/add-passkey` buttons in [crates/bominal-app/src/pages/auth/mod.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/pages/auth/mod.rs) and [crates/bominal-app/src/pages/auth/add_passkey.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/pages/auth/add_passkey.rs) to that passkey client so browser ceremonies now run from the real auth pages.
- Added the JS interop bridge in [crates/bominal-app/assets/passkey-interop.js](/Users/jasonlee/projects/bominal-train/crates/bominal-app/assets/passkey-interop.js), loaded it from [crates/bominal-app/index.html](/Users/jasonlee/projects/bominal-train/crates/bominal-app/index.html), and expanded the browser APIs exposed in [crates/bominal-app/Cargo.toml](/Users/jasonlee/projects/bominal-train/crates/bominal-app/Cargo.toml).
- Reconciled the clean milestone baseline with the current workspace build shape by restoring `wreq` workspace dependencies in [Cargo.toml](/Users/jasonlee/projects/bominal-train/Cargo.toml) and reintroducing [crates/bominal-domain/src/task_event.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-domain/src/task_event.rs), both of which `bominal-app` already depended on indirectly.

## Verification

- `cargo test -p bominal-app --lib`
- `cargo check -p bominal-app --features ssr`
- `cargo check -p bominal-app --target wasm32-unknown-unknown --features hydrate`

All checks passed.
