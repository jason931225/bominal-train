---
phase: 3
slug: shell-and-navigation
status: ready
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-27
---

# Phase 3 — Validation Strategy

## Automated Verification Contract

| Scope | Command | Result |
|-------|---------|--------|
| App unit surface | `cargo test -p bominal-app --lib` | Required |
| App SSR build | `cargo check -p bominal-app --features ssr` | Required |
| App hydrate build | `cargo check -p bominal-app --target wasm32-unknown-unknown --features hydrate` | Required |

## Artifact Checks

| Artifact | Expectation |
|----------|-------------|
| [crates/bominal-app/src/lib.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/lib.rs) | router, auth bootstrap, public/protected shell branching |
| [crates/bominal-app/src/shell_pages.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/shell_pages.rs) | route inventory, stubs, redirects, and public-path logic |
| [crates/bominal-app/src/components/sidebar.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/components/sidebar.rs) | desktop navigation with active state |
| [crates/bominal-app/src/components/bottom_nav.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/components/bottom_nav.rs) | mobile navigation with active state |
| [crates/bominal-app/style/app.css](/Users/jasonlee/projects/bominal-train/crates/bominal-app/style/app.css) | shell, loading, and nav chrome styling for desktop/mobile |

## Manual Verification

Optional browser pass after server integration. For Phase 3, compile and unit validation are sufficient because the server-function transport is intentionally deferred to later phases and the route bodies remain placeholders.
