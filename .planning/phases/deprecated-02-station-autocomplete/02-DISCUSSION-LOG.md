# Phase 2: Station Autocomplete - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-25
**Phase:** 02-station-autocomplete
**Areas discussed:** Component extraction, IME composition handling, Keyboard navigation, Dropdown behavior

---

## Component Extraction

| Option | Description | Selected |
|--------|-------------|----------|
| Extract to StationInput.svelte | Reusable component with $bindable(), DRY | ✓ |
| Keep inline | Less indirection, easier independent customization | |
| You decide | Claude picks | |

**User's choice:** "make best judgement" — deferred to Claude
**Notes:** User said "you don't need to mind the existing code" — building fresh. Claude selected extraction based on Svelte 5 $bindable() pattern and DRY principle.

---

## IME Composition Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Wait for compositionend | Suppress API during composition, fire after compositionend + debounce | ✓ |
| Debounce through composition | Let debounce timer handle naturally, simpler code | |

**User's choice:** Deferred to Claude's research
**Notes:** Claude selected compositionend approach — standard for CJK input, prevents wasted API calls with partial jamo. Used by Google, Naver, Kakao.

---

## Keyboard Navigation Style

| Option | Description | Selected |
|--------|-------------|----------|
| Highlight only | Arrow keys move highlight, Enter commits, text stays as typed | ✓ |
| Live preview | Arrow keys update input text during navigation | |

**User's choice:** Deferred to Claude's research
**Notes:** Claude selected highlight-only per WAI-ARIA combobox pattern. More accessible, cleaner Escape behavior, matches browser native autocomplete.

---

## Dropdown Dismiss/Refocus Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Reshow last results | Cached results reappear on refocus | ✓ |
| Clear and require re-input | Fresh state on refocus | |

**User's choice:** Deferred to Claude's research
**Notes:** Claude selected reshow — reduces friction, matches Korail/SRT booking UX.

---

## Claude's Discretion

- Exact debounce timing, dropdown styling, loading indicator, empty state message, English name display

## Deferred Ideas

None
