---
applyTo: ".github/**,docs/**,AGENTS.md,CHANGELOG.md"
---

# GitHub Project Automation Instructions

Applies to all repository work planning and PR lifecycle updates.

## Mandatory Board Model

Use the canonical three-board policy:
- `bominal Workstreams`
- `bominal Review`
- `bominal Agent Command`

Reference:
- `docs/MANUAL.md#project-tracking`
- `docs/playbooks/GITHUB_PROJECT_AUTOMATION.md`

## Execution Rules

- Do not start implementation without a linked issue in `Ready` state.
- Use `Closes #...` on PRs.
- Keep one `area:*` domain per implementation item and PR path-set.
- Follow command checkpoints in order: `Claimed` -> `Design Note Posted` -> `Draft PR Linked`.
- Respect area WIP cap (`1`) and conflict escalation (`Blocked` + rebase checklist).
- Enforce branch flow: implementation -> `dev`, promotion `dev -> staging -> main`, hotfix `hotfix/* -> main` then back-promote.
- `promotion:auto` is opt-in and enables automatic promotion PR creation only; merge still requires maintainer command.

## Review Rules

- Apply `Review Depth=Secondary Required` for sensitive/risky scope.
- Request `@copilot review` first when policy requires secondary review.
- Request `@codex review` second for cross-check on the same PR.
- Treat material Copilot findings as merge-blocking unless fixed or maintainer-waived with explicit risk note.

## Commands And Tooling

- Use tested commands in `docs/playbooks/GITHUB_PROJECT_AUTOMATION.md`.
- Prefer GitHub MCP tools for automation (`issue_write`, `list_issues`, `pull_request_read`, `request_copilot_review`, `merge_pull_request`) and CLI fallback only when needed.
- If CLI project scope is missing, bootstrap with `GH_PAT_FULL` from `env/dev/test.env` before running project-board commands.
