# Cross-Scope Request Queue

Use this file for any change that requires editing outside your current ACTIVE lock scope.

Rules:
- requester opens `OPEN` request with exact commands
- owner executes and marks `DONE` with commit SHA
- requester verifies and marks `CLOSED`
- each session checks this file every 10 minutes and before commit

Template:

## REQ-001
- status: OPEN
- caller: Session A
- owner: Session B
- created_at_utc: <YYYY-MM-DDTHH:MM:SSZ>
- reason: <why this cross-scope edit is required>
- files:
  - `infra/env/prod/api.env.example`
- exact_commands:
  - `<command 1>`
  - `<command 2>`
- expected_output:
  - <expected result>
- done_at_utc:
- done_commit_sha:
- completion_notes:
