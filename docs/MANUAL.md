# Bominal Manual

## Product Scope And Non-Negotiables

This manual is the canonical policy and operations source for **bominal**.

Mandatory invariants:
- Product name stays `bominal` in UI, docs, and config.
- `third_party/srtgo` and `third_party/catchtable` are read-only.
- Train-provider behavior must remain source-aligned with `third_party/srtgo/srtgo/srt.py` and `third_party/srtgo/srtgo/ktx.py`.
- Runtime baseline is Rust-first (`runtime/`), with no production dependency on retired Python runtime paths.
- Sensitive data must never be logged, serialized to queues, or persisted outside approved boundaries.

Authoritative code map:
- `runtime/crates/api`: API + SSR shell.
- `runtime/crates/worker`: background task runtime.
- `runtime/crates/shared`: shared contracts and integrations.
- `runtime/migrations`: SQL migrations.
- `runtime/frontend`: Tailwind build assets for runtime UI.
- `docs/handoff`: preserved external handoff reference set.

## Security And Data Handling Policy

Required controls:
- Secrets MUST be sourced from environment or managed secret stores. Plaintext secrets in docs, code, logs, and tickets are prohibited.
- Session cookies MUST be `HttpOnly`, `SameSite=Lax`, and `Secure` only in production.
- Payment and credential payloads MUST be encrypted at rest and redacted in logs.
- Provider requests MUST use TLS with certificate validation enabled.
- Internal APIs MUST require explicit service authentication; deny by default.

Prohibited patterns:
- Logging request/response bodies that can contain credentials or payment data.
- Storing raw PAN/CVV or equivalent cardholder fields in database, queue, or artifacts.
- Disabling TLS verification or broadening egress without explicit approval.

Evidence requirements:
- Security-sensitive changes include explicit test coverage for failure/deny paths.
- Every release candidate must include a secrets and log-redaction sanity check.

## Permissions And Approval Model

Risk classes:
- `READ`: inspection only.
- `WRITE`: repository changes.
- `DESTRUCTIVE`: irreversible deletes/resets/migrations.
- `SECURITY`: IAM/auth/secret/egress boundary changes.
- `PRODUCTION`: live-environment mutations.

Approval gates:
- `DESTRUCTIVE`, `SECURITY`, and `PRODUCTION` actions require explicit human approval.
- Break-glass production writes require:
  - owner approval,
  - time-bound window,
  - command list,
  - rollback plan,
  - incident/change reference.

Default behavior:
- Fail closed when permission scope is unclear.
- Prefer read-only validation before any mutating step.

## Engineering Quality Standard

Baseline quality rules:
- Every behavior change MUST include directly relevant tests.
- Critical-path changes (auth, payments, secrets, deployment) MUST include negative-path tests.
- No unresolved warnings in touched scope.
- Deprecated dependency/runtime warnings are actionable and must be triaged before release.

Coverage floors (minimum guardrails):
- Rust API + worker workspace line coverage: >= 80% for changed crates.
- Web test suite coverage: >= 75% lines/functions/branches/statements for changed areas.

Verification contract:
- Run formatting, compile, and tests before completion claims.
- Completion claims require command evidence, not assumptions.

## CI-CD Target State Policy

Platform is intentionally tool-agnostic. Any CI/CD implementation MUST satisfy these outcome gates.

Required pipeline stages:
1. Source integrity gate
- deterministic checkout
- lockfile integrity
- forbidden-secret scan

2. Build and static gate
- compile all runtime services
- type/lint checks for web/runtime assets

3. Test and quality gate
- unit + integration test execution
- coverage threshold enforcement
- critical-path negative test enforcement

4. Supply-chain and security gate
- dependency vulnerability scan
- base image provenance/signing checks

5. Release gate
- immutable artifacts with version metadata
- changelog and docs alignment verification

6. Deploy gate
- pre-deploy config validation
- migration safety validation
- rollout strategy enforcement (progressive/controlled)

