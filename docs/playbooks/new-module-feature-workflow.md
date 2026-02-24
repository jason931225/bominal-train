# New Module or Feature Workflow

## Objective

Provide a deterministic workflow for module/feature delivery with policy, tests, docs, and deploy consistency.

## Preconditions

- Docs-first gate completed (`AGENTS.md`).
- Branch/PR scope defined.
- Required approvals obtained for high-risk actions.

## Deterministic Procedure

1. Docs-first read:
   - `AGENTS.md`
   - `docs/INTENT_ROUTING.md`
   - `docs/README.md`
   - `docs/agents/EXECUTION_PROTOCOL.md`
   - `docs/agents/PERMISSIONS.md`
   - `docs/agents/GUARDRAILS.md`
   - `docs/governance/DOCUMENTATION_POLICY.md`
   - `docs/humans/engineering/ARCHITECTURE.md`
2. Permission and guardrail check.
3. Implementation with TDD and scoped commits.
4. Verification:
   - targeted code tests
   - docs validators when docs changed
5. Docs-last gate:
   - update docs pointers as needed
   - update changelog
6. Deploy consistency check:
   - canonical deploy path remains `infra/scripts/deploy.sh`

## Failure Modes

- Policy ambiguity: stop and request clarification.
- Docs drift: update docs/changelog and rerun validators.

## Pointers

- `docs/README.md`
- `docs/agents/EXECUTION_PROTOCOL.md`
- `docs/agents/PERMISSIONS.md`
- `docs/governance/DOCUMENTATION_POLICY.md`
- `docs/playbooks/daily-operations-chores.md`
- `CHANGELOG.md`
