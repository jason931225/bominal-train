# Rust Backend Parity Continuation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Deliver production-ready Rust backend parity (API + worker + data lifecycle) with no legacy runtime support.

**Architecture:** Split Rust runtime logic into testable modules (`shared`, `api` service layer, `worker` handlers), move queue execution to a DB-backed state machine, and preserve stable HTTP contracts while porting parity-required behavior from legacy endpoints. Enforce RED -> GREEN implementation on every task, with one small commit per task.

**Tech Stack:** Rust 2024, Tokio, axum 0.8, sqlx 0.8 + Postgres, redis-rs, reqwest 0.13, tower/tower-http, tracing, Supabase JWT/JWKS.

---

## Prerequisites

- Use `@superpowers/test-driven-development` inside each implementation subagent task.
- Keep scope limited to files explicitly listed per task.
- Do not touch `third_party/**`.
- Use `CARGO_HOME=/tmp/cargo-home` in all cargo commands.

---

### Task 1: Parity Matrix Baseline (API + Worker)

**Files:**
- Create: `docs/handoff/RUST_BACKEND_PARITY.md`
- Modify: `docs/handoff/README.md`
- Test: `infra/tests/test_docs_consistency.sh`

**Step 1: Write failing parity matrix skeleton check**

Add this required section header list into the new doc:

```md
## Legacy API Route Inventory
## Legacy Worker Inventory
## Rust Implementation Mapping
## Required for Cutover
## Deferred
```

**Step 2: Run doc consistency check to confirm baseline constraints**

Run: `bash infra/tests/test_docs_consistency.sh`
Expected: PASS (or fail only on unrelated pre-existing issues)

**Step 3: Populate parity inventory with concrete mappings**

Add route mappings for:
- `api/app/http/routes/{admin,auth,internal,modules,notifications,wallet}.py`
- `api/app/worker.py`
- `api/app/modules/train/worker.py`
- `api/app/modules/restaurant/worker.py`

Mark each row as `ported | in_progress | required | deferred`.

**Step 4: Re-run docs checks**

Run:
- `bash infra/tests/test_docs_pointers.sh`
- `bash infra/tests/test_docs_consistency.sh`

Expected: PASS

**Step 5: Commit**

```bash
git add docs/handoff/RUST_BACKEND_PARITY.md docs/handoff/README.md
git commit -m "docs(rust): add backend parity matrix for api and worker"
```

---

### Task 2: Shared Error Contract + API Envelope

**Files:**
- Create: `rust/crates/shared/src/error.rs`
- Modify: `rust/crates/shared/src/lib.rs`
- Modify: `rust/crates/api/src/main.rs`
- Test: `rust/crates/shared/src/error.rs` (unit tests module)
- Test: `rust/crates/api/tests/error_envelope_test.rs`

**Step 1: Write failing tests for error envelope contract**

Create test asserting envelope shape:

```rust
assert_eq!(body["code"], "invalid_request");
assert!(body.get("message").is_some());
assert!(body.get("request_id").is_some());
```

**Step 2: Run targeted tests to verify failure**

Run: `cd rust && CARGO_HOME=/tmp/cargo-home cargo test -p bominal-rust-api error_envelope`
Expected: FAIL (missing module/type or shape mismatch)

**Step 3: Implement minimal shared error model and axum conversion**

Implement:
- `ApiErrorCode` enum
- `ApiErrorEnvelope` struct
- `ApiError` type with `IntoResponse`

Export from `shared::lib` and switch `main.rs` string error responses to envelope-based responses for at least:
- auth verify
- webhook auth failure
- queue enqueue failures

**Step 4: Re-run tests**

Run:
- `cd rust && CARGO_HOME=/tmp/cargo-home cargo test -p bominal-rust-api error_envelope`
- `cd rust && CARGO_HOME=/tmp/cargo-home cargo check --workspace`

Expected: PASS

**Step 5: Commit**

```bash
git add rust/crates/shared/src/error.rs rust/crates/shared/src/lib.rs rust/crates/api/src/main.rs rust/crates/api/tests/error_envelope_test.rs
git commit -m "feat(rust): add shared api error envelope and wire core endpoints"
```

---

### Task 3: Runtime Job Persistence Schema

**Files:**
- Create: `rust/migrations/202603020001_runtime_jobs.sql`
- Modify: `rust/migrations/202603010001_bootstrap.sql` (only if needed for FK/bootstrap consistency)
- Test: `rust/crates/api/tests/runtime_jobs_schema_test.rs`

