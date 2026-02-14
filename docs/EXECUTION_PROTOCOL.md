# Execution Protocol (Dynamic Locking)

Canonical protocol for parallel implementation sessions in bominal.

## Required skill

Use `standardized-plan-execution-protocol` as a mandatory requirement before execution.

## Docs-first prerequisite (mandatory)

Before any execution work starts in any session:
1. Read `AGENTS.md`.
2. Read `docs/README.md`.
3. Read `docs/EXECUTION_PROTOCOL.md`.
4. Read all docs listed under `AGENTS.md` section **First files to read**.
5. Resolve task keywords via `docs/INTENT_ROUTING.md` before broad scans.

No lock acquisition, code edits, or deploy actions are allowed before this prerequisite is complete.

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

## Required ledgers

- `docs/LOCK.md` for dynamic ownership scopes.
- `docs/REQUEST.md` for cross-scope requests.

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
- For doc-related ambiguity or exceptions: stop and ask for explicit clarification before continuing.
- `docs/GUARDRAILS.md` hard constraints override all process defaults.

## Deployment policy

- Use `infra/scripts/deploy.sh` as the current canonical deployment script.
- Deploy script must include running-container detection, resource/swap preflight, deploy lock, smoke checks, rollback path.
- Deprecations must be registered in `docs/deprecations/registry.json` and follow `docs/DEPRECATION_WORKFLOW.md`.
- Deploy preflight must enforce production deprecation deadlines before deploy mutation.
