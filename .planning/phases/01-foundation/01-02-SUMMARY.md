---
phase: 01-foundation
plan: 02
status: complete
completed: 2026-03-27T15:29:03-04:00
requirements:
  - FND-04
  - FND-05
files_modified:
  - crates/bominal-app/src/lib.rs
  - crates/bominal-app/src/i18n.rs
  - crates/bominal-app/style/app.css
  - tailwind.config.js
  - crates/bominal-app/assets/.gitkeep
---

# Phase 1 / Plan 02 Summary

Finished the compile-first app shell and validated the end-to-end cargo-leptos build path.

## What Changed

- Replaced the CSR-only app entrypoint in [crates/bominal-app/src/lib.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/lib.rs) with a minimal SSR-safe shell plus a `hydrate()` entrypoint that calls `leptos::mount::hydrate_islands()`.
- Simplified [crates/bominal-app/src/i18n.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/i18n.rs) to an SSR-safe default-locale baseline for Phase 1.
- Created the canonical Phase 1 stylesheet bridge at [crates/bominal-app/style/app.css](/Users/jasonlee/projects/bominal-train/crates/bominal-app/style/app.css), derived from `bominal-ui` ecosystem, `train`, and `auth` CSS payloads.
- Added [tailwind.config.js](/Users/jasonlee/projects/bominal-train/tailwind.config.js) and [crates/bominal-app/assets/.gitkeep](/Users/jasonlee/projects/bominal-train/crates/bominal-app/assets/.gitkeep) so cargo-leptos has stable content-scan and asset paths.

## Outcome

- The app crate now compiles cleanly for both SSR and hydrate targets without pulling browser-only startup code into the server build.
- cargo-leptos emits the WASM and CSS bundle under [target/site/pkg](/Users/jasonlee/projects/bominal-train/target/site/pkg).
- The shared UI source of truth for this phase is `bominal-ui`, not the old SvelteKit stylesheet.

## Verification

- `cargo check -p bominal-app --features ssr`
- `cargo check -p bominal-app --target wasm32-unknown-unknown --features hydrate`
- `cargo leptos build`
- `test -d target/site/pkg && ls target/site/pkg`

All four checks passed on 2026-03-27.
