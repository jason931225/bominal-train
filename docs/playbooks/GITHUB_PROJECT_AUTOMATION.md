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

- `Kanban` (adopted): status flow (`Triage -> Ready -> In Progress -> In Review -> Blocked -> Done`) is the baseline.
- `Iterative development` (adopted): issue checkpoint flow + draft PR milestones.
- `Bug tracker` (partially adopted): severity/risk handling through `priority:*` + `risk:*` taxonomy.
- `Feature release` (partially adopted): `Target Release`, `Release Checkpoint`, `Promotion Flag` fields.
- `Product launch` (selective adoption): release calendar + milestone discipline only.

Priority scope contract:
- `priority:p0`: service-impacting, security-critical, or release-blocking.
- `priority:p1`: high-value near-term delivery.
- `priority:p2`: normal planned backlog.
- `priority:p3`: low urgency or exploratory maintenance.

PR label inheritance policy:
- PRs inherit missing `type:*`, `area:*`, `priority:*`, and `ci:tier:*` labels from the first linked issue (`Closes #...`) via `PR Governance` automation.
- `semver:*` is release metadata and should be applied only when the PR is tied to a production release/promotion target.
- Prefer one coherent PR with multiple focused commits; split into multiple PRs only for independent scopes, merge-order constraints, or risk isolation.

## Required Field Model

### `bominal Workstreams`
- `Status`: `Triage`, `Ready`, `In Progress`, `In Review`, `Blocked`, `Done`
- `Type`: `Bug`, `Enhancement`, `Docs`, `Chore`, `Security`, `Ops`
- `Area`: label-aligned `area:*`
- `Priority`: `P0`, `P1`, `P2`, `P3`
- `Risk`: `Low`, `Medium`, `High`
- `Release Checkpoint`: `Backlog`, `Ready for Staging Gate`, `Gate In Progress`, `Promotion PR Open`, `Promoted`
- `Promotion Flag`: `None`, `Promote`, `Hold`
- `Target Release`
- `Due Date`
- `Linked PR`
- `Gate Issue URL` (text)
- `Merge Order Source` (text)

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

## Promotion Automation Architecture

Promotion is implemented as workflow composition:

1. `promotion-gate-controller.yml`
- Watches Workstreams `Release Checkpoint=Ready for Staging Gate`.
- Creates/refreshes gate issue labeled `gate:open`.
- Assigns reviewer using `PROMOTION_GATE_REVIEWER_MAP_JSON` with fallback `PROMOTION_GATE_DEFAULT_REVIEWER`.

2. `promotion-gate.yml`
- Runs two required jobs on each gate issue:
  - `ci_gate`
  - `promotion_governance_gate`
- Writes pass/fail labels and per-rule bot comments.

3. `promotion-gate-review-loop.yml`
- Verifies review-loop evidence across bundle PRs.
- Requires evidence of `@copilot review` and `@codex review` comments.
- Blocks when `CHANGES_REQUESTED` remains unresolved.

4. `promotion-gate-commands.yml`
- Handles gate and promotion commands.
- `/gate refresh`: re-run gate checks for one issue.
- `/gate promote`: mark promotion intent and set `Promotion Flag=Promote` when project field access is available.
- `/gate waive advisory ...`: advisory-only waiver entry into gate `## Waiver Ledger`.
- `/promote merge`: safeguarded merge command for promotion/back-promotion PRs.

5. `promotion-pr-open-dev-staging.yml`
- Opens `dev -> staging` promotion PR only when gate labels show pass state:
  - `gate:ci-pass`
  - `gate:governance-pass`
  - `gate:review-round-complete`
  - `gate:promote`

6. `promotion-pr-create.yml`
- Handles `staging -> main` auto-create only for merged PRs explicitly labeled `promotion:auto`.

## Review Budget Policy

Copilot review usage controls:
- Monthly budget: `300` invocations.
- Reset boundary: first day of each month (UTC).
- CI tracking: `.github/workflows/ci.yml` job `Copilot Review Budget`.
- Hard gate: CI fails when monthly usage reaches/exceeds budget.
- Warning band: warn at `270` by default (`COPILOT_REVIEW_WARN_THRESHOLD`, override via repo variable).

Judicious-use rules:
- Prefer Copilot review for medium/high risk, secondary-required, or promotion-gate-critical changes.
- Avoid consuming budget on low-risk docs-only or trivial hygiene PRs.
- Default cross-check sequence remains `@copilot review` then `@codex review` when secondary review is warranted.

## Actions Minute Governance

Global Actions budget controls:
- Monthly global cap: `3000` minutes.
- Reserved CD pool: `300` minutes.
- Non-CD cap: `2700` minutes.
- Scope: all repository workflows count toward global usage.

Governance modes:
- `normal`: full non-CD policy checks run.
- `throttle`: only `ci:tier:heavy` (or hotfix/override) PRs run heavy checks; other PRs run cheap checks only.
- `lockdown`: non-hotfix non-CD workflows are blocked; CD reserve remains protected.
- Global lockdown (`>=3000`) blocks CD and release workflows.

Implementation surfaces:
- reusable evaluator: `.github/workflows/actions-budget-governor.yml`
- PR/full CI policy: `.github/workflows/ci.yml`
- push-minimal CI: `.github/workflows/ci-push-minimal.yml`
- command handler: `.github/workflows/actions-budget-commands.yml`
- daily report: `.github/workflows/actions-budget-report.yml`

Operator commands:
- `/budget status`
- `/budget override reason:"..."`

## Gate Manifest Contract

Every promotion gate issue contains a machine-readable manifest block:

