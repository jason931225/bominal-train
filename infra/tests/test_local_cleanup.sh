#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/infra/scripts/local-cleanup.sh"

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

DEFAULT_CALLS="$TMP_DIR/default.calls"
DEFAULT_OUT="$TMP_DIR/default.out"
default_status="$(run_case "$DEFAULT_CALLS" "$DEFAULT_OUT")"
if [[ "$default_status" -ne 0 ]]; then
  echo "FAIL: default local cleanup should succeed" >&2
  cat "$DEFAULT_OUT" >&2
  exit 1
fi

if ! rg -q "compose -f infra/docker-compose.yml down --remove-orphans" "$DEFAULT_CALLS"; then
  echo "FAIL: default cleanup did not down dev compose stack" >&2
  cat "$DEFAULT_CALLS" >&2
  exit 1
fi

if ! rg -q "compose -f infra/docker-compose.prod.yml down --remove-orphans" "$DEFAULT_CALLS"; then
  echo "FAIL: default cleanup did not down prod compose stack" >&2
  cat "$DEFAULT_CALLS" >&2
  exit 1
fi

for cmd in "container prune -f" "network prune -f" "image prune -f" "builder prune -f" "system df"; do
  if ! rg -q "^${cmd}$" "$DEFAULT_CALLS"; then
    echo "FAIL: default cleanup missing command: ${cmd}" >&2
    cat "$DEFAULT_CALLS" >&2
    exit 1
  fi
done

if rg -q "^image prune -a -f$" "$DEFAULT_CALLS"; then
  echo "FAIL: default cleanup unexpectedly ran aggressive image prune" >&2
  cat "$DEFAULT_CALLS" >&2
  exit 1
fi

AGGR_CALLS="$TMP_DIR/aggr.calls"
AGGR_OUT="$TMP_DIR/aggr.out"
aggr_status="$(run_case "$AGGR_CALLS" "$AGGR_OUT" --aggressive --volumes --dev-only)"
if [[ "$aggr_status" -ne 0 ]]; then
  echo "FAIL: aggressive local cleanup should succeed" >&2
  cat "$AGGR_OUT" >&2
  exit 1
fi

if ! rg -q "compose -f infra/docker-compose.yml down --remove-orphans -v" "$AGGR_CALLS"; then
  echo "FAIL: aggressive cleanup with --volumes did not down dev stack with -v" >&2
  cat "$AGGR_CALLS" >&2
  exit 1
fi

if rg -q "compose -f infra/docker-compose.prod.yml down --remove-orphans" "$AGGR_CALLS"; then
  echo "FAIL: --dev-only cleanup should not down prod stack" >&2
  cat "$AGGR_CALLS" >&2
  exit 1
fi

for cmd in "image prune -a -f" "builder prune -a -f" "volume prune -f"; do
  if ! rg -q "^${cmd}$" "$AGGR_CALLS"; then
    echo "FAIL: aggressive cleanup missing command: ${cmd}" >&2
    cat "$AGGR_CALLS" >&2
    exit 1
  fi
done

echo "OK: local cleanup script test passed."
