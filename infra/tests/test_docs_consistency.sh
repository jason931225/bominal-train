#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

AGENTS="$ROOT_DIR/AGENTS.md"
EXEC_PROTOCOL="$ROOT_DIR/docs/EXECUTION_PROTOCOL.md"
DEPLOYMENT="$ROOT_DIR/docs/DEPLOYMENT.md"
RUNBOOK="$ROOT_DIR/docs/RUNBOOK.md"
README="$ROOT_DIR/README.md"

for f in "$AGENTS" "$EXEC_PROTOCOL" "$DEPLOYMENT" "$RUNBOOK" "$README"; do
  if [[ ! -f "$f" ]]; then
    echo "ERROR: expected file missing: $f" >&2
    exit 1
  fi
done

# Current canonical deployment script (temporary policy) must be consistent.
canonical="infra/scripts/deploy.sh"

for f in "$AGENTS" "$EXEC_PROTOCOL" "$DEPLOYMENT" "$RUNBOOK" "$README"; do
  if ! grep -Fq "$canonical" "$f"; then
    echo "ERROR: canonical deploy script missing in $f: $canonical" >&2
    exit 1
  fi
done

# Prohibit active references to not-yet-implemented deployment scripts.
for f in "$AGENTS" "$EXEC_PROTOCOL"; do
  if grep -Eq 'infra/scripts/fetch_ci\.sh|infra/scripts/deploy\.prod\.sh' "$f"; then
    echo "ERROR: non-canonical deploy script reference found in $f" >&2
    exit 1
  fi
done

# Enforce compose command style in high-traffic docs.
for f in "$AGENTS" "$README" "$DEPLOYMENT" "$RUNBOOK" "$ROOT_DIR/docs/CONTRIBUTING.md"; do
  if grep -Eq '(^|[[:space:]`])docker-compose([[:space:]`]|$)' "$f"; then
    echo "ERROR: use 'docker compose' style instead of 'docker-compose' in $f" >&2
    exit 1
  fi
done

echo "OK: docs consistency checks passed."
