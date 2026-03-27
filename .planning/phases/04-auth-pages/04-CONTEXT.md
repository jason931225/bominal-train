# Phase 4: Auth Pages - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Replace the Phase 3 auth/email-route stubs with real authentication page implementations: auth landing, login, signup, forgot password, verify, add-passkey, verify-email, and reset-password.

</domain>

<decisions>
## Implementation Decisions

### the agent's Discretion
- Preserve the route inventory and shell ownership from Phase 3. The new auth pages should slot into the existing router without changing the path map or navigation contract.
- Prefer adapting the partial auth-page work already present in `bominal-app/src/pages/auth` before copying new donor code blindly.
- Use the Phase 2 typed `/api/` proxy layer (`crate::api`) and `AuthState` (`crate::state`) as the source of truth for auth mutations instead of introducing a second auth abstraction.
- Pull the donor passkey WASM flow forward only as far as needed to satisfy the auth-page requirements; broader interop hardening remains Phase 7 work.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/bominal-app/src/pages/auth/{mod,login,signup,forgot}.rs` already contains app-local layout work and form shells for the landing, login, signup, and forgot-password routes.
- `crates/bominal-frontend/src/pages/auth/*.rs` contains the donor Leptos implementations for passkey landing, verify, add-passkey, and the shared auth-shell presentation.
- `crates/bominal-frontend/src/pages/{verify_email_page,reset_password_page}.rs` already covers the token-driven email verification and password reset flows.
- `crates/bominal-frontend/src/api/passkey.rs` contains a workable WASM passkey ceremony client for login and registration.
- `crates/bominal-app/src/api.rs` already exposes typed auth endpoints for login, register, forgot password, resend verification, verify email, reset password, logout, and `get_me()`.

### Established Patterns
- Phase 3 already handles public-route layout and auth bootstrap at the app-shell level, so Phase 4 should focus on page implementations, not router changes.
- Existing `bominal-app` auth page files are UI-complete but still contain placeholder submit handlers.
- The donor auth pages assume a slightly different direct-server-function surface, so they need adaptation to the current `/api/` proxy architecture.

### Integration Points
- Completed auth pages should replace the Phase 3 auth stubs in `shell_pages.rs` or make those shell routes delegate to real page modules.
- Login/register/logout flows must update `AuthState` so the Phase 3 guarded shell transitions correctly.
- Verify-email and reset-password pages need query-param access and async result rendering inside the existing public shell.

</code_context>

<specifics>
## Specific Ideas

- Reuse the existing `bominal-app/src/pages/auth` module as the landing zone for the form-based routes.
- Add a shared auth-shell helper so the routes keep consistent layout and can absorb the donor passkey pages without style drift.
- Treat passkey login/registration as islands rooted in those auth pages, with the low-level WebAuthn transport borrowed from the donor WASM client.

</specifics>

<deferred>
## Deferred Ideas

- Server-side Leptos/Axum transport for page actions remains a later integration concern.
- Deeper interop hardening, broader browser support, and reusable WebAuthn abstractions remain Phase 7 work.

</deferred>
