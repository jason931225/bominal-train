# Phase 5: Core Pages - Research

**Researched:** 2026-03-27
**Confidence:** HIGH

## Summary

Phase 5 can be completed without resurrecting the entire donor frontend crate. The current `bominal-app` already has the typed server-function surface needed for home, search, tasks, and reservations. The donor Leptos pages are still valuable references, but the cheapest correct path is to port the interaction model, not the entire donor component hierarchy.

## Key Findings

### API coverage is already in place

- `crates/bominal-app/src/api.rs` already exposes `list_stations`, `suggest_stations`, `search_trains`, `list_tasks`, `create_task`, `update_task`, `delete_task`, `list_reservations`, `ticket_detail`, `cancel_reservation`, `pay_reservation`, `refund_reservation`, and `list_cards`.
- `CreateTaskInput` in `crates/bominal-domain/src/dto.rs` still represents a single-provider reservation task with ordered `TargetTrain`s, so the search page should keep a provider toggle rather than trying to create mixed-provider tasks.

### Donor pages are reference material, not a drop-in

- `crates/bominal-frontend/src/pages/home_view.rs` and `tasks_view.rs` are light enough to mirror behaviorally, but they depend on donor-only presentation components.
- `crates/bominal-frontend/src/pages/search_panel.rs` contains the needed state machine for search and task creation, but a direct copy would require many component transplants that belong more naturally in Phase 6.
- `crates/bominal-frontend/src/pages/reservations_view.rs` confirms the reservation action flow and card dependency, but its card/panel abstractions can be flattened into simpler app-local markup.

### Minimal shared helpers unblock the whole phase

- A small `SseReload` helper is sufficient to power live updates for home, tasks, and reservations.
- A lightweight status-chip helper plus a few page-shell/swipe CSS utilities are enough to keep the Phase 5 pages readable without expanding the component surface too early.

## Recommendation

Split Phase 5 into four plans:

1. Land shared Phase 5 helpers plus the home dashboard.
2. Build the search workflow with provider toggle, station suggestions, results, and task creation.
3. Build the tasks page with active/completed segmentation, SSE refresh, and swipe-reveal cancel.
4. Build the reservations page with provider filter, ticket details, and pay/cancel/refund actions.
