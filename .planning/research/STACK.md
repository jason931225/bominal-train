# Technology Stack

**Project:** Bominal Train Reservation SaaS
**Researched:** 2026-03-24
**Scope:** Frontend wiring libraries for SvelteKit SPA to Axum REST API

## Current Stack (Fixed)

| Technology | Version | Purpose |
|------------|---------|---------|
| Svelte | 5.55.0 | UI framework with runes reactivity |
| SvelteKit | 2.55.0 | SPA framework (adapter-static) |
| Tailwind CSS | 4.1+ | Styling via @tailwindcss/vite |
| Vite | 6.1+ | Build tooling |
| TypeScript | 5.7+ | Type safety |
| Axum | 0.8 | Rust REST API backend |
| PostgreSQL | 16 | Primary database |
| Valkey | - | Session/cache store |

## Recommended Additions

### 1. Evervault Client-Side Encryption

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `@evervault/js` | latest (CDN v2) | Client-side card field encryption | Official browser SDK with TypeScript defs. Loads from CDN, cannot be bundled. |

**Confidence:** HIGH (verified via official docs at docs.evervault.com/sdks/javascript)

**Current state:** The codebase already has `frontend/src/lib/interop/evervault.ts` that loads via `window.Evervault` from a CDN script tag and reads team/app IDs from meta tags. This pattern is correct and should be kept.

**Do NOT install `@evervault/js` via npm.** The SDK must load from `https://js.evervault.com/v2` (CDN requirement). The existing meta-tag + `window.Evervault` approach in `evervault.ts` is the right pattern for non-React frameworks like SvelteKit. The npm `@evervault/js` package is a thin wrapper that calls the CDN anyway -- it adds a dependency without benefit since the existing interop file already works.

**Refinement needed:** The current `evervault.ts` uses `any` types. Add a local type declaration file (`evervault.d.ts`) for `window.Evervault` to get type safety without an npm dependency.

```typescript
// src/lib/interop/evervault.d.ts
interface EvervaultInstance {
  encrypt(data: string): Promise<string>;
  encrypt(data: Record<string, unknown>): Promise<Record<string, string>>;
}

interface EvervaultConstructor {
  new (teamId: string, appId: string): EvervaultInstance;
}

declare global {
  interface Window {
    Evervault?: EvervaultConstructor;
  }
}
```

### 2. Date Picker -- Custom Build (No Library)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Custom Svelte component | - | Modal date picker for travel date selection | No existing library matches the glass-morphism design system or YYYYMMDD format requirement |

**Confidence:** MEDIUM

**Why NOT use a library:**

- **shadcn-svelte/bits-ui Calendar** (bits-ui 2.16.x): Best Svelte 5 option but requires installing bits-ui + @internationalized/date + shadcn-svelte CLI scaffolding. The Calendar component outputs `DateValue` objects from `@internationalized/date`, not `YYYYMMDD` strings. The styling is shadcn's design system, not the app's glass-morphism CSS variables. Integrating would mean either (a) wrapping bits-ui Calendar and converting formats, adding 2 dependencies for a single component, or (b) restyling shadcn to match the existing design system.

- **date-picker-svelte** (2.17.0): Last meaningful update months ago; unclear Svelte 5 runes compatibility. Returns JS Date objects.

- **@svelte-plugins/datepicker**: Svelte 4 era, no confirmed Svelte 5 support.

**Recommendation:** Build a custom `DatePicker.svelte` modal component. The app already has `BottomSheet.svelte` for modal patterns and the glass-morphism design tokens. A train reservation date picker is simple: single date, no range, no time, 1-2 months forward, YYYYMMDD output. This is ~150 lines of Svelte, well within "build don't buy" territory. The current raw `<input type="text" placeholder="YYYYMMDD">` is the main UX gap.

### 3. Autocomplete/Typeahead -- Custom Build (Already Started)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Custom inline autocomplete | - | Station name typeahead with Korean/English/Japanese support | Already implemented in search page with debounce + suggest API |

**Confidence:** HIGH (code already exists)

**Current state:** The search page (`routes/search/+page.svelte`) already has a working autocomplete implementation:
- 150ms debounced input handlers (`onDepInput`, `onArrInput`)
- Calls `suggestStations()` API which hits `/api/stations/{provider}/suggest`
- Dropdown renders `SuggestMatch[]` with Korean name + English subtitle
- Selection handlers update form state

**What's missing:** The inline implementation should be extracted to a reusable `StationInput.svelte` component to reduce the search page from 780+ lines. The component needs:
- Keyboard navigation (arrow keys, Enter to select, Escape to close)
- Accessibility: `role="combobox"`, `aria-expanded`, `aria-activedescendant`
- Mobile touch handling (the `onblur` 200ms timeout is fragile)

**Do NOT install `simple-svelte-autocomplete` or `svelte-typeahead`.** Both are Svelte 4 era libraries with no confirmed Svelte 5 runes support. The existing implementation is already wired to the backend suggest API and matches the design system.

