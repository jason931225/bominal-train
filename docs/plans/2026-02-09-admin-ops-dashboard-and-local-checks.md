TRACK how much of this has been implemented so far

Plan: Admin Ops Dashboard + Local Smoothness Checks
Goals (Success Criteria)
Admin can quickly answer “why are tasks queued/stuck” from the UI: worker alive, Redis reachable, queue depth, stale tasks, recent failures.
Local dev has a single command that brings up the stack, waits for health, runs backend tests + web typecheck, and fails fast with actionable logs.
Backend Changes (API + Worker)

1. Make train task enqueues idempotent (prevents duplicate recovery enqueues)
   Modify

queue.py
Change

When calling enqueue_job("run_train_task", task_id, ...), pass a deterministic \_job_id:
\_job_id = f"train:{task_id}"
Why

enqueue_recoverable_tasks() currently re-enqueues all active tasks on worker startup. Without deterministic job IDs, worker restarts can create duplicate jobs for the same task.
Add tests

Create test_train_queue.py
Stub get_queue_pool() to a fake pool and assert enqueue_job() receives \_job_id="train:<task_id>" and preserves \_defer_by behavior. 2) Add worker heartbeat in Redis (for “is worker alive?”)
Modify

worker.py
Change

In on_startup(ctx), start a background asyncio task that updates:
Key: bominal:worker:heartbeat
Value: ISO timestamp (UTC)
TTL: 30s
Update interval: 5s or 10s
In on_shutdown(ctx), cancel the heartbeat task cleanly.
Add tests (optional but recommended)

test_worker_heartbeat.py using fakeredis and calling the heartbeat loop function directly. 3) Add admin “ops” endpoints
Modify

admin.py
Add endpoints (all admin-protected by existing router dependency)

GET /api/admin/ops/status
Returns:
Redis ping ok/error
arq:queue zcard, arq:in-progress zcard
Worker heartbeat status + last heartbeat timestamp (or “offline”)
Train active task count and stale task count (e.g. updated_at older than 10m)
GET /api/admin/ops/train/stale-tasks?limit=20
Returns list of stale tasks with safe fields only:
task_id, state, updated_at, created_at, deadline_at, user_id or user_email, last_attempt_at, last_error_code
GET /api/admin/ops/train/recent-failures?hours=24&limit=50
Returns recent failed TaskAttempts:
task_id, action, provider, error_code, error_message_safe, started_at
POST /api/admin/ops/train/recover
Runs enqueue_recoverable_tasks(db) and returns enqueued_count (and if we extend the worker helper to return details, also stale_reset_count, skipped_paused_count)
POST /api/admin/ops/train/tasks/{task_id}/requeue
Validates task is not PAUSED, not terminal, not hidden, not cancelled
Sets state to QUEUED (if needed)
Calls enqueue_train_task(task_id) (now idempotent via \_job_id)
Security

Never return spec_json, provider HTTP traces, secrets, or raw payloads. Only error_message_safe / meta_json_safe style fields.
Add tests

test_admin_ops.py
Non-admin gets 403
Admin gets 200 + expected keys
Requeue endpoint rejects terminal/paused/hidden tasks
Web Changes (Admin UI)

1. Fix admin fetch base URL for local dev
   Right now admin components fetch ..." directly, which works in prod behind Caddy, but breaks in local dev (API is http://localhost:8000).

Modify

system-stats-card.tsx
user-management.tsx
Change

Use clientApiBaseUrl like the train module:
fetch(\${clientApiBaseUrl}/api/admin/stats`, { credentials: "include" })` 2) Add ops section to Maintenance Dashboard
Create

ops-status-card.tsx
stale-tasks-table.tsx
recent-failures-table.tsx
Modify

page.tsx
UI behavior

Show:
Worker: Online/Offline (heartbeat TTL-based)
Redis: OK/Degraded + queue counts
Train: stale task count
Provide buttons:
Refresh
Recover tasks (POST) with confirmation dialog
Per-task Requeue (POST) for stale tasks
Local “Smoothness” Process

