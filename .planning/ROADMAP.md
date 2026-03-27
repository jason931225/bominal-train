# Roadmap: Bominal Leptos SSR Migration

## Overview

Rewrite the bominal-train frontend from SvelteKit 2.16 SPA to Leptos 0.8 SSR with selective hydration (islands architecture). The Leptos app will be served directly by the existing Axum server, eliminating the separate npm build step. All 14 routes, 15 components, 3 stores, 6 API modules, the i18n system (ko/en/ja), SSE real-time updates, and the train/auth surfaces will be ported onto the shared `bominal-ui` design system. No backend API changes required.

**Replaces:** Previous SvelteKit feature-wiring roadmap (2026-03-24)
**Migration scope:** 10 route groups, 15 components, 283 i18n keys x 3 locales, SSE, WebAuthn, Evervault interop, and shared `bominal-ui` adoption

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3...): Planned migration work
- Decimal phases (2.1, 2.2): Urgent insertions if needed

- [x] **Phase 1: Foundation** - Crate setup, cargo-leptos config, workspace deps, `bominal-ui` wiring, dual compilation verified (completed 2026-03-27)
- [x] **Phase 2: Core Infrastructure** - i18n, utils, types, server functions (API layer), state management (completed 2026-03-27)
- [x] **Phase 3: Shell and Navigation** - Root App with router, layout, Sidebar, BottomNav (completed 2026-03-27)
- [ ] **Phase 4: Auth Pages** - All 8 auth-related routes (landing, login, signup, forgot, verify, add-passkey, verify-email, reset-password)
- [ ] **Phase 5: Core Pages** - Home, search, tasks, reservations (the main application)
- [ ] **Phase 6: Settings and Shared Components** - Settings page, remaining SSR and island components
- [ ] **Phase 7: Client-Only Interop** - WebAuthn passkey and Evervault card encryption via web-sys/wasm-bindgen
- [ ] **Phase 8: Server Integration** - Replace SPA serving with leptos_axum, merge state, wire SSR handler
- [ ] **Phase 9: CSS and Build Pipeline** - Tailwind CSS migration (no npm), Dockerfile update, dev workflow
- [ ] **Phase 10: Cleanup** - Remove frontend/, update documentation, final verification

## Phase Details

### Phase 1: Foundation
**Goal**: Leptos 0.8 foundation compiles in both SSR and hydrate modes with cargo-leptos orchestrating the build and `bominal-ui` wired as the shared UI source
**Depends on**: Nothing (first phase)
**Requirements**: FND-01, FND-02, FND-03, FND-04, FND-05
**Success Criteria** (what must be TRUE):
  1. `bominal-frontend` remains outside the active cargo-leptos build path and misleading workspace references are removed
  2. `bominal-app/Cargo.toml` has ssr/hydrate features with islands architecture deps
  3. Workspace `[workspace.dependencies]` includes leptos, leptos_meta, leptos_router, leptos_axum
  4. Root `Cargo.toml` contains `[[workspace.metadata.leptos]]` with the correct bin-package/lib-package/feature config
  5. `bominal-app` depends on `bominal-ui` as the canonical shared design crate
  6. `cargo check --features ssr` succeeds on bominal-app
  7. `cargo check --target wasm32-unknown-unknown --features hydrate` succeeds on bominal-app
**Plans:** 2/2 plans complete
Plans:
- [ ] 01-01-PLAN.md -- Keep donor crate out of the active build path, rewrite bominal-app Cargo.toml with SSR/hydrate features, add workspace deps, wire `bominal-ui`
- [ ] 01-02-PLAN.md -- Configure cargo-leptos build system, create minimal App shell, materialize `bominal-ui`-backed CSS, verify dual compilation

### Phase 2: Core Infrastructure
**Goal**: All foundational modules (i18n, types, utils, API layer, state) are implemented and unit-tested
**Depends on**: Phase 1
**Requirements**: INFRA-01, INFRA-02, INFRA-03, INFRA-04, INFRA-05
**Success Criteria** (what must be TRUE):
  1. i18n `t()` function returns correct translations for all 283 keys in ko/en/ja
  2. All 8 utility functions produce correct output (format_time, format_date, etc.)
  3. All 20 shared types compile and round-trip through serde
  4. Server functions proxy to /api/ endpoints and return typed responses
  5. Auth context, theme context, and SSE store are provided and accessible
