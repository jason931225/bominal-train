> Archive note (2026-02-14): Completed and moved from `docs/plans/active/` during Stage 8 closure.
> See `docs/plans/archive/2026-02-14-program-closure-report.md` for final status.

# Stage 4 Restaurant Policy Enforcement Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enforce restaurant execution policy invariants (auth fallback order, payment-step lease key, and non-committing concurrency boundaries) in backend policy code and worker orchestration.

**Architecture:** Implement policy as pure functions in `api/app/modules/restaurant/policy.py` and invoke them from restaurant worker orchestration. Keep logic deterministic and testable: policy helpers return typed outcomes, while the worker applies those outcomes to task-attempt records and retry scheduling.

**Tech Stack:** FastAPI backend, SQLAlchemy task model, Redis lease coordination, pytest/pytest-asyncio.

---

### Task 1: Create restaurant policy types and pure policy helpers

**Files:**
- Create: `api/app/modules/restaurant/policy.py`
- Create: `api/app/modules/restaurant/types.py`
- Test: `api/tests/test_restaurant_policy.py`

**Step 1: Write failing policy tests**

```python
def test_auth_fallback_order_is_refresh_then_bootstrap_then_fail():
    ...

def test_payment_lease_key_is_provider_account_restaurant():
    ...

def test_non_committing_actions_mark_concurrency_safe():
    ...
```

**Step 2: Run tests to verify RED**

Run: `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_restaurant_policy.py`
Expected: FAIL because policy module/types do not exist.

**Step 3: Implement minimal policy module**

Implement pure helpers:
- `build_payment_lease_key(provider: str, account_ref: str, restaurant_id: str) -> str`
- `resolve_auth_fallback_step(refresh_attempts: int, bootstrap_attempted: bool) -> AuthStep`
- `is_non_committing_phase(action: str) -> bool`

**Step 4: Run tests to verify GREEN**

Run: `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_restaurant_policy.py`
Expected: PASS.

**Step 5: Commit**

```bash
git add api/app/modules/restaurant/policy.py api/app/modules/restaurant/types.py api/tests/test_restaurant_policy.py
git commit -m "feat(restaurant): add policy helpers for auth fallback and lease key"
```

---

### Task 2: Add Redis lease helper for payment-step exclusivity

**Files:**
- Create: `api/app/modules/restaurant/lease.py`
- Modify: `api/app/modules/restaurant/policy.py`
- Test: `api/tests/test_restaurant_policy.py`

**Step 1: Write failing lease tests**

```python
async def test_payment_lease_acquire_blocks_second_holder(...):
    ...

async def test_payment_lease_release_restores_availability(...):
    ...
```

**Step 2: Run tests to verify RED**

Run: `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_restaurant_policy.py -k lease`
Expected: FAIL because lease helper does not exist.

**Step 3: Implement minimal lease helper**

- Redis `SET key value NX EX <ttl>` acquire semantics.
- Release only by holder token match.
- Lease key must be generated from `provider+account+restaurant_id` policy helper.

**Step 4: Run tests to verify GREEN**

Run: `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_restaurant_policy.py -k lease`
Expected: PASS.

**Step 5: Commit**

```bash
git add api/app/modules/restaurant/lease.py api/app/modules/restaurant/policy.py api/tests/test_restaurant_policy.py
git commit -m "feat(restaurant): enforce payment-step lease semantics"
```

---

### Task 3: Integrate policy enforcement into restaurant worker path

**Files:**
- Modify: `api/app/modules/restaurant/worker.py`
- Modify: `api/app/worker_restaurant.py`
- Test: `api/tests/test_restaurant_worker_policy_flow.py`

**Step 1: Write failing integration tests**

```python
async def test_worker_uses_auth_fallback_sequence_before_failure(...):
    ...

async def test_worker_blocks_parallel_payment_with_same_lease_key(...):
    ...

async def test_non_committing_phase_can_run_without_payment_lease(...):
    ...
```

**Step 2: Run tests to verify RED**

Run: `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_restaurant_worker_policy_flow.py`
Expected: FAIL because worker still uses placeholder behavior.

**Step 3: Implement minimal worker orchestration changes**

- Invoke policy helper to decide auth attempt sequence.
- Acquire payment lease only for committing actions.
- Emit safe attempt metadata only (`meta_json_safe`) with no secrets.

**Step 4: Run tests to verify GREEN**

Run: `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_restaurant_worker_policy_flow.py`
Expected: PASS.

**Step 5: Commit**

```bash
git add api/app/modules/restaurant/worker.py api/app/worker_restaurant.py api/tests/test_restaurant_worker_policy_flow.py
git commit -m "feat(restaurant-worker): enforce auth fallback and payment lease policy"
```

---

### Task 4: Add configuration knobs and docs for policy tuning

**Files:**
- Modify: `api/app/core/config.py`
- Modify: `infra/env/dev/api.env.example`
- Modify: `infra/env/prod/api.env.example`
- Modify: `docs/ARCHITECTURE.md`
- Modify: `docs/RUNBOOK.md`

**Step 1: Write failing config and docs expectations**

Add tests for defaults in `api/tests/test_restaurant_policy_config.py`.

**Step 2: Run tests to verify RED**

Run: `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_restaurant_policy_config.py`
Expected: FAIL before new settings exist.

**Step 3: Implement minimal config surface**

Add settings (examples):
- `RESTAURANT_AUTH_REFRESH_RETRIES`
- `RESTAURANT_PAYMENT_LEASE_TTL_SECONDS`
- `RESTAURANT_BOOTSTRAP_TIMEOUT_SECONDS`

**Step 4: Run tests and docs checks**

Run:
- `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_restaurant_policy_config.py`
- `bash infra/tests/test_docs_consistency.sh`

Expected: PASS.

**Step 5: Commit**

```bash
git add api/app/core/config.py infra/env/dev/api.env.example infra/env/prod/api.env.example docs/ARCHITECTURE.md docs/RUNBOOK.md api/tests/test_restaurant_policy_config.py
git commit -m "config/docs: add restaurant policy settings and operator guidance"
```

---

### Task 5: Full verification gate

**Step 1: Run restaurant policy suites**

Run:
- `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_restaurant_policy.py tests/test_restaurant_worker_policy_flow.py tests/test_restaurant_policy_config.py`

Expected: PASS.

**Step 2: Run backend baseline**

Run: `docker compose -f infra/docker-compose.yml run --rm api pytest -q`
Expected: PASS.

**Step 3: Run docs validators**

Run:
- `bash infra/tests/test_docs_pointers.sh`
- `bash infra/tests/test_changelog.sh`
- `bash infra/tests/test_intent_routing.sh`
- `bash infra/tests/test_docs_consistency.sh`

Expected: all PASS.

## Acceptance Criteria

- Auth fallback order is enforced exactly: refresh retries -> bootstrap -> fail.
- Payment operations are mutually exclusive for the same `provider+account+restaurant_id` lease key.
- Non-committing phases remain concurrency-safe without acquiring payment lease.
- Policy behavior is encoded in automated tests and documented.

## Assumptions and Defaults

- Restaurant tasks continue to use existing generic `tasks` table and safe attempt metadata fields.
- Bootstrap mechanism is policy-routed, with implementation hooks defined for later provider integrations.