7. Post-deploy gate
- health checks
- smoke checks
- rollback trigger evaluation

Change-scope routing policy:
- Docs-only deltas MUST bypass heavy CI/CD stages (build, runtime test, artifact publish, deploy).
- Docs-only means changes under `docs/**` or markdown-only file updates.
- Deploy workflows MUST NOT trigger from markdown-only changes even when they occur under otherwise deploy-scoped paths (for example `runtime/**` or `env/prod/**`).

Mandatory artifacts:
- build metadata,
- test and coverage reports,
- security scan report,
- deployment decision log,
- post-deploy verification summary.

## GitHub Project Management Policy

Use GitHub Issues, Pull Requests, Actions, Milestones, and Project tracking as the operational source of truth for active delivery.

Authority model:
- Canonical policy and security/deploy standards remain in-repo (`docs/MANUAL.md`).
- GitHub Wiki is active for operational guidance, onboarding, and coordination pages, and MUST link back to canonical manual sections when policy-sensitive topics are referenced.

### Label Taxonomy

All active Issues and PRs MUST carry:
- exactly one `type:*` label:
  - `type:bug`
  - `type:enhancement`
  - `type:documentation`
  - `type:chore`
  - `type:security`
  - `type:ops`
- exactly one `area:*` label:
  - `area:runtime-api`
  - `area:runtime-worker`
  - `area:runtime-shared`
  - `area:runtime-frontend`
  - `area:payment-crypto`
  - `area:docs`
  - `area:ci-cd`
  - `area:infra`
  - `area:auth`
  - `area:observability`
- exactly one `priority:*` label:
  - `priority:p0`
  - `priority:p1`
  - `priority:p2`
  - `priority:p3`

Optional labels:
- status/risk labels (`status:*`, `risk:*`) are encouraged for routing and incident response.
- `duplicate`, `help wanted`, `question`, `invalid`, `wontfix` remain available for triage outcomes.

Canonical label definitions MUST be kept in `.github/labels.yml`.

Priority scope classification:
- `priority:p0`: active incident, security emergency, data-integrity risk, or release-blocking production risk.
- `priority:p1`: high-impact, near-term committed work with meaningful user/operator risk.
- `priority:p2`: planned standard backlog delivery without immediate service risk.
- `priority:p3`: exploratory or deferred backlog work.

PR label inheritance:
- PR `type:*`, `area:*`, and `priority:*` labels MUST inherit from the primary linked issue (`Closes #...`).
- Label mismatches between PR and linked issue are merge-blocking governance failures.

### Issue Governance

Issue intake MUST use repository issue forms (`.github/ISSUE_TEMPLATE/*.yml`) and include:
- problem statement
- expected outcome
- in-scope / out-of-scope
- acceptance criteria
- risk classification
- verification plan

Blank issues are disabled by default.

### Pull Request Governance

Every PR to `main` MUST:
1. Link at least one issue with closing syntax (`Closes #123`).
2. Include summary, scope, risk/rollback notes, verification evidence, docs impact, and changelog impact in the PR template.
3. Carry required labels (`type:*`, `area:*`, `priority:*`).
4. Resolve all review conversations before merge.
5. Pass required checks and branch protection rules.

Additional PR rules:
- Docs-only PRs:
  - scope must remain docs/markdown only,
  - must include `type:documentation` and `area:docs`,
  - use docs-only CI/CD routing (no heavy build/deploy stages).
- Duplicate PRs:
  - apply `duplicate`,
  - include replacement reference in body/comment,
  - close or keep only as historical context.

### Secondary AI Review

- Every non-trivial PR SHOULD request both AI reviews when scope/risk warrants it; Copilot-first then Codex cross-check is the default sequence:
  1. `@copilot review`
  2. resolve/waive material Copilot findings
  3. `@codex review`
- Every PR MUST request at least a secondary Codex review comment after labels/body/checks are ready:
  - `@codex review`
