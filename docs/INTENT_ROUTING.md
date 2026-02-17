# Intent Routing

Keyword-to-pointer routing map to reduce discovery cost and token usage.

Use this before broad file reads. Resolve intent, then open only mapped canonical docs.

## Fast Path

1. Map user intent keyword to pointers below.
2. Read mapped docs first.
3. Use scoped search only in mapped areas (for example: `rg -n '<pattern>' docs/<target>.md`).
4. Expand scope only if mapped docs are insufficient.

## Keyword Map

| Intent keyword | Primary pointers | Secondary pointers |
|---|---|---|
| `read` | `docs/playbooks/daily-operations-chores.md` | `docs/README.md` |
| `clean` | `docs/playbooks/daily-operations-chores.md` | `docs/DOCUMENTATION_WORKFLOW.md` |
| `hygiene` | `docs/playbooks/daily-operations-chores.md` | `docs/DOCUMENTATION_WORKFLOW.md` |
| `permission` / `approval` / `access` | `docs/PERMISSIONS.md` | `docs/GUARDRAILS.md` |
| `guardrail` / `safety` | `docs/GUARDRAILS.md` | `docs/PERMISSIONS.md` |
| `deploy` | `docs/DEPLOYMENT.md` | `docs/RUNBOOK.md` |
| `rollback` | `docs/DEPLOYMENT.md` | `docs/RUNBOOK.md` |
| `deprecate` / `sunset` / `compatibility window` / `removal` | `docs/DEPRECATION_WORKFLOW.md` | `docs/deprecations/registry.json`, `docs/deprecations/2026-02-14-inventory.md` |
| `lock` / `request` / `conflict` | `docs/EXECUTION_PROTOCOL.md` | `docs/LOCK.md`, `docs/REQUEST.md` |
| `resy` / `widget` / `form data` | `docs/playbooks/resy-widget-form-data-capture.md` | `docs/playbooks/new-module-feature-workflow.md` |
| `opentable` / `otp` / `session` / `human` / `cpr` | `docs/provider-research/opentable-endpoints.md` | `docs/provider-research/restaurant-provider-endpoint-inventory.md` |
| `provider` / `adapter` / `endpoint` / `schema` | `docs/provider-research/restaurant-provider-canonical-contract.md` | `docs/provider-research/restaurant-db-schema-mapping.md`, `docs/playbooks/restaurant-provider-adapter-workflow.md` |
| `catchtable` | `docs/provider-research/restaurant-provider-endpoint-inventory.md` | `docs/provider-research/README.md` |
| `new module` / `new feature` | `docs/playbooks/new-module-feature-workflow.md` | `docs/playbooks/daily-operations-chores.md` |
| `plan` | `docs/plans/active/README.md` | `docs/plans/archive/2026-02-14-program-closure-report.md`, `docs/playbooks/new-module-feature-workflow.md` |

## Search Chores (Token-Efficient)

- Find files: `rg --files | rg '<keyword>'`
- Find text in scoped docs: `rg -n '<pattern>' docs/<target>.md`
- Read only needed ranges: `sed -n '<start>,<end>p' <file>`
- Avoid full-tree scans unless routing fails.
