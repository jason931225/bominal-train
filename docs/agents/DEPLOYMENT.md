# Agent Deployment Overlay

This file defines agent-specific deploy constraints.

- Production is read-only by default.
- Deploy mutations require explicit approval for approval-gated actions.
- Use the canonical deploy script path: `infra/scripts/deploy.sh`.
- Do not use destructive production commands outside approved break-glass windows.
- Every deploy-affecting change must include rollback instructions and verification signals.

Canonical human procedure:
- `docs/humans/operations/DEPLOYMENT.md`

Canonical policy:
- `docs/governance/PRODUCTION_POLICY.md`
- `docs/governance/CHANGE_MANAGEMENT.md`