**Plans:** 3/3 plans complete

### Phase 3: Shell and Navigation
**Goal**: Navigable application shell with auth-guarded routing, sidebar, and bottom nav
**Depends on**: Phase 2
**Requirements**: SHELL-01, SHELL-02, SHELL-03
**Success Criteria** (what must be TRUE):
  1. Root App renders with router, redirecting unauthenticated users to /auth
  2. Authenticated layout shows Sidebar (desktop) and BottomNav (mobile)
  3. Current page is highlighted in navigation
  4. All 14 route paths are defined (even if pages are stubs)
**Plans:** 2/2 plans complete

### Phase 4: Auth Pages
**Goal**: Complete authentication flow — users can sign up, log in, use passkeys, verify email, and reset password
**Depends on**: Phase 3
**Requirements**: AUTH-01, AUTH-02, AUTH-03, AUTH-04, AUTH-05, AUTH-06
**Success Criteria** (what must be TRUE):
  1. Auth landing page renders with passkey login button (island)
  2. Login form submits email/password and authenticates
  3. Signup form registers new users with password strength feedback
  4. Forgot password sends reset email
  5. Email verification and password reset token flows work end-to-end
  6. Add-passkey page triggers WebAuthn registration
**Plans:** 3 plans (estimated)
**UI hint**: yes

### Phase 5: Core Pages
**Goal**: Main application pages — home dashboard, train search with results, task management, reservation list
**Depends on**: Phase 4
**Requirements**: PAGE-01, PAGE-02, PAGE-03, PAGE-04
**Success Criteria** (what must be TRUE):
  1. Home page shows active tasks summary with pull-to-refresh
  2. Search page supports station autocomplete, date/time selection, multi-provider results
  3. Tasks page shows active/completed tabs with SSE real-time updates and swipe-to-cancel
  4. Reservations page shows tickets with provider filter and pay/cancel/refund actions
**Plans:** 4 plans (estimated, one per page — search is the most complex)
**UI hint**: yes

### Phase 6: Settings and Shared Components
**Goal**: Settings page fully ported, all shared components available as SSR or island variants
**Depends on**: Phase 5
**Requirements**: SETT-01, COMP-01, COMP-02
**Success Criteria** (what must be TRUE):
  1. Settings page has provider credential management, card management, appearance toggles, logout
  2. Pure SSR components (GlassPanel, StatusChip, Skeleton, Icon, CardBrand) render correctly
  3. Interactive components (BottomSheet, SelectionPrompt, TicketCard, TaskCard) hydrate correctly
**Plans:** 2 plans (estimated)
**UI hint**: yes

### Phase 7: Client-Only Interop
**Goal**: WebAuthn passkey and Evervault card encryption work in WASM islands
**Depends on**: Phase 4 (passkey pages), Phase 6 (card section in settings)
**Requirements**: INTEROP-01, INTEROP-02
**Success Criteria** (what must be TRUE):
  1. Passkey login via navigator.credentials.get() works from WASM
  2. Passkey registration via navigator.credentials.create() works from WASM
  3. Evervault card encryption via JS SDK wrapper works from WASM
**Plans:** 2 plans (estimated)

### Phase 8: Server Integration
**Goal**: Axum server serves Leptos SSR instead of static SvelteKit files
**Depends on**: Phase 7
**Requirements**: SRV-01, SRV-02, SRV-03
**Success Criteria** (what must be TRUE):
  1. Non-API routes return server-rendered HTML from Leptos
  2. WASM bundle and CSS served as static assets
  3. All existing /api/ endpoints still work unchanged
  4. Server functions can access SharedState (DB pool, encryption key)
**Plans:** 2 plans (estimated)

### Phase 9: CSS and Build Pipeline
**Goal**: Shared `bominal-ui` styling and the remaining CSS pipeline build without npm, Dockerfile updated, dev workflow uses cargo-leptos
**Depends on**: Phase 8
**Requirements**: CSS-01, CSS-02, BUILD-01, BUILD-02
**Success Criteria** (what must be TRUE):
  1. Shared `bominal-ui` ecosystem + `train`/`auth` skins render correctly in Leptos
  2. Tailwind scans .rs files for class names
  3. Docker build produces working image without Node.js
  4. `cargo leptos build` replaces `./dev-build.sh`
