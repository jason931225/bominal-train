# HTTP/3 (Maybe) - Deferred TODO

Status: Deferred
Priority: Low
Date: 2026-03-04

Scope:
- Edge-only HTTP/3 (QUIC at reverse proxy/load balancer).
- Keep bominal app transport unchanged (no native QUIC listener in API).

## Why "maybe"
- Potential benefit: better latency/resilience for mobile networks.
- Potential problem: added ops complexity and compatibility risk may outweigh gains.

## Promotion Criteria (to move from deferred -> active)
- Canary shows either:
  - p95 latency improvement >= 5% on user-facing routes, or
  - connection/protocol failure reduction >= 20%.
- No sustained regression:
  - no error-rate increase > 0.1 percentage points,
  - no auth/session regression signals.

## Rollout Sketch (when activated)
1. Capture 7-day baseline for latency/error/auth-session signals.
2. Enable HTTP/3 at edge for a canary slice with automatic HTTP/2/1.1 fallback.
3. Compare against baseline and decide promote/rollback.

## Out Of Scope
- Native HTTP/3 listener inside bominal API.
- Outbound provider HTTP/3 changes.

## Owner / Trigger
- Owner: TBD
- Trigger: revisit after current runtime refactor/hardening backlog is complete.
