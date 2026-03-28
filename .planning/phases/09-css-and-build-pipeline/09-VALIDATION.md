---
phase: 9
slug: css-and-build-pipeline
status: ready
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-27
---

# Phase 9 — Validation Strategy

## Automated Verification Contract

| Scope | Command | Result |
|-------|---------|--------|
| cargo-leptos frontend path | `./dev-build.sh --frontend-only` | Required |
| App SSR build | `cargo check -p bominal-app --features ssr` | Required |
| App hydrate build | `cargo check -p bominal-app --target wasm32-unknown-unknown --features hydrate` | Required |
| Server SSR build | `cargo check -p bominal-server --features ssr` | Required |
| Deployment script syntax | `bash -n deployment/build.sh && bash -n deployment/bootstrap.sh` | Required |

## Artifact Checks

| Artifact | Expectation |
|----------|-------------|
| [Cargo.toml](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/Cargo.toml) | cargo-leptos metadata no longer depends on the stale Tailwind JS config file |
| [crates/bominal-app/style/app.css](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/style/app.css) | Tailwind v4 import/source contract scoped to the active Leptos app sources |
| [dev-build.sh](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/dev-build.sh) | local build helper routes through cargo-leptos |
| [Makefile](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/Makefile) | build/watch/serve helpers route through cargo-leptos instead of donor frontend CSS/TS tasks |
| [deployment/build.sh](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/deployment/build.sh) | production build entry point uses cargo-leptos release/precompress flow |
| [deployment/bootstrap.sh](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/deployment/bootstrap.sh) | bootstrap no longer treats Node/npm as required for the application build |
| [Dockerfile](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/Dockerfile) | builder uses cargo-leptos + Tailwind standalone and resolves the sibling `bominal-ui` crate through an explicit extra context |

## Manual Verification

If time permits, run a full cold `cargo leptos build --release --precompress` and a full `docker buildx build` to completion. For this migration phase, the faster verification contract above is sufficient because it proves the active frontend/Tailwind pipeline, the server compilation path, and the Docker context strategy all align with the new cargo-leptos workflow.
