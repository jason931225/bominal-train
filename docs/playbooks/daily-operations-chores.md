# Daily Operations and Chores Playbook

## Objective

Provide a deterministic, low-token workflow for routine engineering/ops maintenance.

## Scope

- In scope: health checks, verification, docs/changelog hygiene, deploy-precheck hygiene.
- Out of scope: feature implementation details.

## Preconditions

- Docs-first gate completed (`AGENTS.md`).
- Branch/PR scope identified.
- Access to local compose and infra scripts.

## Deterministic Procedure

1. Read pointers:
   - `docs/INTENT_ROUTING.md`
   - `docs/README.md`
   - `docs/agents/EXECUTION_PROTOCOL.md`
2. Check repository status:
   - `git status --short`
3. Run routine validations:
   - `bash infra/tests/test_docs_pointers.sh`
   - `bash infra/tests/test_docs_audience_split.sh`
   - `bash infra/tests/test_changelog.sh`
   - `bash infra/tests/test_env_utils.sh`
   - `bash infra/tests/test_deprecation_policy.sh`
   - `bash infra/tests/test_deprecation_references.sh`
   - `bash infra/tests/test_predeploy_check.sh`
4. Run docs consistency checks:
   - `bash infra/tests/test_intent_routing.sh`
   - `bash infra/tests/test_docs_consistency.sh`
5. If env/deploy files changed, run:
   - `bash infra/scripts/predeploy-check.sh --skip-smoke-tests`
6. Update docs/changelog for notable changes.
7. Re-run relevant validators before completion.

## Failure Modes and Recovery

- Validation failure:
  - Recovery: fix root cause, rerun failed command, rerun full checklist.
- Docs drift:
  - Recovery: update docs/changelog and rerun checks.

## Security and Redaction

- Never commit secrets.
- Do not store raw sensitive payloads in docs.

## Artifacts and Pointers

- `docs/README.md`
- `docs/agents/EXECUTION_PROTOCOL.md`
- `docs/governance/DOCUMENTATION_POLICY.md`
- `CHANGELOG.md`
