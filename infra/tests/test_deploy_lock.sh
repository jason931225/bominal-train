#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/infra/scripts/deploy.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p "$TMP_DIR/bin" "$TMP_DIR/history"

cat >"$TMP_DIR/bin/docker" <<'DOCKER'
#!/usr/bin/env bash
set -euo pipefail

if [[ "${1:-}" == "compose" && "${2:-}" == "version" ]]; then
  exit 0
fi

if [[ "${1:-}" == "compose" ]]; then
  if [[ "$*" == *" ps"* ]]; then
    sleep "${DOCKER_STUB_SLEEP_SECONDS:-0}"
    echo "api"
    exit 0
  fi
  exit 0
fi

if [[ "${1:-}" == "ps" ]]; then
  echo "NAMES"
  exit 0
fi

exit 0
DOCKER
chmod +x "$TMP_DIR/bin/docker"

LOCK_FILE="$TMP_DIR/deploy.lock"
FIRST_OUT="$TMP_DIR/first.out"
SECOND_OUT="$TMP_DIR/second.out"

PATH="$TMP_DIR/bin:$PATH" \
  REPO_DIR="$ROOT_DIR" \
  DEPLOY_HISTORY_DIR="$TMP_DIR/history" \
  DEPLOY_LOCK_FILE="$LOCK_FILE" \
  DOCKER_STUB_SLEEP_SECONDS=2 \
  bash "$SCRIPT" --status >"$FIRST_OUT" 2>&1 &
first_pid=$!

# Wait until first invocation has created its lock marker.
for _ in $(seq 1 40); do
  if [[ -d "${LOCK_FILE}.d" || -f "$LOCK_FILE" ]]; then
    break
  fi
  sleep 0.05
done

set +e
PATH="$TMP_DIR/bin:$PATH" \
  REPO_DIR="$ROOT_DIR" \
  DEPLOY_HISTORY_DIR="$TMP_DIR/history" \
  DEPLOY_LOCK_FILE="$LOCK_FILE" \
  DOCKER_STUB_SLEEP_SECONDS=0 \
  bash "$SCRIPT" --status >"$SECOND_OUT" 2>&1
second_status=$?
set -e

if [[ "$second_status" -eq 0 ]]; then
  echo "FAIL: second concurrent deploy invocation unexpectedly succeeded" >&2
  cat "$SECOND_OUT" >&2
  kill "$first_pid" >/dev/null 2>&1 || true
  exit 1
fi

if ! rg -qi "lock|already running" "$SECOND_OUT"; then
  echo "FAIL: expected lock contention message in second invocation output" >&2
  cat "$SECOND_OUT" >&2
  kill "$first_pid" >/dev/null 2>&1 || true
  exit 1
fi

wait "$first_pid"

echo "OK: deploy lock enforcement test passed."
