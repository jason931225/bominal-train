# Bominal

## What This Is

Bominal is a Korean train reservation SaaS that searches for available seats across SRT and KTX providers, auto-books them, and handles payment with encrypted card details. The app now runs as an Axum-served Leptos 0.8 SSR application with selective hydration, using a glass-morphism design system and Korean/English/Japanese i18n.

## Core Value

Users can search both train providers simultaneously, create an auto-booking task, and pay securely with encrypted card details -- end to end.

## Current Milestone

**Leptos SSR Migration**: Completed in the working tree. The Axum server now serves the Leptos app directly, all 14 routes and shared components are ported, passkey/Evervault interop is wired through the app-local browser boundary, and the build pipeline is cargo-leptos based.

## Requirements

### Validated

- Station suggest API exists on backend (`/api/stations/{provider}/suggest`)
- Auth flow works end-to-end (signup, login, passkey, email verification)
- Task CRUD API exists (`/api/tasks`)
- SSE real-time task updates (`/api/tasks/events`)
- Reservation API exists (`/api/reservations`)
- Card management API exists (`/api/cards`)
- Provider credential API exists (`/api/providers`)
- Glass morphism design system with light/dark mode and two themes
- Bottom nav with 5 tabs (home, search, tasks, reservations, settings)
- i18n framework (ko/en/ja) in place
- Task runner polls and executes booking tasks
- Evervault backend integration for card tokenization
- AES-256-GCM encryption for provider credentials at rest
- Leptos 0.8 SSR + islands app shell served directly from Axum
- All 14 roadmap routes are defined in `bominal-app`
- cargo-leptos build pipeline replaces the old JS-package-manager frontend build flow
- Shared `bominal-ui` train/auth styling renders through the active app stylesheet
- Final cleanup removed the obsolete `frontend/` and donor Leptos crate trees

### Active

- [ ] Milestone archival/tagging and next-milestone setup

### Out of Scope

- Mobile native app -- web only for now
- New features beyond SvelteKit parity -- faithful port only
- Direct service-layer server functions -- using /api/ proxy for Phase 1
- Backend API changes -- all endpoints remain unchanged

## Context

- **Stack**: Axum 0.8 (Rust) backend + Leptos 0.8 SSR frontend + PostgreSQL 16 + Valkey
- **Frontend crate**: `crates/bominal-app/` (active SSR/islands app)
- **Providers**: SRT and KTX (Korean high-speed rail operators)
- **Encryption**: Evervault for card tokenization, AES-256-GCM for provider API credentials
- **Auth**: WebAuthn passkeys (primary) + email/password (fallback) + Argon2 hashing
- **Design**: Apple-inspired glass morphism with Inter font, two themes (Glass, Clear Sky), light/dark mode
- **Build**: cargo-leptos + Tailwind standalone CLI, static assets emitted to `target/site`

## Constraints

- **Tech stack**: Rust backend + Leptos 0.8 SSR frontend (islands architecture)
- **Providers**: Must support both SRT and KTX APIs
- **Security**: Card data must never touch our servers unencrypted (Evervault requirement)
- **i18n**: Korean is primary language, all user-facing strings must be localized (ko/en/ja)
- **Parity**: The Leptos app must preserve the migrated user flows across auth, search, tasks, reservations, and settings
- **Rust-first build**: Final build pipeline must not require a JS package manager

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Leptos 0.8 islands over full hydration | Minimizes WASM bundle, SSR improves initial load | Adopted |
| Server functions proxy to /api/ (Option B) | Simpler, ships faster than direct service-layer access | Adopted |
| cargo-leptos for build | Standard Leptos build tool, handles dual SSR+WASM | Adopted |
| Tailwind standalone CLI (no npm) | Eliminates Node.js from build pipeline | Adopted |
| Rewrite bominal-app in place | Existing crate has salvageable patterns | Adopted |
| Remove donor frontend trees at migration close | Prevent stale docs/build surfaces from surviving the cutover | Adopted |
| SvelteKit to Leptos migration | Full-stack Rust, eliminate JS toolchain, better performance | Adopted |
| Evervault for card encryption | PCI compliance without self-hosting | Retained |
| Multi-provider simultaneous search | Users want best available across both providers | Retained |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? -> Move to Out of Scope with reason
2. Requirements validated? -> Move to Validated with phase reference
3. New requirements emerged? -> Add to Active
4. Decisions to log? -> Add to Key Decisions
5. "What This Is" still accurate? -> Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check -- still the right priority?
3. Audit Out of Scope -- reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-03-27 -- migration implementation complete, archival pending*
