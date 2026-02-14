# Documentation Workflow Standard

This document defines the production-standard documentation workflow for `bominal`.

## Goals

- Keep documentation discoverable and low-latency for agents/humans.
- Reduce repeated debugging/research for complex tasks.
- Ensure commits, tests, troubleshooting, and rollback procedures stay consistent.

## Source-of-truth hierarchy

1. `AGENTS.md` - mandatory guardrails and execution policy.
2. `docs/EXECUTION_PROTOCOL.md` - lock/request and multi-session protocol.
3. `docs/README.md` - canonical pointer library.
4. Domain docs:
   - `docs/CONTRIBUTING.md`
   - `docs/DEPLOYMENT.md`
   - `docs/RUNBOOK.md`
   - `docs/SECURITY.md`
5. Reusable playbooks:
   - `docs/playbooks/*.md`

If guidance conflicts, follow the stricter rule and document the exception decision.
Guardrails take precedence over all process defaults.

## Standard workflow

### 1) Docs-first

Before planning or coding:
1. Read required files from `AGENTS.md`.
2. Resolve task keywords via `docs/INTENT_ROUTING.md`.
3. Resolve relevant pointers from `docs/README.md`.
4. Check whether an existing playbook already covers the task pattern.

### 2) In-flight updates

During implementation:
1. Update docs in the same stage as the code change.
2. Keep changes minimal and scoped to actual behavior changes.
3. If unclear whether to update or addendum, prefer small additive sections unless structure is broken.

### 3) Docs-last

Before PR/merge/completion:
1. Re-read changed docs for consistency.
2. Verify pointers still resolve and are complete.
3. Add commit-based changelog entries in `CHANGELOG.md`.
4. Run docs validation checks.

## Protocols by concern

### Troubleshooting protocol

For each incident/update:
1. Symptom: user-visible signal and impact.
2. Scope: services/modules affected.
3. Reproduction: exact commands/inputs.
4. Evidence: logs, traces, payloads (redacted).
5. Mitigation: immediate fix/rollback.
6. Root cause: confirmed reason.
7. Prevention: tests, guardrails, monitoring updates.
8. Documentation: runbook or playbook update with links.

### Commit protocol

- Use small, scoped commits with clear intent.
- One logical change set per commit (avoid mixed unrelated changes).
- Every notable behavior/ops/doc change must be reflected in `CHANGELOG.md` with commit reference.
- Keep docs and code changes in the same PR when behavior changed.

### Test protocol

- Test first for behavior changes (TDD for production code).
- Run narrow tests for touched areas, then required baseline suite.
- For docs/protocol changes, run docs validation scripts:
  - `infra/tests/test_docs_pointers.sh`
  - `infra/tests/test_changelog.sh`
  - `infra/tests/test_deprecation_policy.sh`
  - `infra/tests/test_deprecation_references.sh`
- For env/deploy workflow changes, run:
  - `infra/tests/test_env_utils.sh`
  - `infra/tests/test_predeploy_check.sh`

### Deprecation protocol

- Register every deprecation in `docs/deprecations/registry.json` before removal work begins.
- Follow lifecycle and window policy in `docs/DEPRECATION_WORKFLOW.md`.
- Validate policy and references locally before PR and before deploy.
- Production deploy must pass deprecation gate via `predeploy-check.sh`.
- Any bypass requires explicit approval and rollback notes.

### Rollback protocol

- Rollback path must be documented before deploying risky changes.
- Prefer deterministic rollback (known-good commit/image digest).
- Validate health after rollback and document the incident in runbook/playbook.

## Complex-task playbook standard

Use `docs/playbooks/TEMPLATE.md` for any complex task that required non-obvious steps, reverse engineering, or repeated retries.

Mandatory sections:
- Objective and scope
- Preconditions
- Inputs (dependency-derived vs non-dependency)
- Step sequence (deterministic)
- Verification checkpoints
- Failure modes and fallback
- Security/redaction requirements
- Artifacts and references (HAR, scripts, docs)

## Update policy

- Prefer additive updates over broad rewrites.
- Consolidate docs under `docs/` unless strong justification exists.
- Any new canonical document must be registered in `docs/README.md` pointer library.
- Treat `third_party/**` as reference-only documentation; do not treat it as canonical policy for `bominal`.