1. Make docker compose usage consistent in scripts
   Modify

local-setup.sh
local-run.sh
Change

Detect compose like deploy-zero-downtime.sh does:
Prefer docker compose, fallback to docker-compose
Fix the prereq check (current command -v "docker compose" is not reliable). 2) Add one command to verify local dev is healthy
Create

local-check.sh
What it does

Starts dev stack in detached mode (docker-compose.yml)
Waits for:
http://localhost:8000/health
http://localhost:3000
Runs:
docker compose exec -T api pytest -q
docker compose exec -T web npx tsc --noEmit
On failure, prints docker compose logs --tail=200 api worker web
Optional flag --down to stop stack after checks
Definition Of Done / Verification
docker-compose.yml up --build starts cleanly.
local-check.sh passes on a fresh machine with Docker.
Admin maintenance page works in local dev and shows ops status.
Worker heartbeat shows “online” while worker container is running.
Recover/requeue endpoints are admin-only and do not leak sensitive data.
One Decision Needed
Do you want the admin ops UI to include the recovery actions (Recover + Requeue) or keep it view-only at first?

Using skills brainstorming (feature/design clarity) + writing-plans (decision-complete implementation plan).

Remaining feature (recommended): Train task UX/observability + local smoothness checks

Make it obvious what a task is doing, when it will run next, and why it’s failing, without needing to dig into logs.
Add a repeatable “is my local stack healthy?” process (script + checklist) that catches worker/queue/API/web issues early.
Scope Options
Small: UI-only polish (better labels/chips/empty states), plus a local checklist doc.
Medium (Recommended): Add safe task “last attempt summary” + “next run time” to API responses, update Train UI to surface it, plus a local-check script.
Large: Real-time updates (SSE/WebSocket), deeper admin ops dashboard, and worker heartbeat/metrics.
This plan is for Option 2.

Backend Plan (API/worker)

1. Add “last attempt summary” to task summaries
   Goal: show “last action + error” in Train task lists without loading full detail.

Changes

Update schemas.py TaskSummaryOut to include:
last_attempt_action: str | None
last_attempt_ok: bool | None
last_attempt_error_code: str | None
last_attempt_error_message_safe: str | None
last_attempt_finished_at: datetime | None
Update service.py:
Add a helper \_latest_attempt_map(task_ids) -> {task_id: TaskAttempt} (or a small struct) using a Postgres-friendly “latest per task” query strategy.
Extend task_to_summary(...) to populate the new fields.
Ensure list_tasks(...), create_task(...), and any other code returning TaskSummaryOut continues to work.
Tests

Update/add assertions in test_train_tasks.py for the new fields to be present (and correct for tasks with/without attempts). 2) Persist + expose “next run at” for POLLING
Goal: when a task is POLLING, show when it will check again.

Changes

Update worker.py:
In \_schedule_retry(...), write spec_json["next_run_at"] = <utc iso> before commit.
When transitioning into a non-polling active state (e.g. RUNNING, RESERVING, PAYING) or terminal state, clear next_run_at to avoid stale UI.
Update service.py task_to_summary(...) to parse next_run_at from spec_json into a new TaskSummaryOut.next_run_at: datetime | None.
Update schemas.py accordingly.
Tests

Add/adjust a worker test (in test_train_tasks.py) that triggers \_schedule_retry behavior and verifies next_run_at is set on the task summary. 3) Optional but valuable: “Retry now” endpoint (safe requeue)
Goal: user can nudge a task that’s QUEUED/POLLING without waiting.

Changes

Add POST /api/train/tasks/{task_id}/retry in router.py.
Implement retry_task(...) in service.py:
Allowed only when: state in {"QUEUED","POLLING"}, not hidden, not cancelled, not paused, not terminal.
Set state to QUEUED, clear next_run_at, commit, call enqueue_train_task(task_id).
Tests

New tests in test_train_tasks.py:
retry works for QUEUED and POLLING
retry rejected for RUNNING/RESERVING/PAYING/PAUSED and terminal states
enqueue function called (monkeypatch)
Frontend Plan (Train UI) 4) Refactor Train dashboard into components (reduce 1600-line file)
Goal: safer iteration and fewer regressions.

