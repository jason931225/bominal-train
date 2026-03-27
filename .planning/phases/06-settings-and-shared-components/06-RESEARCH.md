# Phase 6: Settings and Shared Components - Research

**Researched:** 2026-03-27
**Confidence:** HIGH

## Summary

Phase 6 is best treated as two connected slices: the `/settings` page itself, and the component/utilities backlog that page and later phases need. The donor settings page is detailed but already broken into sensible subsections, while the remaining shared components fall cleanly into pure SSR helpers and interactive islands.

## Key Findings

### Settings donor surface is already organized

- `crates/bominal-frontend/src/pages/settings_view.rs` already covers the core settings requirements: current-user info, provider credential management, payment cards, appearance controls, and logout.
- That donor page depends primarily on provider/card/auth server functions plus a small component set (`CardBrand`, `GlassPanel`) rather than the heavier search modal stack.
- The current `bominal-app` typed API already exposes the needed provider/card/logout endpoints, so Phase 6 is mostly adaptation and styling work.

### Shared components naturally split into SSR vs interactive

- Pure SSR candidates from the roadmap map well to donor files: `status_chip.rs`, `card_brand.rs`, `skeleton.rs`, and a small app-local `GlassPanel`/`Icon` surface.
- Interactive candidates map to donor components but should be ported selectively: `bottom_sheet.rs`, `selection_prompt.rs`, `ticket_card.rs`, and `task_card.rs`.
- Phase 5 already landed enough page styling that these components can be introduced without recreating the entire donor CSS contract.

### Current app state after Phase 5

- `/home`, `/search`, `/tasks`, and `/reservations` are now real app pages, but `/settings` is still routed to the old shell stub.
- Phase 5 added app-local `SseReload`, `StatusChip`, protected-page helpers, and shared page/card styles, which lowers the amount of setup Phase 6 still needs.
- Card-brand detection and formatted ticket/task cards are the clearest reusable gaps left between the current app and the donor component inventory.

## Recommendation

Use two plans:

1. Port `/settings` with provider/card/auth/appearance flows on top of the existing typed API and current app shell.
2. Formalize the remaining shared component surface in `bominal-app` by porting the pure SSR helpers first and the interactive islands second, then refit the settings page and Phase 5 pages onto those primitives where it buys clarity.
