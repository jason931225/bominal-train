# GitHub Project Automation

This playbook operationalizes `docs/MANUAL.md#project-tracking` and the protected branch model (`dev -> staging -> main`).

## Board Topology

Maintain three active GitHub Project v2 boards:

1. `bominal Workstreams`
- Purpose: issue intake, prioritization, release-stage and promotion tracking.

2. `bominal Review`
- Purpose: PR review-depth, Copilot/Codex disposition, merge readiness.

3. `bominal Agent Command`
- Purpose: deterministic agent dispatch queue, checkpoint tracking, domain-lock and escalations.

## Template Adoption Matrix

Adopt board template ideas by partial composition:

- `Kanban` (adopted): status flow (`Triage -> Ready -> In Progress -> In Review -> Done`) is the backbone.
- `Iterative development` (adopted): queue checkpoints and incremental PR milestones.
- `Bug tracker` (partially adopted): severity/risk handling through `priority:*` + `risk:*` labels.
- `Feature release` (partially adopted): `Target Release`, `Env Stage`, `Promotion State` fields.
- `Product launch` (selective adoption): release-calendar/milestone alignment; no marketing pipeline coupling.

Priority scope contract:
- `priority:p0`: service-impacting, security-critical, or release-blocking.
- `priority:p1`: high-value near-term delivery.
- `priority:p2`: normal planned backlog.
- `priority:p3`: low urgency or exploratory maintenance.

## Required Field Model

### `bominal Workstreams`
- `Status`: `Triage`, `Ready`, `In Progress`, `In Review`, `Blocked`, `Done`
- `Type`: `Bug`, `Enhancement`, `Docs`, `Chore`, `Security`, `Ops`
- `Area`: label-aligned `area:*`
- `Priority`: `P0`, `P1`, `P2`, `P3`
- `Risk`: `Low`, `Medium`, `High`
- `Env Stage`: `Development`, `Staging`, `Production`
- `Promotion State`: `PR Open`, `Merged to dev`, `Merged to staging`, `Released`
- `Promotion Mode`: `Manual`, `Auto`
- `Target Release`
- `Due Date`
- `Linked PR`

### `bominal Review`
- `Review Status`: `Ready for Review`, `Changes Requested`, `Approved`, `Merged`
- `Review Depth`: `Standard`, `Secondary Required`
- `Copilot Required`: `Yes`, `No`
- `Copilot Material State`: `Clear`, `Material Open`, `Material Waived`
- `Linked Issue`

### `bominal Agent Command`
- `Queue Rank`
- `Claim State`: `Ready`, `Claimed`, `Design Note Posted`, `Draft PR Linked`, `Blocked`, `Escalated`
- `Checkpoint`: `Claim`, `Design Note`, `Draft PR`
- `Domain Lock`: `Pass`, `Fail`
- `Conflict State`: `None`, `Rebase Required`
- `Escalation State`: `None`, `Secondary Review`, `Policy Exception`

## Branch Promotion Lifecycle

Branch and promotion policy:
- implementation branches -> `dev`
- `dev` -> `staging`
- `staging` -> `main`
- `hotfix/*` -> `main` then back-promote (`main -> staging -> dev`)

Automation rules:
- issue label `promotion:auto` enables automatic PR creation after merge:
  - merged PR into `dev` opens `dev -> staging`
  - merged PR into `staging` opens `staging -> main`
- promotion PR merge remains maintainer-controlled via `/promote merge` command.
- merged hotfix PR to `main` opens `main -> staging` back-promotion PR.

## Review Sequencing

When secondary review is required:
1. Request `@copilot review`.
2. Resolve material findings or maintainers explicitly waive with risk note.
3. Request `@codex review` for cross-check.
4. Merge only after checks are green and conversations are resolved.

## Orchestrator Issue Authoring Contract

Orchestrator-posted issues MUST include:
- objective and user/operator outcome,
- exact `area:*` domain and allowed path-set,
- risk class, sensitive boundaries, rollback plan,
- in-scope/out-of-scope,
- acceptance criteria,
- verification command checklist,
- agent dispatch instructions (claim/design-note/draft-PR/review sequence).

Use `.github/ISSUE_TEMPLATE/orchestrator-task.yml` for this.

## Tested Agent Bootstrap Commands

GitHub CLI bootstrap (session start):
```bash
gh auth status
gh repo view jason931225/bominal --json name,defaultBranchRef
```

If the default CLI token is missing `read:project`, bootstrap with the local full-scope PAT:
```bash
set -a
source env/dev/test.env
set +a
export GH_TOKEN="$GH_PAT_FULL"
gh project list --owner @me
```

Create and queue a work item:
```bash
gh issue create --repo jason931225/bominal --title "chore: dummy project automation check" --body "Policy smoke test issue" --label type:chore --label area:ci-cd --label priority:p3 --label status:ready
gh project item-add "$BOMINAL_WORKSTREAMS_PROJECT_NUMBER" --owner "$BOMINAL_WORKSTREAMS_PROJECT_OWNER" --url "<ISSUE_URL>"
```

Promotion-review commands:
```bash
gh pr review <PR_NUMBER> --repo jason931225/bominal --comment --body "@copilot review"
gh pr review <PR_NUMBER> --repo jason931225/bominal --comment --body "@codex review"
gh pr comment <PR_NUMBER> --repo jason931225/bominal --body "/promote merge"
```

GitHub MCP equivalents (preferred in agent automation):
- queue/read issues: `list_issues`, `issue_read`, `issue_write`
- PR lifecycle: `list_pull_requests`, `pull_request_read`, `update_pull_request`, `merge_pull_request`
- review operations: `request_copilot_review`, `add_issue_comment`, `pull_request_review_write`
- repo governance checks: `get_me`, `list_branches`, `list_releases`

## Auth And Scope Requirements

Required scopes for CLI PAT fallback (if GitHub App/session auth is unavailable):
- `repo`
- `project`
- `read:project`
- `workflow`

`PROJECT_AUTOMATION_TOKEN` secret must support Project v2 GraphQL reads/writes for configured boards.

## Exception Handling

- Policy violation: move to `Escalated`, apply `status:blocked`, post corrective checklist.
- Emergency hotfix: temporary bypass requires explicit maintainer approval and post-merge incident note.
- Manual override always requires an auditable issue/PR comment trail.