### 4. SSE Real-Time Updates -- Native EventSource (Already Implemented)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Native `EventSource` API | - | Real-time task status updates | Already implemented in `stores/sse.svelte.ts` with Svelte 5 runes |

**Confidence:** HIGH (code already exists)

**Current state:** `sse.svelte.ts` provides a singleton EventSource store with:
- Connection to `/api/tasks/events`
- `task_update` event listener
- Callback-based subscription with auto-disconnect when no subscribers
- Svelte 5 `$state` for connection status

**What's missing:** The SSE store currently fires a generic callback without passing the event data. The `TaskEvent` payload (task_id, status, message, attempt_count, reservation_number) should be parsed and passed to subscribers so the UI can update specific task cards without a full refetch.

**Refinement:**
```typescript
// Enhanced SSE store should parse and forward event data
type TaskUpdateEvent = {
  task_id: string;
  status: string;
  message: string;
  attempt_count: number;
  reservation_number?: string;
};

type SseCallback = (event: TaskUpdateEvent) => void;
```

**Do NOT install `sveltekit-sse` or `@lok/kit-sse`.** Those libraries are designed for SvelteKit SSR endpoints (server-side SSE producers). This app uses adapter-static (SPA mode) -- the SSE producer is the Axum backend, not SvelteKit. The native `EventSource` browser API with the existing store pattern is correct.

### 5. Multi-Provider Search Aggregation -- No Library Needed

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `Promise.all` + sort | - | Simultaneous SRT + KTX search | Already implemented in search page |

**Confidence:** HIGH (code already exists)

**Current state:** The search page already handles multi-provider search:
```typescript
const [srtResults, ktxResults] = await Promise.all([
  searchTrains('SRT', departure, arrival, date, time),
  searchTrains('KTX', departure, arrival, date, time)
]);
const merged = [...srtResults, ...ktxResults];
results = merged.sort((a, b) => a.dep_time.localeCompare(b.dep_time));
```

**What's missing:**
- Error handling per provider: if SRT fails but KTX succeeds, show KTX results with an error banner for SRT (currently any error hides all results)
- Provider credential check: before searching, verify credentials exist and prompt user to add them in settings if missing
- Loading state per provider: show which provider is still loading

### 6. Time Picker -- Custom Build (Enhance TimeSlider)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Custom `TimeSlider.svelte` | - | Departure time band selection | Already exists, needs UX refinement |

**Confidence:** HIGH

The existing `TimeSlider` component maps a slider value to time strings. This is functional. The improvement needed is visual -- showing labeled time bands (e.g., "06:00", "12:00", "18:00") rather than just a raw slider. No library needed.

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Date picker | Custom build | shadcn-svelte Calendar | Adds bits-ui + @internationalized/date deps for one component; output format mismatch (DateValue vs YYYYMMDD); design system clash |
| Date picker | Custom build | date-picker-svelte | Unclear Svelte 5 runes support; returns Date objects not YYYYMMDD |
| Autocomplete | Custom build (existing) | simple-svelte-autocomplete | Svelte 4 era; existing impl already wired to trilingual suggest API |
| SSE | Native EventSource (existing) | sveltekit-sse | Designed for SSR endpoints, not SPA + external API |
| Payment SDK | CDN script (existing) | @evervault/js npm | npm package just wraps CDN load; adds dependency without benefit |
| UI primitives | None | bits-ui | Would only use Calendar; overkill for one component |

## What to Install

**Nothing.** The existing `package.json` has everything needed. All gaps are implementation work, not missing libraries:

1. Extract `StationInput.svelte` component from inline autocomplete code
2. Build `DatePicker.svelte` modal using existing `BottomSheet.svelte` pattern
3. Enhance `sse.svelte.ts` to parse and forward `TaskEvent` data
4. Add per-provider error handling in multi-provider search
5. Add `evervault.d.ts` type declarations
6. Enhance `TimeSlider.svelte` with labeled time bands

## Sources

- [Evervault JavaScript SDK docs](https://docs.evervault.com/sdks/javascript) -- HIGH confidence, official
- [Evervault JS GitHub](https://github.com/evervault/evervault-js) -- HIGH confidence, official
- [shadcn-svelte Date Picker](https://www.shadcn-svelte.com/docs/components/date-picker) -- HIGH confidence, official
- [bits-ui npm](https://www.npmjs.com/package/bits-ui) -- version 2.16.3, HIGH confidence
- [@internationalized/date npm](https://www.npmjs.com/package/@internationalized/date) -- version 3.12.0, HIGH confidence
- [sveltekit-sse GitHub](https://github.com/razshare/sveltekit-sse) -- MEDIUM confidence, community
- [simple-svelte-autocomplete GitHub](https://github.com/pstanoev/simple-svelte-autocomplete) -- LOW confidence for Svelte 5 compat
