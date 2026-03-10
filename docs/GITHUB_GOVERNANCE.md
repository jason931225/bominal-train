# GitHub Governance Policy

Canonical governance for GitHub Issues, Pull Requests, Projects, labels, review sequencing, promotion flow, and release controls.

## Scope And Authority

Use GitHub Issues, Pull Requests, Actions, Milestones, and Project tracking as the operational source of truth for active delivery.

Authority model:
- Policy here is canonical for GitHub governance.
- Security, permission, and deploy standards still anchor to `docs/MANUAL.md`.
- CI/CD execution and budget internals are canonical in `docs/CI_CD_POLICY.md`.
- Wiki pages may operationalize policy, but must link back to canonical anchors in this file.

## Board Topology

Maintain three active GitHub Project v2 boards:
1. `bominal Workstreams`: issue intake, prioritization, release-stage and promotion tracking.
2. `bominal Review`: PR review-depth, Copilot/Codex disposition, merge readiness.
3. `bominal Agent Command`: deterministic agent dispatch queue, checkpoints, domain locks, escalations.

## Template Adoption Matrix

Adopt board template ideas by partial composition:
- `Kanban` (adopted): baseline status flow `Triage -> Ready -> In Progress -> In Review -> Blocked -> Done`.
- `Iterative development` (adopted): issue checkpoints and draft-PR milestones.
- `Bug tracker` (partially adopted): severity/risk handling via `priority:*` + `risk:*`.
- `Feature release` (partially adopted): `Target Release`, `Release Checkpoint`, `Promotion Flag` fields.
- `Product launch` (selective adoption): release calendar and milestone discipline only.
- Non-adoption: do not create duplicate issue systems outside canonical GitHub issues + board fields.

## Label Taxonomy

All active Issues and PRs MUST carry:
- exactly one `type:*`: `type:bug`, `type:enhancement`, `type:documentation`, `type:chore`, `type:security`, `type:ops`.
- exactly one `area:*`: `area:runtime-api`, `area:runtime-worker`, `area:runtime-shared`, `area:runtime-frontend`, `area:payment-crypto`, `area:docs`, `area:ci-cd`, `area:infra`, `area:auth`, `area:observability`.
- exactly one `priority:*`: `priority:p0`, `priority:p1`, `priority:p2`, `priority:p3`.

Optional labels:
- `status:*`, `risk:*` for routing and incident response.
- `ci:tier:*` (`light|standard|heavy`) required on non-promotion PRs.
- `semver:*` (`major|minor|patch|none`) for production release planning metadata only.
- `budget:override`, `budget:lockdown` for Actions minute-governance controls.
- triage outcomes: `duplicate`, `help wanted`, `question`, `invalid`, `wontfix`.

Canonical label definitions MUST be maintained in `.github/labels.yml`.

Priority scope classification:
- `priority:p0`: active incident, security emergency, data-integrity risk, or release-blocking production risk.
- `priority:p1`: high-impact, near-term committed work.
- `priority:p2`: planned standard backlog.
- `priority:p3`: exploratory or deferred work.

PR label inheritance:
- PR `type:*`, `area:*`, and `priority:*` MUST inherit from the primary linked issue (`Closes #...`).
- Label mismatch with linked issue is merge-blocking governance failure.

## Issue Governance

Issue intake MUST use repository issue forms (`.github/ISSUE_TEMPLATE/*.yml`) and include:
- problem statement,
- expected outcome,
- in-scope and out-of-scope,
- acceptance criteria,
- risk classification,
- verification plan.

Blank issues are disabled by default.

## Pull Request Governance

Every implementation PR MUST:
1. Link at least one issue with closing syntax (`Closes #123`).
2. Include summary, scope, risk/rollback notes, verification evidence, docs impact, and changelog impact in the PR template.
3. Carry required labels (`type:*`, `area:*`, `priority:*`).
4. Carry exactly one `ci:tier:*` label unless it is explicit promotion/back-promotion scope.
5. Resolve all review conversations before merge.
6. Pass required checks and branch protection rules.

Additional PR rules:
- Docs-only PRs must remain docs/markdown-only, include `type:documentation` + `area:docs`, and use docs-only CI/CD routing.
- Duplicate PRs must carry `duplicate` and reference the replacement PR/issue.
- One PR should represent one coherent scope; split only for independent scope, merge-order constraints, or risk isolation.
- Do not default to one-commit-per-PR. Multiple focused commits in one coherent PR are preferred.

## Secondary AI Review Policy

Default sequence when dual review is warranted:
1. `@copilot review`
2. resolve or explicitly waive material Copilot findings
3. `@codex review` for cross-check

Rules:
- Every PR MUST request at least a secondary Codex review comment after labels/body/checks are ready.
- For material-risk changes (`SECURITY`, `PRODUCTION`, `DESTRUCTIVE`) or high-complexity refactors, Copilot review is also required.
- Copilot review is required for work items marked `Risk=Medium|High` or `Review Depth=Secondary Required`.
- Material findings are merge-blocking unless fixed or maintainer-waived with rationale/risk note.
- AI review is advisory and does not replace required human approvals.

