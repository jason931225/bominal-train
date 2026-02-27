#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/infra/scripts/ensure-uv-api-venv.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p "$TMP_DIR/bin" "$TMP_DIR/repo/api"
cat >"$TMP_DIR/repo/api/requirements.txt" <<'REQ'
pytest==9.0.2
REQ

cat >"$TMP_DIR/bin/uv" <<'UV'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "${UV_CALLS_FILE:?}"

case "${1:-}" in
  venv)
    venv_path="$2"
    mkdir -p "$venv_path/bin"
    cat >"$venv_path/bin/python" <<'PY'
#!/usr/bin/env bash
exit 0
PY
    chmod +x "$venv_path/bin/python"
    ;;
  run)
    if [[ "$*" == *"import pytest"* ]]; then
      if [[ -f "${PYTEST_INSTALLED_FILE:?}" ]]; then
        exit 0
      fi
      exit 1
    fi
    exit 0
    ;;
  pip)
    if [[ "${2:-}" == "install" ]]; then
      touch "${PYTEST_INSTALLED_FILE:?}"
      exit 0
    fi
    ;;
esac
exit 0
UV
chmod +x "$TMP_DIR/bin/uv"

# First run should create venv and install pytest.
first_out="$({
  env \
    PATH="$TMP_DIR/bin:$PATH" \
    UV_CALLS_FILE="$TMP_DIR/uv.calls" \
    PYTEST_INSTALLED_FILE="$TMP_DIR/pytest.installed" \
    BOMINAL_ROOT_DIR="$TMP_DIR/repo" \
    bash "$SCRIPT"
} 2>&1)"

expected_python="$TMP_DIR/repo/api/.venv/bin/python"
last_line="$(printf '%s\n' "$first_out" | tail -n 1)"
if [[ "$last_line" != "$expected_python" ]]; then
  echo "FAIL: expected output python path '$expected_python', got '$last_line'" >&2
  echo "$first_out" >&2
  exit 1
fi

if ! rg -q '^venv .*/api/\.venv$' "$TMP_DIR/uv.calls"; then
  echo "FAIL: uv venv was not called" >&2
  cat "$TMP_DIR/uv.calls" >&2
  exit 1
fi
if ! rg -q '^pip install --python .*/api/\.venv/bin/python -r .*/api/requirements\.txt$' "$TMP_DIR/uv.calls"; then
  echo "FAIL: uv pip install -r requirements.txt was not called" >&2
  cat "$TMP_DIR/uv.calls" >&2
  exit 1
fi

# Second run should not reinstall pytest.
cp "$TMP_DIR/uv.calls" "$TMP_DIR/uv.calls.before"
second_out="$({
  env \
    PATH="$TMP_DIR/bin:$PATH" \
    UV_CALLS_FILE="$TMP_DIR/uv.calls" \
    PYTEST_INSTALLED_FILE="$TMP_DIR/pytest.installed" \
    BOMINAL_ROOT_DIR="$TMP_DIR/repo" \
    bash "$SCRIPT"
} 2>&1)"

if [[ "$(printf '%s\n' "$second_out" | tail -n 1)" != "$expected_python" ]]; then
  echo "FAIL: second run returned unexpected python path" >&2
  echo "$second_out" >&2
  exit 1
fi

pip_install_count="$(rg -c '^pip install --python .*/api/\.venv/bin/python -r .*/api/requirements\.txt$' "$TMP_DIR/uv.calls")"
if [[ "$pip_install_count" -ne 1 ]]; then
  echo "FAIL: expected exactly one pytest install, got $pip_install_count" >&2
  cat "$TMP_DIR/uv.calls" >&2
  exit 1
fi

echo "OK: ensure-uv-api-venv script tests passed."
