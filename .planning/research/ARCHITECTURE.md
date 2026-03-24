# Architecture Patterns

**Domain:** Train reservation SaaS -- SvelteKit SPA to Axum REST API wiring
**Researched:** 2026-03-24

## Current Architecture

```
Browser (SvelteKit SPA, adapter-static)
  |
  |-- fetch() with credentials: 'include' ---> Axum REST API (:PORT)
  |                                              |-- /api/search (POST)
  |                                              |-- /api/stations/{provider}/suggest (GET)
  |                                              |-- /api/tasks (CRUD)
  |                                              |-- /api/tasks/events (SSE)
  |                                              |-- /api/cards (CRUD)
  |                                              |-- /api/providers (CRUD)
  |                                              |-- /api/reservations (GET)
  |                                              |
  |-- EventSource('/api/tasks/events') --------->|-- SSE broadcast per user
  |                                              |
  |-- CDN: js.evervault.com/v2 (encrypt)         |-- Evervault Outbound Relay (decrypt)
                                                 |-- PostgreSQL 16
                                                 |-- Valkey (sessions, rate limits)
```

### Component Boundaries

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| `lib/api/client.ts` | Typed fetch wrapper with error handling, credentials | All API endpoints |
| `lib/api/*.ts` | Per-domain API functions (search, tasks, cards, etc.) | client.ts |
| `lib/stores/sse.svelte.ts` | Singleton EventSource connection, callback dispatch | /api/tasks/events |
| `lib/interop/evervault.ts` | Lazy Evervault SDK init, card encryption | CDN SDK, /api/cards |
| `lib/interop/passkey.ts` | WebAuthn credential creation/assertion | Browser WebAuthn API |
| `lib/components/*.svelte` | Reusable UI primitives (GlassPanel, BottomSheet, etc.) | Parent components |
| `routes/search/+page.svelte` | Search form, results, task creation flow | API functions, components |

### Data Flow: Search to Task Creation

```
1. User types station name
2. StationInput debounces (150ms) --> GET /api/stations/{provider}/suggest?q=...
3. User selects station, date, time, passengers
4. User taps Search --> POST /api/search (or Promise.all for both providers)
5. Results displayed, user selects trains in priority order
6. User opens review modal, sets seat pref + auto-pay
7. User taps Create Task --> POST /api/tasks
8. Redirect to /tasks page
9. SSE stream delivers TaskEvent updates as runner executes
```

## Patterns to Follow

### Pattern 1: API Module per Domain

**What:** Each API domain (search, tasks, cards, providers, reservations, auth) gets its own `lib/api/{domain}.ts` file that imports from `client.ts`.

**When:** Always. Already established in codebase.

**Example:**
```typescript
// lib/api/search.ts
import { get, post } from './client';
import type { SuggestResult } from '$lib/types';

export function suggestStations(provider: string, query: string): Promise<SuggestResult> {
  return get(`/api/stations/${provider}/suggest`, { q: query });
}
```

### Pattern 2: Svelte 5 Runes for Component State

**What:** Use `$state`, `$derived`, `$effect` for all reactive state. No Svelte 4 stores (`writable`, `readable`).

**When:** Always in new code. The codebase already uses runes exclusively.

**Example:**
```typescript
let value = $state('');
let suggestions = $state<SuggestMatch[]>([]);
const hasSuggestions = $derived(suggestions.length > 0);
```

### Pattern 3: Glass-Morphism Design Tokens via CSS Variables

**What:** All styling uses CSS custom properties (`--color-*`) rather than hardcoded colors. Components wrap content in `GlassPanel` for consistent card styling.

**When:** Every UI component.

**Why:** Enables light/dark mode and theme switching (Glass, Clear Sky) without component changes.

### Pattern 4: Bottom Sheet Modal for Mobile-First Inputs

**What:** Complex inputs (date picker, confirmation dialogs) open as bottom sheets rather than inline forms or browser native pickers.

**When:** Any input that benefits from a larger touch target or dedicated UI space.

**Example:** The review modal in search page already uses this pattern with `sheet-enter` animation.

### Pattern 5: Debounced API Calls for Typeahead

**What:** Use `setTimeout`/`clearTimeout` with 150ms delay for search-as-you-type inputs.

**When:** Any input that triggers an API call on keystroke.

**Why:** Prevents flooding the API with requests. 150ms is fast enough to feel instant but batches rapid typing.

## Anti-Patterns to Avoid

### Anti-Pattern 1: Installing UI Component Libraries for Single Components

**What:** Adding bits-ui, shadcn-svelte, or Flowbite for one or two components.

**Why bad:** Introduces a second design system that conflicts with the existing glass-morphism tokens. Forces choosing between two styling approaches on every new component. Dependencies grow for minimal benefit.

**Instead:** Build custom components using the existing `GlassPanel`, `BottomSheet`, and CSS variable patterns. The design system is cohesive -- keep it that way.

### Anti-Pattern 2: Mixing Svelte 4 Store Patterns with Runes

**What:** Using `writable()` or `$:` reactive declarations alongside `$state`.

**Why bad:** Two reactivity systems in one codebase. Confusing for contributors, impossible to compose cleanly.

**Instead:** Use `$state`, `$derived`, `$effect` exclusively. The SSE store already demonstrates the correct pattern with module-level `$state`.

### Anti-Pattern 3: Monolithic Page Components

**What:** Putting all logic in a single `+page.svelte` file (the search page is 780+ lines).

**Why bad:** Hard to test, hard to review, hard to reuse.

**Instead:** Extract reusable components (StationInput, DatePicker) and keep page components as orchestrators that compose smaller pieces.

### Anti-Pattern 4: Full Refetch on SSE Events

**What:** Calling `listTasks()` on every SSE `task_update` event.

**Why bad:** Wastes bandwidth, causes UI flicker, defeats the purpose of real-time updates.

**Instead:** Parse the `TaskEvent` payload from SSE and update the specific task in the local store. Only full-refetch on page mount.

## Scalability Considerations

| Concern | Current (MVP) | At Scale |
|---------|--------------|----------|
| SSE connections | One EventSource per logged-in user | Axum broadcast channels scale to thousands; add cleanup cron (already exists) |
| Search latency | Sequential per provider; Promise.all for both | Consider caching station lists client-side; search results are not cacheable |
| Station suggest | API call per keystroke (debounced) | Station list is ~50 entries per provider; could prefetch and filter client-side |
| Bundle size | No external UI libs; Svelte 5 compiles small | Keep dependencies minimal; current approach is correct |

## Sources

- Codebase analysis: api/client.ts, stores/sse.svelte.ts, routes/search/+page.svelte
- Axum SSE implementation: crates/bominal-server/src/sse.rs
- Evervault architecture: crates/bominal-server/src/evervault.rs
