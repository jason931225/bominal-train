# Bominal Docs

This folder is the operational and engineering documentation pack for Bominal.

## Quick start

1. `EXECUTION_PROTOCOL.md` - dynamic lock/request protocol for multi-session execution
2. `ARCHITECTURE.md` - system design and module boundaries
3. `CONTRIBUTING.md` - local setup, coding standards, and testing
4. `DEPLOYMENT.md` - production deployment and environment separation
5. `RUNBOOK.md` - operations, troubleshooting, and recovery steps
6. `SECURITY.md` - security controls, secret handling, and hardening priorities
7. `DOCUMENTATION_WORKFLOW.md` - standardized docs workflow and protocol library
8. `PERMISSIONS.md` - approval, scope, and least-privilege protocol
9. `GUARDRAILS.md` - hard safety constraints and fail-closed behavior
10. `INTENT_ROUTING.md` - keyword-to-pointer routing for token-efficient discovery
11. `playbooks/README.md` - complex-task playbook index

## Canonical Pointer Library

This is the single source of truth for operational and implementation pointers.

Pointer format convention (mandatory):
- Format: `- [PTR-<GROUP>-<NNN>] \`<repo-relative-path>\` - <short description>`
- Use uppercase group names (`CORE`, `DOCS`, `PLAN`, `OPS`, etc.).
- Use stable IDs; do not renumber existing IDs.
- One pointer per line; no nested bullets.
- Paths must be repo-relative and exist.
- Keep descriptions concise and factual.

### Core governance

- [PTR-CORE-001] `AGENTS.md` - agent governance and non-negotiables.
- [PTR-CORE-002] `docs/README.md` - canonical pointer library.
- [PTR-CORE-003] `docs/EXECUTION_PROTOCOL.md` - dynamic locking and request workflow.
- [PTR-CORE-004] `docs/LOCK.md` - active lock ledger for dynamic scope ownership.
- [PTR-CORE-005] `docs/REQUEST.md` - cross-scope request queue.
- [PTR-CORE-006] `CHANGELOG.md` - commit-based project changelog (Keep a Changelog).

### Architecture and delivery

- [PTR-DOCS-001] `docs/ARCHITECTURE.md` - system architecture and boundaries.
- [PTR-DOCS-002] `docs/CONTRIBUTING.md` - coding/testing conventions.
- [PTR-DOCS-003] `docs/DEPLOYMENT.md` - production deployment and rollback.
- [PTR-DOCS-004] `docs/RUNBOOK.md` - operations and incident handling.
- [PTR-DOCS-005] `docs/SECURITY.md` - security controls and requirements.
- [PTR-DOCS-006] `docs/DOCUMENTATION_WORKFLOW.md` - standardized documentation workflow and protocols.
- [PTR-DOCS-007] `docs/PERMISSIONS.md` - permission and approval protocol for code/docs/ops changes.
- [PTR-DOCS-008] `docs/GUARDRAILS.md` - non-negotiable hard safety constraints.
- [PTR-DOCS-009] `docs/INTENT_ROUTING.md` - keyword routing map for low-token document discovery.

### Active implementation plans

- [PTR-PLAN-001] `docs/plans/2026-02-11-bominal-grand-restructure-plan.md` - current top-level restructure plan.
- [PTR-PLAN-002] `docs/todo/backend-production-readiness.md` - backend production-readiness hardening backlog.

### Playbooks

- [PTR-PLAY-001] `docs/playbooks/README.md` - playbook index and usage standard.
- [PTR-PLAY-002] `docs/playbooks/TEMPLATE.md` - mandatory template for complex-task playbooks.
- [PTR-PLAY-003] `docs/playbooks/resy-widget-form-data-capture.md` - Resy form-data capture/replay protocol.
- [PTR-PLAY-004] `docs/playbooks/daily-operations-chores.md` - standardized daily engineering and operations chore workflow.
- [PTR-PLAY-005] `docs/playbooks/new-module-feature-workflow.md` - end-to-end workflow for adding modules/features with policy and docs gates.

### Validation

- [PTR-OPS-001] `infra/tests/test_docs_pointers.sh` - validates pointer section and target paths.
- [PTR-OPS-002] `infra/tests/test_intent_routing.sh` - validates required intent keyword mappings.
- [PTR-OPS-003] `infra/tests/test_docs_consistency.sh` - enforces deployment-policy and docs consistency rules.

For agent-focused instructions, also read the root `AGENTS.md`.
