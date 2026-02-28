#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT_SRC="$ROOT_DIR/infra/scripts/run_mutation_api.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

HARNESS_ROOT="$TMP_DIR/harness"
mkdir -p "$HARNESS_ROOT/infra/scripts" "$HARNESS_ROOT/api/scripts"
cp "$SCRIPT_SRC" "$HARNESS_ROOT/infra/scripts/run_mutation_api.sh"
chmod +x "$HARNESS_ROOT/infra/scripts/run_mutation_api.sh"

cat >"$HARNESS_ROOT/api/scripts/mutation_smoke.py" <<'EOF_MUTATION'
print("mutation smoke placeholder")
EOF_MUTATION

ENSURE_SCRIPT="$HARNESS_ROOT/infra/scripts/ensure-uv-api-venv.sh"
SYSTEM_DIRNAME="$(command -v dirname)"

make_uv_ensure() {
  local log_file="$1"
  cat >"$ENSURE_SCRIPT" <<EOF_ENSURE
#!/usr/bin/env bash
set -euo pipefail
echo "ensure" >> "$log_file"
printf '%s\n' "/tmp/fake-venv-python"
EOF_ENSURE
  chmod +x "$ENSURE_SCRIPT"
}

make_nonexec_ensure() {
  cat >"$ENSURE_SCRIPT" <<'EOF_ENSURE'
#!/usr/bin/env bash
set -euo pipefail
exit 1
EOF_ENSURE
  chmod -x "$ENSURE_SCRIPT"
}

# Scenario 1: uv path selected when uv exists.
SC1_BIN="$TMP_DIR/sc1-bin"
SC1_LOG="$TMP_DIR/sc1.log"
mkdir -p "$SC1_BIN"
make_uv_ensure "$SC1_LOG"
cat >"$SC1_BIN/uv" <<EOF_UV
#!/usr/bin/env bash
set -euo pipefail
echo "uv \$*" >> "$SC1_LOG"
EOF_UV
chmod +x "$SC1_BIN/uv"
PATH="$SC1_BIN:/bin:/usr/bin" bash "$HARNESS_ROOT/infra/scripts/run_mutation_api.sh" >"$TMP_DIR/sc1.out" 2>"$TMP_DIR/sc1.err"
if ! grep -q "Using uv-managed API venv for mutation smoke." "$TMP_DIR/sc1.out"; then
  echo "FAIL: uv path message missing" >&2
  exit 1
fi
if ! grep -q "uv run --python /tmp/fake-venv-python python $HARNESS_ROOT/api/scripts/mutation_smoke.py" "$SC1_LOG"; then
  echo "FAIL: uv path invocation mismatch" >&2
  cat "$SC1_LOG" >&2 || true
  exit 1
fi

# Scenario 2: python fallback selected when uv is missing and pytest is available.
SC2_BIN="$TMP_DIR/sc2-bin"
SC2_LOG="$TMP_DIR/sc2.log"
mkdir -p "$SC2_BIN"
make_nonexec_ensure
cat >"$SC2_BIN/python3" <<EOF_PY
#!/usr/bin/env bash
set -euo pipefail
if [[ "\${1:-}" == "-c" ]]; then
  exit 0
fi
echo "python3 \$*" >> "$SC2_LOG"
EOF_PY
chmod +x "$SC2_BIN/python3"
PATH="$SC2_BIN:/bin:/usr/bin" bash "$HARNESS_ROOT/infra/scripts/run_mutation_api.sh" >"$TMP_DIR/sc2.out" 2>"$TMP_DIR/sc2.err"
if ! grep -q "Using host python3 interpreter for mutation smoke." "$TMP_DIR/sc2.out"; then
  echo "FAIL: python fallback message missing" >&2
  exit 1
fi
if ! grep -q "python3 $HARNESS_ROOT/api/scripts/mutation_smoke.py" "$SC2_LOG"; then
  echo "FAIL: python fallback invocation mismatch" >&2
  cat "$SC2_LOG" >&2 || true
  exit 1
fi

# Scenario 3: docker run fallback selected when uv is missing and pytest is unavailable.
SC3_BIN="$TMP_DIR/sc3-bin"
SC3_LOG="$TMP_DIR/sc3.log"
mkdir -p "$SC3_BIN"
make_nonexec_ensure
cat >"$SC3_BIN/python3" <<'EOF_PY'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == "-c" ]]; then
  exit 1
fi
exit 1
EOF_PY
cat >"$SC3_BIN/docker" <<EOF_DOCKER
#!/usr/bin/env bash
set -euo pipefail
echo "docker \$*" >> "$SC3_LOG"
exit 0
EOF_DOCKER
chmod +x "$SC3_BIN/python3" "$SC3_BIN/docker"
PATH="$SC3_BIN:/bin:/usr/bin" bash "$HARNESS_ROOT/infra/scripts/run_mutation_api.sh" >"$TMP_DIR/sc3.out" 2>"$TMP_DIR/sc3.err"
if ! grep -q "Using docker compose run fallback for mutation smoke." "$TMP_DIR/sc3.out"; then
  echo "FAIL: docker fallback message missing" >&2
  exit 1
fi
if ! grep -q "docker compose version" "$SC3_LOG"; then
  echo "FAIL: docker compose version probe missing" >&2
  cat "$SC3_LOG" >&2 || true
  exit 1
fi
if ! grep -q "run --rm --no-deps api python /app/scripts/mutation_smoke.py" "$SC3_LOG"; then
  echo "FAIL: docker run fallback invocation mismatch" >&2
  cat "$SC3_LOG" >&2 || true
  exit 1
fi

# Scenario 4: deterministic failure when no execution path is available.
SC4_BIN="$TMP_DIR/sc4-bin"
mkdir -p "$SC4_BIN"
make_nonexec_ensure
cat >"$SC4_BIN/dirname" <<EOF_DIRNAME
#!/bin/bash
set -euo pipefail
exec "$SYSTEM_DIRNAME" "\$@"
EOF_DIRNAME
chmod +x "$SC4_BIN/dirname"
set +e
PATH="$SC4_BIN" /bin/bash "$HARNESS_ROOT/infra/scripts/run_mutation_api.sh" >"$TMP_DIR/sc4.out" 2>"$TMP_DIR/sc4.err"
SC4_EXIT=$?
set -e
if [[ "$SC4_EXIT" -eq 0 ]]; then
  echo "FAIL: expected no-tool mutation gate path to fail" >&2
  exit 1
fi
if ! grep -q "unable to run API mutation smoke gate" "$TMP_DIR/sc4.err"; then
  echo "FAIL: deterministic no-tool error message missing" >&2
  cat "$TMP_DIR/sc4.err" >&2 || true
  exit 1
fi

echo "OK: run_mutation_api fallback paths verified."
