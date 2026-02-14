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

## Template (Non-live Example)

### LOCK-EXAMPLE-001
- status: EXAMPLE_ACTIVE
- owner_session: Session X
- scope:
  - `path/**`
- reason: <task/stage reason>
- created_at_utc: <YYYY-MM-DDTHH:MM:SSZ>
- released_at_utc:
