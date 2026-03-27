# Phase 2: Core Infrastructure - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement the shared application infrastructure for the Leptos migration: real i18n data and locale handling, reusable formatting helpers, shared typed DTOs, `/api/`-proxy server functions, and baseline auth/theme/SSE state surfaces that later routing and page-port phases can consume.

</domain>

<decisions>
## Implementation Decisions

### the agent's Discretion
- All implementation choices are at the agent's discretion — pure infrastructure phase.
- Preserve the Phase 1 architecture decisions: `bominal-app` stays the active crate, `bominal-frontend` stays donor-only, and the app proxies through existing `/api/` endpoints instead of reviving the donor crate's direct server-function architecture.
- Prefer canonical shared sources where they already exist: translation/domain data from `bominal-domain`, UI tokens from `bominal-ui`, and current Svelte/SvelteKit modules only as migration references.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `frontend/src/lib/i18n/{ko,en,ja}.json` already contains the current translation corpus for the Svelte app.
- `frontend/src/lib/types/index.ts` contains the current frontend-facing DTO inventory that mirrors the backend domain.
- `frontend/src/lib/utils.ts` and `crates/bominal-frontend/src/utils.rs` already define the formatting helpers needed by the app surface.
- `frontend/src/lib/stores/{auth,theme,sse}.svelte.ts` documents the existing client-state behavior to port.
- `crates/bominal-frontend/src/api/*.rs` contains donor Leptos API modules that can be adapted to the roadmap's `/api/` proxy approach.

### Established Patterns
- Phase 1 established `bominal-app` as the only active workspace app crate with explicit `ssr`/`hydrate` features.
- The roadmap locks in `/api/` proxying rather than direct service-layer server functions.
- `bominal-domain` is already the best candidate for shared locale and typed domain ownership.

### Integration Points
- New infrastructure modules should live under `crates/bominal-app/src/` and be consumed by later Shell/Auth/Core Pages phases.
- State providers created here will become the root contexts for the Phase 3 app shell.
- Types and API wrappers created here should align with the existing Axum endpoints and backend DTOs to avoid later route rewrites.

</code_context>

<specifics>
## Specific Ideas

No specific requirements — infrastructure phase. Use the roadmap success criteria and existing donor/frontend modules as implementation references.

</specifics>

<deferred>
## Deferred Ideas

None

</deferred>
