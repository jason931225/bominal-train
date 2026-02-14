# Daily Operations and Chores Playbook

## Objective

Provide a deterministic, low-token workflow for routine engineering/ops tasks so sessions do not repeatedly rediscover the same process.

## Scope

- In scope: daily health checks, light maintenance, verification, docs/changelog hygiene, safe deploy checks.
- Out of scope: feature implementation details, provider-specific reverse engineering.

## Preconditions

- Docs-first gate completed (`AGENTS.md`).
- Lock scope acquired in `docs/LOCK.md` for any edits.
- Access to local compose and infra scripts.

## Inputs

### Dependency-derived inputs

- Current git state, active locks, open requests.
- Service/container health.
- Existing changelog and docs pointers.

### Non-dependency inputs

- Task ticket/goal for the day.
- Environment target (dev/prod simulation).

## Deterministic Procedure

1. Read pointers:
   - `docs/INTENT_ROUTING.md`
   - `docs/README.md`
   - `docs/EXECUTION_PROTOCOL.md`
2. Check active coordination ledgers:
   - `docs/LOCK.md`
   - `docs/REQUEST.md`
3. Check repository status:
   - `git status --short`
4. Run routine validations:
   - `bash infra/tests/test_docs_pointers.sh`
   - `bash infra/tests/test_changelog.sh`
   - `bash infra/tests/test_env_utils.sh`
   - `bash infra/tests/test_deprecation_policy.sh`
   - `bash infra/tests/test_deprecation_references.sh`
   - `bash infra/tests/test_predeploy_check.sh`
5. Run baseline Python infra tests:
   - `python3 -m unittest discover -s infra/tests -p 'test_*.py'`
6. Run docs consistency checks:
   - `bash infra/tests/test_intent_routing.sh`
   - `bash infra/tests/test_docs_consistency.sh`
7. If env/deploy files changed, run:
   - `bash infra/scripts/predeploy-check.sh --skip-smoke-tests`
8. Update docs/changelog for notable changes:
   - `CHANGELOG.md` under `## Unreleased` with commit-based entries.
9. Re-run validations before completion.

## Token-Saving Search and Navigation Chores

Use these before broad file reads:

1. Find files quickly (prefer `rg`):
   - `rg --files | rg '<keyword-or-path-fragment>'`
2. Find exact code/docs locations:
   - `rg -n '<pattern>' <target-dir>`
3. Limit scope before opening files:
   - `rg -n '<pattern>' docs/ api/ infra/ web/`
4. Inspect only relevant lines:
   - `sed -n '<start>,<end>p' <file>`
5. Prioritize pointer-driven reads:
   - resolve `docs/README.md` entries first, then open only linked docs.
6. For repeated chores, capture once in playbook instead of rediscovering.

Default rule: do not scan entire trees when a pointer or `rg` query can narrow scope first.

## Verification Checkpoints

- Pointer/changelog checks are green.
- Infra shell tests are green.
- No unresolved lock conflicts.
- `CHANGELOG.md` includes current notable changes.

## Failure Modes and Recovery

- Validation failure:
  - Detection: non-zero exit from test scripts.
  - Recovery: fix root cause, rerun failed command, then rerun full checklist.
- Lock conflict:
  - Detection: overlapping active scope in `docs/LOCK.md`.
  - Recovery: use `docs/REQUEST.md` with exact commands and wait for owner completion.
- Docs drift:
  - Detection: behavior changed but docs/changelog not updated.
  - Recovery: update relevant docs + changelog, rerun doc checks.

## Security and Redaction

- Never commit secrets from env files.
- Do not copy raw sensitive payloads into docs.
- Keep operational artifacts redacted.

## Artifacts and Pointers

- `docs/README.md`
- `docs/EXECUTION_PROTOCOL.md`
- `docs/DOCUMENTATION_WORKFLOW.md`
- `CHANGELOG.md`

## Change History

- [0d84ae8] Initial daily operations/chores playbook added.
- [0d84ae8] Added token-saving search/navigation chore checklist (`rg`-first workflow).
