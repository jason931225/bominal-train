# Phase 1: Foundation Fixes - Context

**Gathered:** 2026-03-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Fix two blocking bugs in the search form: (1) the suggest API query parameter mismatch that prevents station autocomplete from working, and (2) the search button disabled state logic that doesn't check for a date value. No new features — just make existing wiring work correctly.

</domain>

<decisions>
## Implementation Decisions

### API Parameter Fix
- **D-01:** Fix on the frontend side — change `suggestStations()` to send `q` instead of `query`, matching the existing backend `SuggestQuery` struct. Backend contract stays unchanged.

### Search Button Logic
- **D-02:** Add `!date` to the search button's `disabled` condition so it requires departure, arrival, AND date before enabling.

### Claude's Discretion
- Whether to also rename the local variable in `suggestStations()` for clarity (cosmetic, not functional)
- Whether to add a brief inline comment explaining the disabled logic

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Frontend Search
- `frontend/src/lib/api/search.ts` — `suggestStations()` function (line 18-26) sends `query` param
- `frontend/src/routes/search/+page.svelte` — Search button disabled condition (line 526), station suggest calls (lines 109, 129)

### Backend API
- `crates/bominal-server/src/search.rs` — `SuggestQuery` struct expects `q` field (line 85-90), `suggest_stations` handler (line 117)
- `crates/bominal-server/src/routes.rs` — Route registration at `/stations/{provider}/suggest` (line 109)

### Requirements
- `.planning/REQUIREMENTS.md` — SRCH-07 (param mismatch), SRCH-06 (search button logic)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `frontend/src/lib/api/client.ts` — Typed fetch wrapper with `get<T>()` that appends query params as URLSearchParams
- Glass morphism design system already in place — no UI changes needed for this phase

### Established Patterns
- Frontend API functions in `frontend/src/lib/api/` use typed wrappers (`get`, `post`) from `client.ts`
- Backend uses Axum extractors: `Path(provider)` + `Query(params)` for suggest endpoint

### Integration Points
- `suggestStations()` is called from `+page.svelte` lines 109 and 129 (departure and arrival inputs)
- Search button at `+page.svelte` line 526 gates the `handleSearch` function

</code_context>

<specifics>
## Specific Ideas

No specific requirements — both fixes are mechanical bug fixes with clear before/after behavior.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-foundation*
*Context gathered: 2026-03-24*
