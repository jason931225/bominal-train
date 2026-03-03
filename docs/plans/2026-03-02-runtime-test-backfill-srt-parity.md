# Runtime Test Backfill and SRT Parity Rewrite Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rebuild the Rust runtime test suite so tests are intentional, deterministic, and enforce functional parity with `third_party/srtgo/srtgo/srt.py`.

**Architecture:** Use a layered strategy: module-unit tests for pure logic and security-critical parsing, integration tests for axum routes and worker execution, and explicit parity-contract tests bound to `srtgo` operation coverage. Keep the default suite hermetic (no external network), and avoid flaky dependency coupling.

**Tech Stack:** Rust workspace tests (`cargo test`), `tokio::test`, axum `oneshot` integration testing, deterministic `ReqwestSrtClient`, serde/JSON fixtures, `cargo llvm-cov` coverage gating.

---

### Task 1: Re-baseline and Rewrite the Existing API Error Envelope Suite

**Files:**
- Modify: `runtime/crates/api/tests/error_envelope_test.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn queue_without_db_returns_persistence_unavailable() {
    // Build app with empty database_url and any redis URL.
    // POST /api/runtime/queue/enqueue.
    // Assert status 503 and details.stage == "persist".
}
```

**Step 2: Run test to verify it fails**

Run: `cd runtime && cargo test -p bominal-api --test error_envelope_test queue_without_db_returns_persistence_unavailable -- --nocapture`  
Expected: FAIL if old assertion still expects redis/connect behavior.

**Step 3: Write minimal implementation**

```rust
// Rewrite queue error tests to match current control flow:
// persist happens before redis connect/push.
// Keep auth tests intact, remove assertion drift.
```

**Step 4: Run test to verify it passes**

Run: `cd runtime && cargo test -p bominal-api --test error_envelope_test -- --nocapture`  
Expected: PASS for all envelope branches in this file.

**Step 5: Commit**

```bash
git add runtime/crates/api/tests/error_envelope_test.rs
git commit -m "test(api): rewrite error envelope suite for current queue flow"
```

### Task 2: Make Runtime Queue Branches Fully Testable Without External Infra

**Files:**
- Modify: `runtime/crates/api/src/services/runtime_queue_service.rs`
- Modify: `runtime/crates/api/src/http/runtime_queue.rs`
- Test: `runtime/crates/api/src/services/runtime_queue_service.rs` (inline `#[cfg(test)]`)

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn enqueue_maps_connect_and_push_failures_deterministically() {
    // Fake persistence succeeds.
    // Fake queue returns connect failure, then push failure.
    // Assert exact EnqueueRuntimeJobError variants.
}
```

**Step 2: Run test to verify it fails**

Run: `cd runtime && cargo test -p bominal-api enqueue_maps_connect_and_push_failures_deterministically -- --nocapture`  
Expected: FAIL because current service path is tightly coupled to sqlx/redis clients.

**Step 3: Write minimal implementation**

```rust
// Introduce internal service seams for persistence + queue push
// and keep existing public API as thin wrapper around production adapters.
// Do not change external HTTP contract.
```

**Step 4: Run test to verify it passes**

Run: `cd runtime && cargo test -p bominal-api runtime_queue_service -- --nocapture`  
Expected: PASS for service-level branch tests (validate, persist, duplicate conflict, redis unavailable/connect/push).

**Step 5: Commit**

```bash
git add runtime/crates/api/src/services/runtime_queue_service.rs runtime/crates/api/src/http/runtime_queue.rs
git commit -m "test(api): add deterministic runtime queue service branch tests"
```

### Task 3: Add Internal Service JWT Security Tests (Fail-Closed Paths)

**Files:**
- Modify: `runtime/crates/api/src/http/internal_auth.rs` (inline `#[cfg(test)]`)

**Step 1: Write the failing test**

```rust
#[test]
fn rejects_wrong_alg_and_bad_signature() {
    // Build malformed JWT with alg != HS256 and bad HMAC.
    // Assert InvalidServiceToken.
}
```

**Step 2: Run test to verify it fails**

Run: `cd runtime && cargo test -p bominal-api internal_auth::tests -- --nocapture`  
Expected: FAIL before parser/validator tests exist.