Changes

Split train-dashboard.tsx into:
task-state-chip.tsx (color/label mapping)
train-task-card.tsx (active/completed card UI)
train-task-lists.tsx (two lists + empty states)
(optional) train-search-panel.tsx, train-credentials-panel.tsx
Keep orchestration + data fetch in TrainDashboard, but push rendering to components. 5) Surface progress + failure reason
Changes

Update types.ts TrainTaskSummary with:
last*attempt*\* fields
next_run_at?: string | null
In active task cards:
Show a state chip with human text (“Queued”, “Polling”, etc.)
If last_attempt_ok === false, show a 1-line “Last error: …” (use error_message_safe).
If state === "POLLING" and next_run_at, show “Next check: <KST time>”.
If we add /retry, show a “Retry now” button only for QUEUED/POLLING.
“Smooth Locally” Process 6) Fix + standardize local scripts for Compose v1/v2
Why: current scripts check for compose v2 but still call docker-compose directly.

Changes

Update local-setup.sh and local-run.sh to:
detect compose command (docker compose vs docker-compose)
use the detected command consistently
Fix main.py /health Redis check (it imports a non-existent function today); use redis.py instead so local checks can trust /health. 7) Add a single local “doctor” command
Create local-check.sh:

Bring up dev stack (docker-compose.yml) in the background (or verify running).
Wait for:
http://localhost:8000/health and assert JSON has db=true and redis=true
http://localhost:3000 returns 200
Run:
docker compose exec -T api pytest -q (or a targeted subset for speed)
docker compose exec -T web npx tsc --noEmit
Print next-step hints on failure (show which service/log to inspect). 8) Document it
Update README.md and CONTRIBUTING.md to include:
local-check.sh as the recommended “is it healthy?” step.
Acceptance Criteria
Train active tasks list shows state chip + last error (when applicable).
POLLING tasks show a “next check” time.
Local local-check.sh provides a single pass/fail signal and catches missing dependencies, broken health, failing tests, or TS type errors.
One Decision Needed
Do you want the user-facing “Retry now” button + /api/train/tasks/{id}/retry included in this feature, or should retry remain an admin/ops-only action?

user-facing. rigorously set conditions for when retry now button is available. it should be separate from pause/resume cycle.

continue

<proposed_plan>

Train “Retry Now” + Local Smoothness Checks
For Claude: REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

Goal: Add a user-facing “Retry now” action for Train tasks with strict eligibility (not tied to pause/resume), plus a repeatable local “everything is healthy” check.

Architecture: Add a new authenticated API endpoint to re-queue eligible tasks, make queueing idempotent per task via deterministic arq job IDs, and expose safe scheduling/attempt summary fields so the UI can explain what’s happening. Add a single local checker script that starts the dev stack, verifies health, and runs tests/typecheck.

Tech Stack: FastAPI + SQLAlchemy (SQLite in tests), arq + Redis, Next.js App Router + TS + Tailwind.

Public Interfaces (API/UI)
API additions/changes
New endpoint: POST /api/train/tasks/{task_id}/retry
Task summary fields: extend TaskSummaryOut (schemas.py) with:
next_run_at: datetime | None (when POLLING will run next, UTC)
last_attempt_action: str | None
last_attempt_ok: bool | None
last_attempt_error_code: str | None
last_attempt_error_message_safe: str | None
last_attempt_finished_at: datetime | None
retry_now_allowed: bool
retry_now_reason: str | None (why disabled)
retry_now_available_at: datetime | None (cooldown expiry)
Frontend typing updates
Update types.ts TrainTaskSummary to match the new fields.
Strict “Retry Now” Eligibility (Backend Enforced)
Retry is allowed only if all are true:

