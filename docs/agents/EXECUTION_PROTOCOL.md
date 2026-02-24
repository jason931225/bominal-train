# Agent Execution Protocol

Canonical agent workflow for implementation and documentation updates in bominal.

## Docs-First Prerequisite

Before edits:
1. Read `AGENTS.md`.
2. Read `docs/agents/GUARDRAILS.md`.
3. Read `docs/agents/PERMISSIONS.md`.
4. Read this file.
5. Route intent through `docs/INTENT_ROUTING.md` and `docs/README.md`.

## Mandatory Gate: Docs > Plan > Test

Before staging:
1. Confirm policy/docs requirements.
2. Confirm/refresh implementation plan.
3. Execute tests for touched scope.
4. Verify each behavior change has directly relevant tests.
5. Resolve warnings in touched scope or record explicit owner/rationale.

## Docs-Last Prerequisite

Before completion claims:
1. Re-read changed docs.
2. Ensure docs match final behavior.
3. Ensure `CHANGELOG.md` has commit-based notable entries.

## Quality Bar

- Use behavior-level assertions.
- Require negative/boundary checks for critical areas.
- Enforce hybrid coverage floors as minimum guardrails:
  - API line coverage >= 75%
  - Web lines/functions/branches/statements >= 70%
- Coverage-ignore directives are exception-only and require inline rationale.
- Treat `npm warn deprecated` as actionable.

## Critical Path Requirements

Require stronger verification for:
- auth/session/authorization
- crypto/redaction and sensitive data boundaries
- payment/CDE boundaries
- deploy/rollback paths
- For these paths, require assertiveness and mutation-style verification in addition to baseline test/coverage gates.

## Coordination Model

Default coordination is branch + PR workflow.
The lock/request ledger workflow is retired.

## Deployment Rule

Canonical deploy path remains `infra/scripts/deploy.sh`.
Deprecation lifecycle policy is `docs/governance/DEPRECATION_POLICY.md` with registry `docs/deprecations/registry.json`.
