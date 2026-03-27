---
phase: 4
slug: auth-pages
status: ready
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-27
---

# Phase 4 — Validation Strategy

## Automated Verification Contract

| Scope | Command | Result |
|-------|---------|--------|
| App unit surface | `cargo test -p bominal-app --lib` | Required |
| App SSR build | `cargo check -p bominal-app --features ssr` | Required |
| App hydrate build | `cargo check -p bominal-app --target wasm32-unknown-unknown --features hydrate` | Required |

## Artifact Checks

| Artifact | Expectation |
|----------|-------------|
| [crates/bominal-app/src/pages/auth/mod.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/pages/auth/mod.rs) | shared auth shell plus real `/auth` landing page with passkey entry |
| [crates/bominal-app/src/pages/auth/login.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/pages/auth/login.rs) | email/password login wired to typed auth server function and shared auth state |
| [crates/bominal-app/src/pages/auth/signup.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/pages/auth/signup.rs) | signup flow with password strength feedback and post-signup verify handoff |
| [crates/bominal-app/src/pages/auth/forgot.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/pages/auth/forgot.rs) | forgot-password request flow with success state |
| [crates/bominal-app/src/pages/auth/verify.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/pages/auth/verify.rs) | resend-verification flow and post-signup verify handoff |
| [crates/bominal-app/src/pages/auth/add_passkey.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/pages/auth/add_passkey.rs) | add-passkey route wired to live WebAuthn registration ceremony |
| [crates/bominal-app/src/pages/verify_email.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/pages/verify_email.rs) | token-based email verification flow |
| [crates/bominal-app/src/pages/reset_password.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/pages/reset_password.rs) | token-based password reset flow |
| [crates/bominal-app/src/api/passkey.rs](/Users/jasonlee/projects/bominal-train/crates/bominal-app/src/api/passkey.rs) | minimal WASM passkey login/register client |
| [crates/bominal-app/assets/passkey-interop.js](/Users/jasonlee/projects/bominal-train/crates/bominal-app/assets/passkey-interop.js) | browser bridge for WebAuthn ceremony serialization |

## Manual Verification

Optional browser verification is recommended for the live passkey ceremony and email-token routes once the server is running, but compile/unit validation is sufficient for this milestone phase because the auth pages only proxy the existing backend endpoints.
