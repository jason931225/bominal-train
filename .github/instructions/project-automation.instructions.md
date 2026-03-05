---
applyTo: ".github/**,docs/**,AGENTS.md,CHANGELOG.md"
---

# GitHub Project Automation Instructions

Applies to repository project-governance, promotion, and PR lifecycle automation updates.

## Mandatory Board Model

Use canonical three-board policy:
- `bominal Workstreams`
- `bominal Review`
- `bominal Agent Command`

Reference:
- `docs/MANUAL.md#project-tracking`
- `docs/playbooks/GITHUB_PROJECT_AUTOMATION.md`

## Execution Rules

- Do not start implementation without a linked issue in `Ready` state.
- Use `Closes #...` on implementation PRs.
- Keep one `area:*` domain per implementation item and PR path-set.
- Follow command checkpoints in order: `Claimed` -> `Design Note Posted` -> `Draft PR Linked`.
- Respect area WIP cap (`1`) and conflict escalation (`Blocked` + rebase checklist).
- Enforce branch flow: implementation -> `dev`, promotion `dev -> staging -> main`, hotfix `hotfix/* -> main` then back-promote.
- `dev -> staging` promotion is gate-driven (`promotion-gate*` + `promotion-pr-open-dev-staging.yml`).
- `promotion:auto` applies to `staging -> main` PR auto-create only.

## Review Rules

- Apply `Review Depth=Secondary Required` for sensitive/risky scope.
- Request `@copilot review` first when policy requires secondary review.
- Request `@codex review` second for cross-check on the same PR.
- Keep Copilot usage judicious and under monthly budget (`300` requests, reset on the 1st UTC).
- Treat material Copilot findings as merge-blocking unless fixed or maintainer-waived with explicit risk note.

## Gate Commands

Gate commands are processed in `.github/workflows/promotion-gate-commands.yml`.

- `/gate refresh`
- `/gate promote`
- `/gate waive advisory <finding_id> reason:"..." risk:"..." expires:"YYYY-MM-DD" followup:"#123"`
- `/promote merge` (PR comments only)

## Commands And Tooling

- Use tested command set in `docs/playbooks/GITHUB_PROJECT_AUTOMATION.md`.
- Prefer GitHub MCP tools for automation (`issue_write`, `list_issues`, `pull_request_read`, `request_copilot_review`, `merge_pull_request`) and CLI fallback only when needed.
- If CLI project scope is missing, bootstrap with `GH_PAT_FULL` from `env/dev/test.env` before board commands.
