# Phase 3: Shell and Navigation - Research

**Researched:** 2026-03-27
**Confidence:** HIGH

## Summary

The Phase 3 shell can be built directly from existing repo behavior. The current Svelte layout defines the public-route exceptions and auth-check timing, while the donor Leptos app already contains the desired route list and nav component behavior. The migration work here is mostly adaptation: port the shell into `bominal-app`, keep only stub route bodies, and wire it to the Phase 2 `api` and `state` modules.

## Key Findings

### Shell behavior

- `frontend/src/routes/+layout.svelte` performs an auth check on mount, delays protected rendering until the check completes, and hides nav chrome on `/auth*`, `/verify-email`, and `/reset-password`.
- `crates/bominal-frontend/src/app.rs` already lays out the Leptos router structure, including route ordering and a redirect from `/search/results` back to `/search`.
- The route inventory in the roadmap matches the current Svelte app and the donor Leptos app, so there is no route ambiguity to resolve.

### Navigation behavior

- `crates/bominal-frontend/src/components/sidebar.rs` and `bottom_nav.rs` already use `use_location()` to compute active states and suppress navigation on auth/email-flow pages.
- Both nav components currently use the same canonical sections: home, search, tasks, reservations, settings.
- The current donor nav components are self-contained enough to port with minimal changes into `bominal-app`.

### State and bootstrap behavior

- `frontend/src/lib/stores/auth.svelte.ts` defines the auth state contract: `check()`, `setUser()`, `clear()`, `loading`, `checked`, and derived `isAuthenticated`.
- Phase 2 already provides equivalent Rust-side auth/theme/SSE state primitives plus the typed `get_me()` proxy in `crates/bominal-app/src/api.rs`.
- `crates/bominal-frontend/src/theme.rs` and `browser.rs` show the expected cookie names and DOM root attributes for theme/mode, which are useful to seed and reflect shell state.

## Recommendation

Execute Phase 3 in two plans:

1. Replace the Phase 1 placeholder with a real router, auth-bootstrap flow, public/protected layout branching, and stub route components for all required paths.
2. Port and integrate the responsive Sidebar and BottomNav components, then add the shell-specific CSS needed to make the protected layout look intentional on desktop and mobile.
