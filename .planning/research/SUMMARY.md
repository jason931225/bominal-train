# Research Summary: Bominal Frontend Wiring

**Domain:** Train reservation SaaS -- SvelteKit SPA frontend wiring to Axum REST API
**Researched:** 2026-03-24
**Overall confidence:** HIGH

## Executive Summary

The Bominal frontend-backend wiring milestone requires no new npm dependencies. The existing SvelteKit 5.55 + Tailwind CSS 4 + TypeScript stack has everything needed. All five areas investigated (Evervault integration, autocomplete, date picker, SSE, multi-provider search) are either already implemented or best served by custom components that match the existing glass-morphism design system.

The codebase is further along than the active requirements list suggests. Station autocomplete is working (needs component extraction), multi-provider search with Promise.all is implemented (needs error isolation), SSE real-time updates are connected (needs event data parsing), and Evervault card encryption is wired (needs type safety). The main UX gap is the date picker -- currently a raw YYYYMMDD text input that should become a modal calendar.

Third-party UI libraries (shadcn-svelte, bits-ui, date-picker-svelte) were evaluated and rejected. They introduce design system conflicts with the existing glass-morphism CSS variable pattern, add dependencies for single-component use cases, and several lack confirmed Svelte 5 runes support. The recommendation is to build custom components using the patterns already established in the codebase.

One likely bug was discovered during research: the station suggest API call sends `{ query }` as a URL parameter but the backend expects `?q=...`. This parameter name mismatch would cause autocomplete to silently fail.

## Key Findings

**Stack:** No new dependencies needed. All gaps are implementation work, not missing libraries.
**Architecture:** SPA (adapter-static) connecting to Axum via same-origin fetch/EventSource. Pattern is sound.
**Critical pitfall:** Promise.all for multi-provider search must become Promise.allSettled to avoid all-or-nothing failures.

## Implications for Roadmap

Based on research, suggested phase structure:

1. **Bug fixes and foundation** - Fix suggest API parameter mismatch, add Evervault type declarations
   - Addresses: Station autocomplete reliability, type safety
   - Avoids: Building on broken foundation

2. **Component extraction** - Extract StationInput.svelte from inline search page code, add keyboard navigation and accessibility
   - Addresses: Station autocomplete (table stakes), search page maintainability (780+ lines)
   - Avoids: Monolithic page component anti-pattern

3. **Date picker** - Build custom DatePicker.svelte modal using BottomSheet pattern
   - Addresses: Biggest UX gap (raw YYYYMMDD input)
   - Avoids: Third-party library dependency; design system clash

4. **Search resilience** - Promise.allSettled for per-provider error handling, provider credential check before search
   - Addresses: Multi-provider search reliability, credential setup flow
   - Avoids: All-or-nothing search failures

5. **SSE enhancement** - Parse TaskEvent data in SSE store, targeted UI updates
   - Addresses: Real-time task status without full refetch
   - Avoids: Bandwidth waste, UI flicker

**Phase ordering rationale:**
- Phase 1 first because the suggest API bug blocks autocomplete testing
- Phase 2 before 3 because it reduces search page complexity, making subsequent changes easier
- Phase 3 is independent and can be done in parallel with Phase 2
- Phase 4 depends on provider credential setup which is a settings page concern
- Phase 5 is lowest priority because SSE already works (just inefficiently)

**Research flags for phases:**
- Phase 3 (DatePicker): Standard patterns, unlikely to need research. Calendar grid rendering is a solved problem.
- Phase 4 (Provider credentials): May need research on SRT/KTX API credential validation endpoints
- All other phases: Standard patterns, no additional research needed

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Verified installed versions (Svelte 5.55, SvelteKit 2.55); confirmed Evervault SDK docs; evaluated all major Svelte date picker libraries |
| Features | HIGH | Based on direct codebase analysis; feature gaps clearly identified from PROJECT.md active list |
| Architecture | HIGH | Architecture already implemented and working; recommendations are refinements not redesigns |
| Pitfalls | HIGH for 1-3, MEDIUM for 4-10 | Critical pitfalls verified against code and docs; moderate/minor based on common SPA patterns |

## Gaps to Address

- SRT/KTX API credential validation flow (how does the backend verify provider credentials are valid?)
- Vite proxy configuration for development (needed for SSE cookies to work cross-origin in dev)
- Whether Axum currently injects Evervault meta tags into the SPA index.html
