# Phase 10: Cleanup - Research

**Researched:** 2026-03-27
**Confidence:** HIGH

## Summary

The active application no longer depends on the old `frontend/` tree or the donor `crates/bominal-frontend/` crate. The remaining Phase 10 work is cleanup of stale repo surfaces: delete the dead trees, remove Trunk-era files that still point at deleted CSS, rewrite the repo guide, and then re-run the current cargo-leptos verification flow to prove the migration is self-contained.

## Key Findings

### The live build path is already independent of the removed frontend trees

- [Cargo.toml](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/Cargo.toml) only lists `crates/bominal-app` as the frontend crate in workspace members and cargo-leptos metadata.
- [crates/bominal-server/src/routes.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-server/src/routes.rs) now serves Leptos SSR and static assets from `target/site`, so `frontend/build` is no longer part of production routing.
- [dev-build.sh](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/dev-build.sh), [deployment/build.sh](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/deployment/build.sh), [deployment/bootstrap.sh](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/deployment/bootstrap.sh), and [Dockerfile](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/Dockerfile) already use cargo-leptos and no longer require npm.

### The remaining contradictions are documentation and dead artifacts

- [CLAUDE.md](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/CLAUDE.md) still claims the frontend is a SvelteKit SPA and still prescribes `cd frontend && npm run build`.
- [crates/bominal-app/index.html](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/index.html) still carries `data-trunk` wiring and references [crates/bominal-app/style/main.css](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/style/main.css), which imports `../../frontend/src/styles/liquid-glass.css`.
- The remaining `bominal-frontend` mentions outside deleted code are confined to stale migration docs and one stale service-layer doc comment.

### Final quality-gate closeout can reuse earlier phase evidence plus a few direct measurements

- QG-01/QG-02/QG-03/QG-04/QG-06 already have strong phase-level evidence in the existing verification reports for routing, interactive islands, i18n, SSE-backed pages, and shared component/design-system adoption.
- QG-05 still needs a direct bundle-size measurement on the final release output.
- QG-07 still needs an explicit post-cleanup verification pass proving the active scripts/docs no longer advertise npm/Node.js.

## Recommendation

Use one execution plan:

1. Delete the dead frontend trees and Trunk-era app-shell leftovers.
2. Rewrite current documentation and fix the remaining stale code comments/planning references that contradict the final architecture.
3. Re-run cargo-leptos/app/server verification, measure the gzipped WASM bundle, and close out CLEAN/QG requirements in the planning state.
