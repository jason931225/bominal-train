#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

required=(
  "docs/START_HERE.md"
  "docs/governance/README.md"
  "docs/governance/PRODUCTION_POLICY.md"
  "docs/governance/CHANGE_MANAGEMENT.md"
  "docs/governance/SECURITY_POLICY.md"
  "docs/governance/DEPRECATION_POLICY.md"
  "docs/governance/DOCUMENTATION_POLICY.md"
  "docs/governance/ENGINEERING_QUALITY.md"
  "docs/governance/RELIABILITY_OBSERVABILITY.md"
  "docs/governance/INCIDENT_MANAGEMENT.md"
  "docs/governance/APPROVALS_AND_PERMISSIONS.md"
  "docs/humans/README.md"
  "docs/humans/engineering/ARCHITECTURE.md"
  "docs/humans/engineering/CONTRIBUTING.md"
  "docs/humans/operations/DEPLOYMENT.md"
  "docs/humans/operations/RUNBOOK.md"
  "docs/humans/security/SECURITY.md"
  "docs/agents/README.md"
  "docs/agents/GUARDRAILS.md"
  "docs/agents/PERMISSIONS.md"
  "docs/agents/EXECUTION_PROTOCOL.md"
  "docs/agents/DEPLOYMENT.md"
  "docs/agents/DOCUMENTATION.md"
)

for rel in "${required[@]}"; do
  if [[ ! -f "$ROOT_DIR/$rel" ]]; then
    echo "ERROR: missing required audience-split doc: $rel" >&2
    exit 1
  fi
done

retired=(
  "docs/ARCHITECTURE.md"
  "docs/CONTRIBUTING.md"
  "docs/DEPLOYMENT.md"
  "docs/RUNBOOK.md"
  "docs/SECURITY.md"
  "docs/EXECUTION_PROTOCOL.md"
  "docs/PERMISSIONS.md"
  "docs/GUARDRAILS.md"
  "docs/DOCUMENTATION_WORKFLOW.md"
  "docs/DEPRECATION_WORKFLOW.md"
  "docs/LOCK.md"
  "docs/REQUEST.md"
)

for rel in "${retired[@]}"; do
  if [[ -e "$ROOT_DIR/$rel" ]]; then
    echo "ERROR: retired legacy doc still present: $rel" >&2
    exit 1
  fi
done

for rel in "docs/overhaul1" "docs/overhaul2"; do
  if [[ -e "$ROOT_DIR/$rel" ]]; then
    echo "ERROR: overhaul reference directory must be removed: $rel" >&2
    exit 1
  fi
done

echo "OK: audience-split docs layout is valid."
