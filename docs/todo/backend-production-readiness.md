# Backend TODO (Production Readiness)

Status: Archived and closed on 2026-02-14.  
Closure reference: `docs/plans/archive/2026-02-14-program-closure-report.md`

The implementation checklist below is retained as historical execution evidence.

Date: 2026-02-11  
Scope: `api/` backend quality findings to address before restaurant provider standardization work proceeds.

## Priority 0 (High)

### 1) Harden worker shutdown cancellation handling

Issue:
- `app/worker.py` cancels `heartbeat_task` during shutdown and suppresses `Exception`, but cancellation handling can still bypass recovery path depending on runtime behavior.

Actions:
- Update shutdown cancellation handling so `CancelledError` does not prevent `_recover_in_flight_tasks()` from running.
- Ensure shutdown always reaches final recovery + completion logs.

Files:
- `app/worker.py`
- `tests/test_worker_heartbeat.py` (extend)
- New test file if needed: `tests/test_worker_shutdown_recovery.py`

Acceptance:
- New test proves shutdown with cancelled heartbeat still executes in-flight recovery path.
- Existing heartbeat test remains green.

Verification:
- `docker compose -f /Users/jasonlee/bominal/infra/docker-compose.yml run --rm api pytest -q tests/test_worker_heartbeat.py`
- `docker compose -f /Users/jasonlee/bominal/infra/docker-compose.yml run --rm api pytest -q tests/test_worker_shutdown_recovery.py`

## Priority 1 (Medium)

### 2) Convert auth uniqueness races to deterministic API responses

Issue:
- Register/account update do pre-checks for duplicate email/display_name, but commit-time DB uniqueness races can still surface as 500.

Actions:
- Catch DB `IntegrityError` around commit boundaries in auth route flows.
- Map uniqueness collisions to stable 400/409 responses with safe detail messages.
- Keep existing pre-checks for UX; rely on DB constraint as final guard.

Files:
- `app/http/routes/auth.py`
- `tests/test_auth_flow.py`

Acceptance:
- Concurrent duplicate-write scenarios return stable conflict responses (not 500).

Verification:
- `docker compose -f /Users/jasonlee/bominal/infra/docker-compose.yml run --rm api pytest -q tests/test_auth_flow.py`

### 3) Make auth rate-limit IP source proxy-aware

Issue:
- Rate limiter keys only on `request.client.host`, inconsistent with existing proxy-aware IP extraction used elsewhere.

Actions:
- Add consistent client IP extraction for auth rate limit keys (support `x-forwarded-for` / `cf-connecting-ip` policy).
- Preserve safe behavior when headers are absent/spoofed in local/dev.

Files:
- `app/http/deps.py`
- `tests/test_api_access_control.py` (and/or new rate-limit-focused tests)

Acceptance:
- Rate-limiting behavior is deterministic across direct and proxied deployments.

Verification:
- `docker compose -f /Users/jasonlee/bominal/infra/docker-compose.yml run --rm api pytest -q tests/test_api_access_control.py`

## Priority 2 (Coverage / Confidence)

### 4) Add worker lifecycle regression tests

Issue:
- Current tests verify heartbeat key writes but do not cover shutdown recovery invariants.

Actions:
- Add tests for:
  - shutdown with in-flight task(s) triggers requeue path
  - hidden/cancelled/paused tasks are not requeued during recovery
  - heartbeat cancellation does not short-circuit shutdown sequence

Files:
- `tests/test_worker_shutdown_recovery.py` (new)
- `tests/test_worker_heartbeat.py` (extend as needed)

Acceptance:
- Worker lifecycle invariants are encoded and pass reliably.

Verification:
- `docker compose -f /Users/jasonlee/bominal/infra/docker-compose.yml run --rm api pytest -q tests/test_worker_heartbeat.py tests/test_worker_shutdown_recovery.py`

## Execution Notes

- Keep diffs minimal and surgical.
- Use TDD for each item (failing test first, then implementation).
- Do not mix restaurant feature implementation with these hardening fixes in the same commit.
- After each item, run targeted tests and then full backend suite.

Full-suite verification:
- `docker compose -f /Users/jasonlee/bominal/infra/docker-compose.yml run --rm api pytest -q`