**Step 3: Write minimal implementation**

```rust
// Add focused unit tests for:
// - parse_bool matrix
// - decode_base64url rejects invalid chars/padding
// - role/scope internal checks
// - iat/exp skew handling
// - compatibility_aliases_enabled non-prod debug gating
```

**Step 4: Run test to verify it passes**

Run: `cd runtime && cargo test -p bominal-api internal_auth::tests -- --nocapture`  
Expected: PASS with explicit deny-path coverage.

**Step 5: Commit**

```bash
git add runtime/crates/api/src/http/internal_auth.rs
git commit -m "test(api): backfill internal service JWT fail-closed tests"
```

### Task 4: Add Internal API Route Auth Contract Tests

**Files:**
- Create: `runtime/crates/api/tests/internal_api_auth_contract_test.rs`
- Modify: `runtime/crates/api/tests/error_envelope_test.rs` (shared helpers only if needed)

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn internal_route_without_service_token_returns_401() {
    // PUT /internal/v1/providers/srt/credentials without header.
    // Assert 401 + unauthorized envelope code.
}
```

**Step 2: Run test to verify it fails**

Run: `cd runtime && cargo test -p bominal-api --test internal_api_auth_contract_test -- --nocapture`  
Expected: FAIL before file exists.

**Step 3: Write minimal implementation**

```rust
// Add test fixtures to create signed service JWT for happy-path auth pass-through.
// Assert:
// - missing token => 401
// - malformed token => 401
// - valid token reaches handler and returns downstream error/status (not auth error)
```

**Step 4: Run test to verify it passes**

Run: `cd runtime && cargo test -p bominal-api --test internal_api_auth_contract_test -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add runtime/crates/api/tests/internal_api_auth_contract_test.rs runtime/crates/api/tests/error_envelope_test.rs
git commit -m "test(api): add internal API auth contract coverage"
```

### Task 5: Backfill Worker Executor Parsing and Policy Tests

**Files:**
- Modify: `runtime/crates/worker/src/runtime/executor.rs` (inline `#[cfg(test)]`)

**Step 1: Write the failing test**

```rust
#[test]
fn canonical_operation_aliases_match_expected_matrix() {
    assert_eq!(canonical_operation_name("search"), Some("search_train"));
    assert_eq!(canonical_operation_name("pay_with_card"), Some("pay_with_card"));
}
```

**Step 2: Run test to verify it fails**

Run: `cd runtime && cargo test -p bominal-worker canonical_operation_aliases_match_expected_matrix -- --nocapture`  
Expected: FAIL before tests exist.

**Step 3: Write minimal implementation**

```rust
// Add tests for:
// - canonical alias mapping
// - missing login material => fatal error
// - payment policy blocks auto pay in CI/testing
// - simulated failure kinds map to correct ExecutionErrorKind
```

**Step 4: Run test to verify it passes**

Run: `cd runtime && cargo test -p bominal-worker executor::tests -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add runtime/crates/worker/src/runtime/executor.rs
git commit -m "test(worker): add executor parsing and policy tests"
```

### Task 6: Backfill Retry/Backoff Determinism Tests

**Files:**
- Modify: `runtime/crates/worker/src/runtime/retry.rs` (inline `#[cfg(test)]`)

**Step 1: Write the failing test**

```rust
#[test]
fn exponential_backoff_caps_at_max_delay() {
    // attempt_count high -> delay must not exceed max_delay
}
```

**Step 2: Run test to verify it fails**

Run: `cd runtime && cargo test -p bominal-worker retry::tests -- --nocapture`  
Expected: FAIL before tests exist.

**Step 3: Write minimal implementation**

```rust
// Add tests for:
// - classify_error retryable vs non-retryable
// - compute_backoff_delay boundaries
// - plan_failure schedule vs dead-letter transitions
```

**Step 4: Run test to verify it passes**

Run: `cd runtime && cargo test -p bominal-worker retry::tests -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add runtime/crates/worker/src/runtime/retry.rs
git commit -m "test(worker): cover retry classification and backoff planning"
```

### Task 7: Backfill Shared SRT Adapter and Deterministic Client Tests

