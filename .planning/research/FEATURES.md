# Feature Landscape

**Domain:** Train reservation SaaS (Korean SRT/KTX) -- frontend wiring milestone
**Researched:** 2026-03-24

## Table Stakes

Features users expect. Missing = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Station autocomplete with Korean input | Users type in Korean; raw text input without suggestions is unusable for station names | Low | Already implemented inline, needs extraction to reusable component with keyboard nav |
| Date picker (not raw YYYYMMDD input) | No user should type "20260401" manually; this is a standard UX expectation | Medium | Must support Korean locale, modal/bottom-sheet pattern, YYYYMMDD output format |
| Multi-provider search results | Core value prop: search SRT + KTX simultaneously | Low | Already implemented via Promise.all, needs per-provider error isolation |
| Real-time task status updates | After creating a booking task, users need to see progress without manual refresh | Low | SSE already connected, needs event data parsing passed to UI |
| Payment card management | Users must add cards to enable auto-pay on booking tasks | Low | Evervault encryption already wired in evervault.ts, card CRUD API exists |
| Provider credential setup | Users cannot search without SRT/KTX login credentials | Medium | API exists, settings page shows unset state, needs setup flow UI |

## Differentiators

Features that set product apart. Not expected, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Graceful provider fallback | When one provider fails or creds missing, still show results from the other | Low | Show partial results plus error banner instead of total failure |
| Prioritized train selection with drag-reorder | Users pick multiple trains in priority order; system tries them sequentially | Low | Already implemented with SortableList.svelte |
| Auto-pay toggle with card selection | One-tap auto-payment when seat found, removing manual payment step | Low | Already implemented in review modal |
| Station autocorrect (Korean keyboard layout detection) | Suggest API handles English-keyboard Korean input (e.g., tjtb maps to Seoul) | Low | Backend station_search already supports layout hint detection |
| Trilingual station names | Show Korean plus English plus Japanese names in suggestions | Low | Suggest API returns all three; UI already renders ko plus en |

## Anti-Features

Features to explicitly NOT build.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Inline calendar always visible | Takes too much screen real estate on mobile; conflicts with glass-morphism card layout | Use modal/bottom-sheet date picker that opens on tap |
| Full shadcn-svelte component library | Massive dependency for one or two components; design system clash with existing glass-morphism tokens | Build custom components matching existing design system |
| SvelteKit SSR for SSE | App uses adapter-static (SPA mode); SSE comes from Axum backend, not SvelteKit | Keep native EventSource in browser connecting to /api/tasks/events |
| Complex state management library | Svelte 5 runes handle all reactive state needs | Continue using runes plus stores pattern already in codebase |
| Train schedule browsing | Out of scope per PROJECT.md; users search specific routes, not browse timetables | Only search-to-book flow |

## Feature Dependencies

```
Provider credential setup --> Multi-provider search (cannot search without creds)
Station autocomplete --> Search form submission
Date picker --> Search form submission
Search results --> Train selection --> Task creation
Card management --> Auto-pay toggle in task creation
SSE updates --> Task status display (tasks page)
```

## MVP Recommendation (This Milestone)

Prioritize (in order):
1. Station autocomplete extraction -- reusable StationInput.svelte with keyboard nav (unblocks clean search UX)
2. Date picker modal -- replace YYYYMMDD text input (biggest UX gap)
3. Per-provider error isolation -- show partial results when one provider fails
4. SSE event data parsing -- pass task event payload to subscribers for targeted UI updates
5. Provider credential setup flow -- settings page UX for adding SRT/KTX login

Defer:
- Time picker enhancement: Current TimeSlider works functionally; visual polish is lower priority than missing features
- i18n completeness: Error message localization can be done incrementally

## Sources

- Codebase analysis of search page, SSE store, evervault interop, types
- PROJECT.md active requirements list
