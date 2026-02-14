# New Module or Feature Workflow

## Objective

Provide a deterministic workflow for adding a new module or feature with permissions, guardrails, lock coordination, testing, docs updates, and deploy consistency.

## Scope

- In scope: planning, implementation workflow, approvals, verification, and documentation lifecycle.
- Out of scope: module-specific business logic details.

## Preconditions

- Docs-first gate completed (`AGENTS.md`).
- Active scope lock acquired in `docs/LOCK.md` before file edits.
- Relevant owner approvals received for high-risk actions.

## Deterministic Procedure

1. **Docs-first read**
   - Read `AGENTS.md`.
   - Read `docs/INTENT_ROUTING.md`.
   - Read `docs/README.md` (resolve canonical pointers).
   - Read `docs/EXECUTION_PROTOCOL.md`, `docs/PERMISSIONS.md`, `docs/GUARDRAILS.md`, `docs/DOCUMENTATION_WORKFLOW.md`.
   - Read `docs/ARCHITECTURE.md` and domain docs relevant to the feature.

2. **Permission and guardrail check**
   - Classify planned actions (read/write/destructive/external/security-sensitive).
   - Require explicit approval for risky categories.
   - Fail closed if policy is ambiguous; guardrails override all permission defaults.

3. **Scope lock and cross-scope requests**
   - Add `ACTIVE` lock for minimal scope in `docs/LOCK.md`.
   - If out-of-scope edits are needed, submit exact commands in `docs/REQUEST.md`.
   - Wait for owner completion and close request after verification.

4. **Implementation and tests**
   - Implement within locked scope only.
   - Use TDD for production code changes.
   - Run focused tests first, then required baseline checks.

5. **Verification**
   - Run relevant code tests for touched modules.
   - Run docs checks when docs changed:
     - `bash infra/tests/test_docs_pointers.sh`
     - `bash infra/tests/test_changelog.sh`
   - Run env/deploy checks if relevant:
     - `bash infra/tests/test_env_utils.sh`
     - `bash infra/tests/test_predeploy_check.sh`

6. **Docs-last gate**
   - Update affected docs in `docs/`.
   - Register any new canonical docs/playbooks in `docs/README.md`.
   - Update `CHANGELOG.md` with commit-based entries for notable changes.

7. **Deploy consistency check**
   - Ensure docs and commands align to current canonical deploy path.
   - Current canonical path: `infra/scripts/deploy.sh` (until migration policy changes).

8. **Release lock**
   - Mark lock `RELEASED` in `docs/LOCK.md` and commit lock release.

## Verification Checkpoints

- No edits outside `ACTIVE` lock scope.
- Required approvals are present for risky operations.
- Tests and validators pass.
- Pointer library and changelog are updated.
- Deploy references are consistent with current canonical script.

## Failure Modes and Recovery

- **Lock conflict**
  - Detection: overlapping `ACTIVE` scope.
  - Recovery: use `docs/REQUEST.md`, no direct out-of-scope edits.
- **Policy ambiguity**
  - Detection: unclear whether action is permitted.
  - Recovery: stop and request explicit clarification.
- **Docs drift**
  - Detection: behavior changed but docs/changelog not updated.
  - Recovery: complete docs-last gate and rerun validators.

## Artifacts and Pointers

- `docs/README.md`
- `docs/EXECUTION_PROTOCOL.md`
- `docs/PERMISSIONS.md`
- `docs/DOCUMENTATION_WORKFLOW.md`
- `docs/playbooks/daily-operations-chores.md`
- `CHANGELOG.md`

## Change History

- [0d84ae8] Initial new module/feature standardized workflow.
