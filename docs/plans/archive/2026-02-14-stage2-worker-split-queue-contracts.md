> Archive note (2026-02-14): Completed and moved from `docs/plans/active/` during Stage 8 closure.
> See `docs/plans/archive/2026-02-14-program-closure-report.md` for final status.

# Stage 2 Worker Split + Queue Contracts Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enforce and verify queue-domain isolation for train and restaurant workers so jobs cannot be consumed by the wrong worker domain.

**Architecture:** Keep two ARQ workers (`app.worker_train.WorkerTrainSettings`, `app.worker_restaurant.WorkerRestaurantSettings`) and enforce explicit queue names at every producer/consumer boundary. Encode routing guarantees as tests so future changes cannot silently collapse domains.

**Tech Stack:** FastAPI backend, arq/Redis, pytest/pytest-asyncio, Docker Compose.

---

### Task 1: Add explicit queue-domain constants and wire train/email producers

**Files:**
- Create: `api/app/core/queue_domains.py`
- Modify: `api/app/modules/train/queue.py`
- Modify: `api/app/services/email_queue.py`
- Test: `api/tests/test_queue_domains.py`

**Step 1: Write failing tests for queue selection**

```python
async def test_train_queue_pool_uses_train_queue_name(...):
    ...

async def test_email_queue_pool_uses_train_queue_name(...):
    ...
```

**Step 2: Run tests to verify RED**

Run: `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_queue_domains.py -k "queue_pool_uses"`
Expected: FAIL because queue names are not yet explicitly wired.

**Step 3: Implement minimal queue-domain constants and wiring**

```python
TRAIN_QUEUE_NAME = "train:queue"
RESTAURANT_QUEUE_NAME = "restaurant:queue"
```

Use `create_pool(..., default_queue_name=TRAIN_QUEUE_NAME)` in train and email queue producers.

**Step 4: Run tests to verify GREEN**

Run: `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_queue_domains.py -k "queue_pool_uses"`
Expected: PASS.

**Step 5: Commit**

```bash
git add api/app/core/queue_domains.py api/app/modules/train/queue.py api/app/services/email_queue.py api/tests/test_queue_domains.py
git commit -m "test/infra: enforce explicit train queue domain for producers"
```

---

### Task 2: Make worker settings consume explicit queue domains

**Files:**
- Modify: `api/app/worker.py`
- Modify: `api/app/worker_restaurant.py`
- Test: `api/tests/test_queue_domains.py`

**Step 1: Write failing tests for worker queue contracts**

```python
def test_worker_train_settings_uses_train_queue():
    ...

def test_worker_restaurant_settings_uses_restaurant_queue():
    ...
```

**Step 2: Run tests to verify RED**

Run: `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_queue_domains.py -k "worker_.*queue"`
Expected: FAIL until `queue_name` is explicit in both worker settings.

**Step 3: Implement minimal worker queue-name wiring**

- Set `queue_name = TRAIN_QUEUE_NAME` in `WorkerSettings` (`api/app/worker.py`).
- Set `queue_name = RESTAURANT_QUEUE_NAME` in `WorkerRestaurantSettings` (`api/app/worker_restaurant.py`).

**Step 4: Run tests to verify GREEN**

Run: `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_queue_domains.py -k "worker_.*queue"`
Expected: PASS.

**Step 5: Commit**

```bash
git add api/app/worker.py api/app/worker_restaurant.py api/tests/test_queue_domains.py
git commit -m "test/worker: enforce explicit worker queue domains"
```

---

### Task 3: Add restaurant enqueue helper + cross-domain routing guard tests

**Files:**
- Create: `api/app/modules/restaurant/queue.py`
- Modify: `api/app/modules/restaurant/worker.py`
- Test: `api/tests/test_queue_domains.py`

**Step 1: Write failing routing guard tests**

```python
async def test_enqueue_restaurant_task_uses_restaurant_domain(...):
    ...

def test_train_worker_function_set_excludes_restaurant_runner():
    ...

def test_restaurant_worker_function_set_excludes_train_runner():
    ...
```

**Step 2: Run tests to verify RED**

Run: `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_queue_domains.py -k "restaurant_domain or function_set"`
Expected: FAIL because restaurant producer helper/guards do not exist yet.

**Step 3: Implement minimal restaurant enqueue helper and guards**

- Add `enqueue_restaurant_task(task_id: str, defer_seconds: float = 0.0)` using job id prefix `restaurant:{task_id}`.
- Keep `run_restaurant_task` as placeholder but scoped to restaurant queue only.

**Step 4: Run tests to verify GREEN**

Run: `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_queue_domains.py -k "restaurant_domain or function_set"`
Expected: PASS.

**Step 5: Commit**

```bash
git add api/app/modules/restaurant/queue.py api/app/modules/restaurant/worker.py api/tests/test_queue_domains.py
git commit -m "test/restaurant: add queue producer and routing guard coverage"
```

---

### Task 4: Documentation and full verification

**Files:**
- Modify: `docs/ARCHITECTURE.md`
- Modify: `README.md`

**Step 1: Update docs for queue-domain contracts**
- Document queue names and worker ownership (`train:queue`, `restaurant:queue`).
- Document that email delivery jobs are consumed by train worker domain.

**Step 2: Run focused backend verification**

Run: `docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_queue_domains.py`
Expected: PASS.

**Step 3: Run baseline backend verification**

Run: `docker compose -f infra/docker-compose.yml run --rm api pytest -q`
Expected: PASS.

**Step 4: Run docs verifiers**

Run:
- `bash infra/tests/test_docs_pointers.sh`
- `bash infra/tests/test_changelog.sh`
- `bash infra/tests/test_intent_routing.sh`
- `bash infra/tests/test_docs_consistency.sh`

Expected: all PASS.

**Step 5: Commit**

```bash
git add docs/ARCHITECTURE.md README.md
git commit -m "docs: record queue-domain isolation contracts"
```

## Acceptance Criteria

- Train, email, and restaurant producers enqueue into explicit intended queue domains.
- Worker settings declare explicit queue names.
- Routing guard tests fail when cross-domain function/queue leakage is introduced.
- Architecture docs match runtime queue-domain behavior.

## Assumptions and Defaults

- Queue domain names are `train:queue` and `restaurant:queue`.
- Email jobs remain in train worker domain unless a dedicated notifications worker is introduced in a future stage.