```yaml
<!-- promotion-gate-manifest:start -->
bundle_id: bundle-<milestone>-<area>-<issue>
milestone: <milestone-title>
area: area:<domain>
risk: low|medium|high
ordered_prs:
  - 0
depends_on:
  - issue: 0
release_summary_required: true
<!-- promotion-gate-manifest:end -->
```

Required gate sections outside the manifest:
- `## Merge Order Notes`
- `## Risk Assessment`
- `## Release Summary`
- `## Round Review Log`
- `## Waiver Ledger`

## Command Processing And Waivers

Command processing location:
- Workflow: `.github/workflows/promotion-gate-commands.yml`
- Trigger: `issue_comment` (`created`, `edited`)

Command authorization:
- Requires repository permission `write|maintain|admin`.

Waiver policy:
- Only advisory findings can be waived by command.
- Format:
  ```text
  /gate waive advisory <finding_id> reason:"..." risk:"..." expires:"YYYY-MM-DD" followup:"#123"
  ```
- Result:
  - Adds label `gate:advisory-waived`.
  - Appends immutable ledger entry in gate issue `## Waiver Ledger`.

## Orchestrator Issue Contract

Orchestrator-created issues MUST include:
- objective and expected operator outcome,
- one `area:*` domain lock and explicit allowed path-set,
- risk class, sensitive boundaries, rollback and blast radius,
- in-scope/out-of-scope,
- acceptance criteria and exact verification commands,
- bundle manifest, dependency notes, and merge-order notes when sequence is required,
- review sequencing instructions (`@copilot review` then `@codex review`).

Use `.github/ISSUE_TEMPLATE/orchestrator-task.yml`.

## Tested Agent Bootstrap Commands

GitHub CLI bootstrap:
```bash
gh auth status
gh repo view jason931225/bominal --json name,defaultBranchRef
```

PAT bootstrap fallback (`env/dev/test.env`):
```bash
set -a
source env/dev/test.env
set +a
export GH_TOKEN="$GH_PAT_FULL"
gh auth status
gh project list --owner "$BOMINAL_WORKSTREAMS_PROJECT_OWNER"
```

Create + add issue to Workstreams:
```bash
ISSUE_URL=$(gh issue create --repo jason931225/bominal --title "chore: board automation smoke test" --body "Policy smoke test issue" --label type:chore --label area:ci-cd --label priority:p3 --label ci:tier:standard --label status:ready)
gh project item-add "$BOMINAL_WORKSTREAMS_PROJECT_NUMBER" --owner "$BOMINAL_WORKSTREAMS_PROJECT_OWNER" --url "$ISSUE_URL"
```

Promotion/gate commands:
```bash
gh issue comment <GATE_ISSUE_NUMBER> --repo jason931225/bominal --body "/gate refresh"
gh issue comment <GATE_ISSUE_NUMBER> --repo jason931225/bominal --body "/gate promote"
gh issue comment <GATE_ISSUE_NUMBER> --repo jason931225/bominal --body '/gate waive advisory finding-123 reason:"non-blocking style issue" risk:"low" expires:"2026-04-01" followup:"#999"'
gh pr comment <PR_NUMBER> --repo jason931225/bominal --body "/promote merge"
```

Budget commands:
```bash
gh issue comment <ISSUE_OR_PR_NUMBER> --repo jason931225/bominal --body "/budget status"
gh issue comment <ISSUE_OR_PR_NUMBER> --repo jason931225/bominal --body '/budget override reason:"urgent release validation required"'
```

Review requests (cross-check):
```bash
gh pr comment <PR_NUMBER> --repo jason931225/bominal --body "@copilot review"
gh pr comment <PR_NUMBER> --repo jason931225/bominal --body "@codex review"
```

GitHub MCP equivalents for agents:
- queue and issue lifecycle: `list_issues`, `issue_read`, `issue_write`, `add_issue_comment`
- PR lifecycle and validation: `list_pull_requests`, `pull_request_read`, `update_pull_request`, `merge_pull_request`
- review automation: `request_copilot_review`, `pull_request_review_write`, `add_comment_to_pending_review`
- governance checks: `list_branches`, `list_releases`, `get_me`

## Auth Scope Requirements

`PROJECT_AUTOMATION_TOKEN` secret must support Project v2 GraphQL reads/writes.

Optional review-budget variables:
- `COPILOT_REVIEW_MONTHLY_BUDGET` (default `300`)
- `COPILOT_REVIEW_WARN_THRESHOLD` (default `270`)

Optional Actions-minute governance variables:
- `ACTIONS_MINUTES_MONTHLY_BUDGET` (default `3000`)
- `ACTIONS_CD_RESERVED_MINUTES` (default `300`)
- `ACTIONS_BURNRATE_ENFORCE` (default `true`)
- `ACTIONS_BURNRATE_BUFFER_PCT` (default `10`)
- `ACTIONS_CD_WORKFLOW_NAMES` (default `CD,CD Non-Production,Release Tag`)
- `ACTIONS_HOTFIX_BRANCH_PREFIX` (default `hotfix/`)
- `ACTIONS_BUDGET_REPORT_ISSUE_NUMBER` (issue number for daily report updates)

PAT fallback scopes (for CLI sessions):
- `repo`
- `project`
- `read:project`
- `workflow`

## Failure Handling

- Policy violation: add `status:blocked`, keep gate in fail state, and post corrective checklist.
- Emergency override: requires explicit maintainer comment trail with reason and rollback plan.
- Missing project-field capability: gate/command workflows continue with explicit warning comments instead of silent skip.