## Copilot Review Budget

- Monthly cap: `300` Copilot review invocations.
- Reset: first day of each UTC month.
- Enforcement: CI `Copilot Review Budget` job fails once cap is reached.
- Warning threshold default: `270` (override via `COPILOT_REVIEW_WARN_THRESHOLD`).
- Use Copilot review judiciously; avoid low-risk docs-only hygiene usage.

## Actions Minute Governance

- GitHub Actions usage is billed by minute and must be actively governed.
- Monthly global cap: `3000` minutes.
- Reserved CD budget: `300` minutes.
- Non-CD cap: `2700` minutes.

Governance modes:
- `normal`: full non-CD policy checks.
- `throttle`: only `ci:tier:heavy` (or hotfix/override) PRs run heavy checks.
- `lockdown`: non-hotfix non-CD workflows blocked; CD reserve remains protected.
- Global lockdown (>= `3000`) blocks CD and release workflows unless emergency override policy is used.

Operational surfaces:
- evaluator workflow: `.github/workflows/actions-budget-governor.yml`
- daily reporting: `.github/workflows/actions-budget-report.yml`
- commands: `/budget status`, `/budget override reason:"..."`

## Project Tracking

### Required Field Model

`bominal Workstreams`:
- `Status`: `Triage`, `Ready`, `In Progress`, `In Review`, `Blocked`, `Done`
- `Type`: `Bug`, `Enhancement`, `Docs`, `Chore`, `Security`, `Ops`
- `Area`
- `Priority`: `P0`, `P1`, `P2`, `P3`
- `Risk`: `Low`, `Medium`, `High`
- `Release Checkpoint`: `Backlog`, `Ready for Staging Gate`, `Gate In Progress`, `Promotion PR Open`, `Promoted`
- `Promotion Flag`: `None`, `Promote`, `Hold`
- `Target Release`
- `Due Date`
- `Linked PR`
- `Gate Issue URL`
- `Merge Order Source`

`bominal Review`:
- `Review Status`: `Ready for Review`, `Changes Requested`, `Approved`, `Merged`
- `Review Depth`: `Standard`, `Secondary Required`
- `Copilot Required`: `Yes`, `No`
- `Copilot Material State`: `Clear`, `Material Open`, `Material Waived`
- `Linked Issue`

`bominal Agent Command`:
- `Queue Rank`
- `Claim State`: `Ready`, `Claimed`, `Design Note Posted`, `Draft PR Linked`, `Blocked`, `Escalated`
- `Checkpoint`: `Claim`, `Design Note`, `Draft PR`
- `Domain Lock`: `Pass`, `Fail`
- `Conflict State`: `None`, `Rebase Required`
- `Escalation State`: `None`, `Secondary Review`, `Policy Exception`

### Automation Expectations

- New issues auto-add to Workstreams with `Status=Triage`.
- Implementation starts only from `Ready` issues with required metadata.
- Agent pickup order is deterministic: highest `priority:*`, then FIFO oldest `Ready`.
- Claim flow is checkpoint-driven: `Claimed -> Design Note Posted -> Draft PR Linked`.
- Hard domain lock: one `area:*` per implementation item and PR path-set.
- Area WIP cap is `1`; same-area conflicts auto-transition active claims to `Blocked` with rebase checklist.
- Linked PR review-ready state moves issue to `In Review` (with fallback mapping where board options differ).
- Merged linked PR moves issue to `Done`.
- Promotion to `staging` is gate-driven from release checkpoints, not direct merge side effect.

## Promotion Gate And Commands

Promotion governance is enforced by deterministic workflows:
- `promotion-gate-controller.yml`: creates/refreshes gate issue from Workstreams `Release Checkpoint=Ready for Staging Gate`.
- `promotion-gate.yml`: runs `ci_gate` and `promotion_governance_gate`.
- `promotion-gate-review-loop.yml`: enforces review-loop evidence (`@copilot review`, `@codex review`) and no unresolved `CHANGES_REQUESTED`.
- `promotion-gate-commands.yml`: handles `/gate ...` and `/promote merge`.
- `promotion-pr-open-dev-staging.yml`: opens `dev -> staging` PR only after gate pass and promotion intent.
- `promotion-pr-create.yml`: handles `staging -> main` auto-create for merged PRs explicitly labeled `promotion:auto`.

Gate pass prerequisites for `dev -> staging`:
- `gate:ci-pass`
- `gate:governance-pass`
- `gate:review-round-complete`
- `gate:promote`

### Gate Manifest Contract

Each gate issue must include:

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

Required sections in the gate issue body:
- `## Merge Order Notes`
- `## Risk Assessment`
- `## Release Summary`
- `## Round Review Log`
- `## Waiver Ledger`

### Command Processing And Waivers

Processing:
- workflow: `.github/workflows/promotion-gate-commands.yml`
- trigger: `issue_comment` (`created`, `edited`)
- required permission: `write|maintain|admin`

Supported commands:
- `/gate refresh`
- `/gate promote`
- `/gate waive advisory <finding_id> reason:"..." risk:"..." expires:"YYYY-MM-DD" followup:"#123"`
- `/promote merge` (PR comments)
- `/budget status`
- `/budget override reason:"..."`

