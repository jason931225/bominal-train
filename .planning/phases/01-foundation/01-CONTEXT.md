# Phase 1: Foundation - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Convert the active frontend crate from CSR-only Leptos to a real Leptos SSR/islands foundation, wire the repo to current `cargo-leptos` workspace conventions, adopt `bominal-ui` as the canonical shared design source, and establish a server/WASM build path that the later routing and page-port phases can build on.

This phase is compile-first infrastructure. It does **not** need to preserve the existing auth pages, replace Axum's SvelteKit fallback routes yet, or deliver visual parity. Those belong to later migration phases.

</domain>

<decisions>
## Implementation Decisions

### Active App Crate
- **D-01:** `crates/bominal-app` remains the active migration target. `crates/bominal-frontend` is treated as a donor/reference crate only during Phase 1, not as the workspace app crate and not as the build target.

### Foundation Scope
- **D-02:** Phase 1 is compile-first. Replace the current CSR-only `bominal-app` surface with a minimal SSR/hydrate shell and temporarily gate or stub unfinished modules rather than trying to keep current auth flows live in this phase.

### cargo-leptos Configuration
- **D-03:** Use current cargo-leptos workspace configuration in the root `Cargo.toml` via `[[workspace.metadata.leptos]]`. Do not introduce a separate `Cargo-leptos.toml` file.
- **D-04:** Add a real `ssr` feature path to `bominal-server` in Phase 1 so `cargo leptos build` targets the actual server crate. Defer the route-level handoff from `ServeDir(frontend/build)` to `leptos_axum` until Phase 8.

### CSS and Assets
- **D-05:** `bominal-ui` is the canonical UI source of truth for Bominal Train. Phase 1 should consume or deterministically derive its CSS inputs from `../bominal-ui`, not from `frontend/src/app.css` as a primary source.
- **D-06:** Carry the shared UI style inputs into `crates/bominal-app` during Phase 1 so cargo-leptos has stable CSS/Tailwind inputs, but treat visual fidelity validation as Phase 9 work.

### Data Flow Architecture
- **D-07:** Keep the roadmap-level decision that the active app will proxy through existing `/api/` endpoints. Do not adopt the dormant `crates/bominal-frontend` direct server-function pattern as the Phase 1 baseline.

### Claude's Discretion
- Exact `cfg(feature = "...")` gating versus temporary stubs for the current CSR modules
- Whether `crates/bominal-frontend` gets a short archival note during the execution phase
- Exact minimal shell markup and file layout inside `crates/bominal-app`

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Planning
- `.planning/ROADMAP.md` — Leptos migration scope, phase boundaries, and dependencies
- `.planning/PROJECT.md` — locked architecture decisions and migration constraints
- `.planning/REQUIREMENTS.md` — FND-01 through FND-05 and the no-npm/end-state constraints
- `.planning/STATE.md` — current milestone state and known migration concerns

### Canonical Shared UI
- `../bominal-ui/Cargo.toml` — canonical shared UI crate manifest and feature surface (`leptos`, `train`, `auth`)
- `../bominal-ui/.planning/PROJECT.md` — shared UI product intent and cross-product design direction
- `../bominal-ui/.planning/ROADMAP.md` — canonical UI library roadmap and skin/archetype direction
- `../bominal-ui/src/lib.rs` — crate entrypoint exposing CSS, icons, palette, and Leptos surfaces
- `../bominal-ui/src/css/mod.rs` — shared CSS API (`ECOSYSTEM_CSS`, product overrides, `full_css`)
- `../bominal-ui/src/css/ecosystem.rs` — ecosystem tokens and shared Liquid Glass utility patterns
- `../bominal-ui/src/css/train.rs` — canonical train skin overrides
- `../bominal-ui/src/css/auth.rs` — canonical auth skin overrides
- `../bominal-ui/src/leptos_components/mod.rs` — currently exported shared Leptos component surface

### Active Workspace State
- `Cargo.toml` — current workspace members and missing Leptos workspace metadata/dependencies
- `crates/bominal-app/Cargo.toml` — current CSR-only app manifest (`leptos` with `csr`, `gloo-net`, no `ssr`/`hydrate`)
- `crates/bominal-app/src/lib.rs` — current `mount_to_body` entrypoint and auth-router prototype
- `crates/bominal-app/src/api.rs` — browser-only API client built around `gloo-net`
- `crates/bominal-app/src/i18n.rs` — SSR-unsafe locale detection through `window.navigator`

### Dormant Donor Crate
- `crates/bominal-frontend/Cargo.toml` — off-workspace Leptos manifest that already expects workspace-scoped Leptos dependencies
- `crates/bominal-frontend/src/lib.rs` — donor crate module map
- `crates/bominal-frontend/src/app.rs` — richer SSR shell prototype and route inventory
- `crates/bominal-frontend/src/hydrate.rs` — `hydrate_body` entrypoint showing pre-islands assumptions
- `crates/bominal-frontend/src/api/mod.rs` — direct server-function orientation that conflicts with the active `/api/` proxy decision

### Server and Build Integration
- `crates/bominal-server/Cargo.toml` — current server manifest with no Leptos build path
- `crates/bominal-server/src/main.rs` — active server binary entrypoint
- `crates/bominal-server/src/routes.rs` — current SvelteKit `frontend/build` static fallback
- `frontend/src/app.css` — current train-local stylesheet; compatibility reference only, not the new source of truth
- `frontend/src/styles/liquid-glass.css` — current train-local design-system CSS; compatibility reference only
- `frontend/package.json` — current Node/Vite/Tailwind toolchain
- `dev-build.sh` — current npm-based build script
- `Dockerfile` — current npm + SvelteKit production build path
- `Makefile` — legacy Tailwind/esbuild flow pointing at `crates/bominal-frontend`

### External References
- No additional local ADR/spec files exist for this phase. External Leptos references used for planning are captured in `01-RESEARCH.md`.

</canonical_refs>

<specifics>
## Specific Ideas

- `cargo check -p bominal-app` currently succeeds, but only in CSR mode.
- `cargo-leptos 0.3.5` is installed locally.
- `wasm32-unknown-unknown` is already installed locally.
- `tailwindcss` is already available on the machine.
- `cargo check --manifest-path ../bominal-ui/Cargo.toml --features 'leptos train auth'` succeeds.
- `crates/bominal-frontend` is not dead code; it is a dormant prototype with useful page/component work that should be mined deliberately rather than deleted blindly.

</specifics>

<deferred>
## Deferred Ideas

- Fully deleting or archiving `crates/bominal-frontend` after code extraction and build-script cleanup
- Replacing Axum's static SPA fallback with `leptos_axum` route rendering
- Updating `dev-build.sh`, `Dockerfile`, and deployment scripts to remove Node.js from the build pipeline
- Visual parity and design-system verification
- Final cleanup of any train-local CSS still duplicated from the SvelteKit app once `bominal-ui` integration is complete

</deferred>

---

*Phase: 01-foundation*
*Context gathered: 2026-03-27*
