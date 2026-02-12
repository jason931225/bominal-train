# Bominal Grand Restructure Plan (Dynamic Locking, Two Sessions)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.
> **Required protocol skill:** `standardized-plan-execution-protocol`

**Goal:** Restructure bominal runtime and infra for production operation on e2-micro with split workers, partial restaurant exposure, payment-step lease safety, and conflict-safe two-session execution.

**Architecture:** Keep one API control plane and two execution workers (`worker-train`, `worker-restaurant`). Use dynamic lock scopes (`docs/LOCK.md`) for per-stage file ownership and `docs/REQUEST.md` for cross-scope changes. Enforce auth fallback order (`HTTP refresh retries -> Playwright bootstrap -> fail task`) and keep deployment aligned to current canonical script (`infra/scripts/deploy-zero-downtime.sh`). Track `fetch_ci.sh` + `deploy.prod.sh` as a future migration, not active runtime dependency.

**Tech Stack:** FastAPI, SQLAlchemy async ORM, Redis/arq, Docker Compose, Bash/Bats, pytest/pytest-asyncio.

---

## Required execution protocol

1. Both sessions MUST follow `standardized-plan-execution-protocol`.
2. `docs/LOCK.md` is mandatory for dynamic scope ownership.
3. `docs/REQUEST.md` is mandatory for cross-scope edits.
4. Subagent request processing is allowed, but only within request scope.
5. TDD + verification-before-completion are mandatory.

## Session model

- Session A: API/runtime-heavy tasks by lock scope.
- Session B: infra/deploy/docs-heavy tasks by lock scope.
- No static permanent ownership map; lock scopes are re-evaluated each stage.

## Stage sequence

### Stage 0: coordination bootstrap

- Create/initialize `docs/LOCK.md` and `docs/REQUEST.md` templates.
- Acquire initial non-overlapping `ACTIVE` lock scopes.
- Commit lock-only changes first.

### Stage 1: backend hardening baseline (must finish first)

Implement `docs/todo/backend-production-readiness.md` in this order:
1. worker shutdown recovery robustness
2. auth uniqueness race handling
3. proxy-aware rate-limit keying
4. worker lifecycle regression coverage

### Stage 2: worker split + queue contracts

- Add `worker-train` and `worker-restaurant` entrypoints.
- Ensure queue domains are isolated (`train:*`, `restaurant:*`).
- Add routing tests to prevent cross-domain consumption.

### Stage 3: restaurant partial exposure

- Add capability flags on module endpoint.
- Expose only implemented restaurant operations.
- Keep unimplemented operations explicit and safe.

### Stage 4: restaurant policy enforcement

- Enforce auth fallback: refresh retries, then Playwright bootstrap, then fail.
- Enforce payment-step lease key: `provider+account+restaurant_id`.
- Allow non-committing phase concurrency.

### Stage 5: infra overhaul

- Harden `infra/scripts/deploy-zero-downtime.sh` as the active production deploy path.
- Add running-container detection path.
- Add resource/swap preflight checks.
- Add deploy lock to prevent concurrent deploys.
- Add post-deploy smoke + rollback trigger path.
- Keep `fetch_ci.sh` + `deploy.prod.sh` as optional future migration (separate tracked work).

### Stage 6: safe deprecation cleanup

- Inventory deprecated files/features first.
- Add compatibility shim window.
- Migrate callers/docs.
- Remove deprecated files only after verification.

### Stage 7: docs canonization

- Make bominal docs canonical for execution protocol.
- Add protocol references in `AGENTS.md` and `docs/README.md`.
- Keep external/reference repo docs informational only.

## Verification gates

Per task:
- RED test observed and expected.
- GREEN test observed and expected.

Per batch:
- relevant domain suite passes.
- no conflicting new `ACTIVE` lock entries.
- no stale owned `OPEN` requests.

Pre-PR:
- full relevant test suites pass.
- infra script tests pass.
- rebase conflict-free.
- lock/request ledgers consistent.

## Required commands (minimum)

```bash
# Backend suite
docker compose -f infra/docker-compose.yml run --rm api pytest -q

# Infra/script suite
bash infra/tests/test_docs_pointers.sh
bash infra/tests/test_changelog.sh
bash infra/tests/test_env_utils.sh
bash infra/tests/test_predeploy_check.sh
bash infra/tests/test_intent_routing.sh
bash infra/tests/test_docs_consistency.sh
```

## Completion criteria

- All planned tasks complete with verification evidence.
- No unresolved `OPEN` requests.
- No lingering `ACTIVE` locks for completed scopes.
- No out-of-lock edits in final diff.
- Deployment/docs paths updated to current canonical script model.
