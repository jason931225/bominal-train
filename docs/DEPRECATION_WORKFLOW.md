# Deprecation Workflow

Standard lifecycle for safe deprecation and removal across local tooling, GitHub CI/workflows, and production deployment/runtime paths.

## Scope

This workflow applies to all interface classes:
- shell scripts and commands
- GitHub workflows/automation interfaces
- deployment/runtime artifacts
- env keys and config contract fields
- API fields/endpoints and docs-facing operator procedures

## Canonical Sources

- Policy and lifecycle: `docs/DEPRECATION_WORKFLOW.md` (this document)
- Machine registry: `docs/deprecations/registry.json`
- Historical inventory notes: `docs/deprecations/2026-02-14-inventory.md`

## Lifecycle States

Every deprecation entry must follow these states:

1. `deprecated`
- artifact still allowed during compatibility window.
- replacement path must be documented.

2. `removal_scheduled`
- compatibility window has started; callers are expected to migrate.
- removal date is fixed in registry.

3. `removed`
- artifact removed from active runtime paths.
- removal evidence is recorded (`removed_on`, optional `removal_commit`).

4. `blocked`
- deprecation cannot proceed due to unresolved dependency/risk.
- unblock conditions must be documented in `notes`.

## Default Compatibility Windows

Window policy: `prod30_github14_local2`.

- Production-facing interfaces (`scope=production`): 30-day minimum window.
- GitHub/CI interfaces (`scope=github`): 14-day minimum window.
- Local-only interfaces (`scope=local`): 2 local release cycles minimum before removal.
- Mixed scope (`scope=mixed`): apply strictest defaults (production minimum).

Exceptions:
- allowed only with explicit approval.
- registry must include:
  - `window_exception_approved: true`
  - `window_exception_reason: "<approved rationale>"`

## Registry Contract

Each entry in `docs/deprecations/registry.json` must include:

- `id`
- `surface`
- `scope`
- `artifact`
- `replacement`
- `owner`
- `status`
- `deprecated_on`
- `remove_after`
- `window_policy`
- `callers_scan_paths`
- `notes`

Optional lifecycle fields:
- `removed_on`
- `removal_commit`
- `allow_reference_paths`
- `local_release_cycles_required`
- `local_release_cycles_completed`
- `window_exception_approved`
- `window_exception_reason`

## Enforcement Matrix

### Local development

- Run:
  - `bash infra/tests/test_deprecation_policy.sh`
  - `bash infra/tests/test_deprecation_references.sh`
- Trigger: local validation chores and infra/doc change validation.
- Expected behavior: fail fast on invalid registry schema, expired windows, or active references to removed artifacts.

### GitHub CI

- Workflow: `.github/workflows/infra-tests.yml`
- Gate type: blocking on both `pull_request` and `push` to `main`.
- Required checks:
  - deprecation policy validation
  - active reference scan

### Production deployment

- Gate location: `infra/scripts/predeploy-check.sh` (invoked by `infra/scripts/deploy.sh`).
- Guard command:
  - `python3 infra/scripts/deprecation_guard.py enforce-deploy ...`
- Failure conditions:
  - removed production artifact still referenced in active runtime paths
  - production deprecation past `remove_after` and not finalized
- Escape hatch (approval required): set `PREDEPLOY_ALLOW_DEPRECATION_BYPASS=true`.

## Operational Procedure

1. Register deprecation in `docs/deprecations/registry.json` before changing runtime references.
2. Publish migration/replacement notes in relevant docs (`README.md`, `docs/DEPLOYMENT.md`, `docs/RUNBOOK.md`, API docs).
3. Run policy/reference checks locally.
4. Remove callers incrementally.
5. Mark status as `removed` only after active references are gone and checks pass.
6. Record change in `CHANGELOG.md` with commit SHA entry.

## Rollback and Incident Handling

- If deprecation removal causes production impact:
  - use documented rollback path (`docs/DEPLOYMENT.md`).
  - restore last known-good runtime configuration.
  - keep registry status accurate (`blocked` or revert to `removal_scheduled` with note).
- For emergency bypass:
  - explicit approval and rollback plan are required by `docs/GUARDRAILS.md` and `docs/PERMISSIONS.md`.
