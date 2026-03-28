# Phase 10: Cleanup - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Finish the migration by removing the dead SvelteKit and donor-Leptos frontend trees, updating repo documentation to the `bominal-app` + cargo-leptos SSR reality, and proving the final build no longer depends on the removed artifacts.

</domain>

<decisions>
## Implementation Decisions

### the agent's Discretion
- Treat `frontend/` and `crates/bominal-frontend/` as removable migration residue. They are no longer part of the workspace or active build path.
- Update documentation to describe the current architecture, not the historical migration staging.
- Prefer deleting stale one-off migration documents that directly prescribe the removed frontend structure over keeping contradictory docs around.
- Validate cleanup with the active cargo-leptos build, app/server compile checks, and an explicit grep for Node/npm references in build surfaces.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Signals
- [crates/bominal-app/src/lib.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/lib.rs) already owns the SSR shell and route table, so cleanup should document `bominal-app` as the app entry point instead of any donor crate.
- [Cargo.toml](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/Cargo.toml), [dev-build.sh](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/dev-build.sh), [deployment/build.sh](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/deployment/build.sh), and [Dockerfile](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/Dockerfile) already point at cargo-leptos and `target/site`.

### Stale Surfaces
- [CLAUDE.md](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/CLAUDE.md) still describes a SvelteKit SPA, npm/Vite workflows, and `frontend/build`.
- [crates/bominal-app/index.html](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/index.html) plus [crates/bominal-app/style/main.css](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/style/main.css) are leftover Trunk-era files, and `main.css` still imports deleted SvelteKit styles.
- [docs/WASM_HYDRATION_PLAN.md](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/docs/WASM_HYDRATION_PLAN.md) and [docs/superpowers/specs/2026-03-12-frontend-rewrite-design.md](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/docs/superpowers/specs/2026-03-12-frontend-rewrite-design.md) are migration-era docs that still prescribe `crates/bominal-frontend`.

</code_context>

<specifics>
## Specific Ideas

- Keep this as a single-plan cleanup phase: doc rewrite, stale artifact deletion, then final build/size/reference verification.
- Use Phase 3, 6, 8, and 9 verification outputs as supporting evidence for the final quality-gate closeout instead of inventing a separate manual QA track.

</specifics>

<deferred>
## Deferred Ideas

- Milestone archival/tagging can follow once the cleanup verification is complete and the user is ready to publish/archive the milestone state.

</deferred>
