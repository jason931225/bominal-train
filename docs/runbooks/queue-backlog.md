# Queue Backlog Runbook

## Purpose

Diagnose and mitigate runtime queue backlog growth before it becomes a user-facing incident.

## Signals

- Rising queued/running job counts without completion recovery.
- Increasing end-to-end task latency.
- Worker saturation or repeated retries.

## Procedure

1. Confirm backlog scope:
   - queue depth trend
   - stuck job age distribution
   - provider concentration
2. Validate worker health and dependencies:
   - database connectivity
   - Redis/queue connectivity
   - provider path availability
3. Apply mitigation in priority order:
   - stop introducing new load if required
   - increase safe processing capacity
   - resolve hot failure class causing retries/dead-letter events
4. Verify drain trend and user-impact reduction.
5. Capture incident timeline and preventive follow-up work.

## Guardrails

- Keep retries bounded and explicit.
- Avoid destructive queue resets without explicit approval.
- Preserve auditability of state transitions.

## Exit Criteria

- Backlog trend is declining.
- Queue latency returns to acceptable range.

## References

- `docs/MANUAL.md#operations-runbook-core`
- `docs/MANUAL.md#deployment-and-rollback-standard`
