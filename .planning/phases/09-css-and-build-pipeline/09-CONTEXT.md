# Phase 9: CSS and Build Pipeline - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Move the remaining operational build surface from the old `bominal-frontend` and Node-based pipeline onto cargo-leptos, while keeping the current Leptos app styling working through the cargo-leptos/Tailwind path and updating Docker/dev workflows to match.

</domain>

<decisions>
## Implementation Decisions

### the agent's Discretion
- Treat `cargo leptos build` and `cargo leptos watch/serve` as the canonical build/dev entry points for this phase.
- Remove stale Node/esbuild/TS assumptions from scripts that are now building the wrong crate (`bominal-frontend`), but avoid Phase 10 documentation cleanup unless it directly blocks the build workflow.
- Keep the current `bominal-app` CSS bridge if it is sufficient to make the pipeline build. This phase is about a working pipeline first, not a full redesign of the styling source-of-truth.
- Update the Docker path so it can build the existing workspace shape, including the sibling `bominal-ui` dependency, without reintroducing Node.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- Root [Cargo.toml](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/Cargo.toml) already contains `[[workspace.metadata.leptos]]` with `site-root = "target/site"`, `site-pkg-dir = "pkg"`, and the active style file at [crates/bominal-app/style/app.css](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/style/app.css).
- `cargo-leptos` is already installed locally and `cargo leptos --help` confirms the release/precompress flow needed for production builds.
- [crates/bominal-app/style/app.css](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/style/app.css) already carries the active train/auth design tokens used by the Leptos app.

### Established Patterns
- [dev-build.sh](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/dev-build.sh), [deployment/build.sh](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/deployment/build.sh), [Makefile](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/Makefile), and [Dockerfile](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/Dockerfile) still build `crates/bominal-frontend`, call `esbuild`, run manual `wasm-bindgen`, and rely on Node/npm-era steps.
- The current server integration from Phase 8 expects runtime static assets under `target/site`, so build outputs must now land there consistently.
- The sibling dependency [bominal-ui](/Users/jasonlee/projects/bominal-ui/Cargo.toml) lives outside this repo at `../bominal-ui`, which means Docker cannot keep using a repo-root-only copy strategy.

### Integration Points
- The first `cargo leptos build` probe emitted a Tailwind v4 warning that JavaScript config files are no longer required, pointing directly at [tailwind.config.js](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/tailwind.config.js) as stale pipeline baggage.
- [deployment/bootstrap.sh](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/deployment/bootstrap.sh) still installs/checks Node/npm and advertises `npm run dev`, so the environment bootstrap also needs to move to cargo-leptos.

</code_context>

<specifics>
## Specific Ideas

- Plan 1 should normalize the local build path: Tailwind v4 stylesheet entry, cargo-leptos metadata cleanup, `dev-build.sh`, and `Makefile`.
- Plan 2 should normalize the deployment path: release build script, Docker image, and bootstrap instructions/tooling.
- Verification should include at least one real `cargo leptos build` run plus the existing app/server cargo checks. If Docker is available, attempt a real image build after the Dockerfile update.

</specifics>

<deferred>
## Deferred Ideas

- Full documentation cleanup in [CLAUDE.md](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/CLAUDE.md) and broader donor/frontend removal remain Phase 10.
- Replacing the current hand-maintained app CSS bridge with a fully generated `bominal-ui` pipeline can wait unless the build cannot be stabilized without it.

</deferred>