- For material-risk changes (`SECURITY`, `PRODUCTION`, `DESTRUCTIVE`) or high-complexity refactors, also request:
  - `@copilot review`
- Copilot review is required for:
  - PRs linked to work items with `Risk=Medium` or `Risk=High`,
  - any PR classified as `Review Depth=Secondary Required`.
- Copilot review should be used judiciously:
  - avoid routine docs-only or low-risk hygiene PRs unless explicitly warranted,
  - prefer risk-driven usage (security/auth/payment/deploy/sensitive migrations),
  - default cross-check order is `@copilot review` then `@codex review`.
- Copilot findings are classified as:
  - `Material`: security/auth/payment/session/data-loss/deploy/test-gap scope,
  - `Advisory`: quality/style/non-blocking scope.
- Merge is blocked while any material Codex or Copilot finding is open.
- Material findings may only be waived by a maintainer with explicit rationale and risk note.
- AI review is advisory and does not replace required human approval policy where applicable.
- Copilot review monthly budget is capped at `300` requests per month (UTC month boundary, reset on day `1`).
- CI MUST track monthly `@copilot review` invocation count and fail when budget is exceeded.

### Project Tracking

Maintain three active GitHub Project v2 boards:
- `bominal Workstreams`: issue intake and delivery tracking.
- `bominal Review`: PR review-depth and merge-readiness tracking.
- `bominal Agent Command`: automation control-plane for dispatch, claim checkpoints, and policy escalations.

Operational runbooks:
- `docs/playbooks/GITHUB_PROJECT_AUTOMATION.md`
- `docs/playbooks/GITHUB_PROJECT_OPERATIONS.md`

`bominal Workstreams` required fields:
- `Status`: `Triage`, `Ready`, `In Progress`, `In Review`, `Blocked`, `Done`
- `Type`: `Bug`, `Enhancement`, `Docs`, `Chore`, `Security`, `Ops`
- `Area`: aligned to `area:*` taxonomy
- `Priority`: `P0`, `P1`, `P2`, `P3`
- `Risk`: `Low`, `Medium`, `High`
- `Release Checkpoint`: `Backlog`, `Ready for Staging Gate`, `Gate In Progress`, `Promotion PR Open`, `Promoted`
- `Promotion Flag`: `None`, `Promote`, `Hold`
- `Target Release`: semver target (for example `v0.2.0`)
- `Due Date`
- `Linked PR`
- `Gate Issue URL` (text)
- `Merge Order Source` (text)

`bominal Review` required fields:
- `Review Status`: `Ready for Review`, `Changes Requested`, `Approved`, `Merged`
- `Review Depth`: `Standard`, `Secondary Required`
- `Copilot Required`: `Yes`, `No`
- `Copilot Material State`: `Clear`, `Material Open`, `Material Waived`
- `Linked Issue`

`bominal Agent Command` required fields:
- `Queue Rank`
- `Claim State`: `Ready`, `Claimed`, `Design Note Posted`, `Draft PR Linked`, `Blocked`, `Escalated`
- `Checkpoint`: `Claim`, `Design Note`, `Draft PR`
- `Domain Lock`: `Pass`, `Fail`
- `Conflict State`: `None`, `Rebase Required`
- `Escalation State`: `None`, `Secondary Review`, `Policy Exception`

Template adoption strategy:
- Full adoption: iterative development + kanban flow on `bominal Workstreams`.
- Partial adoption: feature-release, bug-tracker, and product-launch templates as saved views/filters over the same canonical issues (not separate duplicate issue systems).
- Non-adoption: avoid template mechanics that duplicate labeling/project state outside canonical issue fields.