**Plans:** 2 plans (estimated)
**UI hint**: yes (visual verification of design system)

### Phase 10: Cleanup
**Goal**: SvelteKit frontend removed, documentation updated, migration complete
**Depends on**: Phase 9 (all verification passed)
**Requirements**: CLEAN-01, CLEAN-02
**Success Criteria** (what must be TRUE):
  1. `frontend/` directory deleted
  2. CLAUDE.md reflects Leptos architecture
  3. No npm/Node.js references remain in build scripts
  4. All quality gates pass (QG-01 through QG-07)
**Plans:** 1 plan

## Dependency Graph

```
Phase 1 (Foundation)
  |
Phase 2 (Core Infra)
  |
Phase 3 (Shell)
  |
Phase 4 (Auth Pages)
  |           \
Phase 5 (Core Pages)  Phase 7a (Passkey Interop)
  |
Phase 6 (Settings + Components)
  |           \
Phase 7b (Evervault)   Phase 7a (if not done)
  |           /
Phase 8 (Server Integration)
  |
Phase 9 (CSS + Build)
  |
Phase 10 (Cleanup)
```

Phases 1-6 are strictly sequential (each builds on the previous).
Phase 7 (interop) can partially overlap with Phase 5-6 since passkey interop is needed by Phase 4 pages.
Phases 8-10 are sequential post-integration.

## Inventory Being Ported

### Routes (14 total)
| SvelteKit Route | Leptos Path | Phase |
|-----------------|-------------|-------|
| `/` (redirect) | `/` | 3 |
| `/auth` | `/auth` | 4 |
| `/auth/login` | `/auth/login` | 4 |
| `/auth/signup` | `/auth/signup` | 4 |
| `/auth/forgot` | `/auth/forgot` | 4 |
| `/auth/verify` | `/auth/verify` | 4 |
| `/auth/add-passkey` | `/auth/add-passkey` | 4 |
| `/home` | `/home` | 5 |
| `/search` | `/search` | 5 |
| `/tasks` | `/tasks` | 5 |
| `/reservations` | `/reservations` | 5 |
| `/settings` | `/settings` | 6 |
| `/verify-email` | `/verify-email` | 4 |
| `/reset-password` | `/reset-password` | 4 |

### Components (15 total)
| Svelte Component | Classification | Phase |
|------------------|---------------|-------|
| GlassPanel | Pure SSR | 6 |
| StatusChip | Pure SSR | 6 |
| Skeleton | Pure SSR | 6 |
| Icon | Pure SSR | 6 |
| CardBrand | Pure SSR | 6 |
| Sidebar | Pure SSR | 3 |
| BottomNav | Pure SSR | 3 |
| StationInput | Island | 5 |
| CalendarPicker | Island | 5 |
| TimeSlider | Island | 5 |
| SortableList | Island | 5 |
| BottomSheet | Island | 6 |
| SelectionPrompt | Island | 5 |
| TicketCard | Island | 5 |
| TaskCard | Island | 5 |

### Key Decisions

| Decision | Rationale |
|----------|-----------|
| Leptos 0.8 islands (not CSR or full hydration) | Selective hydration minimizes WASM bundle, SSR improves initial load |
| Server functions proxy to /api/ (Option B) | Simpler, looser coupling, ships faster than direct service-layer access |
| cargo-leptos for build | Standard Leptos build tool, handles dual SSR+WASM compilation |
| Tailwind standalone CLI (no npm) | Eliminates Node.js from entire build pipeline |
| Rewrite bominal-app in place | Existing crate has salvageable patterns (i18n, API, auth context) |
| Keep `bominal-frontend` as donor code outside the active build path | Preserve reusable prototype work without splitting the migration target |
| `bominal-ui` is the UI source of truth | Shared Bominal UI must stay aligned across products; train app should consume/reference the canonical crate |

---
*Roadmap created: 2026-03-26*
*Replaces SvelteKit feature-wiring roadmap (2026-03-24)*
