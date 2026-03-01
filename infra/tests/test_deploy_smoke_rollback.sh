#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/infra/scripts/deploy.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p "$TMP_DIR/bin"

cat >"$TMP_DIR/bin/gcloud" <<'GCLOUD'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == "auth" && "${2:-}" == "configure-docker" ]]; then
  exit 0
fi
exit 0
GCLOUD
chmod +x "$TMP_DIR/bin/gcloud"

cat >"$TMP_DIR/bin/predeploy-check.sh" <<'PREFLIGHT'
#!/usr/bin/env bash
set -euo pipefail
exit 0
PREFLIGHT
chmod +x "$TMP_DIR/bin/predeploy-check.sh"

cat >"$TMP_DIR/bin/curl" <<'CURL'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${SIMULATE_SMOKE_FAILURE:-0}" == "1" ]]; then
  url="${*: -1}"
  if [[ "$url" == *"/health/ready" ]]; then
    exit 0
  fi
  exit 1
fi
exit 0
CURL
chmod +x "$TMP_DIR/bin/curl"

cat >"$TMP_DIR/bin/docker" <<'DOCKER'
#!/usr/bin/env bash
set -euo pipefail

printf '%s\n' "$*" >> "${DOCKER_CALLS_FILE:?}"

if [[ "${1:-}" == "compose" && "${2:-}" == "version" ]]; then
  exit 0
fi

if [[ "${1:-}" == "compose" ]]; then
  if [[ "$*" == *"ps --services --filter status=running"* ]]; then
    printf 'api\nweb\n'
    exit 0
  fi
  exit 0
fi

if [[ "${1:-}" == "pull" ]]; then
  exit 0
fi

if [[ "${1:-}" == "inspect" ]]; then
  if [[ "$*" == *"org.opencontainers.image.revision"* ]]; then
    echo "newcommit"
    exit 0
  fi
  if [[ "$*" == *".RepoDigests"* ]]; then
    echo "${2}@sha256:cccccccc"
    exit 0
  fi
  exit 0
fi

if [[ "${1:-}" == "image" && "${2:-}" == "prune" ]]; then
  exit 0
fi

if [[ "${1:-}" == "images" ]]; then
  exit 0
fi

if [[ "${1:-}" == "ps" ]]; then
  echo "NAMES"
  exit 0
fi

exit 0
DOCKER
chmod +x "$TMP_DIR/bin/docker"

prepare_history() {
  local dir="$1"
  mkdir -p "$dir"
  echo "currentcommit" > "$dir/current"
  echo "oldcommit" > "$dir/previous"
}

run_case() {
  local smoke_fail="$1"
  local calls_file="$2"
  local out_file="$3"
  local history_dir="$4"

  prepare_history "$history_dir"

  set +e
  PATH="$TMP_DIR/bin:$PATH" \
    REPO_DIR="$ROOT_DIR" \
    DEPLOY_HISTORY_DIR="$history_dir" \
    DEPLOY_LOCK_FILE="$TMP_DIR/lock-$smoke_fail" \
    PREDEPLOY_CHECK_SCRIPT="$TMP_DIR/bin/predeploy-check.sh" \
    DOCKER_CALLS_FILE="$calls_file" \
    SIMULATE_SMOKE_FAILURE="$smoke_fail" \
    GCP_PROJECT_ID="bominal" \
    AUTO_ROLLBACK_ON_SMOKE_FAILURE=true \
    SMOKE_MAX_ATTEMPTS=1 \
    SMOKE_RETRY_DELAY_SECONDS=0 \
    bash "$SCRIPT" >"$out_file" 2>&1
  local status=$?
  set -e
  echo "$status"
}

FAIL_CALLS="$TMP_DIR/fail.calls"
FAIL_OUT="$TMP_DIR/fail.out"
FAIL_HISTORY="$TMP_DIR/history-fail"

SUCCESS_CALLS="$TMP_DIR/success.calls"
SUCCESS_OUT="$TMP_DIR/success.out"
SUCCESS_HISTORY="$TMP_DIR/history-success"

fail_status="$(run_case 1 "$FAIL_CALLS" "$FAIL_OUT" "$FAIL_HISTORY")"
if [[ "$fail_status" -eq 0 ]]; then
  echo "FAIL: smoke failure case unexpectedly succeeded" >&2
  cat "$FAIL_OUT" >&2
  exit 1
fi

if ! rg -q "oldcommit" "$FAIL_CALLS"; then
  echo "FAIL: rollback path did not pull previous deployment image" >&2
  cat "$FAIL_CALLS" >&2
  cat "$FAIL_OUT" >&2
  exit 1
fi

success_status="$(run_case 0 "$SUCCESS_CALLS" "$SUCCESS_OUT" "$SUCCESS_HISTORY")"
if [[ "$success_status" -ne 0 ]]; then
  echo "FAIL: smoke success case should succeed" >&2
  cat "$SUCCESS_OUT" >&2
  exit 1
fi

if rg -q "oldcommit" "$SUCCESS_CALLS"; then
  echo "FAIL: rollback path triggered despite smoke success" >&2
  cat "$SUCCESS_CALLS" >&2
  cat "$SUCCESS_OUT" >&2
  exit 1
fi

echo "OK: smoke rollback trigger test passed."
