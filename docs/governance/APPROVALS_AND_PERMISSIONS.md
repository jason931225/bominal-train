# Approvals and Permissions

## Approval-Gated Actions

Always require explicit approval and audit context for:
- destructive actions
- production writes
- security/IAM/firewall/access-control changes
- payment/CDE boundary changes
- egress allowlist or TLS verification policy changes
- external messaging and financial actions

## Approval Record Requirements

Record must include:
- intended change (diff or exact commands)
- reason and blast radius
- rollback plan
- approval reference

## Environment Defaults

- dev: permissive iteration, still no secret leakage
- staging: limited writes
- prod: read-only by default
