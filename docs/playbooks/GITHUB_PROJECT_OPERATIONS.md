# GitHub Project Operations

Date validated: 2026-03-05
Repo: `jason931225/bominal`

## Board Suite

Use this 3-board model:
- `#1 bominal Workstreams` (delivery intake/execution board, automation target)
- `#2 bominal Review` (review state and secondary-review tracking)
- `#3 bominal Agent Command` (agent queue/claim/checkpoint operations)

Template adoption:
- Full adoption:
  - Iterative development: Workstreams grouped by `Status`.
  - Kanban: default Workstreams lane flow (`Todo` -> `In Progress` -> `Done`).
- Partial adoption:
  - Bug tracker: Workstreams saved view filtered `type:bug`, sorted `Priority` then `Risk`.
  - Feature release: Workstreams saved view grouped by `Target Release`.
  - Product launch: Workstreams saved view filtered `priority:p0|p1` + `area:ci-cd|infra|auth`.
- Non-adoption:
  - Do not create duplicate issue systems outside canonical GitHub issues + board fields.

## Auth Bootstrap For Agents

Project v2 item/field operations are done with `gh` using PAT (`GH_PAT_FULL`).

```bash
set -a
source /Users/jasonlee/bominal/env/dev/test.env
set +a
export GH_TOKEN="$GH_PAT_FULL"
gh auth status -h github.com
```

Expected: token has `project`, `repo`, `workflow` scopes.  
Current PAT works for personal-owner projects; `read:org` is optional unless org APIs are needed.

## Tested GH CLI Commands

Discovery:

```bash
gh project list --owner @me
gh project field-list 1 --owner @me --format json
gh variable list --repo jason931225/bominal
gh secret list --repo jason931225/bominal
```

Board item operations:

```bash
gh project item-add 1 --owner @me --url https://github.com/jason931225/bominal/issues/82
gh project item-list 1 --owner @me --limit 200 --format json
```

Workflow checks:

```bash
gh run list --repo jason931225/bominal --workflow "Project Automation" --limit 20
XDG_CACHE_HOME=/tmp/gh-cache gh run view <run_id> --repo jason931225/bominal --log
```

PR flow checks:

```bash
gh pr create --repo jason931225/bominal --base chore/project-automation-live --head <branch> --title "<title>" --body "<body>"
gh pr merge <pr_number> --repo jason931225/bominal --squash --delete-branch
```

## Tested GitHub MCP Access

Verified read/write MCP tools:
- `mcp__github__get_me`
- `mcp__github__list_issues`
- `mcp__github__pull_request_read`
- `mcp__github__add_issue_comment`

Important limitation:
- GitHub MCP currently used here does not expose Project v2 field/item administration.
- For Project v2 operations, agents must use `gh` CLI with PAT bootstrap above.

## Automation Contract

Workflow: `.github/workflows/project-automation.yml`

Behavior:
- Uses `BOMINAL_WORKSTREAMS_PROJECT_OWNER/NUMBER` first.
- Falls back to legacy `BOMINAL_PROJECT_OWNER/NUMBER`.
- Syncs issues and linked PR status to Workstreams board.
- Handles personal-owner projects safely (user/org GraphQL lookup without hard failure).
- PR governance enforces `type:*`, `area:*`, and `priority:*` inheritance from linked issue.
- Supports status fallback mapping when board uses default options:
  - `Triage` -> `Todo`
  - `Ready` -> `Todo`
  - `In Review` -> `In Progress` (fallback)
  - `Blocked` -> `In Progress` (fallback)

## Live Validation Evidence

Validated with dummy issues `#78`-`#82` and PR `#83` on 2026-03-05:
- PR open event moved linked issue `#78` to `In Progress` (fallback from `In Review`).
- PR merged event moved linked issue `#78` to `Done`.
- Issue-event failures observed on `main` before this patch confirm the old owner-query bug.

## Secondary Review Policy (Mandatory)

Review order when dual review is warranted:
1. Post `@copilot review`
2. Resolve or explicitly waive material Copilot findings
3. Post `@codex review` for independent cross-check

After labels/body/checks are ready, request secondary AI review in PR comments:

```text
@codex review
```

When risk is material (`SECURITY`, `PRODUCTION`, `DESTRUCTIVE`, or high-complexity refactor), also request:

```text
@copilot review
```

Merge only after:
- required human approval policy is satisfied,
- material Codex/Copilot findings are resolved or explicitly waived with rationale.

## Orchestrator Agent Contract

Orchestrator agents must open/update a GitHub issue before dispatching work.
Preferred intake form: `.github/ISSUE_TEMPLATE/orchestrator-job.yml`.

Required issue payload:
- Labels: exactly one `type:*`, one `area:*`, one `priority:*`, plus `risk:*` and `status:*`.
- Scope block: `Problem`, `Expected Outcome`, `In Scope`, `Out Of Scope`.
- Risk/domain block: auth/security/payment/deploy boundaries and forbidden paths.
- Delivery block: acceptance criteria, verification plan, rollback note, dependencies.
- Assignment block: execution instructions, non-goals, and completion evidence expectations.

Minimum CLI sequence:

```bash
gh issue create --repo jason931225/bominal --title "<type>: <scope>" --body-file <issue.md> \
  --label type:<...> --label area:<...> --label priority:<...> --label risk:<...> --label status:ready
gh project item-add 1 --owner @me --url https://github.com/jason931225/bominal/issues/<issue_number>
```
