# CI/CD Policy

Canonical CI/CD execution and resource-governance policy for **bominal**.

For GitHub issue/PR/project governance, use `docs/GITHUB_GOVERNANCE.md`.

## Scope

Platform implementation is tool-agnostic, but outcomes are mandatory. Any CI/CD implementation MUST satisfy this policy.

## Required Pipeline Stages

0. Trigger Gate
- Full CI on pull requests to protected branches: `dev`, `staging`, `main`.
- Minimal post-merge guardrail CI on pushes to protected branches.
- Docs-only/markdown-only changes bypass heavy CI/CD stages.

1. Source Integrity Gate
- deterministic checkout,
- lockfile integrity,
- forbidden-secret scan.

2. Build And Static Gate
- compile all runtime services with locked dependencies,
- type/lint checks for runtime/web assets,
- scheduled clean-build verification without cache for dependency drift.

3. Test And Quality Gate
- unit/static checks before integration-heavy checks,
- coverage threshold enforcement for changed scope,
- critical-path negative-path enforcement.

4. Supply-Chain And Security Gate
- dependency vulnerability review on every pull request,
- base-image provenance/signing controls.

5. Release Gate
- immutable artifacts with version metadata,
- changelog/docs alignment verification.

6. Deploy Gate
- pre-deploy config validation,
- migration safety validation,
- controlled rollout strategy enforcement.

7. Post-Deploy Gate
- health checks,
- smoke checks,
- rollback trigger evaluation.

## Change-Scope Routing

- Docs-only deltas MUST bypass heavy CI/CD stages (build/runtime tests/artifact publish/deploy).
- Docs-only means `docs/**` or markdown-only file updates.
- Deploy workflows MUST NOT trigger from markdown-only changes, including markdown edits under otherwise deploy-scoped paths.

## Mandatory CI/CD Artifacts

- build metadata,
- test and coverage reports,
- security scan report,
- deployment decision log,
- post-deploy verification summary.

## Protected Branch CI Model

- PR full CI is the primary validation path.
- Protected-branch push CI is minimal guardrail only (avoid duplicate heavy execution).
- Branch policy gate requires named checks on protected branches.

## CI Execution Tiers

PRs use exactly one non-promotion `ci:tier:*` label:
- `ci:tier:light`: cheap checks only.
- `ci:tier:standard`: default policy checks.
- `ci:tier:heavy`: full heavy checks.

Tiering controls heavy-job admission and cost governance; tier policy is enforced by PR governance automation.

## Actions Minute Governance

GitHub Actions billing controls:
- monthly global cap: `3000` minutes,
- reserved CD pool: `300` minutes,
- non-CD cap: `2700` minutes.

Modes:
- `normal`: full non-CD checks,
- `throttle`: heavy checks only for `ci:tier:heavy` or hotfix/override paths,
- `lockdown`: block non-hotfix non-CD workflows; preserve CD reserve.

Global lockdown (`>=3000`) blocks CD/release workflows unless emergency override policy applies.

Implementation surfaces:
- `.github/workflows/actions-budget-governor.yml`
- `.github/workflows/actions-budget-report.yml`
- `.github/workflows/actions-budget-commands.yml`
- `.github/workflows/ci.yml`
- `.github/workflows/ci-push-minimal.yml`

## Copilot Review Budget In CI

- monthly cap: `300` invocations, reset at UTC month boundary (day 1),
- enforced by CI `Copilot Review Budget` job,
- warning threshold default `270` (configurable).

This budget gate is part of CI governance because it consumes workflow resources and review bandwidth.

## CD Efficiency And Scope

- CD should selectively rebuild changed runtime images and reuse stable refs for unchanged services.
- Non-production and production CD workflows should support concurrency cancellation where safe.
- Remove unnecessary build steps from CD when already guaranteed by upstream CI gates.

## Deploy Safety Contract

Deploy workflows MUST enforce:
- same-commit prerequisite validation,
- immutable image digests/artifacts,
- migration-before-restart discipline where applicable,
- health-gated rollback criteria.

## Command Interface

Supported governance commands:
- `/budget status`
- `/budget override reason:"..."`

Promotion commands are defined in `docs/GITHUB_GOVERNANCE.md`.
