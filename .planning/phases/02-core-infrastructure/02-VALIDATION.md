---
phase: 2
slug: core-infrastructure
status: ready
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-27
---

# Phase 2 — Validation Strategy

## Automated Verification Contract

| Scope | Command | Result |
|-------|---------|--------|
| App unit surface | `cargo test -p bominal-app --lib` | Required |
| App SSR build | `cargo check -p bominal-app --features ssr` | Required |
| App hydrate build | `cargo check -p bominal-app --target wasm32-unknown-unknown --features hydrate` | Required |

## Artifact Checks

| Artifact | Expectation |
|----------|-------------|
| [crates/bominal-app/src/i18n.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/i18n.rs) | locale cookie parsing plus domain-backed translation helpers |
| [crates/bominal-app/src/utils.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/utils.rs) | all shared formatting/status helpers present |
| [crates/bominal-app/src/types.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/types.rs) | canonical app-facing typed surface exported |
| [crates/bominal-app/src/api.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/api.rs) | typed `/api/` proxy server functions available |
| [crates/bominal-app/src/state.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/state.rs) | auth/theme/SSE state contexts provided |

## Manual Verification

None required for Phase 2. This is shared infrastructure work and is considered complete when the foundational modules compile, the tests pass, and the phase success criteria are satisfied from the app crate boundary.
