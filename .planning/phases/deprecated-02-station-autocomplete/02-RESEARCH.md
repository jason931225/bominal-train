# Phase 2: Station Autocomplete - Research

**Researched:** 2026-03-25
**Domain:** SvelteKit component architecture, WAI-ARIA combobox, Korean IME composition
**Confidence:** HIGH

## Summary

This phase extracts the inline station suggestion code from `+page.svelte` into a reusable `StationInput.svelte` component with three key capabilities: (1) debounced autocomplete wired to the existing `/api/stations/{provider}/suggest?q=` endpoint, (2) Korean IME composition awareness to avoid premature API calls during Hangul syllable assembly, and (3) full WAI-ARIA combobox keyboard navigation. No new backend endpoints or npm dependencies are needed -- everything builds on Svelte 5 runes (`$state`, `$derived`, `$bindable`) and the existing `suggestStations()` API function.

The existing code in `+page.svelte` lines 40-148 (script) and 330-423 (template) contains a working but duplicated and accessibility-deficient suggest implementation. The component extraction consolidates ~200 lines of duplicated departure/arrival logic into a single reusable component, adds IME composition guards, and layers on ARIA attributes and keyboard event handling.

**Primary recommendation:** Build a single `StationInput.svelte` component with `$bindable()` value prop, internal composition/debounce state machine, and WAI-ARIA combobox role attributes. Replace both departure and arrival blocks in `+page.svelte` with this component.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Extract a reusable `StationInput.svelte` component using Svelte 5 `$bindable()` rune for the value prop. Departure and arrival both use this component with different labels/props. Callback prop for selection event.
- **D-02:** Suppress API calls during IME composition. Listen for `compositionstart`/`compositionend` events -- set a flag during composition, skip debounce triggers while flag is true. Only fire API call after `compositionend` + debounce.
- **D-03:** Debounce timer of 200-300ms after composition ends (slightly longer than the current 150ms to account for IME settle time).
- **D-04:** Follow WAI-ARIA combobox pattern -- arrow keys move a visual highlight (`aria-activedescendant`), input text stays as typed, Enter commits the highlighted selection, Escape dismisses the dropdown.
- **D-05:** Proper ARIA attributes: `role="combobox"`, `aria-expanded`, `aria-controls`, `aria-activedescendant` on the input; `role="listbox"` on the dropdown; `role="option"` + `aria-selected` on each item.
- **D-06:** On re-focus, reshow last cached results immediately (no re-typing required). Clear cached results only when input value changes.
- **D-07:** Minimum query length of 1 character before triggering API call (consistent with current behavior).

### Claude's Discretion
- Exact debounce timing (200-300ms range)
- Dropdown max height and scroll behavior
- Loading indicator style (spinner vs skeleton)
- Empty state message when no matches found
- Whether to show English name alongside Korean in dropdown items

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SRCH-01 | Station input connects to suggest API with debounced autocomplete dropdown | Existing `suggestStations()` API, `$bindable()` component pattern, debounce with `setTimeout` |
| SRCH-02 | Station autocomplete handles Korean IME composition events (compositionend) | `compositionstart`/`compositionend` event listeners, `isComposing` flag pattern |
| SRCH-03 | Autocomplete dropdown supports keyboard navigation (arrow keys, enter, escape) | WAI-ARIA combobox pattern with `aria-activedescendant` focus management |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Svelte | ^5.19.0 | Component framework with runes | Already in project, `$state`/`$derived`/`$bindable` runes required |
| SvelteKit | ^2.16.0 | App framework | Already in project, provides routing and build |
| Tailwind CSS | ^4.1.0 | Styling | Already in project, all components use it |

### Supporting
No new dependencies needed. All functionality is achievable with:
- Browser `compositionstart`/`compositionend` events (Web API)
- `setTimeout`/`clearTimeout` for debounce (built-in)
- ARIA attributes (HTML standard)
- Existing `suggestStations()` from `$lib/api/search.ts`

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Custom debounce | lodash debounce | Adds dependency for 5 lines of code -- not worth it |
| Custom combobox | Melt UI / Bits UI | Adds dependency; project already has custom glass morphism components; ARIA combobox is ~50 lines of keyboard handling |
| Custom IME handling | No library exists | compositionstart/compositionend is the standard browser API -- no alternative |