Task exists, belongs to the requesting user, module == "train".
hidden_at is NULL
cancelled_at is NULL
paused_at is NULL and state != "PAUSED" (explicitly separate from pause/resume)
state in {"QUEUED","POLLING"}
current time < deadline_at (if past deadline, mark EXPIRED and reject retry)
manual retry cooldown has elapsed (default 30s since last manual retry)
Cooldown storage: spec_json["manual_retry_last_at"] (UTC ISO string).

HTTP errors:

404 not found / not owned / hidden
409 not eligible state (RUNNING/RESERVING/PAYING/PAUSED/terminal)
410 deadline passed (also mark expired)
429 cooldown active
Queue Safety (Prevent Duplicate Jobs)
Make train task queueing idempotent per task by using a deterministic arq job id:

job id: train:{task_id}
This ensures:

At most one queued job per task (reschedules by updating the zset score)
“Retry now” can safely override a deferred POLLING schedule without creating duplicates
Implement in queue.py:

enqueue_train_task(task_id, defer_seconds) calls:
pool.enqueue_job("run_train_task", task_id, \_job_id=f"train:{task_id}", \_defer_by=defer_seconds)
Implementation Tasks (TDD)
Task 1: Queue idempotency per task
Files

Modify: queue.py
Test: test_train_queue.py (new)
Test (failing first)

Monkeypatch get_queue_pool() to return a fake object with enqueue_job.
Assert \_job_id == f"train:{task_id}" and \_defer_by is passed when defer_seconds > 0.
Implement

Update enqueue_train_task accordingly.
Verify

Run: test_train_queue.py
Expected: PASS
Task 2: Add retry endpoint + service function with strict conditions
Files

Modify: router.py
Modify: service.py
Modify: schemas.py
Test: test_train_tasks.py
Test (failing first)
Add tests for:

Allowed: QUEUED and POLLING tasks -> returns 200, task state becomes QUEUED, and enqueue called (monkeypatch enqueue_train_task)
Not allowed: PAUSED, RUNNING, RESERVING, PAYING, COMPLETED/FAILED/CANCELLED/EXPIRED -> 409
Deadline passed -> 410 and state becomes EXPIRED
Cooldown active -> 429
Implement

Add retry_task_now(db, task_id, user):
Load task, validate ownership + constraints above.
Clear spec_json["next_run_at"] if present.
Set spec_json["manual_retry_last_at"]=utc_now().isoformat()
Set state QUEUED, updated_at=utc_now(), commit.
await enqueue_train_task(str(task.id))
Return TaskActionResponse(task=task_to_summary(...))
Add route POST /tasks/{task_id}/retry.
Verify

Run: test_train_tasks.py
Expected: PASS
Task 3: Persist + expose next_run_at
Files

Modify: worker.py
Modify: service.py
Modify: schemas.py
Test: test_train_tasks.py
Test (failing first)

Exercise the path where \_schedule_retry is called (existing tests already cover polling/retry behavior; extend to assert task.spec_json["next_run_at"] is set and summary exposes it).
Implement

In \_schedule_retry(...):
compute next_run_at = utc_now() + timedelta(seconds=delay_seconds)
set spec_json["next_run_at"] = next_run_at.isoformat()
Clear next_run_at when transitioning into RUNNING/RESERVING/PAYING and when entering terminal states (ensure it’s not stale).
In task_to_summary, parse next_run_at into a datetime | None.
Task 4: Add “last attempt summary” to TaskSummaryOut
Files

Modify: service.py
Modify: schemas.py
Test: test_train_tasks.py
Test (failing first)

Create a task + attempts in DB (some tests already assert attempts exist); extend to assert the summary includes the latest attempt fields and that they match the newest attempt.
Implement

Add \_latest_attempt_map(db, task_ids) -> dict[UUID, TaskAttempt] using SQLite-friendly query:
select attempts where task_id IN (...)
order by task_id, finished_at desc
take first per task in Python
Populate the new fields in task_to_summary.
Task 5: Compute retry availability fields in summaries
Files

Modify: service.py
Modify: schemas.py
Test: test_train_tasks.py
Test (failing first)

