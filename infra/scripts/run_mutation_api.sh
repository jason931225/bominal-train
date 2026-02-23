#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
API_DIR="$ROOT_DIR/api"

cd "$API_DIR"

echo "→ Running API mutation smoke gate"
if python3 -c "import pytest" >/dev/null 2>&1; then
  python3 "$API_DIR/scripts/mutation_smoke.py"
  exit $?
fi

if command -v docker >/dev/null 2>&1; then
  echo "→ Local Python deps missing; using api container for mutation smoke."
  docker compose -f "$ROOT_DIR/infra/docker-compose.yml" exec -T api python /app/scripts/mutation_smoke.py
  exit $?
fi

echo "ERROR: pytest dependency is unavailable and docker fallback is not available." >&2
exit 1
