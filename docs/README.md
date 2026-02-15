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
11. `DEPRECATION_WORKFLOW.md` - deprecation lifecycle policy and enforcement model
12. `playbooks/README.md` - complex-task playbook index

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
- [PTR-DOCS-010] `docs/plans/README.md` - active/archive planning lifecycle policy.
- [PTR-DOCS-011] `docs/deprecations/2026-02-14-inventory.md` - deprecation inventory with owner/replacement/removal-gate tracking.
- [PTR-DOCS-012] `docs/DEPRECATION_WORKFLOW.md` - canonical deprecation lifecycle policy across local, GitHub, and production.
- [PTR-DOCS-013] `docs/deprecations/registry.json` - machine-validated deprecation registry used by CI/deploy guards.

### Program Plans and Closure Artifacts

- [PTR-PLAN-001] `docs/plans/archive/2026-02-11-bominal-grand-restructure-plan.md` - archived top-level restructure umbrella with stage links.
- [PTR-PLAN-002] `docs/todo/backend-production-readiness.md` - archived backend production-readiness tracker and implementation record.
- [PTR-PLAN-003] `docs/plans/archive/2026-02-14-backlog-status-report.md` - archived restructure/backlog execution snapshot.
- [PTR-PLAN-004] `docs/plans/archive/2026-02-14-stage2-worker-split-queue-contracts.md` - archived Stage 2 queue-domain isolation execution plan.
- [PTR-PLAN-005] `docs/plans/archive/2026-02-14-stage3-restaurant-partial-exposure.md` - archived Stage 3 module capability exposure execution plan.
- [PTR-PLAN-006] `docs/plans/archive/2026-02-14-stage4-restaurant-policy-enforcement.md` - archived Stage 4 restaurant policy/lease execution plan.
- [PTR-PLAN-007] `docs/plans/archive/2026-02-14-stage5-infra-deploy-hardening.md` - archived Stage 5 deploy hardening execution plan.
- [PTR-PLAN-008] `docs/plans/archive/2026-02-14-stage6-safe-deprecation-cleanup.md` - archived Stage 6 deprecation cleanup execution plan.
- [PTR-PLAN-009] `docs/plans/archive/2026-02-14-stage7-docs-canonization.md` - archived Stage 7 docs canonization execution plan.
- [PTR-PLAN-010] `docs/plans/archive/2026-02-14-stage8-program-closure-and-archival-hygiene.md` - archived Stage 8 closure and archival hygiene execution plan.
- [PTR-PLAN-011] `docs/plans/archive/2026-02-14-program-closure-report.md` - final closure report for the restructure program.
- [PTR-PLAN-012] `docs/plans/active/README.md` - current active-plan state marker.
- [PTR-PLAN-013] `docs/plans/active/2026-02-14-stage9-performance-optimization.md` - active Stage 9 backend-first performance optimization execution plan.

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
- [PTR-OPS-004] `infra/tests/test_execution_ledgers.sh` - validates lock/request ledger structure and template safety.

For agent-focused instructions, also read the root `AGENTS.md`.
