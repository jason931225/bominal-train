# Requirements: Bominal

**Defined:** 2026-03-24
**Core Value:** Users can search both train providers simultaneously, create an auto-booking task, and pay securely with encrypted card details -- end to end.

## v1 Requirements

Requirements for this milestone. Each maps to roadmap phases.

### Search Form

- [ ] **SRCH-01**: Station input connects to suggest API with debounced autocomplete dropdown
- [ ] **SRCH-02**: Station autocomplete handles Korean IME composition events (compositionend)
- [ ] **SRCH-03**: Autocomplete dropdown supports keyboard navigation (arrow keys, enter, escape)
- [ ] **SRCH-04**: Date picker is a modal/overlay calendar replacing raw YYYYMMDD text input
- [ ] **SRCH-05**: Time band selection has proper UI (not just raw slider value)
- [ ] **SRCH-06**: Search button enables when departure, arrival, and date are filled
- [ ] **SRCH-07**: Fix suggest API parameter mismatch (frontend sends `query`, backend expects `q`)

### Multi-Provider Search

- [ ] **PROV-01**: Search fires against both SRT and KTX simultaneously
- [ ] **PROV-02**: Per-provider error isolation via Promise.allSettled (one failure doesn't block other)
- [ ] **PROV-03**: Prompt user to add provider credentials when missing (link to settings)
- [ ] **PROV-04**: Station name normalization across SRT and KTX providers
- [ ] **PROV-05**: Search results display merged from both providers with provider label

### Task Management

- [ ] **TASK-01**: User can create a booking task from search results (selecting train + options)
- [ ] **TASK-02**: SSE delivers real-time task status updates to the frontend
- [ ] **TASK-03**: SSE events include granular task-specific payloads (not full list refresh)
- [ ] **TASK-04**: Task detail view shows schedule info, provider, and current status

### Payment

- [ ] **PAY-01**: Card input form with custom-styled fields, encrypted via `evervault.encrypt()` (not UI.Card iframe)
- [ ] **PAY-02**: Encrypted ciphertext stored on backend, never plaintext
- [ ] **PAY-03**: Server-side validation that stored card data is valid Evervault ciphertext format
- [ ] **PAY-04**: Card management in settings (add card, view masked number, remove card)
- [ ] **PAY-05**: Auto-pay toggle available when creating a booking task

### Settings & Polish

- [ ] **SETT-01**: Provider credential setup flow in settings (SRT/KTX login credentials)
- [ ] **SETT-02**: All user-facing error messages localized (no English fallbacks in Korean UI)
- [ ] **SETT-03**: Provider credential validation on save (verify credentials work)

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Notifications

- **NOTF-01**: Push notifications when task finds available seat
- **NOTF-02**: Email notification on successful booking

### Advanced Search

- **ADVS-01**: Fare comparison across providers
- **ADVS-02**: Flexible date search (search +/- days)
- **ADVS-03**: Saved search presets

### Monitoring

- **MNTR-01**: Granular SSE events for booking progress steps (searching, found, reserving, reserved)
- **MNTR-02**: Task history with timeline view

## Out of Scope

| Feature | Reason |
|---------|--------|
| Mobile native app | Web SPA only for this milestone |
| Real-time seat availability push | Requires provider webhook support (not available) |
| Multi-user / team accounts | Single-user focus for v1 |
| Train schedule browsing | Only search-to-book flow, not general timetable |
| Evervault UI.Card iframe | Using `evervault.encrypt()` on custom form fields instead |
| Provider API rate limit handling | Defer until scale requires it |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| SRCH-01 | Phase 2 | Pending |
| SRCH-02 | Phase 2 | Pending |
| SRCH-03 | Phase 2 | Pending |
| SRCH-04 | Phase 3 | Pending |
| SRCH-05 | Phase 3 | Pending |
| SRCH-06 | Phase 1 | Pending |
| SRCH-07 | Phase 1 | Pending |
| PROV-01 | Phase 5 | Pending |
| PROV-02 | Phase 5 | Pending |
| PROV-03 | Phase 4 | Pending |
| PROV-04 | Phase 5 | Pending |
| PROV-05 | Phase 5 | Pending |
| TASK-01 | Phase 6 | Pending |
| TASK-02 | Phase 6 | Pending |
| TASK-03 | Phase 6 | Pending |
| TASK-04 | Phase 6 | Pending |
| PAY-01 | Phase 7 | Pending |
| PAY-02 | Phase 7 | Pending |
| PAY-03 | Phase 7 | Pending |
| PAY-04 | Phase 7 | Pending |
| PAY-05 | Phase 7 | Pending |
| SETT-01 | Phase 4 | Pending |
| SETT-02 | Phase 8 | Pending |
| SETT-03 | Phase 4 | Pending |

**Coverage:**
- v1 requirements: 24 total
- Mapped to phases: 24
- Unmapped: 0

---
*Requirements defined: 2026-03-24*
*Last updated: 2026-03-24 after roadmap creation*
