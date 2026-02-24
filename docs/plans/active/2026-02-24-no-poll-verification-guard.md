# No-Poll Verification Guard (Local + CI)

## Summary

Establish a deterministic verification guard that proves train task UIs are event-driven and do not perform periodic polling. The guard runs locally and in CI and fails on regression.

## Goal and Success Criteria

Goal:
- Prevent reintroduction of timer-based polling for train task status surfaces.

Success criteria:
- Dashboard, task detail, and top-nav attention pass automated checks for no periodic task polling.
- CI fails if interval polling is reintroduced for task status refresh paths.
- Local command exists and is documented for quick validation before commit/deploy.

## In Scope

- Train task status surfaces:
  - `web/components/train/train-dashboard.tsx`
  - `web/components/train/train-task-detail.tsx`
  - `web/components/top-nav-task-attention.tsx`
- Existing SSE endpoint behavior:
  - `GET /api/train/tasks/events`
- Test and guard scripts:
  - web unit tests and optional static guard script
- Docs:
  - `docs/ARCHITECTURE.md`
  - `docs/CONTRIBUTING.md`
  - `docs/RUNBOOK.md`
  - `CHANGELOG.md`

## Out of Scope

- Re-architecture of SSE transport.
- New push channel types (WebSocket migration).
- Payment flow changes.
- Cross-module UI polling behavior unrelated to train task status.

## Implementation Approach

1. Define verification policy.
- No periodic polling means:
  - no `setInterval` or recurring timers that trigger task status fetches for dashboard/task detail/top-nav attention.
- Allowed refresh triggers:
  - initial load
  - SSE `task_state` events
  - explicit user actions (pause/retry/pay/cancel/hide)
  - `visibilitychange` to visible (single recovery fetch)
- Document this contract in architecture/contributing docs.

2. Add deterministic test coverage for each surface.
- Dashboard:
  - assert no interval timer with task polling cadence is registered
  - assert refresh occurs on SSE event
- Task detail:
  - assert fetch occurs on mount and matching `task_state` event only
  - assert no periodic timer registration
- Top-nav attention:
  - assert no 60s task polling interval
  - assert reload on SSE + visibility restore
  - assert hidden tab suppresses SSE-triggered fetch until visible
- Keep tests resilient to framework internal timers by checking for task-polling intervals specifically.

3. Add static regression guard (optional but recommended).
- Add a lightweight script under `infra/scripts/` to scan targeted files for disallowed timer patterns tied to train task fetch codepaths.
- Guard behavior:
  - scope to known files/components
  - allow non-task-related timers
  - fail with actionable path/line output
- Use this as a fail-fast complement to behavior tests.

4. Wire CI gate.
- Ensure no-poll unit tests run in existing web CI test job.
- Add static no-poll guard script to infra quality gates if implemented.
- Fail-closed behavior: any regression blocks deploy prerequisites.

5. Local developer workflow.
- Add one command to run no-poll checks quickly (unit tests + static guard if present).
- Document in runbook/contributing as required pre-commit check for train UI status changes.

6. Observability sanity guidance.
- Add runbook verification snippet for manual spot-check:
  - idle page should show no periodic `/api/train/tasks*` network loop
  - SSE stream may remain open
  - one-time refresh on visibility restore is expected
- Keep this operator validation secondary to automated enforcement.

## Important Public API / Interface / Type Notes

- No API schema changes required.
- Behavioral contract clarification:
  - `task_state` events are the trigger for UI refresh.
  - high-frequency internal worker states remain suppressed from publish path.
- No new frontend public props/types required unless testability utilities are added.

## Test Cases and Scenarios

1. Dashboard no-poll.
- Given mounted dashboard, task list fetch happens initially.
- When idle (no events), no periodic task fetch increments.
- When `task_state` SSE event arrives, one refresh occurs.

2. Task detail no-poll.
- Given mounted detail for `taskId=X`, initial fetch runs once.
- On SSE event for `taskId=Y`, no reload.
- On SSE event for `taskId=X`, reload executes.
- No interval-based reload timer is registered.

3. Top-nav attention no-poll.
- Initial attention fetch runs once.
- No interval with polling cadence is registered.
- SSE event triggers fetch only when tab visible.
- Visibility restore triggers one fetch.

4. Static guard (if added).
- Fails when disallowed interval polling pattern is introduced in targeted files.
- Passes with event-driven code.

5. CI integration.
- Pipeline fails when any no-poll test or guard fails.
- Pipeline passes with current event-driven behavior.

## Rollout and Safety

Rollout sequence:
1. Add or adjust tests first.
2. Add static guard and CI wiring.
3. Update docs.
4. Validate locally and in CI.

Risk:
- false positives from generic timers.

Mitigation:
- scope checks to task-status codepaths and known interval values/patterns.
- keep behavioral tests as primary source of truth.

## Assumptions and Defaults

- Scope is local plus CI enforcement.
- SSE remains the canonical refresh mechanism for train task status surfaces.
- Visibility-change one-shot refresh is allowed as recovery behavior.
- Existing backend SSE endpoint and suppressed-state policy remain unchanged.
