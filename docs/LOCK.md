# Dynamic Lock Ledger

Use this file to claim dynamic edit scopes during multi-session execution.

Rules:
- Acquire minimal lock scope before writing code.
- Commit lock acquisition separately.
- Re-check this file before writing and before each commit.
- Release lock after stage completion.
- Commit lock release separately.

Template:

## LOCK-001
- status: ACTIVE
- owner_session: Session A
- scope:
  - `api/app/modules/restaurant/**`
- reason: <task/stage reason>
- created_at_utc: <YYYY-MM-DDTHH:MM:SSZ>
- released_at_utc:

## LOCK-2026-02-14-STAGE8
- status: ACTIVE
- owner_session: Codex Session
- scope:
  - `docs/**`
  - `infra/tests/**`
  - `.github/workflows/infra-tests.yml`
  - `CHANGELOG.md`
- reason: Stage 8 program closure and archival hygiene implementation
- created_at_utc: 2026-02-14T20:09:47Z
- released_at_utc:
