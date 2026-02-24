# Intent Routing

Keyword-to-pointer routing map to reduce discovery cost and token usage.

## Fast Path

1. Map intent keyword to pointers below.
2. Open mapped canonical docs first.
3. Use scoped search in mapped areas.

## Keyword Map

| Intent keyword | Primary pointers | Secondary pointers |
|---|---|---|
| `governance` / `policy` | `docs/governance/README.md` | `docs/README.md` |
| `approval` / `permission` / `access` | `docs/governance/APPROVALS_AND_PERMISSIONS.md` | `docs/agents/PERMISSIONS.md` |
| `guardrail` / `safety` | `docs/agents/GUARDRAILS.md` | `docs/governance/SECURITY_POLICY.md` |
| `deploy` | `docs/humans/operations/DEPLOYMENT.md` | `docs/governance/CHANGE_MANAGEMENT.md` |
| `rollback` | `docs/humans/operations/DEPLOYMENT.md` | `docs/humans/operations/RUNBOOK.md` |
| `incident` | `docs/governance/INCIDENT_MANAGEMENT.md` | `docs/humans/operations/RUNBOOK.md` |
| `quality` / `testing` / `warnings` | `docs/governance/ENGINEERING_QUALITY.md` | `docs/humans/engineering/CONTRIBUTING.md` |
| `reliability` / `observability` | `docs/governance/RELIABILITY_OBSERVABILITY.md` | `docs/governance/PRODUCTION_POLICY.md` |
| `security` / `cde` / `pci` | `docs/governance/SECURITY_POLICY.md` | `docs/humans/security/SECURITY.md`, `docs/security/compliance-matrix.md` |
| `deprecate` / `removal` | `docs/governance/DEPRECATION_POLICY.md` | `docs/deprecations/registry.json` |
| `docs` / `documentation` | `docs/governance/DOCUMENTATION_POLICY.md` | `docs/README.md` |
| `architecture` | `docs/humans/engineering/ARCHITECTURE.md` | `docs/governance/PRODUCTION_POLICY.md` |
| `contributing` | `docs/humans/engineering/CONTRIBUTING.md` | `docs/governance/ENGINEERING_QUALITY.md` |
| `plan` | `docs/plans/active/README.md` | `docs/plans/README.md` |
| `playbook` | `docs/playbooks/README.md` | `docs/governance/DOCUMENTATION_POLICY.md` |
| `read` / `clean` / `hygiene` | `docs/playbooks/daily-operations-chores.md` | `docs/governance/DOCUMENTATION_POLICY.md` |
| `resy` / `provider` | `docs/provider-research/README.md` | `docs/playbooks/restaurant-provider-adapter-workflow.md` |

## Search Chores

- Find files: `rg --files | rg '<keyword>'`
- Find scoped text: `rg -n '<pattern>' docs/<target>`
- Read ranges: `sed -n '<start>,<end>p' <file>`
