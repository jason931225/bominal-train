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
- [PTR-GOV-010] `docs/governance/SECRETS_RESIDENCY_POLICY.md` - authoritative secret residency and source-of-truth contracts.
- [PTR-GOV-011] `docs/governance/FREE_TIER_BUDGET_POLICY.md` - hosted free-tier threshold and evidence policy.

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
- [PTR-PLAN-004] `docs/plans/active/2026-03-01-rust-leptos-ssr-cutover.md` - executable plan for Rust 2024 + Leptos SSR migration track.
- [PTR-PLAN-005] `docs/plans/active/2026-03-02-rust-backend-parity-continuation.md` - task-by-task implementation plan for Rust backend parity completion (API + worker + data lifecycle).
- [PTR-PLAY-001] `docs/playbooks/README.md` - playbook index.
- [PTR-PLAY-002] `docs/playbooks/TEMPLATE.md` - playbook template.
- [PTR-PLAY-003] `docs/playbooks/release-1.0.0-deploy-readiness.md` - release readiness and deployment execution playbook for version 1.0.0.
- [PTR-PLAY-004] `docs/playbooks/db-deep-dive-production.md` - production DB-path latency investigation and deploy-gate workflow.
- [PTR-PLAY-005] `docs/playbooks/train-realtime-cutover.md` - phased train live-event cutover playbook for Supabase Realtime primary transport with SSE fallback/deprecation lifecycle.

### Registry and Compliance

- [PTR-DOCS-001] `docs/deprecations/registry.json` - deprecation registry.
- [PTR-DOCS-002] `docs/deprecations/2026-02-14-inventory.md` - historical deprecation inventory.
- [PTR-DOCS-003] `docs/security/compliance-matrix.md` - PCI/OWASP compliance mapping.
- [PTR-DOCS-004] `docs/releases/README.md` - versioning policy.
- [PTR-DOCS-005] `docs/releases/version-map.json` - version map.
- [PTR-DOCS-006] `docs/artifacts/2026-02-24-gcp-free-tier-offering.md` - user-captured snapshot of GCP free tier products and limits.
- [PTR-DOCS-007] `docs/artifacts/2026-02-27-full-audit.md` - full release/deploy readiness audit snapshot with gate outcomes and blockers.
- [PTR-DOCS-008] `docs/handoff/README.md` - handoff discoverability index entrypoint.
- [PTR-DOCS-009] `docs/handoff` - handoff docs folder container for external rewrite references.
- [PTR-DOCS-010] `docs/handoff/RESOURCE_INDEX.md` - pointer index of source, capture, and probe resources for external rewrite.
- [PTR-DOCS-011] `docs/handoff/EXTERNAL_REWRITE_RUNBOOK.md` - execution sequence and acceptance checks for rewrite teams.
- [PTR-DOCS-012] `docs/handoff/NON_REPO_ARTIFACTS.md` - explicit list of non-portable external artifacts and handling rules.
- [PTR-DOCS-013] `docs/handoff/HANDOFF_EXTERNAL_REWRITE.md` - canonical external rewrite handoff package for provider parity work.
- [PTR-DOCS-014] `docs/handoff/PROVIDER_CONTRACT.md` - provider contract reference migrated from tools.
- [PTR-DOCS-015] `docs/handoff/PROVIDER_FIELD_MAP.json` - machine-readable provider endpoint/field/auth map.
- [PTR-DOCS-016] `docs/handoff/PROVIDER_FIELD_MAP.md` - human-readable provider endpoint/field/auth map.
- [PTR-DOCS-017] `docs/handoff/RESOURCE_MANIFEST.json` - machine-readable handoff resource manifest.
- [PTR-DOCS-018] `docs/handoff/output/auth_scope_probe/latest_auth_scope_probe.json` - auth-scope probe evidence snapshot.
- [PTR-DOCS-019] `docs/handoff/RUST_IMPLEMENTATION_FILE_MANIFEST.md` - authoritative inventory of files created for Rust migration scope.
- [PTR-DOCS-020] `docs/handoff/RUST_BACKEND_PARITY.md` - legacy-to-rust backend parity matrix for route/worker cutover tracking.

### Validation

- [PTR-OPS-001] `infra/tests/test_docs_pointers.sh` - pointer library validator.
- [PTR-OPS-002] `infra/tests/test_intent_routing.sh` - intent routing validator.
- [PTR-OPS-003] `infra/tests/test_docs_consistency.sh` - docs consistency validator.
- [PTR-OPS-004] `infra/tests/test_docs_audience_split.sh` - audience-split structure validator.
- [PTR-OPS-005] `infra/tests/test_versioning.sh` - version mapping validator.
- [PTR-OPS-006] `infra/tests/test_docs_no_duplicate_security_sections.sh` - duplicate section guard for security documentation.
- [PTR-OPS-007] `infra/tests/test_policy_runtime_parity.sh` - policy/runtime parity validator for payment and secret-source contracts.
- [PTR-OPS-008] `infra/tests/test_secret_residency_contract.sh` - secret residency contract validator for env/policy alignment.
- [PTR-OPS-009] `infra/tests/test_payment_boundary_regressions.sh` - payment boundary regression and forbidden-pattern validator.
- [PTR-OPS-010] `infra/tests/test_sync_edge_secrets_from_gsm.sh` - edge secret sync automation validator.
- [PTR-OPS-011] `infra/tests/test_free_tier_status_report.sh` - free-tier status report automation validator.
- [PTR-OPS-012] `infra/tests/test_ensure_uv_api_venv.sh` - uv-managed API venv/bootstrap validator.