## Architecture Patterns

### Recommended Project Structure
```
frontend/src/lib/components/
├── StationInput.svelte   # NEW: reusable autocomplete combobox
├── GlassPanel.svelte     # existing: glass morphism container
├── Skeleton.svelte        # existing: loading skeleton
└── ...                    # other existing components
```

### Pattern 1: Svelte 5 $bindable() Component
**What:** Two-way binding for the station name value between parent and child component
**When to use:** When the parent needs to read the current value (for search form submission) and the child needs to update it (on selection)
**Example:**
```svelte
<!-- StationInput.svelte -->
<script lang="ts">
  import type { SuggestMatch } from '$lib/types';

  interface Props {
    value: string;
    label: string;
    placeholder?: string;
    provider: string;
    onselect?: (match: SuggestMatch) => void;
  }

  let { value = $bindable(), label, placeholder = '', provider, onselect }: Props = $props();
</script>
```

```svelte
<!-- Parent usage in +page.svelte -->
<StationInput
  bind:value={departure}
  label={t('search.from')}
  placeholder={t('search.select_station')}
  {provider}
  onselect={(match) => { /* optional side effect */ }}
/>
```
**Source:** [Svelte $bindable docs](https://svelte.dev/docs/svelte/$bindable)

### Pattern 2: IME Composition Guard
**What:** Flag-based suppression of API calls during Korean IME composition
**When to use:** Any text input that triggers server calls where CJK input is expected
**Example:**
```svelte
<script lang="ts">
  let isComposing = $state(false);
  let debounceTimer: ReturnType<typeof setTimeout> | undefined;

  function handleInput(e: Event): void {
    const query = (e.target as HTMLInputElement).value;
    value = query;

    clearTimeout(debounceTimer);
    if (isComposing) return; // Skip during IME composition
    scheduleFetch(query);
  }

  function handleCompositionStart(): void {
    isComposing = true;
  }

  function handleCompositionEnd(e: CompositionEvent): void {
    isComposing = false;
    // The input value is already updated by this point
    scheduleFetch(value);
  }

  function scheduleFetch(query: string): void {
    clearTimeout(debounceTimer);
    if (query.length < 1) {
      suggestions = [];
      return;
    }
    debounceTimer = setTimeout(() => fetchSuggestions(query), 250);
  }
</script>
```
**Source:** [MDN compositionend](https://developer.mozilla.org/en-US/docs/Web/API/Element/compositionend_event)

### Pattern 3: WAI-ARIA Combobox with aria-activedescendant
**What:** DOM focus stays on the input element; visual focus in the listbox is managed via `aria-activedescendant` pointing to the active option's `id`
**When to use:** Any autocomplete/combobox where the user types and navigates suggestions
**Example:**
```svelte
<script lang="ts">
  let activeIndex = $state(-1);
  let expanded = $state(false);

  const activeDescendant = $derived(
    activeIndex >= 0 ? `station-option-${activeIndex}` : undefined
  );

  function handleKeydown(e: KeyboardEvent): void {
    if (!expanded) {
      if (e.key === 'ArrowDown') {
        expanded = true;
        activeIndex = 0;
        e.preventDefault();
      }
      return;
    }

    switch (e.key) {
      case 'ArrowDown':
        activeIndex = (activeIndex + 1) % suggestions.length;
        e.preventDefault();
        break;
      case 'ArrowUp':
        activeIndex = activeIndex <= 0 ? suggestions.length - 1 : activeIndex - 1;
        e.preventDefault();
        break;
      case 'Enter':
        if (activeIndex >= 0) {
          selectStation(suggestions[activeIndex]);
          e.preventDefault();
        }
        break;
      case 'Escape':
        expanded = false;
        activeIndex = -1;
        e.preventDefault();
        break;
    }
  }
</script>

<input
  role="combobox"
  aria-expanded={expanded}
  aria-controls="station-listbox"
  aria-activedescendant={activeDescendant}
  aria-autocomplete="list"
  onkeydown={handleKeydown}
/>

{#if expanded && suggestions.length > 0}
  <ul id="station-listbox" role="listbox">
    {#each suggestions as match, i}
      <li
        id="station-option-{i}"
        role="option"
        aria-selected={i === activeIndex}
      >
        {match.name_ko}
      </li>
    {/each}
  </ul>
{/if}
```
**Source:** [W3C WAI-ARIA Combobox Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/combobox/examples/combobox-autocomplete-list/)

### Anti-Patterns to Avoid
- **Filtering on `input` event without IME guard:** Fires on every keystroke including partial jamo (ㅅ, 서) during Korean composition, triggering wasted API calls that return no results
- **Moving DOM focus to listbox:** DOM focus must stay on the input; use `aria-activedescendant` for visual focus. Moving DOM focus breaks typing flow.
- **Using `onblur` without delay for dropdown dismissal:** Current code uses `setTimeout(() => { showSuggest = false }, 200)` -- this is necessary so `onmousedown` on options fires before the dropdown hides. Keep this pattern.
- **Separate debounce per instance:** The current code duplicates debounce logic for departure and arrival. The component encapsulates its own debounce timer internally.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Debounce | Full debounce utility library | `setTimeout`/`clearTimeout` inline | Only 5 lines, no edge cases beyond IME (which we handle separately) |
| Unique IDs for ARIA | UUID generator | Counter-based ID with component prefix | `station-dep-option-0`, `station-arr-option-0` -- simple, deterministic, unique enough for same-page use |

**Key insight:** This phase is pure frontend component work with no complex data transformations. The backend API already exists and works correctly (fixed in Phase 1). The complexity is entirely in browser event handling (IME composition) and ARIA attribute management.

## Common Pitfalls

### Pitfall 1: compositionend Event Order Varies by Browser
**What goes wrong:** In Chrome/Safari, the `input` event fires BEFORE `compositionend`. In Firefox, `input` fires AFTER `compositionend`. This means checking `isComposing` in the `input` handler catches the Chrome case, but the Firefox `input` event after `compositionend` may be missed.
**Why it happens:** The W3C spec was ambiguous; browsers implemented differently.
**How to avoid:** Handle BOTH paths: (1) in `input` handler, skip if `isComposing` is true; (2) in `compositionend` handler, explicitly trigger the fetch with the current value. This covers both orderings.
**Warning signs:** Suggestions not appearing after completing a Korean syllable in Firefox.

### Pitfall 2: aria-activedescendant Requires Matching IDs
**What goes wrong:** `aria-activedescendant` points to a DOM id that doesn't exist (e.g., after suggestions list re-renders) -- screen readers announce nothing.
**Why it happens:** When suggestions update, old IDs are removed and new ones created. If `activeIndex` isn't reset, it may point to a stale index.
**How to avoid:** Reset `activeIndex` to -1 whenever the suggestions array changes. Generate IDs deterministically from index position.
**Warning signs:** Screen reader not announcing the highlighted option.

### Pitfall 3: Multiple StationInput Instances Need Unique ARIA IDs
**What goes wrong:** Two `StationInput` components on the same page (departure + arrival) share the same listbox ID, violating HTML id uniqueness.
**Why it happens:** Hard-coded IDs like `"station-listbox"` in the component.
**How to avoid:** Accept an `id` prop or generate a unique prefix per instance (e.g., `dep-station-listbox`, `arr-station-listbox`). Use a prop like `name` to namespace IDs.
**Warning signs:** ARIA `aria-controls` pointing to wrong listbox.

### Pitfall 4: Scroll-Into-View for Keyboard Navigation
**What goes wrong:** User presses ArrowDown many times; the highlighted option scrolls out of the visible area of the dropdown.
**Why it happens:** The listbox has `max-h-48 overflow-y-auto` but no scroll management.
**How to avoid:** After updating `activeIndex`, call `element.scrollIntoView({ block: 'nearest' })` on the active option element.
**Warning signs:** User navigates with arrows but can't see the highlighted item.

### Pitfall 5: Race Conditions from Stale Async Responses
**What goes wrong:** User types "서울", then quickly types "대전". The "서울" response arrives after "대전" response, overwriting correct results.
**Why it happens:** Network latency variance; earlier request completes after later one.
**How to avoid:** Track the query string that triggered each request. On response, compare against current value -- discard if stale. Alternatively, use an incrementing request counter.
**Warning signs:** Dropdown briefly shows wrong suggestions then corrects, or shows results for a previous query.

## Code Examples

### Complete StationInput Component Skeleton
```svelte
<!-- frontend/src/lib/components/StationInput.svelte -->
<script lang="ts">
  import { suggestStations } from '$lib/api/search';
  import type { SuggestMatch } from '$lib/types';

  interface Props {
    value: string;
    label: string;
    placeholder?: string;
    provider: string;
    name: string; // 'dep' | 'arr' -- used for unique ARIA IDs
    onselect?: (match: SuggestMatch) => void;
  }

  let {
    value = $bindable(),
    label,
    placeholder = '',
    provider,
    name,
    onselect
  }: Props = $props();

  // Suggest state
  let suggestions = $state<SuggestMatch[]>([]);
  let expanded = $state(false);
  let activeIndex = $state(-1);
  let loading = $state(false);

  // IME composition state
  let isComposing = $state(false);
  let debounceTimer: ReturnType<typeof setTimeout> | undefined;

  // ARIA IDs
  const listboxId = `${name}-station-listbox`;
  const activeDescendant = $derived(
    activeIndex >= 0 ? `${name}-station-option-${activeIndex}` : undefined
  );

  // Cache for re-focus behavior (D-06)
  let lastQuery = $state('');
  let cachedSuggestions = $state<SuggestMatch[]>([]);

  function scheduleFetch(query: string): void {
    clearTimeout(debounceTimer);
    if (query.length < 1) {
      suggestions = [];
      expanded = false;
      return;
    }
    loading = true;
    debounceTimer = setTimeout(() => fetchSuggestions(query), 250);
  }

  async function fetchSuggestions(query: string): Promise<void> {
    try {
      const result = await suggestStations(provider, query);
      // Guard against stale responses
      if (query !== value) return;
      suggestions = result.matches;
      cachedSuggestions = result.matches;
      lastQuery = query;
      expanded = suggestions.length > 0;
      activeIndex = -1;
    } catch {
      suggestions = [];
    } finally {
      loading = false;
    }
  }

  function handleInput(e: Event): void {
    const query = (e.target as HTMLInputElement).value;
    value = query;
    if (isComposing) return;
    scheduleFetch(query);
  }

  function handleCompositionStart(): void {
    isComposing = true;
  }

  function handleCompositionEnd(): void {
    isComposing = false;
    scheduleFetch(value);
  }

  function handleFocus(): void {
    if (cachedSuggestions.length > 0 && value === lastQuery) {
      suggestions = cachedSuggestions;
      expanded = true;
    }
  }

  function handleBlur(): void {
    // Delay to allow onmousedown on options to fire
    setTimeout(() => { expanded = false; activeIndex = -1; }, 200);
  }

  function selectStation(match: SuggestMatch): void {
    value = match.name_ko;
    expanded = false;
    activeIndex = -1;
    onselect?.(match);
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (!expanded || suggestions.length === 0) {
      if (e.key === 'ArrowDown' && suggestions.length > 0) {
        expanded = true;
        activeIndex = 0;
        e.preventDefault();
      }
      return;
    }
    switch (e.key) {
      case 'ArrowDown':
        activeIndex = (activeIndex + 1) % suggestions.length;
        scrollActiveIntoView();
        e.preventDefault();
        break;
      case 'ArrowUp':
        activeIndex = activeIndex <= 0
          ? suggestions.length - 1
          : activeIndex - 1;
        scrollActiveIntoView();
        e.preventDefault();
        break;
      case 'Enter':
        if (activeIndex >= 0 && activeIndex < suggestions.length) {
          selectStation(suggestions[activeIndex]);
          e.preventDefault();
        }
        break;
      case 'Escape':
        expanded = false;
        activeIndex = -1;
        e.preventDefault();
        break;
    }
  }

  function scrollActiveIntoView(): void {
    // Use tick() or requestAnimationFrame to wait for DOM update
    requestAnimationFrame(() => {
      const el = document.getElementById(`${name}-station-option-${activeIndex}`);
      el?.scrollIntoView({ block: 'nearest' });
    });
  }
</script>

<div class="flex-1 relative">
  <label
    class="text-xs font-medium mb-1 block"
    style="color: var(--color-text-tertiary)"
    for="{name}-station-input"
  >
    {label}
  </label>
  <input
    id="{name}-station-input"
    type="text"
    role="combobox"
    aria-expanded={expanded}
    aria-controls={listboxId}
    aria-activedescendant={activeDescendant}
    aria-autocomplete="list"
    class="w-full rounded-xl px-3 py-2.5 text-sm font-medium outline-none"
    style="background: var(--color-bg-sunken); color: var(--color-text-primary); border: 1px solid var(--color-border-default)"
    {placeholder}
    {value}
    oninput={handleInput}
    onkeydown={handleKeydown}
    onfocus={handleFocus}
    onblur={handleBlur}
    oncompositionstart={handleCompositionStart}
    oncompositionend={handleCompositionEnd}
  />
  {#if expanded && suggestions.length > 0}
    <ul
      id={listboxId}
      role="listbox"
      class="absolute z-20 left-0 right-0 mt-1 glass-panel rounded-xl overflow-hidden max-h-48 overflow-y-auto"
    >
      {#each suggestions as match, i}
        <li
          id="{name}-station-option-{i}"
          role="option"
          aria-selected={i === activeIndex}
        >
          <button
            type="button"
            class="w-full text-left px-3 py-2.5 text-sm transition-colors"
            class:bg-[var(--color-interactive-hover)]={i === activeIndex}
            style="color: var(--color-text-primary)"
            onmousedown={() => selectStation(match)}
          >
            {match.name_ko}
            <span class="text-xs ml-1" style="color: var(--color-text-tertiary)">
              {match.name_en}
            </span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>
```

### Parent Integration in +page.svelte
```svelte
<!-- Replace departure/arrival blocks (lines 330-423) with: -->
<GlassPanel class="mb-4 page-enter stagger-1">
  <div class="flex items-center gap-2">
    <StationInput
      bind:value={departure}
      label={t('search.from')}
      placeholder={t('search.select_station')}
      provider={providerFilter === 'Both' ? 'SRT' : providerFilter}
      name="dep"
    />

    <!-- Swap button (unchanged) -->
    <button ...>...</button>

    <StationInput
      bind:value={arrival}
      label={t('search.to')}
      placeholder={t('search.select_station')}
      provider={providerFilter === 'Both' ? 'SRT' : providerFilter}
      name="arr"
    />
  </div>
</GlassPanel>
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Svelte 4 `export let` | Svelte 5 `$props()` + `$bindable()` | Svelte 5 (Oct 2024) | Two-way binding requires explicit opt-in |
| Svelte 4 reactive stores | Svelte 5 `$state` / `$derived` runes | Svelte 5 (Oct 2024) | Fine-grained reactivity, no `$:` syntax |
| `e.isComposing` check | `compositionstart`/`compositionend` flag | Always | Flag approach is more reliable cross-browser |

**Deprecated/outdated:**
- Svelte 4 `export let` bindings: replaced by `$props()` + `$bindable()` in Svelte 5
- `$:` reactive declarations: replaced by `$derived` rune
- Writable stores (`writable()`): replaced by `$state` rune for component-local state

## Open Questions

1. **Provider reactivity when `providerFilter` changes**
   - What we know: The `provider` prop is passed to `StationInput`. If the user switches provider filter, suggestions should refresh or clear.
   - What's unclear: Should cached suggestions be invalidated when provider changes? Current code doesn't handle this.
   - Recommendation: Clear cached suggestions when `provider` prop changes. Use `$effect` to watch `provider` and reset state.

2. **Empty state UX in dropdown**
   - What we know: Current code hides the dropdown when there are no matches. The user decided this is Claude's discretion.
   - Recommendation: Show a brief "No matches" message in the dropdown rather than hiding it entirely -- this gives the user feedback that the search executed but found nothing, vs. the input not working.

## Project Constraints (from CLAUDE.md)

- **Svelte 5 runes:** Use `$state`, `$derived`, `$bindable` -- not legacy stores or `$:` syntax
- **Tailwind CSS 4:** Use Tailwind classes, CSS custom properties for theming
- **Glass morphism styling:** Dropdown should use `glass-panel` class for consistency
- **i18n:** All user-facing strings via `t()` function from `$lib/i18n`
- **Immutability:** Create new arrays/objects, never mutate existing ones
- **File size:** Components should be under 200-400 lines (StationInput will be ~180 lines)
- **Error handling:** Handle API errors gracefully, show empty state
- **No new dependencies:** Project decision from roadmap -- all custom components
- **Commit format:** `feat:`, `fix:`, `refactor:` etc.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | None currently installed for frontend |
| Config file | none -- see Wave 0 |
| Quick run command | `cd frontend && npx vitest run --reporter=verbose` |
| Full suite command | `cd frontend && npx vitest run` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SRCH-01 | Typing triggers debounced API call and shows dropdown | unit | `npx vitest run src/lib/components/StationInput.test.ts -t "debounce"` | No -- Wave 0 |
| SRCH-02 | IME composition suppresses API calls until compositionend | unit | `npx vitest run src/lib/components/StationInput.test.ts -t "composition"` | No -- Wave 0 |
| SRCH-03 | Arrow keys navigate, Enter selects, Escape dismisses | unit | `npx vitest run src/lib/components/StationInput.test.ts -t "keyboard"` | No -- Wave 0 |

### Sampling Rate
- **Per task commit:** `cd frontend && npx vitest run src/lib/components/StationInput.test.ts`
- **Per wave merge:** `cd frontend && npx vitest run`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `frontend/vitest.config.ts` -- Vitest config with Svelte plugin and jsdom environment
- [ ] `frontend/src/lib/components/StationInput.test.ts` -- covers SRCH-01, SRCH-02, SRCH-03
- [ ] Install dev deps: `cd frontend && npm install -D vitest @testing-library/svelte @testing-library/jest-dom jsdom`
- [ ] Note: `@sveltejs/vite-plugin-svelte` already installed -- Vitest can use it for Svelte component testing

## Sources

### Primary (HIGH confidence)
- [Svelte 5 $bindable docs](https://svelte.dev/docs/svelte/$bindable) -- bindable rune API and usage
- [W3C WAI-ARIA Combobox Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/combobox/examples/combobox-autocomplete-list/) -- ARIA attributes, keyboard behavior, focus management
- [MDN compositionstart](https://developer.mozilla.org/en-US/docs/Web/API/Element/compositionstart_event) -- IME composition event API
- [MDN compositionend](https://developer.mozilla.org/en-US/docs/Web/API/Element/compositionend_event) -- IME composition event API
- Existing codebase: `frontend/src/routes/search/+page.svelte`, `frontend/src/lib/api/search.ts`, `frontend/src/lib/types/index.ts`

### Secondary (MEDIUM confidence)
- [W3C uievents issue #202](https://github.com/w3c/uievents/issues/202) -- compositionend vs input event ordering discussion
- [Detecting IME input guide](https://www.javaspring.net/blog/detecting-ime-input-before-enter-pressed-in-javascript/) -- cross-browser IME handling patterns

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- already installed, versions verified from package.json
- Architecture: HIGH -- $bindable() pattern is documented in official Svelte 5 docs; WAI-ARIA combobox is a W3C standard with reference implementations
- Pitfalls: HIGH -- IME composition event ordering is well-documented; race condition patterns are standard async UI concerns

**Research date:** 2026-03-25
**Valid until:** 2026-04-25 (stable -- Svelte 5 runes API is finalized, WAI-ARIA is a stable standard)
