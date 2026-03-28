# Phase 9: CSS and Build Pipeline - Research

**Date:** 2026-03-27
**Phase:** 9 - CSS and Build Pipeline

## Research Summary

The codebase already has the Leptos-side styling entry points and cargo-leptos metadata needed for a unified build, but the operational scripts still point at the removed `bominal-frontend` crate. The main work for this phase is replacing those stale scripts with cargo-leptos-driven commands and updating Docker/bootstrap to match the current workspace topology.

## Findings

### 1. cargo-leptos is the active build tool

- `cargo leptos --help` is available locally and exposes the exact commands needed for this phase:
  - `cargo leptos build`
  - `cargo leptos build --release --precompress`
  - `cargo leptos watch`
  - `cargo leptos serve`
- Root [Cargo.toml](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/Cargo.toml) already has the active `[[workspace.metadata.leptos]]` block pointing at `bominal-server` + `bominal-app`, with `target/site` as the runtime asset root.

### 2. Tailwind v4 is in play

- The first `cargo leptos build` probe immediately emitted: JavaScript config files are no longer required in Tailwind CSS v4.
- That makes [tailwind.config.js](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/tailwind.config.js) stale by current Tailwind behavior.
- The active stylesheet [crates/bominal-app/style/app.css](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/style/app.css) still uses legacy `@tailwind` directives, so Phase 9 should move it to the current v4-style import/source contract.

### 3. Script surface is still bound to the donor frontend

- [dev-build.sh](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/dev-build.sh) manually runs esbuild, tailwindcss, wasm-bindgen, and `cargo build -p bominal-frontend`.
- [deployment/build.sh](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/deployment/build.sh) repeats the same old flow for releases.
- [Makefile](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/Makefile) still exposes `css`, `css-watch`, and `ts` targets aimed at `crates/bominal-frontend`.

### 4. Docker needs the new workspace shape

- [Dockerfile](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/Dockerfile) currently installs Node, compiles `bominal-frontend`, and manually copies `pkg/` plus donor CSS outputs.
- The active app depends on sibling [bominal-ui](/Users/jasonlee/projects/bominal-ui/Cargo.toml), so a repo-root-only Docker copy step is no longer sufficient for a successful image build.
- Phase 9 therefore needs a Docker strategy that includes the sibling dependency without reintroducing Node.

## Implementation Direction

### Plan 01

Normalize the local build path around cargo-leptos:
- remove the stale Tailwind JS config from metadata/repo
- update the active app stylesheet to the Tailwind v4 import/source style
- replace old local build/watch helpers with cargo-leptos wrappers

### Plan 02

Normalize the deployment path:
- replace manual release build logic with `cargo leptos build --release --precompress`
- update Docker to build the actual Leptos workspace without Node
- update bootstrap to stop requiring npm/Node for the app build flow

## Risks

- Docker verification may require a build context change or named extra context because `bominal-ui` lives beside this repo, not inside it.
- Tailwind v4 source scanning must stay tightly scoped to the Leptos app sources so the build remains deterministic and avoids scanning the donor frontend.
