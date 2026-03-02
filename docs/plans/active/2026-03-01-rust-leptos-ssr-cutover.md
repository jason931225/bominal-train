# 2026-03-01 Rust Leptos SSR Cutover

## Goal

Complete full runtime cutover to Rust-only services (no legacy runtime fallback):

1. Frontend -> Leptos SSR + Tailwind.
2. API -> axum 0.8 + Tokio.
3. Worker -> Rust async binary.
4. Data/Auth -> Supabase Postgres + Supabase Auth from day one.
5. Runtime queue/rate-limit leases -> Redis via `redis-rs`.

## Required Stack Contract

- Language/runtime: Rust 2024 + Tokio async runtime.
- HTTP/API framework: axum 0.8 with middleware-style routing/handlers.
- Data layer: sqlx 0.8 + Postgres + migrations + typed query mapping.
- HTTP clients: reqwest 0.13 with optional curl transport behind `curl-transport` feature.
- Observability/security: tower, tower-http, tracing, tracing-subscriber, jsonwebtoken, hmac, secrecy.
- Binaries: `api` + `worker`.
- Integrations: Supabase Auth (JWKS + webhook sync), Supabase Postgres, Evervault Relay, optional Resend notifier.
- Runtime controls: queue polling/reconcile/watch/rotation schedules via env knobs, JSON logs, graceful shutdown.

## Safety Model

1. Keep strict file-level tracking for every Rust implementation artifact in `docs/handoff/RUST_IMPLEMENTATION_FILE_MANIFEST.md`.
2. Maintain provider contract parity using `docs/handoff/PROVIDER_CONTRACT.md` and `docs/handoff/PROVIDER_FIELD_MAP.{json,md}`.
3. Retire legacy runtime support in compose/runtime docs and make Rust services canonical.
4. Gate every change with compile/test/doc validation before promoting runtime changes.

## Phase Plan

### Phase 0: Docs and Registry Hygiene

- Remove stale references to removed deployment artifacts in active operator docs.
- Register this plan in `docs/plans/active/README.md` and pointer library.
- Add/maintain Rust file manifest (`docs/handoff/RUST_IMPLEMENTATION_FILE_MANIFEST.md`).

### Phase 1: Rust Workspace Bootstrap

Target paths:
- `rust/Cargo.toml`
- `rust/crates/api/**`
- `rust/crates/worker/**`
- `rust/crates/shared/**`
- `rust/migrations/**`

Deliverables:
- workspace compiles (`cargo check`) on stable Rust 2024 edition.
- baseline config model for Supabase, Redis, Evervault, Resend, and scheduling knobs.

### Phase 2: Leptos SSR + Tailwind

Target paths:
- `rust/crates/api/src/web/**`
- `rust/frontend/**`

Deliverables:
- SSR shell rendered by Leptos via axum route.
- Tailwind pipeline and CSS output served from Rust API static route.
- responsive desktop/mobile baseline without dark-mode assumptions.

### Phase 3: API/Worker Runtime Contracts

Target paths:
- `rust/crates/api/src/http/**`
- `rust/crates/api/src/integrations/**`
- `rust/crates/worker/src/**`

Deliverables:
- liveness/readiness endpoints.
- Supabase JWKS validator wiring + auth webhook sync endpoint scaffold.
- Redis queue poll/reconcile/watch loops controlled by env durations.
- graceful shutdown for API + worker.

### Phase 4: Rust-Only Runtime Operations

Target paths:
- `infra/docker-compose*.yml`
- `infra/env/**`
- `docs/humans/engineering/ARCHITECTURE.md`
- `docs/humans/operations/DEPLOYMENT.md`
- `docs/humans/operations/RUNBOOK.md`

Deliverables:
- Rust-only compose/runtime wiring (`api`, `worker`, `web`) in dev/prod.
- documented rollback path and Rust runtime gate checklist.
- explicit "delete candidates" list for retired legacy runtime files.

## Verification

Minimum checks per milestone:

```bash
cd rust && cargo fmt --all -- --check
cd rust && cargo check --workspace
cd rust && cargo test --workspace
```

Docs and pointer checks:

```bash
bash infra/tests/test_docs_pointers.sh
bash infra/tests/test_docs_consistency.sh
bash infra/tests/test_intent_routing.sh
```

## Acceptance Criteria

1. Rust API + worker compile and run as canonical runtime services.
2. Leptos SSR frontend is served by the Rust API with Tailwind styles.
3. Supabase and Redis contracts are represented in Rust config/runtime scaffolding.
4. Rust implementation file manifest is complete and kept in sync.
5. Active/operator docs reflect Rust-only runtime policy and contain no stale deployment references.
