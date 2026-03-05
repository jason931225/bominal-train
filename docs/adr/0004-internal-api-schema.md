# ADR 0004: Versioned Internal API Schema and SDK

- Status: Accepted
- Date: 2026-03-05

## Context

Internal service-to-service routes are security-sensitive and evolve faster than public API surfaces. Without an explicit schema and generated client, cross-service integration drift is likely.

## Decision

- Version internal API contracts explicitly (`/internal/v1/...`).
- Keep a versioned OpenAPI artifact at `sdk/openapi/bominal-internal.v1.json`.
- Generate typed internal SDK artifacts under `sdk/ts/internal/*`.
- Enforce schema/SDK drift checks in CI with a generation check script.

## Consequences

- Internal integrations consume a stable contract instead of hand-maintained request wiring.
- Contract changes are explicit and reviewable.
- CI catches schema/SDK drift before merge.
