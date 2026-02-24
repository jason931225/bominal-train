# Deprecation Policy

Applies to local tooling, CI/workflows, deployment/runtime artifacts, env keys, API contracts, and docs entrypoints.

Machine source of truth: `docs/deprecations/registry.json`.

## Lifecycle States

- `deprecated`
- `removal_scheduled`
- `removed`
- `blocked`

## Default Compatibility Windows

Policy: `prod30_github14_local2`
- production scope: 30 days minimum
- github/CI scope: 14 days minimum
- local scope: 2 local release cycles minimum
- mixed scope: use strictest window

Exceptions require explicit approval metadata in the registry.

## Registry Requirements

Each deprecation entry must include:
- `id`, `surface`, `scope`, `artifact`, `replacement`, `owner`
- `status`, `deprecated_on`, `remove_after`, `window_policy`
- `callers_scan_paths`, `notes`

## Enforcement

Local checks:
- `bash infra/tests/test_deprecation_policy.sh`
- `bash infra/tests/test_deprecation_references.sh`

CI:
- blocking in `.github/workflows/ci-infra-quality-gates.yml`

Production predeploy:
- enforced by `infra/scripts/predeploy-check.sh`
- bypass only with explicit approval (`PREDEPLOY_ALLOW_DEPRECATION_BYPASS=true`)

## Procedure

1. Register deprecation before caller changes.
2. Publish migration/replacement notes in relevant docs.
3. Run local policy/reference checks.
4. Remove callers incrementally.
5. Mark `removed` only after references are gone and checks pass.
6. Add changelog entry with commit reference.

## Rollback

If removal causes production impact:
- rollback using `docs/humans/operations/DEPLOYMENT.md`
- set entry to `blocked` or back to `removal_scheduled` with notes