For tasks in different states, assert retry_now_allowed/reason/available_at are correct.
Assert PAUSED never allows retry (even if state mismatched).
Implement

Central helper compute_retry_now_status(task, now) -> (allowed, reason, available_at) used by:
list responses
task detail response
action responses
Task 6: Frontend “Retry now” button (separate from pause/resume)
Files

Modify: types.ts
Modify: train-dashboard.tsx
Modify: train-task-detail.tsx
UI rules

Show “Retry now” only when task.retry_now_allowed === true
If retry_now_allowed === false but task is QUEUED/POLLING, show disabled button with tooltip text from retry_now_reason (optional, but recommended for clarity)
Never show retry button for PAUSED; PAUSED uses Resume only (separate cycles)
Behavior

POST /api/train/tasks/{id}/retry
On success: refresh tasks list / detail
On error: show returned detail in the existing error banner (do not log sensitive data)
Verify

Run: docker-compose.yml exec -T web npx tsc --noEmit
Manual: create task, observe button enable/disable + state updates
Local “Smoothness” Process
Task 7: Fix API /health Redis check (so local checks are trustworthy)
Files

Modify: main.py
Change

Replace the broken import (get_redis_pool from train queue) with:
from app.core.redis import get_redis_client
redis = await get_redis_client(); await redis.ping()
Verify

curl -sS http://localhost:8000/health returns redis: true when dev stack is up.
Task 8: Add a single local checker script
Files

Create: local-check.sh
Modify: local-setup.sh
Modify: local-run.sh
(Optional) Modify: README.md, CONTRIBUTING.md
local-check.sh responsibilities

Detect compose command (docker compose vs docker-compose)
Start dev stack in background: up -d --build
Wait for:
http://localhost:8000/health with db=true and redis=true
http://localhost:3000 returns 200/302
Run:
pytest -q (API container)
npx tsc --noEmit (web container)
Print actionable failure hints (which container logs to tail)
Acceptance Tests / Scenarios
Create a Train task; it queues and transitions states normally.
If task is POLLING with next_run_at in the future, “Retry now” is available and triggers immediate queue reschedule.
Retry is rejected for PAUSED and all terminal/processing states.
Spam-click retry is blocked by cooldown (429), and UI communicates why/when it becomes available.
local-check.sh provides a clean pass/fail signal on a fresh machine with Docker installed.
Assumptions / Defaults
Manual retry cooldown: 30 seconds (can be moved to config later).
Retry does not create a new task; it only re-queues the existing task.
“Retry now” never unpauses tasks; users must use Resume.
</proposed_plan>

Using skills brainstorming (lock spec) + writing-plans (decision-complete implementation plan).

Train “Retry Now” + Local Smoothness Checks (Plan)
Product Spec: “Retry Now”
User-facing behavior

“Retry now” is a separate action from Pause/Resume.
It never unpauses a task. If a task is paused, user must press Resume (existing behavior).
It forces an immediate worker run for an eligible task and updates UI quickly.
Eligibility (backend is source of truth)
Retry is allowed only if all are true:

Task belongs to current user and module == "train".
hidden_at is null.
Not cancelled: cancelled_at is null and state != "CANCELLED".
Not paused: paused_at is null and state != "PAUSED".
Not terminal: state not in {"COMPLETED","EXPIRED","FAILED","CANCELLED"}.
Not actively processing: state not in {"RUNNING","RESERVING","PAYING"}.
State is one of: QUEUED or POLLING.
Not expired by deadline: utc_now() < deadline_at.
Cooldown: last manual retry (stored in spec_json.manual_retry_requested_at) must be older than TRAIN_MANUAL_RETRY_COOLDOWN_SECONDS (default: 15s).
On success

Set state="QUEUED".
Clear spec_json.next_run_at (if present).
Set spec_json.manual_retry_requested_at=<now UTC iso>.
Enqueue the job immediately.
On rejection

Return a clear safe error message (detail) so the UI can show why it’s disabled.
API/Backend Work
Task 1: Add /api/train/tasks/{task_id}/retry
Files

