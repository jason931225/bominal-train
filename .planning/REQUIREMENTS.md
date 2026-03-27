# Requirements: Bominal Leptos SSR Migration

**Defined:** 2026-03-26
**Core Value:** Replace the SvelteKit SPA with a Leptos 0.8 SSR application using islands architecture, faithfully porting all pages, components, i18n, and real-time features while eliminating the Node.js build dependency.

## Migration Requirements

Requirements for the SvelteKit-to-Leptos rewrite. Each maps to roadmap phases.

### Foundation

- [x] **FND-01**: Keep `bominal-frontend` out of the active workspace/build path and clean misleading references while it remains donor code
- [x] **FND-02**: Rewrite `bominal-app/Cargo.toml` with SSR/hydrate feature flags and islands architecture
- [x] **FND-03**: Add Leptos workspace dependencies (leptos, leptos_meta, leptos_router, leptos_axum)
- [x] **FND-04**: Configure cargo-leptos build system in the root `Cargo.toml` (`[[workspace.metadata.leptos]]` with bin/lib packages, Tailwind integration)
- [x] **FND-05**: Verify dual compilation: `cargo check --features ssr` and `cargo check --target wasm32-unknown-unknown --features hydrate`

### Core Infrastructure

- [x] **INFRA-01**: Port i18n system — embed ko/en/ja JSON files, reactive locale signal, cookie-based SSR locale detection
- [x] **INFRA-02**: Port all 8 utility functions (format_time, format_date, format_cost, etc.)
- [x] **INFRA-03**: Port shared types — all 20 TypeScript interfaces as Rust structs with Serialize/Deserialize
- [x] **INFRA-04**: Implement server functions (API layer) proxying to existing /api/ endpoints
- [x] **INFRA-05**: Implement state management — auth context, theme context, SSE store (client-only)

### Shell and Navigation

- [x] **SHELL-01**: Root App component with leptos_router, auth guard, layout branching (auth vs main)
- [x] **SHELL-02**: Port Sidebar (desktop) and BottomNav (mobile) as pure SSR components
- [x] **SHELL-03**: Active page highlighting via use_location()

### Auth Pages

- [ ] **AUTH-01**: Port auth landing page with passkey login island
- [ ] **AUTH-02**: Port login page — email/password form island
- [ ] **AUTH-03**: Port signup page — registration form with password strength meter island
- [ ] **AUTH-04**: Port forgot password page
- [ ] **AUTH-05**: Port verify page (post-signup) and add-passkey page
- [ ] **AUTH-06**: Port verify-email (token) and reset-password pages

### Core Application Pages

- [ ] **PAGE-01**: Port home page — dashboard with active tasks, pull-to-refresh island, SSE subscription
- [ ] **PAGE-02**: Port search page — station inputs, calendar picker, time slider, results, review sheet (multiple islands)
- [ ] **PAGE-03**: Port tasks page — task list with tabs, swipe-to-cancel, SSE updates
- [ ] **PAGE-04**: Port reservations page — provider filter, expandable tickets, pay/cancel/refund actions

### Settings and Components

- [ ] **SETT-01**: Port settings page — provider section, card section, appearance section, logout
- [ ] **COMP-01**: Port pure SSR components (GlassPanel, StatusChip, Skeleton, Icon, CardBrand) using `bominal-ui` primitives/equivalents where available
- [ ] **COMP-02**: Port interactive components as islands (BottomSheet, SelectionPrompt, TicketCard, TaskCard)

### Client-Only Interop

- [ ] **INTEROP-01**: Port passkey interop — WebAuthn via web-sys (navigator.credentials.get/create)
- [ ] **INTEROP-02**: Port Evervault interop — JS SDK wrapper via wasm-bindgen

### Server Integration

- [ ] **SRV-01**: Replace SPA static serving in bominal-server with leptos_axum SSR handler
- [ ] **SRV-02**: Serve WASM bundle and static assets from cargo-leptos output directory
- [ ] **SRV-03**: Merge SharedState with Leptos context for server function access