**Files:**
- Modify: `runtime/crates/shared/src/providers/srt/mod.rs` (inline `#[cfg(test)]`)
- Modify: `runtime/crates/shared/src/providers/srt/reqwest_client.rs` (inline `#[cfg(test)]`)

**Step 1: Write the failing test**

```rust
#[test]
fn deterministic_client_consumes_planned_failure_once() {
    // failure once for SearchTrain => first call fails, second succeeds
}
```

**Step 2: Run test to verify it fails**

Run: `cd runtime && cargo test -p bominal-shared deterministic_client_consumes_planned_failure_once -- --nocapture`  
Expected: FAIL before tests exist.

**Step 3: Write minimal implementation**

```rust
// Add tests for:
// - operation_name mapping for every SrtOperationRequest variant
// - adapter relogin-on-auth-failure flow
// - clear/login/logout session transitions
// - deterministic canned response shape used by worker execution
```

**Step 4: Run test to verify it passes**

Run: `cd runtime && cargo test -p bominal-shared providers::srt -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add runtime/crates/shared/src/providers/srt/mod.rs runtime/crates/shared/src/providers/srt/reqwest_client.rs
git commit -m "test(shared): add SRT adapter and deterministic client contracts"
```

### Task 8: Add Explicit SRTGO Parity Contract Test

**Files:**
- Create: `runtime/crates/shared/tests/srtgo_parity_contract_test.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn srtgo_core_methods_are_present_and_mapped() {
    // Read third_party/srtgo/srtgo/srt.py.
    // Assert required def names exist:
    // login, logout, search_train, reserve, reserve_standby,
    // reserve_standby_option_settings, get_reservations, ticket_info,
    // cancel, pay_with_card, reserve_info, refund, clear.
}
```

**Step 2: Run test to verify it fails**

Run: `cd runtime && cargo test -p bominal-shared --test srtgo_parity_contract_test -- --nocapture`  
Expected: FAIL before file exists.

**Step 3: Write minimal implementation**

```rust
// Add parity test that binds required SRT operations to runtime contract.
// Include explicit failure message naming missing method(s) to prevent silent drift.
```

**Step 4: Run test to verify it passes**

Run: `cd runtime && cargo test -p bominal-shared --test srtgo_parity_contract_test -- --nocapture`  
Expected: PASS.

**Step 5: Commit**

```bash
git add runtime/crates/shared/tests/srtgo_parity_contract_test.rs
git commit -m "test(shared): enforce srtgo operation parity contract"
```

### Task 9: Enforce Workspace Test and Coverage Gates

**Files:**
- Modify: `runtime/README.md`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing test**

```bash
cd runtime
cargo test --workspace
```

Expected: Fails before all above tasks land.

**Step 2: Run test to verify it fails**

Run: `cd runtime && cargo test --workspace -- --nocapture`  
Expected: FAIL until full backfill is complete.

**Step 3: Write minimal implementation**

```md
Add test matrix + commands in runtime/README.md:
- cargo test --workspace
- cargo llvm-cov --workspace --lcov --output-path target/coverage/lcov.info
```

**Step 4: Run test to verify it passes**

Run:
- `cd runtime && cargo test --workspace -- --nocapture`
- `cd runtime && cargo llvm-cov --workspace --summary-only`

Expected: PASS and coverage summary generated.

**Step 5: Commit**

```bash
git add runtime/README.md CHANGELOG.md
git commit -m "docs(test): document runtime test matrix and coverage gate"
```

---

## Execution Guardrails

- Keep `third_party/**` read-only.
- Do not add flaky network-dependent tests.
- Fail-closed behavior must always be covered by explicit negative tests.
- Prefer unit tests near logic for private helper coverage; use integration tests only for contract boundaries.
- Preserve existing runtime HTTP response envelope contract unless user approves a behavior change.

## Definition of Done

- `cargo test --workspace` passes from `runtime/`.
- All new tests are deterministic and intentional (each test maps to one behavior contract).
- SRT operation parity with `srtgo/srt.py` is explicitly asserted by test.
- Coverage evidence is produced for changed crates and reviewed against `docs/MANUAL.md` quality floor.

