---
phase: 7
slug: client-only-interop
status: ready
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-27
---

# Phase 7 — Validation Strategy

## Automated Verification Contract

| Scope | Command | Result |
|-------|---------|--------|
| App unit surface | `cargo test -p bominal-app --lib` | Required |
| App SSR build | `cargo check -p bominal-app --features ssr` | Required |
| App hydrate build | `cargo check -p bominal-app --target wasm32-unknown-unknown --features hydrate` | Required |

## Artifact Checks

| Artifact | Expectation |
|----------|-------------|
| [crates/bominal-app/src/browser.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/browser.rs) | normalized Rust-facing browser interop boundary for passkeys and card submission |
| [crates/bominal-app/src/api/passkey.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/api/passkey.rs) | WASM start/finish flow for login, conditional login, and registration |
| [crates/bominal-app/src/pages/auth/login.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/pages/auth/login.rs) | conditional passkey mediation plus manual passkey fallback on the email form |
| [crates/bominal-app/src/pages/auth/add_passkey.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/pages/auth/add_passkey.rs) | passkey registration flow still calling the WASM client from the protected auth route |
| [crates/bominal-app/src/pages/settings.rs](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/src/pages/settings.rs) | encrypted card-add flow still routed through the Rust/browser bridge |
| [crates/bominal-app/assets/interop.js](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/assets/interop.js) | single browser runtime asset for Evervault and WebAuthn |
| [crates/bominal-app/index.html](/Users/jasonlee/projects/bominal-train/.claude/worktrees/gsd-autonomous-phase4/crates/bominal-app/index.html) | only the unified interop asset loaded at runtime |

## Manual Verification

Optional browser validation is recommended for real passkey ceremonies and the encrypted card-add path, but compile and code-path verification are sufficient for this migration phase because the browser runtime is now explicitly centralized and still targets the same backend endpoints already used in earlier phases.
