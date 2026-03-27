# Phase 2: Core Infrastructure - Research

**Researched:** 2026-03-27
**Confidence:** HIGH

## Summary

Phase 2 does not need to invent new foundations from scratch. The repo already contains:

- A canonical domain crate at `crates/bominal-domain` with locale support, translation tables, station registries, DTOs, task enums, auth payloads, reservation types, and SSE event payloads.
- A donor Leptos crate at `crates/bominal-frontend` with SSR-oriented wrappers for i18n, utilities, and API/server-function surfaces.
- A Svelte frontend at `frontend/src/lib` with the latest API route contracts, utility behavior, and auth/theme/SSE store semantics.

The correct move is to make `bominal-app` consume or re-export the canonical domain assets where possible, while adapting the Svelte and donor Leptos modules into the app crate's Phase 1 SSR/hydrate architecture.

## Key Findings

### i18n

- `crates/bominal-domain/src/i18n/mod.rs` already defines `Locale`, `t(locale, key)`, locale metadata, and station-name translation helpers.
- The current Svelte app still owns the practical locale UX via `frontend/src/lib/i18n/*.json`.
- The donor crate wraps the domain i18n surface with cookie parsing and Leptos context access in `crates/bominal-frontend/src/i18n.rs`.

### Utilities

- `frontend/src/lib/utils.ts` contains 8 frontend-facing helpers: `formatTime`, `formatDate`, `formatCost`, `statusVariant`, `slotToTimeString`, `formatTimeSlot`, `formatDisplayDate`, and `taskStatusI18nKey`.
- `crates/bominal-frontend/src/utils.rs` already ports 6 of those into Rust.
- `crates/bominal-domain/src/task.rs` already provides some lower-level domain helpers such as `TaskStatus::i18n_key()` and typed enums.

### Shared Types

- `crates/bominal-domain/src/dto.rs`, `task.rs`, `auth.rs`, `reservation.rs`, `evervault.rs`, and `task_event.rs` already cover most of the "20 shared types" called out in the roadmap.
- The current Svelte app's `frontend/src/lib/types/index.ts` is the best checklist for what the app layer actually expects to consume.

### API Layer

- The Svelte app already proxies the live server through `/api/...` via `frontend/src/lib/api/client.ts` and the route-specific modules in `frontend/src/lib/api/`.
- The donor Leptos crate's `crates/bominal-frontend/src/api/*.rs` proves the route surface and data shapes, but it uses direct service-layer server functions in places where the current roadmap prefers `/api/` proxying.

### State

- `frontend/src/lib/stores/auth.svelte.ts`, `theme.svelte.ts`, and `sse.svelte.ts` capture the current auth/session check pattern, theme cookie semantics, and EventSource lifecycle.
- Those behaviors can be ported into Leptos contexts/resources/stores without requiring the Phase 3 shell to be fully present yet.

## Recommendation

Break execution into three plans:

1. Rebuild the i18n and utility surface in `bominal-app` on top of `bominal-domain`.
2. Expose the typed domain/data surface through app-local modules with serde round-trip tests.
3. Add the typed `/api/` proxy layer plus auth/theme/SSE state foundations that later phases can plug into the root shell.

This keeps Phase 2 focused on shared infrastructure and avoids smuggling Phase 3 routing or Phase 4 page work into the foundation layer.
