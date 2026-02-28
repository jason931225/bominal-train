#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${BOMINAL_ROOT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
API_DIR="$ROOT_DIR/api"
VENV_PATH="${API_VENV_PATH:-$API_DIR/.venv}"
VENV_PYTHON="$VENV_PATH/bin/python"

log_info() { echo "[INFO] $*" >&2; }
log_error() { echo "[ERROR] $*" >&2; }

if ! command -v uv >/dev/null 2>&1; then
  log_error "uv is required. Install it first (for example: curl -LsSf https://astral.sh/uv/install.sh | sh)."
  exit 1
fi

if [[ ! -d "$API_DIR" ]]; then
  log_error "API directory not found: $API_DIR"
  exit 1
fi

if [[ ! -x "$VENV_PYTHON" ]]; then
  log_info "Creating API virtualenv via uv: $VENV_PATH"
  uv venv "$VENV_PATH" >/dev/null
fi

if ! uv run --python "$VENV_PYTHON" python -c "import pytest" >/dev/null 2>&1; then
  log_info "pytest is missing in API venv; installing via uv pip"
  if [[ -f "$API_DIR/requirements.txt" ]]; then
    uv pip install --python "$VENV_PYTHON" -r "$API_DIR/requirements.txt" >/dev/null
  else
    uv pip install --python "$VENV_PYTHON" pytest >/dev/null
  fi
fi

printf '%s\n' "$VENV_PYTHON"
