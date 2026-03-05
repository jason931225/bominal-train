# AGENTS.md

Guidance for automated contributors in this repository.

## Mandatory Order Before Changes

1. `docs/START_HERE.md`
2. `docs/MANUAL.md`
3. `docs/README.md`
4. `docs/INTENT_ROUTING.md`

## Non-Negotiables

1. Preserve the product name `bominal`.
2. Treat `third_party/srtgo` and `third_party/catchtable` as read-only.
3. Keep train-provider integration behavior source-aligned with `third_party/srtgo/srtgo/srt.py` and `third_party/srtgo/srtgo/ktx.py`.
4. Never log or persist secrets, passwords, tokens, PAN/CVV, or raw sensitive provider payloads.
5. Preserve session-cookie security behavior (`HttpOnly`, `SameSite=Lax`, `Secure` only in production).
6. Follow `docs/MANUAL.md` for security, permissions, quality, CI/CD target controls, deployment standards, and docs governance.
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

## GitHub Project Operating Policy (Mandatory)

Agents MUST follow the three-board model documented in:
- `docs/MANUAL.md#project-tracking`
- `docs/playbooks/GITHUB_PROJECT_AUTOMATION.md`

Execution rules:
- Pull work from `bominal Agent Command` queue state, not ad-hoc branch-first selection.
- Do not implement without a linked issue in `Ready` state.
- Keep one `area:*` domain per implementation item and PR path-set (hard domain lock).
- Follow claim checkpoints in order: `Claimed` -> `Design Note Posted` -> `Draft PR Linked`.
- Respect area WIP cap (`1`) to avoid same-domain merge conflicts.
- Ensure PRs use `Closes #...`, pass required checks, and resolve review conversations before merge.
- For policy-scoped PRs, enforce Copilot material-finding disposition (fix or maintainer waiver with risk note).

## Current Infrastructure Reality

Infrastructure automation is being rebuilt. Policy is prescriptive and target-state in `docs/MANUAL.md`, while some CI/CD/deploy tooling is currently absent from tracked repo files.
