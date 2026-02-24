# Change Management Standard

## Goals

- ship safely with clear rollback
- keep verification objective
- maintain auditable approvals for risky changes

## Risk Classification

- low: docs-only/UI text/no behavior change
- medium: feature behavior/performance/provider integration
- high: auth, crypto/redaction, payment/CDE, deploy/rollback, security boundaries

## Required Workflow

1. Docs-first: open canonical policy/procedure docs.
2. Plan: bounded scope, risks, tests, rollback.
3. Implement in small reviewable steps.
4. Docs > Plan > Test before staging.
5. Docs-last before completion claims.

## Coordination Model

This repository uses branch/PR coordination as the default concurrency model.
`docs/governance/CHANGE_MANAGEMENT.md` and `docs/governance/CHANGE_MANAGEMENT.md` are retired.

## Migration Safety

Schema changes must follow expand -> migrate/backfill -> contract.

## Deploy Safety

Deploy via `infra/scripts/deploy.sh` and keep rollback path documented.
