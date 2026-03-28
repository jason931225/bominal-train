# Phase 6: Settings and Shared Components - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Port the settings page and the remaining shared UI components so the protected shell has a complete account/settings surface and the reusable SSR/island primitives needed by the remaining migration phases.

</domain>

<decisions>
## Implementation Decisions

### the agent's Discretion
- Keep Phase 6 focused on settings plus reusable components explicitly named in the roadmap. Avoid leaking Phase 7 interop and Phase 8 server integration work into this pass unless a tiny compatibility shim is unavoidable.
- Reuse the Phase 5 helper surface (`ProtectedPage`, `SseReload`, `StatusChip`, page-card styles) instead of inventing another presentation layer for settings.
- Prefer porting donor utility logic such as card-brand detection and bottom-sheet behavior as app-local components only when the current phase actually uses them.
- Maintain the typed `/api` proxy boundary from `crate::api`; the settings page should orchestrate existing provider/card/auth endpoints, not bypass them.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/bominal-frontend/src/pages/settings_view.rs` already models provider credentials, card management, appearance toggles, and logout in Leptos.
- `crates/bominal-frontend/src/components/{card_brand,glass_panel,skeleton,bottom_sheet,selection_prompt,ticket_card,task_card}.rs` contains most of the shared component behavior the roadmap names for this phase.
- `crates/bominal-app/src/api.rs` already exposes the needed provider, card, and logout endpoints.
- Phase 5 already added app-local shared helpers and page/card styles that the settings page can build on immediately.

### Established Patterns
- Protected routes now live under `crates/bominal-app/src/pages` and are auth-guarded through `ProtectedPage`.
- The current app styling direction is a mix of page-local markup plus a relatively small shared CSS layer in `crates/bominal-app/style/app.css`.
- `bominal-ui` remains the long-term shared visual source of truth, so Phase 6 should keep components small and composable instead of hard-coding page-specific presentation into each one.

### Integration Points
- `crates/bominal-app/src/lib.rs` still points `/settings` at the Phase 3 shell stub and will need to move onto a real page module.
- Search and reservations already consume card/task concepts that can benefit from the shared card-brand, ticket-card, and task-card surfaces once they exist app-locally.
- Theme and locale controls should hook into the existing browser/state helpers instead of forking new preference storage.

</code_context>

<specifics>
## Specific Ideas

- Split the phase into one settings-page plan and one reusable-component plan so the settings route can ship while the broader shared surface is formalized.
- Port the donor card-brand utility and the most immediately useful shared components first, then adapt Phase 5 pages to reuse them only where the churn is justified.
- Keep the first settings pass focused on real provider/card/logout flows plus appearance toggles; deeper payment UX polish can remain incremental.

</specifics>

<deferred>
## Deferred Ideas

- Evervault card encryption remains Phase 7 work even if the settings page needs the form shell earlier.
- Axum/SSR integration and final CSS/build cleanup remain later phases.

</deferred>
