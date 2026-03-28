# Phase 7: Client-Only Interop - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Normalize the browser-only interop surface so passkey login/registration and Evervault-backed card encryption run through a coherent WASM bridge instead of a collection of one-off globals.

</domain>

<decisions>
## Implementation Decisions

### the agent's Discretion
- Treat this phase as an interop-normalization pass, not a new feature pass. Reuse the existing `/api/auth/passkey/*` and `/api/cards` backend contracts.
- Keep the inevitable JavaScript runtime surface small because Evervault is JS-only and WebAuthn binary conversions are still awkward from raw Leptos code. The Rust side should still own the public interop API.
- Collapse the current split browser assets (`passkey-interop.js` and `interop.js`) into one source of truth so later phases are not routing through duplicate runtime helpers.
- Restore the donor conditional-passkey login behavior on the email form if it can be done without destabilizing the existing manual passkey button flow.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/bominal-frontend/ts/interop.ts` already combines Evervault helpers, view-transition support, passkey registration, passkey login, and conditional passkey login in one browser bundle.
- `crates/bominal-frontend/src/api/passkey.rs` includes the conditional-passkey login flow that the current `bominal-app` passkey client does not yet expose.
- `crates/bominal-app/src/browser.rs` already owns theme, locale, redirect, and card-submission browser helpers, making it the right place for the remaining passkey/browser wrappers too.

### Established Patterns
- `crates/bominal-app/src/api/passkey.rs` currently fetches challenges and completes server requests from WASM, but it reaches into browser globals directly with `Reflect`.
- `crates/bominal-app/assets/passkey-interop.js` and `crates/bominal-app/assets/interop.js` are currently split by feature even though both are the browser boundary for the same app.
- `crates/bominal-app/src/pages/auth/login.rs` already owns the email/password entry point and is the natural place to reintroduce conditional passkey mediation/autofill support.

### Integration Points
- `crates/bominal-app/index.html` currently loads both interop assets; Phase 7 should reduce that to one durable browser runtime entry.
- `crates/bominal-app/src/pages/auth/mod.rs` and `crates/bominal-app/src/pages/auth/add_passkey.rs` already call the passkey WASM client, so API churn should be kept low there.
- The settings card-add flow added in Phase 6 already depends on `browser::submit_card`, so any interop consolidation must preserve that runtime contract.

</code_context>

<specifics>
## Specific Ideas

- Use two plans: first normalize the passkey/browser interop surface, then collapse Evervault + passkey into one shared runtime asset and remove the legacy split load path.
- Preserve the current typed `Result`-returning Rust API so auth/settings pages do not have to learn about raw JS values.
- Keep manual passkey login/register intact even if conditional mediation is unavailable or unsupported.

</specifics>

<deferred>
## Deferred Ideas

- Server-side Leptos/Axum integration remains Phase 8.
- CSS/build-pipeline cleanup and npm removal remain Phase 9.
- Full removal of donor artifacts remains Phase 10 cleanup work.

</deferred>
