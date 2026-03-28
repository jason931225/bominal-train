---
phase: 8
slug: server-integration
status: ready
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-27
---

# Phase 8 — Validation Strategy

## Automated Verification Contract

| Scope | Command | Result |
|-------|---------|--------|
| App unit surface | `cargo test -p bominal-app --lib` | Required |
| App SSR build | `cargo check -p bominal-app --features ssr` | Required |
| App hydrate build | `cargo check -p bominal-app --target wasm32-unknown-unknown --features hydrate` | Required |
| Server SSR build | `cargo check -p bominal-server --features ssr` | Required |

## Artifact Checks

| Artifact | Expectation |
|----------|-------------|
| [crates/bominal-app/src/lib.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/lib.rs) | exposes the SSR shell document and continues to mount the same Leptos app routes |
| [crates/bominal-app/src/state.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/state.rs) | provides request-derived theme preferences for SSR shell rendering |
| [crates/bominal-server/src/routes.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-server/src/routes.rs) | serves Leptos SSR routes, preserves `/api`, and serves static assets from the cargo-leptos output root |
| [crates/bominal-server/src/state.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-server/src/state.rs) | carries `LeptosOptions` through shared server state |
| [crates/bominal-server/src/runner.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-server/src/runner.rs) | compiles against the current typed task model and shared Evervault/server state shape |
| [crates/bominal-server/Cargo.toml](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-server/Cargo.toml) | includes the runtime dependency needed by the SSR server entry point |

## Manual Verification

Optional browser smoke validation is still useful for confirming that `/home`, `/search`, and `/settings` return rendered HTML plus the expected `/pkg` and `/assets` requests in a live server session. For this migration phase, compile verification plus route/static-asset inspection is sufficient because the SSR integration intentionally preserved the existing `/api` contract and middleware stack.