Modify: router.py
Modify: service.py
Test: test_train_tasks.py
TDD steps

Add failing tests covering:
Allowed for QUEUED and POLLING
Reject PAUSED with message “Resume instead”
Reject RUNNING/RESERVING/PAYING with 409
Reject terminal states with 400
Reject expired deadline (and ensure state becomes EXPIRED if we choose to auto-mark)
Cooldown: first retry ok, immediate second retry returns 429 and includes seconds remaining
Implement retry_task(...):
Reuse get_task_for_user(...) to enforce ownership
Enforce the eligibility rules exactly
Apply state/spec updates, commit, then enqueue_train_task(str(task.id))
Wire router endpoint to retry_task.
Run tests.
Commit.
Task 2: Make the queue job stable per task (prevents duplicate runs)
Files

Modify: queue.py
Test: new test_train_queue.py
Change

enqueue_train_task(task_id, defer_seconds) should call:
await pool.enqueue_job("run_train_task", task_id, \_job_id=f"train:{task_id}", \_defer_by=...)
Tests

Monkeypatch pool and assert \_job_id is passed and \_defer_by is respected.
Task 3: Store and expose next_run_at (for POLLING UX)
Files

Modify: worker.py
Modify: schemas.py
Modify: service.py
Test: test_train_tasks.py
Change

In \_schedule_retry(...):
Compute next_run_at = utc_now() + timedelta(seconds=delay_seconds) (UTC)
Write it into task.spec_json["next_run_at"] (ISO string), then commit, then enqueue with defer
When moving into non-polling states or terminal states, clear next_run_at to avoid stale UI.
Task 4: Add “last attempt summary” to task list responses
Files

Modify: schemas.py
Modify: service.py
Test: test_train_tasks.py
Change

Extend TaskSummaryOut with safe fields:
last_attempt_action, last_attempt_ok, last_attempt_error_code, last_attempt_error_message_safe, last_attempt_finished_at
Implement a \_latest_attempt_map(...) helper and populate these fields in list_tasks(...) and other endpoints returning summaries.
Task 5: Fix API /health Redis check (unblocks local smooth checks)
File

Modify: main.py
Change

Replace the broken import with:
from app.core.redis import get_redis_client
redis = await get_redis_client(); await redis.ping()
Web/UI Work
Task 6: Add Retry Now button (separate from Pause/Resume)
Files

Modify: train-dashboard.tsx
Modify: train-task-detail.tsx
Modify: types.ts
UI rules

Button only renders for tasks in QUEUED or POLLING.
Button is disabled (with tooltip text) if:
task is paused
deadline passed
cooldown not elapsed
Button does not change pause state and does not replace Pause/Resume controls.
Endpoint

Call POST ${clientApiBaseUrl}/api/train/tasks/${taskId}/retry with credentials: "include".
Task 7: Show “Next check” and “Last error” in task cards
Files

Modify: train-dashboard.tsx
Modify: types.ts
Change

For POLLING tasks with next_run_at, show Next check: <KST>.
If last_attempt_ok === false, show Last error: <error_message_safe> (single line).
Local “Smoothness” Process
Task 8: Standardize Compose usage in local scripts
Files

Modify: local-setup.sh
Modify: local-run.sh
Change

Detect compose command like deploy script does and use it consistently.
Task 9: Add local-check command
Files

Create: local-check.sh
Modify: README.md and CONTRIBUTING.md (briefly)
What it runs

Start dev stack (docker-compose.yml) detached.
Wait for http://localhost:8000/health and assert db=true, redis=true.
Wait for http://localhost:3000.
Run:
docker compose exec -T api pytest -q
docker compose exec -T web npx tsc --noEmit
On failure, print docker compose logs --tail=200 api worker web.
Assumptions / Defaults
TRAIN_MANUAL_RETRY_COOLDOWN_SECONDS = 15.
Retry now is only for QUEUED and POLLING (no “force retry” for RUNNING/RESERVING/PAYING; those remain worker-recovery concerns).
Any attempt/error strings surfaced are from existing error_message_safe fields only.
