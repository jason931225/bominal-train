#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
API_DIR="$ROOT_DIR/api"
ENSURE_UV_API_VENV_SCRIPT="$ROOT_DIR/infra/scripts/ensure-uv-api-venv.sh"

cd "$API_DIR"

echo "→ Running API mutation smoke gate"
if command -v uv >/dev/null 2>&1 && [[ -x "$ENSURE_UV_API_VENV_SCRIPT" ]]; then
  venv_python="$(bash "$ENSURE_UV_API_VENV_SCRIPT")"
  uv run --python "$venv_python" python "$API_DIR/scripts/mutation_smoke.py"
  exit $?
fi

if command -v docker >/dev/null 2>&1; then
  echo "→ uv-managed venv unavailable; using api container for mutation smoke."
  docker compose -f "$ROOT_DIR/infra/docker-compose.yml" exec -T api python /app/scripts/mutation_smoke.py
  exit $?
fi

echo "ERROR: uv (with api/.venv) is unavailable and docker fallback is not available." >&2
exit 1
