# Dynamic Lock Ledger

Use this file to claim dynamic edit scopes during multi-session execution.

Rules:
- Acquire minimal lock scope before writing code.
- Commit lock acquisition separately.
- Re-check this file before writing and before each commit.
- Release lock after stage completion.
- Commit lock release separately.
- Template/example entries must never use live statuses.

## Current Entries

### LOCK-2026-02-14-STAGE8
- status: RELEASED
- owner_session: Codex Session
- scope:
  - `docs/**`
  - `infra/tests/**`
  - `.github/workflows/infra-tests.yml`
  - `CHANGELOG.md`
- reason: Stage 8 program closure and archival hygiene implementation
- created_at_utc: 2026-02-14T20:09:47Z
- released_at_utc: 2026-02-14T20:17:48Z

### LOCK-2026-02-14-STAGE9
- status: RELEASED
- owner_session: Codex Session
- scope:
  - `api/**`
  - `web/**`
  - `infra/**`
  - `docs/**`
  - `CHANGELOG.md`
- reason: Stage 9 backend-first then frontend performance optimization implementation
- created_at_utc: 2026-02-14T20:23:23Z
- released_at_utc: 2026-02-14T20:54:52Z

### LOCK-2026-02-14-STAGE9B
- status: RELEASED
- owner_session: Codex Session
- scope:
  - `api/**`
  - `web/**`
  - `infra/**`
  - `docs/**`
  - `CHANGELOG.md`
- reason: Stage 9 follow-up implementation (migration verify, benchmarks, reset workflow, frontend verification, push/review)
- created_at_utc: 2026-02-14T21:03:19Z
- released_at_utc: 2026-02-15T03:43:25Z

### LOCK-2026-02-14-STAGE9C
- status: ACTIVE
- owner_session: Codex Session
- scope:
  - `.github/workflows/infra-tests.yml`
  - `docs/**`
  - `CHANGELOG.md`
- reason: Stage 9 CI follow-up to add new shell script validations to infra workflow
- created_at_utc: 2026-02-15T03:48:05Z
- released_at_utc:

## Template (Non-live Example)

### LOCK-EXAMPLE-001
- status: EXAMPLE_ACTIVE
- owner_session: Session X
- scope:
  - `path/**`
- reason: <task/stage reason>
- created_at_utc: <YYYY-MM-DDTHH:MM:SSZ>
- released_at_utc:
