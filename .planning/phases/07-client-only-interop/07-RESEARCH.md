# Phase 7: Client-Only Interop - Research

**Researched:** 2026-03-27
**Confidence:** HIGH

## Summary

Most of Phase 7 already exists in fragments. The current app can perform passkey login/registration and card encryption, but those capabilities are split across two browser assets and partially bypass the app-local browser boundary. The donor `bominal-frontend` code shows the cleaner end state: one interop asset, browser helpers as the stable Rust-facing contract, and conditional passkey login restored on the email form.

## Key Findings

### The current app already has the raw capabilities

- `crates/bominal-app/src/api/passkey.rs` fetches WebAuthn challenges and finishes the server-side ceremony for login and registration.
- `crates/bominal-app/assets/passkey-interop.js` provides the binary conversion and `navigator.credentials.get/create()` glue for those ceremonies.
- `crates/bominal-app/assets/interop.js` provides Evervault initialization and encrypted card submission for the Phase 6 settings page.

### The main gap is normalization, not greenfield behavior

- The current passkey client reaches directly into browser globals with `Reflect` instead of routing through the app-local browser helper layer.
- `index.html` currently loads two separate runtime assets even though the donor code keeps them unified in one browser bundle.
- The current login page lacks the donor conditional-passkey mediation flow, so passkey autofill support regressed even though the underlying JS bridge function already exists in donor code.

### The donor code gives a low-risk migration target

- `crates/bominal-frontend/ts/interop.ts` is already the combined passkey + Evervault source of truth, so the app-local runtime asset can converge toward that shape.
- `crates/bominal-frontend/src/api/passkey.rs` includes `do_conditional_passkey_login()`, which can be ported with only small app-local adjustments.
- The existing Phase 6 `browser::submit_card()` helper proves the current app is already comfortable with a Rust-owned interop boundary around JS-only SDKs.

## Recommendation

Use two plans:

1. Move passkey ceremony calls behind `browser.rs`, add conditional-passkey login support, and keep the auth pages using the same high-level Rust API.
2. Consolidate passkey and Evervault helpers into one app-local runtime asset, update `index.html` to load only that asset, and delete the legacy split passkey script.
