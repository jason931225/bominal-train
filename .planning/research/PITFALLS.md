# Domain Pitfalls

**Domain:** Train reservation SaaS -- SvelteKit SPA frontend wiring
**Researched:** 2026-03-24

## Critical Pitfalls

Mistakes that cause rewrites or major issues.

### Pitfall 1: Evervault SDK Loading Race Condition

**What goes wrong:** The Evervault SDK loads from CDN asynchronously. If a user navigates to the card management page and tries to encrypt before the SDK finishes loading, `window.Evervault` is undefined and encryption silently fails or throws.

**Why it happens:** The current `getEvervault()` returns `null` if the SDK is not loaded yet, and callers may not handle the null case gracefully.

**Consequences:** Card data sent unencrypted (PCI violation) or user gets cryptic error.

**Prevention:** Add a loading state and retry mechanism. Before showing the card form, verify `window.Evervault` is available. Show a loading indicator if the SDK is still loading. Never fall back to submitting unencrypted data.

**Detection:** Test card submission on slow networks (throttle to 3G in DevTools).

### Pitfall 2: SSE EventSource Cookie Authentication on SPA

**What goes wrong:** `EventSource` does not support custom headers. Authentication relies on session cookies sent via `credentials: 'include'` equivalent. If the SPA is served from a different origin than the API, cookies are not sent.

**Why it happens:** In development, the SvelteKit dev server (port 5173) and Axum (different port) are different origins. `EventSource` has no `credentials` option in its constructor.

**Consequences:** SSE connection returns 401 or connects unauthenticated, receiving no events.

**Prevention:** Ensure same-origin deployment (Axum serves the SPA static files from `frontend/build/`). In development, use Vite proxy to forward `/api/*` to Axum. The `EventSource` constructor will send cookies automatically when same-origin.

**Detection:** Check browser DevTools Network tab for SSE connection -- verify cookies are attached and response is 200, not 401.

### Pitfall 3: Multi-Provider Search All-or-Nothing Error Handling

**What goes wrong:** Currently `Promise.all([searchSRT, searchKTX])` rejects if either provider fails. One provider having invalid credentials or being temporarily down kills the entire search.

**Why it happens:** `Promise.all` short-circuits on first rejection.

**Consequences:** Users see "search failed" even though one provider had valid results.

**Prevention:** Use `Promise.allSettled()` instead. Show results from successful providers and display a targeted error for the failed one.

```typescript
const [srt, ktx] = await Promise.allSettled([
  searchTrains('SRT', ...),
  searchTrains('KTX', ...)
]);
// Collect fulfilled results, show error banners for rejected
```

**Detection:** Test with only one provider's credentials configured.

## Moderate Pitfalls

### Pitfall 4: Autocomplete Dropdown Z-Index vs Bottom Sheet

**What goes wrong:** The station suggestion dropdown renders with `z-20`, but the review bottom sheet uses `z-50`. If the user opens suggestions while a bottom sheet is visible (unlikely but possible with keyboard nav), dropdowns appear behind the sheet.

**Prevention:** Use a consistent z-index system. Define z-index tiers as CSS variables or constants: dropdown (20), floating bar (30), backdrop (40), sheet (50).

### Pitfall 5: Date Picker YYYYMMDD Validation

**What goes wrong:** Users enter invalid dates like "20261301" (month 13) or past dates. The backend may or may not validate; the frontend currently does no validation.

**Prevention:** The custom DatePicker component should constrain selection to valid future dates. On manual entry (if allowed), validate format and range before enabling the search button.

### Pitfall 6: Station Suggest API Parameter Name Mismatch

**What goes wrong:** The frontend `suggestStations()` function sends `{ query: q }` as the URL parameter, but the backend expects `?q=...` (the struct field is `pub q: String`).

**Prevention:** Verify the parameter name matches. The current `search.ts` sends `{ query }` but the Axum handler deserializes `SuggestQuery { q: String }`. This is a bug -- the frontend should send `{ q: query }` or the param key should be `q`.

**Detection:** Test station autocomplete -- if suggestions never appear, this is likely the cause.

### Pitfall 7: onblur Timeout Race Condition in Autocomplete

**What goes wrong:** The current autocomplete uses `onblur` with `setTimeout(200ms)` to allow click events on dropdown items before hiding. On fast taps (mobile), 200ms may not be enough, causing the dropdown to close before the selection registers.

**Prevention:** Use `onmousedown` (already done correctly) for desktop, but also handle `ontouchstart` for mobile. Alternatively, track whether a suggestion was clicked and only hide if no selection occurred.

## Minor Pitfalls

### Pitfall 8: Korean IME Composition Events

**What goes wrong:** Korean input uses an IME that composes characters. The `oninput` event fires during composition, triggering API calls for partial characters.

**Prevention:** The 150ms debounce mitigates this, but for optimal behavior, check `event.isComposing` and skip API calls during composition. Only fire on the final composed character.

### Pitfall 9: SSE Memory Leak on Page Navigation

**What goes wrong:** If the SSE store connects but the component that subscribed unmounts without calling the unsubscribe function, the callback remains in the array.

**Prevention:** The current implementation returns an unsubscribe function from `subscribe()`. Ensure every subscriber calls it in Svelte's `onDestroy` or uses `$effect` cleanup. The auto-disconnect when callbacks.length is 0 is correct.

### Pitfall 10: Evervault Meta Tags Missing in SPA

**What goes wrong:** The Evervault interop reads team/app IDs from meta tags. In SPA mode with adapter-static, the HTML is generated at build time. If the meta tags are not in app.html or injected by the server, Evervault initialization fails silently.

**Prevention:** Verify that Axum injects the meta tags when serving index.html, or hardcode them in `frontend/src/app.html`. Do not rely on SvelteKit server hooks (they do not run in SPA mode).

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| StationInput component extraction | Breaking existing search page behavior | Write integration test for search flow before refactoring; keep same API surface |
| DatePicker modal build | YYYYMMDD format edge cases (leap years, month boundaries) | Use simple calendar grid logic; test Feb 29 and month transitions |
| Per-provider error handling | Promise.allSettled unfamiliar pattern | Simple pattern; document in code comments |
| SSE enhancement | Backward compatibility if event format changes | Version the event schema; parse defensively with fallback |
| Provider credential setup | Sensitive data in forms (passwords) | Never log credentials; use type=password; clear form on submit |

## Sources

- Codebase analysis: search page, SSE store, evervault interop, API client
- Evervault docs: CDN loading requirements
- MDN: EventSource credentials behavior, IME composition events
- Svelte 5 docs: runes lifecycle and cleanup patterns