**Step 1: Write failing schema expectation test**

Add SQL-level test that expects columns:
- `job_id`
- `status`
- `attempt_count`
- `next_run_at`
- `last_error`
- `processed_at`

**Step 2: Run targeted test to ensure failure**

Run: `cd rust && CARGO_HOME=/tmp/cargo-home cargo test -p bominal-rust-api runtime_jobs_schema`
Expected: FAIL (table/columns missing)

**Step 3: Implement migration**

Create migration with:
- `runtime_jobs` table (unique `job_id`, json payload, status indexes)
- `runtime_job_events` append-only table
- `runtime_job_leases` table for reconcile claims

**Step 4: Re-run schema tests**

Run:
- `cd rust && CARGO_HOME=/tmp/cargo-home cargo test -p bominal-rust-api runtime_jobs_schema`
- `cd rust && CARGO_HOME=/tmp/cargo-home cargo check --workspace`

Expected: PASS

**Step 5: Commit**

```bash
git add rust/migrations/202603020001_runtime_jobs.sql rust/crates/api/tests/runtime_jobs_schema_test.rs
git commit -m "feat(rust): add runtime jobs persistence schema"
```

---

### Task 4: Repository Layer for Auth Sync and Runtime Jobs

**Files:**
- Create: `rust/crates/shared/src/repo/mod.rs`
- Create: `rust/crates/shared/src/repo/auth_sync_repo.rs`
- Create: `rust/crates/shared/src/repo/runtime_job_repo.rs`
- Modify: `rust/crates/shared/src/lib.rs`
- Modify: `rust/crates/api/src/main.rs`
- Test: `rust/crates/shared/src/repo/runtime_job_repo.rs` (unit tests module)

**Step 1: Write failing repo tests**

Define tests for repository API:
- `upsert_auth_sync`
- `insert_runtime_job`
- `transition_runtime_job_status`

**Step 2: Run targeted tests to confirm failure**

Run: `cd rust && CARGO_HOME=/tmp/cargo-home cargo test -p bominal-shared runtime_job_repo`
Expected: FAIL (repo module missing)

**Step 3: Implement minimal sqlx repositories**

Use `sqlx::query` with typed binds and explicit status transition guards.

Minimal contract:

```rust
pub enum RuntimeJobStatus { Queued, Running, Completed, Failed }
pub async fn insert_runtime_job(pool: &PgPool, ...) -> Result<()>;
pub async fn mark_running(pool: &PgPool, job_id: &str, lease_owner: &str) -> Result<bool>;
pub async fn mark_terminal(pool: &PgPool, job_id: &str, status: RuntimeJobStatus, last_error: Option<&str>) -> Result<()>;
```

**Step 4: Re-run tests**

Run:
- `cd rust && CARGO_HOME=/tmp/cargo-home cargo test -p bominal-shared runtime_job_repo`
- `cd rust && CARGO_HOME=/tmp/cargo-home cargo check --workspace`

Expected: PASS

**Step 5: Commit**

```bash
git add rust/crates/shared/src/repo rust/crates/shared/src/lib.rs rust/crates/api/src/main.rs
git commit -m "feat(rust): add shared repositories for auth sync and runtime jobs"
```

---

### Task 5: Queue Enqueue API -> DB + Redis Contract

**Files:**
- Modify: `rust/crates/api/src/main.rs`
- Create: `rust/crates/api/src/runtime_queue_service.rs`
- Test: `rust/crates/api/tests/enqueue_runtime_job_test.rs`

**Step 1: Write failing API test for enqueue side effects**

Test should assert:
- HTTP `202`
- DB row exists in `runtime_jobs`
- Redis list receives payload

**Step 2: Run targeted test to verify failure**

Run: `cd rust && CARGO_HOME=/tmp/cargo-home cargo test -p bominal-rust-api enqueue_runtime_job`
Expected: FAIL (DB persistence not yet wired in service)

**Step 3: Implement service and wire route**

Implement service function:

```rust
pub async fn enqueue_runtime_job(state: &AppState, req: EnqueueRuntimeJobRequest) -> Result<EnqueueRuntimeJobResponse, ApiError>
```

Order of operations:
1. Validate request
2. Persist queued job in Postgres
3. Push serialized payload to Redis
4. Return response envelope

**Step 4: Re-run tests**

