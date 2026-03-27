# Phase 3: Shell and Navigation - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Build the first navigable Leptos application shell in `bominal-app`: router setup, public vs protected layout branching, all route definitions, and responsive desktop/mobile navigation chrome.

</domain>

<decisions>
## Implementation Decisions

### the agent's Discretion
- Preserve the existing Svelte route inventory exactly: `/`, `/auth`, `/auth/login`, `/auth/signup`, `/auth/forgot`, `/auth/verify`, `/auth/add-passkey`, `/home`, `/search`, `/tasks`, `/reservations`, `/settings`, `/verify-email`, `/reset-password`.
- Keep the migration layered: Phase 3 provides route stubs and shell/navigation wiring; Phase 4 and later phases will replace those stubs with real page implementations.
- Reuse Phase 2 foundations instead of introducing a second auth/theme abstraction. The new shell should consume `crate::api`, `crate::i18n`, `crate::state`, and `crate::types`.
- Preserve the established Bominal train visual language from the donor Leptos app and current Svelte layout rather than inventing a new shell design.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `frontend/src/routes/+layout.svelte` contains the current auth-check flow and the public-route exceptions that suppress nav chrome.
- `crates/bominal-frontend/src/app.rs` already demonstrates the desired Leptos route inventory and the overall shell split between auth routes and app routes.
- `crates/bominal-frontend/src/components/sidebar.rs` and `bottom_nav.rs` already capture the active-state and hide/show logic for navigation chrome.
- `frontend/src/lib/stores/auth.svelte.ts` defines the current check/set/clear auth semantics that the Leptos shell should preserve.
- `crates/bominal-frontend/src/theme.rs` and `browser.rs` show the expected theme cookie names and root DOM attribute behavior for `data-theme` and `data-mode`.

### Established Patterns
- Phase 2 already introduced `AuthState`, `ThemeState`, `SseState`, `get_me()`, and the shared typed route/data surface in `bominal-app`.
- The current `bominal-app` root still renders a Phase 1 placeholder; Phase 3 should replace that with router-driven layout while keeping the compile-first verification style.
- `bominal-ui` supplies the visual tokens, but `crates/bominal-app/style/app.css` still needs local shell/navigation classes for this migration stage.

### Integration Points
- Routing and layout should live in `crates/bominal-app/src/lib.rs`, with supporting components/pages split into local modules.
- Navigation components should use `leptos_router::hooks::use_location()` so active-page styling stays reactive.
- App shell auth gating should source from Phase 2 state plus the typed `get_me()` proxy, even though full server-function routing is not integrated until Phase 8.

</code_context>

<specifics>
## Specific Ideas

- Route `/` should redirect to `/auth` for unauthenticated users and `/home` for authenticated users.
- Public routes are the auth/email flows; protected routes are the main application pages.
- Desktop should render a sidebar, mobile should render a floating bottom nav, and protected content should sit inside a reusable shell frame.

</specifics>

<deferred>
## Deferred Ideas

- Real auth form/page implementations stay in Phase 4.
- Real home/search/tasks/reservations/settings content stays in Phases 5 and 6.
- Full server-side redirect integration and Leptos/Axum SSR wiring stay in Phase 8.

</deferred>
