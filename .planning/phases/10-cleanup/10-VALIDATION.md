---
phase: 10
slug: cleanup
status: ready
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-27
---

# Phase 10 — Validation Strategy

## Automated Verification Contract

| Scope | Command | Result |
|-------|---------|--------|
| App unit + route inventory tests | `cargo test -p bominal-app --lib` | Required |
| Final frontend release output | `cargo leptos build --release --frontend-only` | Required |
| App SSR build | `cargo check -p bominal-app --features ssr` | Required |
| App hydrate build | `cargo check -p bominal-app --target wasm32-unknown-unknown --features hydrate` | Required |
| Server SSR build | `cargo check -p bominal-server --features ssr` | Required |
| Deployment script syntax | `bash -n deployment/build.sh && bash -n deployment/bootstrap.sh` | Required |
| Final stale-reference sweep | `rg -n "npm|Node\\.js|frontend/build|bominal-frontend|SvelteKit|vite"` on the active repo docs/build surfaces | Required |

## Artifact Checks

| Artifact | Expectation |
|----------|-------------|
| [CLAUDE.md](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/CLAUDE.md) | describes the current Leptos SSR/cargo-leptos architecture and commands |
| `frontend/` | removed |
| `crates/bominal-frontend/` | removed |
| [crates/bominal-service/src/lib.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-service/src/lib.rs) | no longer references the removed donor crate |
| [crates/bominal-app/index.html](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/index.html) and [crates/bominal-app/style/main.css](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/style/main.css) | removed so no build path points at deleted SvelteKit styles |
| [docs/WASM_HYDRATION_PLAN.md](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/docs/WASM_HYDRATION_PLAN.md) and [docs/superpowers/specs/2026-03-12-frontend-rewrite-design.md](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/docs/superpowers/specs/2026-03-12-frontend-rewrite-design.md) | no longer present as active contradictory guidance |

## Manual Verification

No new manual UI pass is required for this cleanup phase. Phase-level verification from earlier phases supplies the routing, interaction, i18n, and SSE evidence; this phase only needs the final cleanup/build/reference proof plus the release bundle-size measurement.