Automation expectations:
- new issues are auto-added to `bominal Workstreams` with `Status=Triage`,
- implementation starts only from `Ready` issues with required labels/acceptance/verification metadata,
- agent pickup order is deterministic: highest `priority:*`, then FIFO oldest `Ready`,
- claim flow is checkpoint-driven: `Claimed` -> `Design Note Posted` -> `Draft PR Linked`,
- hard domain lock applies: one `area:*` per implementation item and PR path-set,
- area WIP cap is `1`; same-area merge conflicts auto-transition active claims to `Blocked` with rebase checklist,
- linked PR review-ready state moves issue status to `In Review` (or mapped fallback when board options differ),
- merged linked PR moves issue status to `Done`,
- promotion to `staging` is gate-driven from milestone checkpoints, not direct branch merge side effects.

Repository automation prerequisites:
- preferred variables:
  - `BOMINAL_WORKSTREAMS_PROJECT_OWNER`
  - `BOMINAL_WORKSTREAMS_PROJECT_NUMBER`
- review/command board variables:
  - `BOMINAL_REVIEW_PROJECT_OWNER`
  - `BOMINAL_REVIEW_PROJECT_NUMBER`
  - `BOMINAL_COMMAND_PROJECT_OWNER`
  - `BOMINAL_COMMAND_PROJECT_NUMBER`
  - optional: `COPILOT_REVIEW_MONTHLY_BUDGET` (default `300`)
  - optional: `COPILOT_REVIEW_WARN_THRESHOLD` (default `270`)
- repository secret `PROJECT_AUTOMATION_TOKEN` with `repo`, `project`, and `read:project` scopes.
- transition compatibility while workflow migration is in progress:
  - keep legacy `BOMINAL_PROJECT_OWNER`
  - keep legacy `BOMINAL_PROJECT_NUMBER`

Agent policy:
- agents MUST pull work from `bominal Agent Command` queue state, not ad-hoc branch choice,
- no implementation PR without a linked issue (`Closes #...`),
- secondary review is required when risk/sensitive scope or large-diff policy is triggered,
- orchestrator-authored issues MUST include scope, risk, domain lock, dependency notes, and merge-order notes.

### Promotion Gate And Commands

Promotion governance is enforced by deterministic workflows:
- `promotion-gate-controller.yml`: creates/refreshes gate issues from Workstreams `Release Checkpoint=Ready for Staging Gate`.
- `promotion-gate.yml`: runs `ci_gate` and `promotion_governance_gate`.
- `promotion-gate-review-loop.yml`: enforces review-loop evidence (`@copilot review`, `@codex review`) and no unresolved `CHANGES_REQUESTED`.
- `promotion-gate-commands.yml`: handles `/gate ...` and `/promote merge`.
- `promotion-pr-open-dev-staging.yml`: opens `dev -> staging` PR only after gate pass + promotion intent.

Gate pass prerequisites for `dev -> staging`:
- `gate:ci-pass`
- `gate:governance-pass`
- `gate:review-round-complete`
- `gate:promote`

Command policy:
- `/gate refresh` reruns gate validation for a specific issue.
- `/gate promote` sets promotion intent (and project `Promotion Flag=Promote` where available).
- `/gate waive advisory ...` records advisory-only waivers in gate `## Waiver Ledger`.
- `/promote merge` is restricted to write/maintain/admin and enforces green checks + no active `CHANGES_REQUESTED`.

Orchestrator policy:
- orchestrator agents MUST create or update a GitHub issue before dispatching execution agents.
- orchestrator-created issues MUST include:
  - explicit `type:*`, `area:*`, `priority:*`, and `risk:*` labels,
  - scope (`in-scope`, `out-of-scope`),
  - domain constraints (security/auth/payment/deploy boundaries),
  - acceptance criteria and verification plan,
  - rollback note and dependency context,
  - clear implementation instructions and non-goals.

### Wiki Governance

Required active wiki pages:
- Home
- Roadmap
- Release Calendar
- Incident Index
- Contributor Workflow

Wiki pages that describe policy MUST link back to canonical manual anchors instead of redefining policy independently.

Wiki bootstrap note:
- if `<repo>.wiki.git` is not yet provisioned, initialize the first wiki page via GitHub UI, then manage pages through git or web edits.

### Milestones, Releases, And Tags

