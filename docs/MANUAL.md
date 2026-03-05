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

### Copilot Review

- Copilot review is required for:
  - PRs linked to work items with `Risk=Medium` or `Risk=High`,
  - any PR classified as `Review Depth=Secondary Required`.
- Copilot findings are classified as:
  - `Material`: security/auth/payment/session/data-loss/deploy/test-gap scope,
  - `Advisory`: quality/style/non-blocking scope.
- Merge is blocked while any material Copilot finding is open.
- Material findings may only be waived by a maintainer with explicit rationale and risk note.
- Copilot does not replace required human review policy where applicable.

### Project Tracking

Maintain three active GitHub Project v2 boards:
- `bominal Workstreams`: issue intake and delivery tracking.
- `bominal Review`: PR review-depth and merge-readiness tracking.
- `bominal Agent Command`: automation control-plane for dispatch, claim checkpoints, and policy escalations.

`bominal Workstreams` required fields:
- `Status`: `Triage`, `Ready`, `In Progress`, `In Review`, `Blocked`, `Done`
- `Type`: `Bug`, `Enhancement`, `Docs`, `Chore`, `Security`, `Ops`
- `Area`: aligned to `area:*` taxonomy
- `Priority`: `P0`, `P1`, `P2`, `P3`
- `Risk`: `Low`, `Medium`, `High`
- `Target Release`: semver target (for example `v0.2.0`)
- `Due Date`
- `Linked PR`

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

Automation expectations:
- new issues are auto-added to `bominal Workstreams` with `Status=Triage`,
- implementation starts only from `Ready` issues with required labels/acceptance/verification metadata,
- agent pickup order is deterministic: highest `priority:*`, then FIFO oldest `Ready`,
- claim flow is checkpoint-driven: `Claimed` -> `Design Note Posted` -> `Draft PR Linked`,
- hard domain lock applies: one `area:*` per implementation item and PR path-set,
- area WIP cap is `1`; same-area merge conflicts auto-transition active claims to `Blocked` with rebase checklist,
- linked PR review-ready state moves issue status to `In Review`,
- merged linked PR moves issue status to `Done`.

Repository automation prerequisites:
- repository variables:
  - `BOMINAL_WORKSTREAMS_PROJECT_OWNER`
  - `BOMINAL_WORKSTREAMS_PROJECT_NUMBER`
  - `BOMINAL_REVIEW_PROJECT_OWNER`
  - `BOMINAL_REVIEW_PROJECT_NUMBER`
  - `BOMINAL_COMMAND_PROJECT_OWNER`
  - `BOMINAL_COMMAND_PROJECT_NUMBER`
- repository secret `PROJECT_AUTOMATION_TOKEN` with `repo`, `project`, and `read:project` scopes.
- transition compatibility while workflow migration is in progress:
  - keep legacy `BOMINAL_PROJECT_OWNER`
  - keep legacy `BOMINAL_PROJECT_NUMBER`

Agent policy:
- agents MUST pull work from `bominal Agent Command` queue state, not ad-hoc branch choice,
- no implementation PR without a linked issue (`Closes #...`),
- secondary review is required when risk/sensitive scope or large-diff policy is triggered.

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
