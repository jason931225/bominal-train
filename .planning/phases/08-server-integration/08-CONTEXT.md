# Phase 8: Server Integration - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Replace the current SvelteKit static-site fallback in `bominal-server` with Leptos SSR served directly by Axum, while preserving the existing `/api` surface and wiring the shared server state into the Leptos request pipeline.

</domain>

<decisions>
## Implementation Decisions

### the agent's Discretion
- Keep the `/api` router tree intact. Phase 8 should change how non-API routes are served, not rewrite backend endpoint behavior.
- Use the existing cargo-leptos workspace metadata (`site-root = target/site`, `site-pkg-dir = pkg`) as the static asset contract instead of inventing another output location.
- Treat `SharedState` as the source of truth for database, email, SSE, encryption, Evervault, and WebAuthn dependencies. Leptos SSR should consume that state rather than creating a parallel server context.
- Prefer introducing a small SSR shell entry point in `bominal-app` or a thin server wrapper over trying to force the current `index.html` SPA shell into server rendering.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- Root [Cargo.toml](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/Cargo.toml) already has cargo-leptos metadata pointing the build at `target/site` and `pkg`.
- [crates/bominal-server/Cargo.toml](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-server/Cargo.toml) already exposes an `ssr` feature that depends on `bominal-app`, `leptos`, and `leptos_axum`.
- [crates/bominal-server/src/state.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-server/src/state.rs) already includes `leptos_options` plus `FromRef<SharedState> for LeptosOptions`, and the server test helpers already populate it.

### Established Patterns
- [crates/bominal-server/src/routes.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-server/src/routes.rs) currently nests `/api`, exposes `/health` and `/metrics`, and then falls back to `ServeDir("frontend/build")` with `index.html`.
- [crates/bominal-app/src/lib.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/lib.rs) exposes the `App` component and hydrate entry point but does not yet expose a full SSR shell document.
- The current `bominal-app` pages assume the same typed server/API contract from earlier phases, so SSR integration can focus on routing and context rather than page rewrites.

### Integration Points
- `create_router()` in [crates/bominal-server/src/routes.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-server/src/routes.rs) still hardcodes the old SPA build directory and will need to switch to Leptos SSR plus static asset serving from the cargo-leptos output.
- `SharedState` already expects `leptos_options`, but the production state construction in [crates/bominal-server/src/routes.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-server/src/routes.rs) does not yet populate that field.
- Phase 8 must keep CSP, compression, CORS, request IDs, and tracing intact while changing the fallback routing behavior.

</code_context>

<specifics>
## Specific Ideas

- Split the phase into router/static integration first and shared-state/Leptos-context integration second.
- Use the existing `bominal-app` crate as the SSR view source and add only the missing shell/route-list pieces required by `leptos_axum`.
- Verify the server crate itself with `cargo check -p bominal-server --features ssr` once the integration lands, in addition to the app checks already used in earlier phases.

</specifics>

<deferred>
## Deferred Ideas

- Tailwind CLI/dev-build/Docker changes remain Phase 9.
- Donor `frontend/` removal and documentation cleanup remain Phase 10.

</deferred>
