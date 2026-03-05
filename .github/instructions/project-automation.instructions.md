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

## Review Rules

- Apply `Review Depth=Secondary Required` for sensitive/risky scope.
- Request Copilot review when policy requires it.
- Treat material Copilot findings as merge-blocking unless fixed or maintainer-waived with explicit risk note.
