# Phase 2: Station Autocomplete - Context

**Gathered:** 2026-03-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Build a production-quality station autocomplete component that handles Korean IME composition correctly, supports full keyboard navigation per WAI-ARIA combobox pattern, and replaces the inline suggest code in the search page. No new backend endpoints needed — wires to existing `/api/stations/{provider}/suggest?q=`.

</domain>

<decisions>
## Implementation Decisions

### Component Architecture
- **D-01:** Extract a reusable `StationInput.svelte` component using Svelte 5 `$bindable()` rune for the value prop. Departure and arrival both use this component with different labels/props. Callback prop for selection event.

### IME Composition Handling
- **D-02:** Suppress API calls during IME composition. Listen for `compositionstart`/`compositionend` events — set a flag during composition, skip debounce triggers while flag is true. Only fire API call after `compositionend` + debounce. This prevents wasted calls with partial jamo (ㅅ, 서) that won't match station names.
- **D-03:** Debounce timer of 200-300ms after composition ends (slightly longer than the current 150ms to account for IME settle time).

### Keyboard Navigation
- **D-04:** Follow WAI-ARIA combobox pattern — arrow keys move a visual highlight (`aria-activedescendant`), input text stays as typed, Enter commits the highlighted selection, Escape dismisses the dropdown.
- **D-05:** Proper ARIA attributes: `role="combobox"`, `aria-expanded`, `aria-controls`, `aria-activedescendant` on the input; `role="listbox"` on the dropdown; `role="option"` + `aria-selected` on each item.

### Dropdown Behavior
- **D-06:** On re-focus, reshow last cached results immediately (no re-typing required). Clear cached results only when input value changes.
- **D-07:** Minimum query length of 1 character before triggering API call (consistent with current behavior).

### Claude's Discretion
- Exact debounce timing (200-300ms range)
- Dropdown max height and scroll behavior
- Loading indicator style (spinner vs skeleton)
- Empty state message when no matches found
- Whether to show English name alongside Korean in dropdown items

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Backend Contract
- `crates/bominal-server/src/search.rs` (lines 85-90) — `SuggestQuery` struct defines `q: String` and `mode: Option<String>` fields
- `frontend/src/lib/api/search.ts` — `suggestStations(provider, query, mode?)` function, fixed in Phase 1

### Frontend Patterns
- `frontend/src/lib/components/GlassPanel.svelte` — glass morphism panel used throughout the app
- `frontend/src/routes/search/+page.svelte` — current inline suggest code to be replaced (lines 40-148 script, 330-423 template)

### Standards
- WAI-ARIA Combobox Pattern — https://www.w3.org/WAI/ARIA/apg/patterns/combobox/

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `GlassPanel.svelte` — dropdown should match glass morphism style
- `suggestStations()` API function — already correctly wired (Phase 1 fix)
- `SuggestMatch` type — has `name_ko` and `name_en` fields
- Svelte 5 runes (`$state`, `$derived`, `$bindable`) — project-wide pattern

### Established Patterns
- CSS custom properties for theming (`var(--color-*)`)
- Glass panel styling with `glass-panel` class
- `squish` class for press interaction feedback
- i18n via `t()` function from `$lib/i18n`

### Integration Points
- `+page.svelte` will import `StationInput.svelte` and replace the inline departure/arrival input blocks
- Component needs to accept `provider` prop for multi-provider support (Phase 5)

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. User deferred all decisions to Claude's best judgment based on research and best practices.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-station-autocomplete*
*Context gathered: 2026-03-25*
