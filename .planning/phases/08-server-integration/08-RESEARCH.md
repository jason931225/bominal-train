# Phase 8: Server Integration - Research

**Researched:** 2026-03-27
**Confidence:** MEDIUM-HIGH

## Summary

The server crate is already structurally close to SSR integration, but it still serves the old SvelteKit build directory and has not completed the Leptos wiring. The main work is to replace the SPA fallback with `leptos_axum`, surface a proper SSR shell from `bominal-app`, and finish the `SharedState`/`LeptosOptions` connection that the types already anticipate.

## Key Findings

### The router is still serving the legacy SPA output

- [crates/bominal-server/src/routes.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-server/src/routes.rs) still falls back to `ServeDir("frontend/build")` with `index.html`, which is the exact legacy behavior Phase 8 is meant to replace.
- The existing middleware stack around that fallback is valuable and should remain in place when the fallback becomes Leptos SSR + cargo-leptos static assets.
- Root [Cargo.toml](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/Cargo.toml) already points the build output at `target/site`, so the static-serving target is already known.

### The state layer is partially prepared for Leptos

- [crates/bominal-server/src/state.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-server/src/state.rs) already includes `leptos_options` and a `FromRef` bridge, which strongly suggests the intended Phase 8 shape is to keep one Axum state type shared across API and SSR.
- The test harness in [crates/bominal-server/tests/common/mod.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-server/tests/common/mod.rs) already constructs `SharedState` with `LeptosOptions`, but the production router path has not caught up yet.
- That mismatch means Phase 8 is likely more integration completion than greenfield design.

### The app crate still needs an SSR shell contract

- [crates/bominal-app/src/lib.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/lib.rs) exposes `App` and the hydrate entry point, but not the full document/shell function the earlier donor crate used.
- The current `index.html` and Trunk-style setup are still driving the shell for client builds, so Phase 8 will need to decide whether the shell lives in `bominal-app` or is wrapped in `bominal-server`.
- Because the current app already uses `leptos_meta`, the shell can stay thin: title/meta/css/script asset wiring is already conceptually in place.

## Recommendation

Use two plans:

1. Replace the legacy SPA fallback with Leptos SSR routing and cargo-leptos static asset serving, including the missing `LeptosOptions` initialization.
2. Finish the shared-state integration so server functions and SSR rendering can access the existing server dependencies without splitting the application state model.