### CSS and Build

- [ ] **CSS-01**: Adopt `bominal-ui` ecosystem + `train`/`auth` skin CSS in the cargo-leptos pipeline (no npm)
- [ ] **CSS-02**: Configure Tailwind content scanning for .rs files in view! macros
- [ ] **BUILD-01**: Update Dockerfile for cargo-leptos + wasm32 target (no Node.js stage)
- [ ] **BUILD-02**: Update dev-build.sh to use cargo leptos build

### Cleanup

- [ ] **CLEAN-01**: Remove frontend/ directory after full verification
- [ ] **CLEAN-02**: Update CLAUDE.md and all documentation to reflect Leptos architecture

## Quality Gates

- [ ] **QG-01**: All 14 routes render correctly via SSR
- [ ] **QG-02**: Interactive islands hydrate and function (forms, search, gestures)
- [ ] **QG-03**: i18n works for all 3 locales, switchable at runtime
- [ ] **QG-04**: SSE real-time updates work on home and tasks pages
- [ ] **QG-05**: WASM bundle size < 500 KB gzipped
- [ ] **QG-06**: Shared `bominal-ui` design system renders correctly for train and auth surfaces in Leptos
- [ ] **QG-07**: No npm/Node.js required in build pipeline

## Out of Scope

| Feature | Reason |
|---------|--------|
| New features beyond SvelteKit parity | This is a faithful port, not a feature release |
| Backend API changes | All existing endpoints remain unchanged |
| Direct service-layer server functions | Phase 1 uses proxy-to-/api/ approach; direct access is future optimization |
| Mobile native app | Web only |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| FND-01 | Phase 1 | Complete |
| FND-02 | Phase 1 | Complete |
| FND-03 | Phase 1 | Complete |
| FND-04 | Phase 1 | Complete |
| FND-05 | Phase 1 | Complete |
| INFRA-01 | Phase 2 | Complete |
| INFRA-02 | Phase 2 | Complete |
| INFRA-03 | Phase 2 | Complete |
| INFRA-04 | Phase 2 | Complete |
| INFRA-05 | Phase 2 | Complete |
| SHELL-01 | Phase 3 | Complete |
| SHELL-02 | Phase 3 | Complete |
| SHELL-03 | Phase 3 | Complete |
| AUTH-01 | Phase 4 | Pending |
| AUTH-02 | Phase 4 | Pending |
| AUTH-03 | Phase 4 | Pending |
| AUTH-04 | Phase 4 | Pending |
| AUTH-05 | Phase 4 | Pending |
| AUTH-06 | Phase 4 | Pending |
| PAGE-01 | Phase 5 | Pending |
| PAGE-02 | Phase 5 | Pending |
| PAGE-03 | Phase 5 | Pending |
| PAGE-04 | Phase 5 | Pending |
| SETT-01 | Phase 6 | Pending |
| COMP-01 | Phase 6 | Pending |
| COMP-02 | Phase 6 | Pending |
| INTEROP-01 | Phase 7 | Pending |
| INTEROP-02 | Phase 7 | Pending |
| SRV-01 | Phase 8 | Pending |
| SRV-02 | Phase 8 | Pending |
| SRV-03 | Phase 8 | Pending |
| CSS-01 | Phase 9 | Pending |
| CSS-02 | Phase 9 | Pending |
| BUILD-01 | Phase 9 | Pending |
| BUILD-02 | Phase 9 | Pending |
| CLEAN-01 | Phase 10 | Pending |
| CLEAN-02 | Phase 10 | Pending |

**Coverage:**
- Migration requirements: 37 total
- Quality gates: 7
- Mapped to phases: 37
- Unmapped: 0

---
*Requirements defined: 2026-03-26*
*Replaces previous SvelteKit feature-wiring requirements*
