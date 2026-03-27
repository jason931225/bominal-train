# Phase 4: Auth Pages - Research

**Researched:** 2026-03-27
**Confidence:** HIGH

## Summary

Phase 4 is mostly adaptation work, not greenfield UI work. `bominal-app` already has partial landing/login/signup/forgot pages with the right styling direction, while the donor Leptos crate already has the missing verify/add-passkey/email-token pages and the passkey WASM client. The missing work is unifying those assets around the Phase 2 `/api/` proxy layer and the Phase 3 shell/auth state.

## Key Findings

### Existing app-local auth pages

- `crates/bominal-app/src/pages/auth/mod.rs` already contains a branded `/auth` landing screen with passkey and email-entry buttons.
- `crates/bominal-app/src/pages/auth/login.rs`, `signup.rs`, and `forgot.rs` already provide structured form UIs, but their submit handlers are still placeholder logic.
- Those files are currently outside the active Phase 3 shell path because the shell routes still point at `shell_pages.rs` stubs.

### Donor Leptos auth coverage

- `crates/bominal-frontend/src/pages/auth/passkey_page.rs` and `add_passkey_page.rs` already show how the passkey login/register buttons should behave from WASM.
- `crates/bominal-frontend/src/pages/auth/verify_page.rs` already models the post-signup verify page using current-user lookup and resend-verification behavior.
- `crates/bominal-frontend/src/pages/verify_email_page.rs` and `reset_password_page.rs` already cover the token-query flows needed for `/verify-email` and `/reset-password`.

### API and state alignment

- `crates/bominal-app/src/api.rs` already has the auth endpoint surface Phase 4 needs: `login`, `register`, `forgot_password`, `verify_email`, `reset_password`, `resend_verification`, `logout`, and `get_me`.
- `AuthState` in `crates/bominal-app/src/state.rs` already exposes the `set_user`, `clear`, and `is_authenticated` semantics the pages need after successful auth mutations.
- `crates/bominal-frontend/src/api/passkey.rs` is donor-only right now, but it already targets the same `/api/auth/passkey/...` endpoints the Svelte app uses, so it can be adapted with limited churn.

## Recommendation

Break Phase 4 into three plans:

1. Land a shared auth-shell/page-module structure in `bominal-app` and wire the landing/login/signup/forgot routes to the typed auth APIs plus `AuthState`.
2. Port the remaining verify/add-passkey/verify-email/reset-password routes from the donor crate and adapt them to the app-local state/API surface.
3. Integrate the donor passkey WASM client so the landing and add-passkey pages can actually trigger browser credential ceremonies without waiting for the full Phase 7 interop pass.
