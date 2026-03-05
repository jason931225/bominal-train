# GitHub Project Automation

This playbook operationalizes the canonical policy in `docs/MANUAL.md#project-tracking`.

## Board Topology

Maintain three active GitHub Project v2 boards:

1. `bominal Workstreams`
- Purpose: issue intake, prioritization, delivery status.

2. `bominal Review`
- Purpose: PR review-depth, Copilot disposition, merge readiness.

3. `bominal Agent Command`
- Purpose: automation control-plane for dispatch, claim checkpoints, domain-lock enforcement, and escalations.

## Required Field Model

### `bominal Workstreams`
- `Status`: `Triage`, `Ready`, `In Progress`, `In Review`, `Blocked`, `Done`
- `Type`: `Bug`, `Enhancement`, `Docs`, `Chore`, `Security`, `Ops`
- `Area`: label-aligned `area:*`
- `Priority`: `P0`, `P1`, `P2`, `P3`
- `Risk`: `Low`, `Medium`, `High`
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

## Automation Contract

### Intake and Readiness
- Auto-add every new issue to `bominal Workstreams` with `Status=Triage`.
- Transition to `Ready` only when issue includes:
  - required labels (`type:*`, `area:*`, `priority:*`),
  - acceptance criteria,
  - verification plan,
  - risk class.

### Agent Dispatch
- Source of truth for pickup order: `bominal Agent Command`.
- Pickup order: highest `priority:*` first, then FIFO oldest `Ready`.
- Agents may claim only items in `Claim State=Ready`.
- Claim checkpoints are mandatory and ordered:
  1. `Claimed`
  2. `Design Note Posted`
  3. `Draft PR Linked`

### Domain Lock and Parallelism
- Hard lock: one implementation item maps to one `area:*`.
- Hard lock: PR changed-path set must remain inside that area's allowed path map.
- Area WIP cap is `1` active implementation item.
- On conflicting same-area merge, active claim moves to `Blocked` with `Conflict State=Rebase Required`.

### Review and Merge
- Linked PR enters `bominal Review`.
- Linked issue status moves to `In Review` when PR is review-ready.
- Linked issue status moves to `Done` when PR is merged.
- Merge is blocked unless all are true:
  - linked issue exists (`Closes #...`),
  - required labels are present,
  - required checks are green,
  - all review conversations are resolved,
  - required review depth is satisfied,
  - material Copilot findings are fixed or maintainer-waived.

## Secondary Review Trigger Matrix

Set `Review Depth=Secondary Required` when any trigger matches:
- `risk:high`
- `type:security`
- `area:auth`, `area:payment-crypto`, `area:ci-cd`, `area:infra`
- Sensitive paths changed (auth/session/payment/deploy/migrations/security)
- Large-diff threshold exceeded (policy-defined file/line threshold)

## Copilot Disposition Policy

### Copilot Required
- Required for:
  - `risk:medium` and `risk:high` PRs
  - any PR with `Review Depth=Secondary Required`

### Material Classification
- Material by default if finding touches:
  - security boundaries
  - auth/session behavior
  - payment or secret handling
  - data-loss or rollback safety
  - deploy/infra safety
  - missing negative-path tests in risk-sensitive scope

### Waiver
- Only maintainers may waive material findings.
- Waiver must include:
  - reason,
  - risk note,
  - follow-up issue if deferring remediation.

## Agent Operating Protocol

1. Pull next item from `bominal Agent Command` queue.
2. Confirm issue is linked and `Ready`.
3. Post short design note on issue before opening draft PR.
4. Open draft PR with `Closes #...`.
5. Keep board fields current while progressing checkpoints.
6. Do not bypass `Blocked`/`Escalated` states; resolve gating conditions first.

## Setup Checklist

Repository variables:
- `BOMINAL_WORKSTREAMS_PROJECT_OWNER`
- `BOMINAL_WORKSTREAMS_PROJECT_NUMBER`
- `BOMINAL_REVIEW_PROJECT_OWNER`
- `BOMINAL_REVIEW_PROJECT_NUMBER`
- `BOMINAL_COMMAND_PROJECT_OWNER`
- `BOMINAL_COMMAND_PROJECT_NUMBER`
- Transition compatibility while workflow migration is in progress:
  - `BOMINAL_PROJECT_OWNER`
  - `BOMINAL_PROJECT_NUMBER`

Repository secret:
- `PROJECT_AUTOMATION_TOKEN` (`repo`, `read:project`)

## Exception Handling

- Policy violation: move to `Escalated`, label `status:blocked`, post corrective checklist.
- Emergency ops hotfix: permit temporary bypass only with explicit maintainer approval and post-merge incident note.
- Any manual override must leave an auditable comment trail on issue/PR.
