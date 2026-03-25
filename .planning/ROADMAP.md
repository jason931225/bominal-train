# Roadmap: Bominal

## Overview

Bominal's frontend wiring milestone connects the existing SvelteKit SPA to the Axum REST API across the full search-to-book-to-pay flow. Work progresses from fixing a blocking API bug, through building search form components (station autocomplete, date/time pickers), wiring multi-provider search with credential management, enabling task creation with real-time SSE updates, integrating Evervault payment encryption, and finishing with localization polish. No new backend APIs are needed -- this is frontend wiring to existing endpoints.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Foundation Fixes** - Fix suggest API parameter bug and search button disabled state (completed 2026-03-25)
- [ ] **Phase 2: Station Autocomplete** - Debounced autocomplete with IME handling and keyboard navigation
- [ ] **Phase 3: Date & Time Picker** - Custom modal calendar and time band selector replacing raw inputs
- [ ] **Phase 4: Provider Credential Setup** - Settings flow for adding/validating SRT and KTX credentials
- [ ] **Phase 5: Multi-Provider Search** - Simultaneous SRT+KTX search with per-provider error isolation
- [ ] **Phase 6: Task Creation & Real-Time Updates** - Create booking tasks from results with SSE status streaming
- [ ] **Phase 7: Payment Integration** - Evervault card encryption, ciphertext validation, and card management
- [ ] **Phase 8: Localization Polish** - All user-facing error messages localized with no English fallbacks

## Phase Details

### Phase 1: Foundation Fixes
**Goal**: Search form inputs work correctly against the backend API
**Depends on**: Nothing (first phase)
**Requirements**: SRCH-07, SRCH-06
**Success Criteria** (what must be TRUE):
  1. Station suggest API returns results when user types a station name (query parameter matches backend expectation)
  2. Search button enables as soon as departure, arrival, and date fields are populated
**Plans:** 1 plan
Plans:
- [x] 01-01-PLAN.md -- Fix suggest API parameter key and search button disabled state

### Phase 2: Station Autocomplete
**Goal**: Users can quickly find and select train stations by typing partial names in Korean
**Depends on**: Phase 1
**Requirements**: SRCH-01, SRCH-02, SRCH-03
**Success Criteria** (what must be TRUE):
  1. Typing a partial station name shows a dropdown of matching stations after a short debounce
  2. Typing Korean characters with IME composition (e.g., typing "se" to form a Hangul syllable) does not trigger premature API calls or clear input
  3. User can navigate the dropdown with arrow keys, select with Enter, and dismiss with Escape
**Plans:** 2 plans
Plans:
- [ ] 02-01-PLAN.md -- Create StationInput.svelte component with tests (Vitest setup, TDD, IME + ARIA + debounce)
- [ ] 02-02-PLAN.md -- Integrate StationInput into search page, add i18n keys, visual verification
**UI hint**: yes

### Phase 3: Date & Time Picker
**Goal**: Users can select travel date and time range through intuitive modal controls
**Depends on**: Phase 1
**Requirements**: SRCH-04, SRCH-05
**Success Criteria** (what must be TRUE):
  1. Tapping the date field opens a modal calendar overlay where user can select a date by tapping a day
  2. Selected date displays in a human-readable format (not raw YYYYMMDD)
  3. Time band selection provides a visual UI showing departure time range (not a raw slider value)
**Plans**: TBD
**UI hint**: yes

### Phase 4: Provider Credential Setup
**Goal**: Users can configure their SRT and KTX provider login credentials before searching
**Depends on**: Phase 1
**Requirements**: SETT-01, SETT-03, PROV-03
**Success Criteria** (what must be TRUE):
  1. Settings page shows SRT and KTX credential status (configured vs unconfigured)
  2. User can enter and save provider credentials through a setup flow
  3. Saved credentials are validated against the provider (user sees success/failure feedback)
  4. Starting a search without credentials for a provider prompts the user to add them (with link to settings)
**Plans**: TBD
**UI hint**: yes

### Phase 5: Multi-Provider Search
**Goal**: Users get combined search results from both SRT and KTX in a single view
**Depends on**: Phase 4
**Requirements**: PROV-01, PROV-02, PROV-04, PROV-05
**Success Criteria** (what must be TRUE):
  1. Search fires requests to both SRT and KTX simultaneously
  2. If one provider fails (timeout, auth error), the other provider's results still appear with an inline error for the failed one
  3. Station names are normalized across providers (e.g., both use the same name for Seoul Station)
  4. Results display with a clear provider label (SRT or KTX) on each train option
**Plans**: TBD
**UI hint**: yes

### Phase 6: Task Creation & Real-Time Updates
**Goal**: Users can create auto-booking tasks and watch their status update in real time
**Depends on**: Phase 5
**Requirements**: TASK-01, TASK-02, TASK-03, TASK-04
**Success Criteria** (what must be TRUE):
  1. User can select a train from search results and create a booking task with chosen options
  2. Task list page updates in real time as the backend processes tasks (no manual refresh needed)
  3. SSE events carry task-specific data (status change, attempt count) rather than triggering a full list refetch
  4. Task detail view shows the selected train schedule, provider, and current processing status
**Plans**: TBD
**UI hint**: yes

### Phase 7: Payment Integration
**Goal**: Users can securely add payment cards and attach them to booking tasks
**Depends on**: Phase 6
**Requirements**: PAY-01, PAY-02, PAY-03, PAY-04, PAY-05
**Success Criteria** (what must be TRUE):
  1. User can enter card details in a styled form that encrypts data client-side via evervault.encrypt() before submission
  2. Backend stores only Evervault ciphertext -- plaintext card data never reaches the server
  3. Backend validates that stored card data is valid Evervault ciphertext format (rejects raw numbers)
  4. Settings card management page shows masked card numbers with options to add or remove cards
  5. When creating a booking task, user can toggle auto-pay to charge the saved card on successful booking
**Plans**: TBD
**UI hint**: yes

### Phase 8: Localization Polish
**Goal**: Korean-language users never encounter unexpected English text in the UI
**Depends on**: Phase 7
**Requirements**: SETT-02
**Success Criteria** (what must be TRUE):
  1. All error messages (validation, API errors, network failures) display in the user's selected locale
  2. No English fallback strings appear when the UI language is set to Korean
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8
Note: Phases 2 and 3 can execute in parallel (both depend only on Phase 1).

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation Fixes | 1/1 | Complete | 2026-03-25 |
| 2. Station Autocomplete | 0/2 | Planned | - |
| 3. Date & Time Picker | 0/TBD | Not started | - |
| 4. Provider Credential Setup | 0/TBD | Not started | - |
| 5. Multi-Provider Search | 0/TBD | Not started | - |
| 6. Task Creation & Real-Time Updates | 0/TBD | Not started | - |
| 7. Payment Integration | 0/TBD | Not started | - |
| 8. Localization Polish | 0/TBD | Not started | - |
