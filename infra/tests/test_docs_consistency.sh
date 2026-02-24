#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

AGENTS="$ROOT_DIR/AGENTS.md"
EXEC_PROTOCOL="$ROOT_DIR/docs/agents/EXECUTION_PROTOCOL.md"
DEPLOYMENT="$ROOT_DIR/docs/humans/operations/DEPLOYMENT.md"
RUNBOOK="$ROOT_DIR/docs/humans/operations/RUNBOOK.md"
README="$ROOT_DIR/README.md"
DEPRECATION_POLICY="$ROOT_DIR/docs/governance/DEPRECATION_POLICY.md"
DOCS_INDEX="$ROOT_DIR/docs/README.md"
INTENT_ROUTING="$ROOT_DIR/docs/INTENT_ROUTING.md"
CONTRIBUTING="$ROOT_DIR/docs/humans/engineering/CONTRIBUTING.md"

for f in "$AGENTS" "$EXEC_PROTOCOL" "$DEPLOYMENT" "$RUNBOOK" "$README" "$DEPRECATION_POLICY" "$DOCS_INDEX" "$INTENT_ROUTING" "$CONTRIBUTING"; do
  if [[ ! -f "$f" ]]; then
    echo "ERROR: expected file missing: $f" >&2
    exit 1
  fi
done

canonical="infra/scripts/deploy.sh"
for f in "$AGENTS" "$EXEC_PROTOCOL" "$DEPLOYMENT" "$RUNBOOK" "$README"; do
  if ! grep -Fq "$canonical" "$f"; then
    echo "ERROR: canonical deploy script missing in $f: $canonical" >&2
    exit 1
  fi
done

for f in "$AGENTS" "$EXEC_PROTOCOL"; do
  if grep -Eq 'infra/scripts/fetch_ci\.sh|infra/scripts/deploy\.prod\.sh' "$f"; then
    echo "ERROR: non-canonical deploy script reference found in $f" >&2
    exit 1
  fi
done

for f in "$AGENTS" "$README" "$DEPLOYMENT" "$RUNBOOK" "$CONTRIBUTING"; do
  if grep -Eq '(^|[[:space:]`])docker-compose([[:space:]`]|$)' "$f"; then
    echo "ERROR: use '\''docker compose'\'' style instead of '\''docker-compose'\'' in $f" >&2
    exit 1
  fi
done

for f in "$README" "$DEPLOYMENT" "$RUNBOOK" "$AGENTS"; do
  if ! grep -Fq "docs/governance/DEPRECATION_POLICY.md" "$f"; then
    echo "ERROR: missing canonical deprecation policy reference in $f" >&2
    exit 1
  fi
done

if ! grep -Fq "docs/plans/archive/2026-02-14-program-closure-report.md" "$DOCS_INDEX"; then
  echo "ERROR: docs/README.md must reference the Stage 8 closure report pointer" >&2
  exit 1
fi

if ! grep -Fq "docs/plans/active/README.md" "$INTENT_ROUTING"; then
  echo "ERROR: docs/INTENT_ROUTING.md must route plan intent to active plan state marker" >&2
  exit 1
fi

for f in "$AGENTS" "$EXEC_PROTOCOL" "$DOCS_INDEX"; do
  if grep -Eq 'docs/(LOCK|REQUEST)\.md' "$f"; then
    echo "ERROR: retired lock/request ledgers referenced in $f" >&2
    exit 1
  fi
done

echo "OK: docs consistency checks passed."