Release planning model:
- create one milestone per planned semver release (`vX.Y.Z`),
- assign all release-scoped issues/PRs to that milestone,
- close milestone only after acceptance criteria are met.

Tagging model:
- create annotated semver tags only (`vMAJOR.MINOR.PATCH`) from `main`,
- create tags through protected manual workflow dispatch (not ad-hoc local tagging),
- each release tag MUST have release notes and changelog alignment evidence.

### Branch Protection Baseline For `main`

Required branch settings:
- require pull request before merge,
- require configured approving review count (solo mode may be `0`; policy-driven secondary review gates still apply),
- dismiss stale approvals on new commits,
- require conversation resolution,
- require strict up-to-date status checks,
- enforce for admins,
- disallow force-pushes and deletions.

Required status checks:
- `Workflow Lint`
- `PR Governance`
- `Copilot Review Budget`
- `Branch Policy Gate`

Merge strategy default:
- prefer squash merge for routine feature/fix PRs,
- merge commit and rebase merge should remain disabled unless a documented exception is approved.

## Deployment And Rollback Standard

Deployment policy:
- Use immutable artifacts.
- Prefer progressive rollout with health-gated advancement.
- Database migrations must be forward-safe and operationally reversible.

Minimum deployment checks:
- API liveness and readiness endpoints return expected states.
- Worker startup and queue connectivity validated.
- SSR/web asset availability confirmed.
- VM memory baseline is enforced before rollout:
  - `/swapfile` active at 2G,
  - `vm.swappiness=10`,
  - `vm.vfs_cache_pressure=50`.

Rollback policy:
- Immediate rollback on sustained health failure, data integrity risk, or security regression.
- Rollback path must be documented before rollout.
- Post-rollback incident note is mandatory for any production revert.

## Operations Runbook Core

Incident baseline:
- classify severity (`SEV1`-`SEV4`),
- assign incident owner,
- stabilize service first,
- then remediate root cause.

Triage sequence:
1. Confirm scope and user impact.
2. Validate health/readiness and upstream dependencies.
3. Check recent deploy/config deltas.
4. Execute the least-risk mitigation.
5. Capture timeline and decisions.

Operational expectations:
- Every production-impacting event produces a concise incident summary.
- Repeated incidents must generate a preventive action item.

## Documentation Governance

Single-source rule:
- `docs/MANUAL.md` is canonical policy + operations guidance.
- Other docs should reference this manual instead of redefining policy.

Change discipline:
- Docs-first for policy-sensitive work.
- Docs-last verification before completion.
- `CHANGELOG.md` must record notable docs/policy changes.

Pointer discipline:
- Keep `docs/README.md` as a compact pointer index.
- Keep `docs/START_HERE.md` as orientation entrypoint.
- Keep `docs/INTENT_ROUTING.md` as keyword router into manual sections.
- Keep `docs/PROD_ENV_CONTRACT.md` aligned with production env key contracts and secret-handling posture.

## Current Gap Register And Backfill Targets

Current repository state:
- Runtime source is present and buildable.
- Env templates are present under `env/**`.
- Baseline GitHub Actions workflows are present as `.github/workflows/ci.yml` and `.github/workflows/cd.yml`.
- VM deploy contract is fail-closed and environment-defined: CD requires protected deploy script-path variables, executes remote scripts over IAP SSH, runs SQL migrations before service restart, deploys immutable image digests, and triggers rollback when post-deploy health checks fail.
- Full infra validation suites and richer rollback automation are not yet fully restored.

Mandatory backfill targets:
1. CI quality pipeline implementation aligned to this manual's gates.
2. Deployment orchestration with preflight checks and rollback automation.
3. Infra validation suite for docs, security, deployment, and runtime contracts.
4. Production runbook command set with deterministic evidence capture.

Backfill completion criteria:
- All target-state pipeline gates are executable in-repo.
- Deployment + rollback paths are testable and documented.
- Manual and operational commands reflect actual committed infrastructure.
