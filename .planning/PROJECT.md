# Bominal

## What This Is

Bominal is a Korean train reservation SaaS that searches for available seats across SRT and KTX providers, auto-books them, and handles payment with encrypted card details. The app has an Axum REST API backend with a SvelteKit SPA frontend using a glass morphism design system with Korean/English/Japanese i18n.

## Core Value

Users can search both train providers simultaneously, create an auto-booking task, and pay securely with encrypted card details -- end to end.

## Requirements

### Validated

- Station suggest API exists on backend (`/api/stations/{provider}/suggest`)
- Auth flow works end-to-end (signup, login, passkey, email verification)
- Task CRUD API exists (`/api/tasks`)
- SSE real-time task updates (`/api/tasks/events`)
- Reservation API exists (`/api/reservations`)
- Card management API exists (`/api/cards`)
- Provider credential API exists (`/api/providers`)
- UI shell complete: home, search, tasks, reservations, settings pages
- Glass morphism design system with light/dark mode and two themes
- Bottom nav with 5 tabs (home, search, tasks, reservations, settings)
- i18n framework (ko/en/ja) in place
- Task runner polls and executes booking tasks
- Evervault backend integration for card tokenization
- AES-256-GCM encryption for provider credentials at rest

### Active

- [ ] Station input autocomplete connected to suggest API
- [ ] Date picker — modal or full-page overlay replacing raw YYYYMMDD input
- [ ] Time band selection — proper UI replacing raw slider
- [ ] Multi-provider search — search both SRT + KTX simultaneously
- [ ] Provider auth flow — graceful handling when credentials missing (prompt to add)
- [ ] Task scheduling — merge results from multiple providers
- [ ] Payment — Evervault JS SDK on frontend, store encrypted ciphertext
- [ ] i18n completeness — all error messages in user's locale (no English fallbacks)
- [ ] Fix search button disabled state logic
- [ ] Settings provider credential setup flow (currently shows "미설정")

### Out of Scope

- Mobile native app (React Native / Flutter) -- web SPA only for now
- Real-time seat availability push notifications
- Multi-user / team accounts
- Fare comparison / price alerts
- Train schedule browsing (only search-to-book flow)

## Context

- **Stack**: Axum 0.8 (Rust) + SvelteKit 2.16 (TypeScript) + PostgreSQL 16 + Valkey
- **Providers**: SRT and KTX (Korean high-speed rail operators)
- **Migration**: Recently migrated from Leptos SSR+WASM to SvelteKit SPA. Legacy `bominal-frontend` crate is dead code.
- **Encryption**: Evervault for card tokenization, AES-256-GCM for provider API credentials
- **Auth**: WebAuthn passkeys (primary) + email/password (fallback) + Argon2 hashing
- **Design**: Apple-inspired glass morphism with Inter font, two themes (Glass, Clear Sky), light/dark mode
- **Build**: `./dev-build.sh` builds SvelteKit frontend + Rust server. Frontend at `frontend/`, served from `frontend/build/` by Axum.

## Constraints

- **Tech stack**: Rust backend + SvelteKit frontend -- no changes to this
- **Providers**: Must support both SRT and KTX APIs
- **Security**: Card data must never touch our servers unencrypted (Evervault requirement)
- **i18n**: Korean is primary language, all user-facing strings must be localized

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| SvelteKit over Leptos for frontend | Faster iteration, better ecosystem for UI work | -- Pending |
| Evervault for card encryption | PCI compliance without self-hosting | -- Pending |
| adapter-static (SPA) not SSR | Simpler deployment, API-first architecture | -- Pending |
| Multi-provider simultaneous search | Users want best available across both providers | -- Pending |
| Date/time picker as modal overlay | Better mobile UX than inline form controls | -- Pending |

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
*Last updated: 2026-03-24 after initialization*