Waiver rules:
- advisory findings only (material findings are not command-waivable),
- command appends immutable ledger entry in `## Waiver Ledger`,
- adds label `gate:advisory-waived`.

## Orchestrator Agent Contract

Orchestrator agents MUST open/update an issue before dispatching execution agents.
Use `.github/ISSUE_TEMPLATE/orchestrator-task.yml` (or equivalent structured issue body).

Required issue payload:
- objective and expected outcome,
- one `area:*` domain lock and allowed path-set,
- risk class, sensitive boundaries, rollback/blast radius,
- in-scope and out-of-scope,
- acceptance criteria and exact verification commands,
- dependency notes and merge-order notes for sequence-required work,
- review sequencing instructions (`@copilot review` then `@codex review`) when warranted.

## Tested Commands For Agents

CLI bootstrap:

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

Create and add issue:

```bash
ISSUE_URL=$(gh issue create --repo jason931225/bominal --title "chore: board automation smoke test" --body "Policy smoke test issue" --label type:chore --label area:ci-cd --label priority:p3 --label ci:tier:standard --label status:ready)
gh project item-add "$BOMINAL_WORKSTREAMS_PROJECT_NUMBER" --owner "$BOMINAL_WORKSTREAMS_PROJECT_OWNER" --url "$ISSUE_URL"
```

Review/promotion commands:

```bash
gh pr comment <PR_NUMBER> --repo jason931225/bominal --body "@copilot review"
gh pr comment <PR_NUMBER> --repo jason931225/bominal --body "@codex review"
gh issue comment <GATE_ISSUE_NUMBER> --repo jason931225/bominal --body "/gate refresh"
gh issue comment <GATE_ISSUE_NUMBER> --repo jason931225/bominal --body "/gate promote"
gh pr comment <PR_NUMBER> --repo jason931225/bominal --body "/promote merge"
```

## GitHub MCP Capability Contract

Verified read/write MCP tools:
- `mcp__github__get_me`
- `mcp__github__list_issues`
- `mcp__github__pull_request_read`
- `mcp__github__add_issue_comment`

Project v2 limitation:
- current MCP toolset in this repository flow does not manage Project v2 fields/items.
- use `gh` CLI with PAT bootstrap for Project v2 field/item administration.

## Auth Scope Requirements

Repository secret:
- `PROJECT_AUTOMATION_TOKEN` with Project v2 GraphQL read/write support.

Optional variables:
- `COPILOT_REVIEW_MONTHLY_BUDGET` (default `300`)
- `COPILOT_REVIEW_WARN_THRESHOLD` (default `270`)
- `ACTIONS_MINUTES_MONTHLY_BUDGET` (default `3000`)
- `ACTIONS_CD_RESERVED_MINUTES` (default `300`)
- `ACTIONS_BURNRATE_ENFORCE` (default `true`)
- `ACTIONS_BURNRATE_BUFFER_PCT` (default `10`)
- `ACTIONS_CD_WORKFLOW_NAMES` (default `CD,CD Non-Production,Release Tag`)
- `ACTIONS_HOTFIX_BRANCH_PREFIX` (default `hotfix/`)
- `ACTIONS_BUDGET_REPORT_ISSUE_NUMBER` (daily report issue number)

PAT fallback scopes:
- `repo`
- `project`
- `read:project`
- `workflow`

## Wiki Governance

Required active wiki pages:
- Home
- Roadmap
- Release Calendar
- Incident Index
- Contributor Workflow

Policy-sensitive wiki content MUST link back to canonical sections in this document.

Wiki bootstrap note:
- if `<repo>.wiki.git` is not provisioned, initialize first page in UI, then manage via git/web edits.

## Milestones, Releases, And Tags

Release planning:
- one milestone per planned semver release (`vX.Y.Z`),
- assign release-scoped issues/PRs to milestone,
- close milestone only after acceptance criteria are met.

Tagging:
- annotated semver tags only (`vMAJOR.MINOR.PATCH`) from `main`,
- tags created through protected workflow dispatch, not ad-hoc local tagging,
- each release tag requires release notes and changelog alignment evidence.

## Branch Protection Baseline For `main`

Required branch settings:
- require pull request before merge,
- require configured approving review count (solo mode may be `0`; policy secondary-review rules still apply),
- dismiss stale approvals on new commits,
- require conversation resolution,
- require strict up-to-date status checks,
- enforce for admins,
- disallow force-push and deletion.

Required status checks:
- `Workflow Lint`
- `PR Governance`
- `CI Budget Policy`
- `PR Execution Policy`
- `Dependency Review`
- `Copilot Review Budget`
- `Branch Policy Gate`

Merge strategy default:
- prefer squash merge for routine PRs,
- keep merge commit/rebase merge disabled unless documented exception is approved.

## Failure Handling

- Policy violation: apply `status:blocked`, keep gate in fail state, and post corrective checklist.
- Missing project-field capability: workflows continue with explicit warning comments (never silent skip).
- Emergency override requires explicit maintainer comment trail with reason and rollback plan.
