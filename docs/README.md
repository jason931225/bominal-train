# Bominal Docs

This folder is the operational and engineering documentation pack for Bominal.

Audience split:
- Governance (policy): `docs/governance/**`
- Humans (procedures): `docs/humans/**`
- Agents (overlays): `docs/agents/**`

Start here: `docs/START_HERE.md`.

## Canonical Pointer Library

Pointer format convention (mandatory):
- Format: `- [PTR-<GROUP>-<NNN>] \`<repo-relative-path>\` - <short description>`
- Use uppercase group names.
- Use stable IDs; do not renumber existing IDs.
- One pointer per line; no nested bullets.
- Paths must be repo-relative and exist.

### Audience Entrypoints

- [PTR-AUD-001] `docs/START_HERE.md` - entrypoint and taxonomy.
- [PTR-AUD-002] `docs/governance/README.md` - governance index and precedence.
- [PTR-AUD-003] `docs/humans/README.md` - human procedures index.
- [PTR-AUD-004] `docs/agents/README.md` - agent overlays index.

### Core Governance

- [PTR-CORE-001] `AGENTS.md` - agent governance and non-negotiables.
- [PTR-CORE-002] `docs/README.md` - canonical pointer library.
- [PTR-CORE-003] `docs/INTENT_ROUTING.md` - keyword routing map.
- [PTR-CORE-004] `CHANGELOG.md` - commit-based changelog.

### Canonical Policy

- [PTR-GOV-001] `docs/governance/PRODUCTION_POLICY.md` - production baseline policy.
- [PTR-GOV-002] `docs/governance/ENGINEERING_QUALITY.md` - testing/quality/warning policy.
- [PTR-GOV-003] `docs/governance/RELIABILITY_OBSERVABILITY.md` - reliability and observability policy.
- [PTR-GOV-004] `docs/governance/CHANGE_MANAGEMENT.md` - change, migration, and rollback policy.
- [PTR-GOV-005] `docs/governance/SECURITY_POLICY.md` - security and CDE policy.
- [PTR-GOV-006] `docs/governance/INCIDENT_MANAGEMENT.md` - incident and break-glass policy.
- [PTR-GOV-007] `docs/governance/DEPRECATION_POLICY.md` - deprecation lifecycle policy.
- [PTR-GOV-008] `docs/governance/DOCUMENTATION_POLICY.md` - documentation policy and anti-drift rules.
- [PTR-GOV-009] `docs/governance/APPROVALS_AND_PERMISSIONS.md` - approvals and auditability policy.

### Human Procedures

- [PTR-HUM-001] `docs/humans/engineering/ARCHITECTURE.md` - architecture reference.
- [PTR-HUM-002] `docs/humans/engineering/CONTRIBUTING.md` - contributor workflow.
- [PTR-HUM-003] `docs/humans/operations/DEPLOYMENT.md` - deployment and rollback procedures.
- [PTR-HUM-004] `docs/humans/operations/RUNBOOK.md` - operations and troubleshooting.
- [PTR-HUM-005] `docs/humans/security/SECURITY.md` - security reference details.

### Agent Overlays

- [PTR-AGT-001] `docs/agents/GUARDRAILS.md` - hard constraints.
- [PTR-AGT-002] `docs/agents/PERMISSIONS.md` - agent approval gates.
- [PTR-AGT-003] `docs/agents/EXECUTION_PROTOCOL.md` - agent execution workflow.
- [PTR-AGT-004] `docs/agents/DEPLOYMENT.md` - deploy overlay for agents.
- [PTR-AGT-005] `docs/agents/DOCUMENTATION.md` - docs workflow overlay for agents.

### Plans and Playbooks

- [PTR-PLAN-001] `docs/plans/README.md` - plans lifecycle policy.
- [PTR-PLAN-002] `docs/plans/active/README.md` - active plans state marker.
- [PTR-PLAN-003] `docs/plans/archive/2026-02-14-program-closure-report.md` - restructure closure report.
- [PTR-PLAY-001] `docs/playbooks/README.md` - playbook index.
- [PTR-PLAY-002] `docs/playbooks/TEMPLATE.md` - playbook template.
- [PTR-PLAY-003] `docs/playbooks/release-1.0.0-deploy-readiness.md` - release readiness and deployment execution playbook for version 1.0.0.

### Registry and Compliance

- [PTR-DOCS-001] `docs/deprecations/registry.json` - deprecation registry.
- [PTR-DOCS-002] `docs/deprecations/2026-02-14-inventory.md` - historical deprecation inventory.
- [PTR-DOCS-003] `docs/security/compliance-matrix.md` - PCI/OWASP compliance mapping.
- [PTR-DOCS-004] `docs/releases/README.md` - versioning policy.
- [PTR-DOCS-005] `docs/releases/version-map.json` - version map.
- [PTR-DOCS-006] `docs/artifacts/2026-02-24-gcp-free-tier-offering.md` - user-captured snapshot of GCP free tier products and limits.
- [PTR-DOCS-007] `docs/artifacts/2026-02-27-full-audit.md` - full release/deploy readiness audit snapshot with gate outcomes and blockers.

### Validation

- [PTR-OPS-001] `infra/tests/test_docs_pointers.sh` - pointer library validator.
- [PTR-OPS-002] `infra/tests/test_intent_routing.sh` - intent routing validator.
- [PTR-OPS-003] `infra/tests/test_docs_consistency.sh` - docs consistency validator.
- [PTR-OPS-004] `infra/tests/test_docs_audience_split.sh` - audience-split structure validator.
- [PTR-OPS-005] `infra/tests/test_versioning.sh` - version mapping validator.
