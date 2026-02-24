# Execution Protocol (Dynamic Locking)

Canonical protocol for parallel implementation sessions in bominal.

## Required execution approach

For multi-step tasks, use explicit planning and execution workflow:
- write/refresh a plan
- execute in bounded steps
- verify with objective evidence before completion

## Docs-first prerequisite (mandatory)

Before any execution work starts in any session:
1. Read `AGENTS.md`.
2. Read `docs/README.md`.
3. Read `docs/EXECUTION_PROTOCOL.md`.
4. Read all docs listed under `AGENTS.md` section **First files to read**.
5. Resolve task keywords via `docs/INTENT_ROUTING.md` before broad scans.

No lock acquisition, code edits, or deploy actions are allowed before this prerequisite is complete.

## Docs > Plan > Test gate (mandatory)

Before staging any commit:
1. Confirm documentation requirements first (docs pointers + policy).
2. Confirm/refresh the implementation plan for current scope.
3. Execute tests for the exact touched scope before staging.
4. Verify test relevance and quality for every staged behavior change before `git add` / commit staging.
   - Favor behavior-level assertions over implementation-detail assertions.
   - Require strong negative-path and boundary assertions for critical areas.
   - Use coverage as a risk signal, not a blanket 100% target.
5. Treat coverage-ignore annotations as exception-only:
   - `c8 ignore`, `pragma: no cover`, and similar directives require inline rationale and explicit review justification.
   - If deterministic in-repo tests can cover the branch, add tests instead of introducing ignores.
6. Resolve warning debt in touched scope before staging:
   - runtime warnings
   - deprecation warnings
   - npm/package-manager deprecation warnings
   - toolchain warnings that indicate pending breakage
   If any warning cannot be removed safely in-scope, record explicit rationale and follow-up owner before staging.
7. Run targeted quality gates before staging:
   - Mandatory for critical paths (auth/session, authorization, crypto/redaction, payment/CDE boundaries, deploy/rollback paths):
     - relevant mutation checks and/or invariant tests
   - Optional for low-risk UI/content-only changes when behavior risk is low and covered by direct tests.
   - Avoid vacuous assertions (for example tautological `assert True` / `expect(true).toBe(true)`).

NPM warning policy:
- `npm warn deprecated` must be treated as actionable.
- If not directly fixable in-scope (for example transitive upstream dependency), record:
  - warning text,
  - dependency chain evidence (`npm ls ... --all`),
  - owner and target removal date/version.
- Silent acceptance/suppression of npm warnings is not permitted.

Future-proofing rule:
- During dependency modernization stages, default target is latest stable runtime + package versions.
- Targeted overhauls (including dependency replacement) are permitted when a dependency is deprecated or unmaintained.
- Any deferred upgrade must include an explicit owner and revisit target before staging.

Staging is blocked unless this sequence (`Docs > Plan > Test`) is satisfied.

## Docs-last prerequisite (mandatory)

Before any completion claim, PR creation, or merge:
1. Re-read all relevant docs touched by the task.
2. Ensure docs reflect the final implemented behavior.
3. Ensure no drift against this protocol and active plan requirements.
4. Ensure `CHANGELOG.md` includes commit-based entries for all notable in-scope changes.

No completion/merge action is allowed before this prerequisite is complete.

## Pointer library requirement

Use `docs/README.md` as the canonical pointer library and follow its pointer format convention.
Any new canonical doc/plan introduced by execution must be registered there before completion.

## Changelog requirement (mandatory)

`CHANGELOG.md` is mandatory and commit-based.
For each notable change in behavior, operations, interfaces, docs governance, or deployment:
1. Add an entry under `## Unreleased` in Keep a Changelog categories.
2. Include commit SHA reference in each entry line (short SHA accepted).
3. Keep entries factual and minimal; avoid narrative text.
4. If exclusion is requested, stop and ask for explicit approval.

## Versioning requirement (mandatory)

- Human-readable versioning is commit-parity based and canonicalized in `docs/releases/version-map.json`.
- Validate version mapping before completion for version-related changes:
  - `python3 infra/scripts/version_guard.py validate`
- Commit-to-version resolution must be deterministic and CI-enforced via `infra/tests/test_versioning.sh`.

## Required ledgers

- `docs/LOCK.md` for dynamic ownership scopes.
- `docs/REQUEST.md` for cross-scope requests.
- Keep `Current Entries` separate from non-live template examples in both ledgers.

## Lock lifecycle

1. Acquire minimal `ACTIVE` lock in `docs/LOCK.md` for current stage.
2. Commit lock acquisition (lock-only commit).
3. Implement only within lock scope.
4. Re-check `docs/LOCK.md` before writes and before commits.
5. Release lock (`RELEASED`) at stage completion.
6. Commit lock release (unlock-only commit).

## Request lifecycle

1. If scope conflict exists, open `docs/REQUEST.md` entry with exact commands.
2. Owner executes request (subagent allowed within scope), marks `DONE` with commit SHA.
3. Requester verifies and marks `CLOSED`.

## Guardrails

- No edits outside `ACTIVE` lock scope.
- No silent conflict resolution.
- No completion claims without fresh verification evidence.
- TDD required for production code changes.
- High-quality tests for critical areas are the required quality bar before staging commits.
- Global 100% coverage is not required; apply risk-based thresholds and ratchet over time.
- Test relevance review is mandatory before staging: every staged behavior change must be covered by at least one directly relevant test (not just incidental suite pass).
- Warning hygiene is mandatory before staging: do not carry forward avoidable runtime/deprecation warnings in touched scope.
- For doc-related ambiguity or exceptions: stop and ask for explicit clarification before continuing.
- `docs/GUARDRAILS.md` hard constraints override all process defaults.

## Deployment policy

- Use `infra/scripts/deploy.sh` as the current canonical deployment script.
- Deploy script must include running-container detection, resource/swap preflight, deploy lock, smoke checks, rollback path.
- Deprecations must be registered in `docs/deprecations/registry.json` and follow `docs/DEPRECATION_WORKFLOW.md`.
- Deploy preflight must enforce production deprecation deadlines before deploy mutation.
