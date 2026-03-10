# AGENTS.md

Guidance for automated contributors in this repository.

## Mandatory Order Before Changes

1. `docs/START_HERE.md`
2. `docs/MANUAL.md`
3. `docs/GITHUB_GOVERNANCE.md`
4. `docs/CI_CD_POLICY.md`
5. `docs/README.md`
6. `docs/INTENT_ROUTING.md`

## Non-Negotiables

1. Preserve the product name `bominal`.
2. Treat `third_party/srtgo` and `third_party/catchtable` as read-only.
3. Keep train-provider integration behavior source-aligned with `third_party/srtgo/srtgo/srt.py` and `third_party/srtgo/srtgo/ktx.py`.
4. Never log or persist secrets, passwords, tokens, PAN/CVV, or raw sensitive provider payloads.
5. Preserve session-cookie security behavior (`HttpOnly`, `SameSite=Lax`, `Secure` only in production).
6. Follow canonical policy docs:
   - `docs/MANUAL.md` for security, permissions, quality, deployment standards, and docs governance.
   - `docs/GITHUB_GOVERNANCE.md` for issue/PR/project/release governance.
   - `docs/CI_CD_POLICY.md` for CI/CD execution and resource policy.
7. Keep `CHANGELOG.md` commit-based and append notable updates under `## Unreleased`.

## Repository Scope

Primary implementation paths:
- `runtime/crates/api`
- `runtime/crates/worker`
- `runtime/crates/shared`
- `runtime/migrations`
- `runtime/frontend`

Preserved external reference docs:
- `docs/handoff/**`

## Workflow Expectations

- Default to smallest safe change.
- Validate changes with build/test commands relevant to touched scope.
- Treat critical-path areas (auth, security, payment boundary, deployment) as high-rigor paths.
- Do not perform destructive or production/security boundary operations without explicit human approval.
- For GitHub Project v2 operations, follow `docs/GITHUB_GOVERNANCE.md` and use PAT-bootstrapped `gh` commands (Project v2 field/item admin is CLI-first in this repo flow).
- For PRs, request AI reviews generously when warranted: run `@copilot review` first for medium/high-risk or complex scope, then `@codex review` as cross-check before merge.

## GitHub Project Operating Policy (Mandatory)

Agents MUST follow the three-board model documented in:
- `docs/GITHUB_GOVERNANCE.md#project-tracking`

Execution rules:
- Pull work from `bominal Agent Command` queue state, not ad-hoc branch-first selection.
- Do not implement without a linked issue in `Ready` state.
- Orchestrator agents must post/update the issue first with required labels (`type:*`, `area:*`, `priority:*`, `risk:*`, `status:*`) and full scope/risk/domain/verification instructions before dispatching workers.
- Keep one `area:*` domain per implementation item and PR path-set (hard domain lock).
- Follow claim checkpoints in order: `Claimed` -> `Design Note Posted` -> `Draft PR Linked`.
- Respect area WIP cap (`1`) to avoid same-domain merge conflicts.
- Use branch flow strictly: implementation -> `dev`, promotion `dev -> staging -> main`, hotfix `hotfix/* -> main` then back-promote.
- Do not default to one-commit-per-PR; group related commits into one coherent PR and split only for independent/risk-isolated scopes.
- `dev -> staging` promotion is gate-driven (`promotion-gate*` workflows + `/gate promote`), not direct merged-PR side effect.
- Ensure PRs use `Closes #...` (except explicit promotion/back-promotion PRs), pass required checks, and resolve review conversations before merge.
- Ensure PR `type:*`, `area:*`, and `priority:*` labels inherit from the linked issue.
- For policy-scoped PRs, enforce review sequence: `@copilot review` first, then `@codex review`; material findings must be fixed or explicitly waived with a maintainer risk note.
- Apply exactly one `ci:tier:*` label for non-promotion PRs (`ci:tier:light|standard|heavy`), and use `semver:*` only for production-release planning/promotion metadata.
- Keep dependency vulnerability review always-on for PRs; do not bypass it with repo variables.
- Preserve CI ordering: unit/static checks before integration-heavy checks.
- Keep `@copilot review` usage judicious and within monthly budget (`300`, resets on the 1st UTC); rely on CI budget tracking before requesting additional reviews.
- Keep Actions usage within minute-governance policy: `3000` monthly global cap with `300` reserved for CD workflows.
- For project operations commands and MCP tooling, follow `docs/GITHUB_GOVERNANCE.md`.
- If `gh` lacks project scopes, load `GH_PAT_FULL` from `env/dev/test.env` as documented in `docs/GITHUB_GOVERNANCE.md` before board operations.

## Current Infrastructure Reality

Infrastructure automation is being rebuilt. Policy is prescriptive and target-state in `docs/MANUAL.md`, while some CI/CD/deploy tooling is currently absent from tracked repo files.
