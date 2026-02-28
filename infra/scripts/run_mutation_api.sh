#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
API_DIR="$ROOT_DIR/api"
ENSURE_UV_API_VENV_SCRIPT="$ROOT_DIR/infra/scripts/ensure-uv-api-venv.sh"

cd "$API_DIR"

echo "→ Running API mutation smoke gate"
if command -v uv >/dev/null 2>&1 && [[ -x "$ENSURE_UV_API_VENV_SCRIPT" ]]; then
  if venv_python="$(bash "$ENSURE_UV_API_VENV_SCRIPT")"; then
    echo "→ Using uv-managed API venv for mutation smoke."
    uv run --python "$venv_python" python "$API_DIR/scripts/mutation_smoke.py"
    exit $?
  fi
  echo "→ uv detected, but API venv bootstrap failed; attempting fallback paths."
fi

if command -v python3 >/dev/null 2>&1; then
  if python3 -c "import pytest" >/dev/null 2>&1; then
    echo "→ Using host python3 interpreter for mutation smoke."
    python3 "$API_DIR/scripts/mutation_smoke.py"
    exit $?
  fi
  echo "→ python3 detected without pytest; attempting docker fallback."
fi

if command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
  echo "→ Using docker compose run fallback for mutation smoke."
  docker compose -f "$ROOT_DIR/infra/docker-compose.yml" run --rm --no-deps api python /app/scripts/mutation_smoke.py
  exit $?
fi

echo "ERROR: unable to run API mutation smoke gate (uv/python3/docker paths unavailable)." >&2
exit 1