Run:
- `cd rust && CARGO_HOME=/tmp/cargo-home cargo test -p bominal-rust-api enqueue_runtime_job`
- `cd rust && CARGO_HOME=/tmp/cargo-home cargo check --workspace`

Expected: PASS

**Step 5: Commit**

```bash
git add rust/crates/api/src/main.rs rust/crates/api/src/runtime_queue_service.rs rust/crates/api/tests/enqueue_runtime_job_test.rs
git commit -m "feat(rust): persist runtime jobs on enqueue and publish to redis"
```

---

### Task 6: Worker State Machine + Retry/DLQ

**Files:**
- Modify: `rust/crates/worker/src/main.rs`
- Create: `rust/crates/worker/src/job_runner.rs`
- Create: `rust/crates/worker/src/reconcile.rs`
- Test: `rust/crates/worker/tests/job_runner_test.rs`

**Step 1: Write failing worker tests**

Test scenarios:
- valid queued job transitions `queued -> running -> completed`
- malformed payload moves to DLQ + event row
- retryable failure increments `attempt_count` and schedules `next_run_at`

**Step 2: Run targeted tests to verify failure**

Run: `cd rust && CARGO_HOME=/tmp/cargo-home cargo test -p bominal-rust-worker job_runner`
Expected: FAIL (state machine missing)

**Step 3: Implement job runner and reconcile logic**

Add deterministic transitions and terminal handling:

```rust
match result {
  Ok(_) => mark_terminal(...Completed...)
  Err(e) if retryable => schedule_retry(...)
  Err(e) => mark_terminal(...Failed...) and push_dlq(...)
}
```

**Step 4: Re-run tests**

Run:
- `cd rust && CARGO_HOME=/tmp/cargo-home cargo test -p bominal-rust-worker job_runner`
- `cd rust && CARGO_HOME=/tmp/cargo-home cargo check --workspace`

Expected: PASS

**Step 5: Commit**

```bash
git add rust/crates/worker/src/main.rs rust/crates/worker/src/job_runner.rs rust/crates/worker/src/reconcile.rs rust/crates/worker/tests/job_runner_test.rs
git commit -m "feat(rust-worker): implement job lifecycle state machine with retry and dlq"
```

---

### Task 7: Supabase Webhook/Auth Verification Service Layer

**Files:**
- Create: `rust/crates/api/src/auth_service.rs`
- Modify: `rust/crates/api/src/main.rs`
- Test: `rust/crates/api/tests/supabase_auth_service_test.rs`

**Step 1: Write failing tests for auth service behavior**

Cases:
- webhook secret mismatch -> `401` envelope
- valid webhook persists auth sync row
- invalid bearer token -> `401`
- JWKS unavailable -> `503`

**Step 2: Run targeted tests and capture failure**

Run: `cd rust && CARGO_HOME=/tmp/cargo-home cargo test -p bominal-rust-api supabase_auth_service`
Expected: FAIL

**Step 3: Implement service extraction**

Move verify/webhook logic from `main.rs` into `auth_service.rs` and keep routes as thin adapters.

**Step 4: Re-run tests**

Run:
- `cd rust && CARGO_HOME=/tmp/cargo-home cargo test -p bominal-rust-api supabase_auth_service`
- `cd rust && CARGO_HOME=/tmp/cargo-home cargo check --workspace`

Expected: PASS

**Step 5: Commit**

```bash
git add rust/crates/api/src/main.rs rust/crates/api/src/auth_service.rs rust/crates/api/tests/supabase_auth_service_test.rs
git commit -m "refactor(rust-api): extract supabase auth verification and webhook service"
```

---

### Task 8: Legacy Route Decision Closure (Required vs Deferred)

**Files:**
- Modify: `docs/handoff/RUST_BACKEND_PARITY.md`
- Modify: `docs/humans/engineering/ARCHITECTURE.md`
- Modify: `docs/humans/operations/RUNBOOK.md`
- Test: `infra/tests/test_docs_consistency.sh`

**Step 1: Write failing checklist item (manual) for unresolved required routes**

Set target: zero `required` routes left without a Rust mapping owner/date.

**Step 2: Run docs consistency check**

Run: `bash infra/tests/test_docs_consistency.sh`
Expected: PASS after edits

**Step 3: Close decisions**

For each legacy endpoint from Task 1, mark one of:
- `ported now`
- `deferred with reason`
- `retired with replacement`

