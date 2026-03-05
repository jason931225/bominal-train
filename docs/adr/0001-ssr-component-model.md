# ADR 0001: SSR Component Model Boundary

- Status: Accepted
- Date: 2026-03-05

## Context

`bominal` serves user and admin experiences from the Rust API runtime. Reusable UI fragments were historically mixed with route handlers, making ownership and review scope unclear.

## Decision

- Keep SSR-first rendering in `runtime/crates/api`.
- Keep shared visual components and view helpers in `runtime/crates/ui_patterns`.
- Route handlers own HTTP behavior and data shaping; UI-pattern crates own reusable markup and composition primitives.
- Do not introduce a separate frontend framework runtime for core SSR surfaces.

## Consequences

- UI refactors can be reviewed independently from HTTP service logic.
- Shared component changes are reused consistently across auth/dashboard/admin surfaces.
- SSR route code remains focused on policy, auth, and data orchestration.
