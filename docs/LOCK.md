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
