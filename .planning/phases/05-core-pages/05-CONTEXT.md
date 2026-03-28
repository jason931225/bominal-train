# Phase 5: Core Pages - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Replace the Phase 3 protected-page stubs for `/home`, `/search`, `/tasks`, and `/reservations` with working app pages backed by the typed `/api` proxy layer and the shared shell from Phase 3.

</domain>

<decisions>
## Implementation Decisions

### the agent's Discretion
- Keep Phase 5 bounded to the four route surfaces named in the roadmap; avoid dragging Phase 6 settings/components work into this pass unless a tiny helper is required to make the pages usable.
- Reuse the existing `crate::api` typed server functions as the only data boundary. Do not introduce a second API client or donor-only module structure.
- Prefer intentionally simpler page-local UI over a wholesale transplant of the older `bominal-frontend` component tree when the donor code would pull in unnecessary dependencies.
- Preserve the Phase 3 router and navigation contract. Phase 5 should replace route bodies, not redesign the shell.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/bominal-app/src/api.rs` already exposes the Phase 5 backend surface: station lookup, train search, task CRUD, reservation CRUD, ticket detail, and card listing.
- `crates/bominal-app/src/types.rs` already re-exports the app-facing task, train, reservation, ticket, and card types from `bominal-domain`.
- `crates/bominal-app/src/utils.rs` already contains shared formatting helpers for time, date, cost, time-slot conversion, and task-status badge mapping.
- `crates/bominal-frontend/src/pages/{home_view,search_panel,tasks_view,reservations_view}.rs` and its component directory remain useful donor references for interaction flow and state shape.

### Established Patterns
- Protected pages currently live under the Phase 3 shell and should remain auth-guarded.
- Auth pages already use app-local modules under `crates/bominal-app/src/pages`, which is the right landing zone for the Phase 5 routes as well.
- The CSS direction in `crates/bominal-app/style/app.css` favors glass surfaces plus utility classes rather than a heavy component stylesheet.

### Integration Points
- `crates/bominal-app/src/lib.rs` must swap the `/home`, `/search`, `/tasks`, and `/reservations` routes away from `shell_pages.rs` and onto real page modules.
- Home, tasks, and reservations should respond to `/api/tasks/events` so task-state changes propagate through the shell without manual refresh loops.
- Search task creation must emit the current `CreateTaskInput` shape from `bominal-domain`, which means a single-provider task with ordered target trains.

</code_context>

<specifics>
## Specific Ideas

- Add a minimal SSE helper in `bominal-app` rather than copying the entire donor component set.
- Build the search surface around the current typed API contract: provider toggle, station suggestion input, native date/time controls, and explicit task-review summary instead of donor modals.
- Implement task cancellation with a mobile swipe-reveal affordance that still leaves a clear desktop action path.

</specifics>

<deferred>
## Deferred Ideas

- The broader shared component port (`GlassPanel`, `TicketCard`, `TaskCard`, `BottomSheet`, etc.) remains Phase 6 unless a tiny subset is unavoidable.
- Deep browser-only interaction polish beyond the core flows can wait until later UI/interoperability phases.

</deferred>