**Step 4: Re-run docs checks**

Run:
- `bash infra/tests/test_docs_pointers.sh`
- `bash infra/tests/test_docs_consistency.sh`

Expected: PASS

**Step 5: Commit**

```bash
git add docs/handoff/RUST_BACKEND_PARITY.md docs/humans/engineering/ARCHITECTURE.md docs/humans/operations/RUNBOOK.md
git commit -m "docs(rust): finalize legacy parity decisions for backend cutover"
```

---

### Task 9: Verification Gate and CI Alignment

**Files:**
- Modify: `infra/scripts/local-check.sh`
- Modify: `infra/scripts/predeploy-check.sh`
- Modify: `infra/tests/test_policy_runtime_parity.sh`
- Test: `infra/tests/test_worker_runtime_guards.sh`

**Step 1: Write failing runtime gate expectation tests**

Ensure tests assert:
- API/worker checks run Rust tests + checks
- No legacy Python/Next checks remain

**Step 2: Run targeted infra tests to verify failure (if expectations changed)**

Run:
- `bash infra/tests/test_policy_runtime_parity.sh`
- `bash infra/tests/test_worker_runtime_guards.sh`

Expected: FAIL before gate updates (if any delta introduced)

**Step 3: Implement or adjust gate behavior**

Keep these commands authoritative:
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`

**Step 4: Re-run full verification matrix**

Run:
- `cd rust && CARGO_HOME=/tmp/cargo-home cargo fmt --all -- --check`
- `cd rust && CARGO_HOME=/tmp/cargo-home cargo check --workspace`
- `cd rust && CARGO_HOME=/tmp/cargo-home cargo test --workspace`
- `bash infra/tests/test_policy_runtime_parity.sh`
- `bash infra/tests/test_worker_runtime_guards.sh`
- `bash infra/tests/test_docs_pointers.sh`
- `bash infra/tests/test_docs_consistency.sh`

Expected: PASS

**Step 5: Commit**

```bash
git add infra/scripts/local-check.sh infra/scripts/predeploy-check.sh infra/tests/test_policy_runtime_parity.sh infra/tests/test_worker_runtime_guards.sh
git commit -m "chore(rust): align runtime verification gates with backend parity completion"
```

---

### Task 10: Final Release Readiness Snapshot

**Files:**
- Modify: `CHANGELOG.md`
- Modify: `docs/handoff/RUST_IMPLEMENTATION_FILE_MANIFEST.md`
- Modify: `docs/plans/active/README.md`
- Test: `infra/tests/test_changelog.sh`

**Step 1: Add changelog entry for backend parity completion scope**

Use Keep-a-Changelog categories and commit-based bullets.

**Step 2: Update manifest for every new Rust file added in Tasks 2-9**

Ensure each file has status and notes.

**Step 3: Re-run final documentation and changelog checks**

Run:
- `bash infra/tests/test_changelog.sh`
- `bash infra/tests/test_docs_pointers.sh`
- `bash infra/tests/test_docs_consistency.sh`

Expected: PASS

**Step 4: Final workspace verification**

Run:
- `cd rust && CARGO_HOME=/tmp/cargo-home cargo fmt --all -- --check`
- `cd rust && CARGO_HOME=/tmp/cargo-home cargo check --workspace`
- `cd rust && CARGO_HOME=/tmp/cargo-home cargo test --workspace`

Expected: PASS

**Step 5: Commit**

```bash
git add CHANGELOG.md docs/handoff/RUST_IMPLEMENTATION_FILE_MANIFEST.md docs/plans/active/README.md
git commit -m "docs(release): finalize rust backend parity readiness snapshot"
```

---

## Controller Execution Notes (for Subagent-Driven Run)

- Execute tasks sequentially with `@superpowers/subagent-driven-development` (fresh implementer subagent per task).
- After each task:
  1. spec compliance review
  2. code quality review
  3. only then mark task complete.
- Use `@superpowers/requesting-code-review` for final whole-branch review.
- Use `@superpowers/finishing-a-development-branch` after Task 10.

## Done Criteria

- Required backend parity rows in `docs/handoff/RUST_BACKEND_PARITY.md` are all `ported` or explicitly `retired`.
- Rust API/worker have DB-backed job lifecycle with retry + DLQ behavior.
- Auth/queue/error contracts are test-covered and stable.
- Infra gates and docs reflect Rust-only production posture.
- Full verification matrix passes.
