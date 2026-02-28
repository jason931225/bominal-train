#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/infra/scripts/local-run.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
mkdir -p "$TMP_DIR/bin"

cat >"$TMP_DIR/bin/docker" <<'DOCKER'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "${DOCKER_CALLS_FILE:?}"

if [[ "${1:-}" == "compose" && "${2:-}" == "version" ]]; then
  exit 0
fi
exit 0
DOCKER
chmod +x "$TMP_DIR/bin/docker"

run_case() {
  local calls_file="$1"
  local out_file="$2"
  shift 2

  set +e
  PATH="$TMP_DIR/bin:$PATH" \
    DOCKER_CALLS_FILE="$calls_file" \
    bash "$SCRIPT" "$@" >"$out_file" 2>&1
  local status=$?
  set -e
  echo "$status"
}

match_file() {
  local pattern="$1"
  local target="$2"
  if command -v rg >/dev/null 2>&1; then
    rg -q -- "$pattern" "$target"
  else
    grep -Eq -- "$pattern" "$target"
  fi
}

DEFAULT_CALLS="$TMP_DIR/default.calls"
DEFAULT_OUT="$TMP_DIR/default.out"
default_status="$(run_case "$DEFAULT_CALLS" "$DEFAULT_OUT")"
if [[ "$default_status" -ne 0 ]]; then
  echo "FAIL: default local-run should succeed" >&2
  cat "$DEFAULT_OUT" >&2
  exit 1
fi
if ! match_file "compose -f infra/docker-compose.yml up --build --remove-orphans" "$DEFAULT_CALLS"; then
  echo "FAIL: default local-run did not start compose stack" >&2
  cat "$DEFAULT_CALLS" >&2
  exit 1
fi
if ! match_file "compose -f infra/docker-compose.yml down --remove-orphans" "$DEFAULT_CALLS"; then
  echo "FAIL: default local-run should down stack on exit" >&2
  cat "$DEFAULT_CALLS" >&2
  exit 1
fi

DETACH_CALLS="$TMP_DIR/detach.calls"
DETACH_OUT="$TMP_DIR/detach.out"
detach_status="$(run_case "$DETACH_CALLS" "$DETACH_OUT" -d)"
if [[ "$detach_status" -ne 0 ]]; then
  echo "FAIL: detached local-run should succeed" >&2
  cat "$DETACH_OUT" >&2
  exit 1
fi
if match_file "compose -f infra/docker-compose.yml down --remove-orphans" "$DETACH_CALLS"; then
  echo "FAIL: detached local-run should not auto-down by default" >&2
  cat "$DETACH_CALLS" >&2
  exit 1
fi

FORCED_DOWN_CALLS="$TMP_DIR/forced-down.calls"
FORCED_DOWN_OUT="$TMP_DIR/forced-down.out"
forced_down_status="$(run_case "$FORCED_DOWN_CALLS" "$FORCED_DOWN_OUT" -d --down-on-exit)"
if [[ "$forced_down_status" -ne 0 ]]; then
  echo "FAIL: detached local-run with --down-on-exit should succeed" >&2
  cat "$FORCED_DOWN_OUT" >&2
  exit 1
fi
if ! match_file "compose -f infra/docker-compose.yml down --remove-orphans" "$FORCED_DOWN_CALLS"; then
  echo "FAIL: --down-on-exit should force cleanup in detached mode" >&2
  cat "$FORCED_DOWN_CALLS" >&2
  exit 1
fi

KEEP_CALLS="$TMP_DIR/keep.calls"
KEEP_OUT="$TMP_DIR/keep.out"
keep_status="$(run_case "$KEEP_CALLS" "$KEEP_OUT" --keep-containers)"
if [[ "$keep_status" -ne 0 ]]; then
  echo "FAIL: local-run --keep-containers should succeed" >&2
  cat "$KEEP_OUT" >&2
  exit 1
fi
if match_file "compose -f infra/docker-compose.yml down --remove-orphans" "$KEEP_CALLS"; then
  echo "FAIL: --keep-containers should disable auto-down cleanup" >&2
  cat "$KEEP_CALLS" >&2
  exit 1
fi

echo "OK: local-run cleanup behavior test passed."
