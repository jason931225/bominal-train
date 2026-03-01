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

  if [[ "$*" == *" up "* ]] || [[ "$*" == *" up -"* ]]; then
    compose_up_count=0
    if [[ -f "${COMPOSE_UP_COUNT_FILE:?}" ]]; then
      compose_up_count="$(cat "${COMPOSE_UP_COUNT_FILE:?}")"
    fi
    compose_up_count="$((compose_up_count + 1))"
    echo "$compose_up_count" > "${COMPOSE_UP_COUNT_FILE:?}"

    # Initial deploy on a running stack executes 6 compose "up" calls
    # (redis, api, worker, web-canary, web, caddy). Fail rollback deploy
    # services by failing the first "up" after that point.
    if [[ "${SIMULATE_ROLLBACK_DEPLOY_FAILURE:-0}" == "1" ]] && [[ "$compose_up_count" -ge 7 ]]; then
      exit 1
    fi
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
    echo "${2}@sha256:bbbbbbbb"
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
  local stage="$1"
  local smoke_fail="$2"
  local rollback_deploy_fail="$3"
  local calls_file="$4"
  local out_file="$5"
  local history_dir="$6"
  local compose_up_count_file="$7"

  prepare_history "$history_dir"
  rm -f "$compose_up_count_file"

  set +e
  PATH="$TMP_DIR/bin:$PATH" \
    REPO_DIR="$ROOT_DIR" \
    DEPLOY_HISTORY_DIR="$history_dir" \
    DEPLOY_LOCK_FILE="$TMP_DIR/lock-$stage-$smoke_fail-$rollback_deploy_fail" \
    PREDEPLOY_CHECK_SCRIPT="$TMP_DIR/bin/predeploy-check.sh" \
    DOCKER_CALLS_FILE="$calls_file" \
    COMPOSE_UP_COUNT_FILE="$compose_up_count_file" \
    SIMULATE_SMOKE_FAILURE="$smoke_fail" \
    SIMULATE_ROLLBACK_DEPLOY_FAILURE="$rollback_deploy_fail" \
    GCP_PROJECT_ID="bominal" \
    AUTO_ROLLBACK_ON_SMOKE_FAILURE=true \
    SMOKE_MAX_ATTEMPTS=1 \
    SMOKE_RETRY_DELAY_SECONDS=0 \
    bash "$SCRIPT" --canary-stage="$stage" >"$out_file" 2>&1
  local status=$?
  set -e
  echo "$status"
}

CASE_CALLS="$TMP_DIR/stage1.fail.calls"
CASE_OUT="$TMP_DIR/stage1.fail.out"
CASE_HISTORY="$TMP_DIR/history-stage1-fail"
CASE_COMPOSE_COUNT="$TMP_DIR/compose-up-stage1-fail.count"

status="$(run_case 1 1 1 "$CASE_CALLS" "$CASE_OUT" "$CASE_HISTORY" "$CASE_COMPOSE_COUNT")"
if [[ "$status" -eq 0 ]]; then
  echo "FAIL: canary stage 1 rollback deploy failure case unexpectedly succeeded" >&2
  cat "$CASE_OUT" >&2
  exit 1
fi

if ! rg -q "Auto rollback attempt failed\." "$CASE_OUT"; then
  echo "FAIL: rollback failure marker missing from output" >&2
  cat "$CASE_OUT" >&2
  exit 1
fi

if [[ "$(cat "$CASE_HISTORY/current")" != "currentcommit" ]]; then
  echo "FAIL: current pointer was unexpectedly swapped after rollback deploy failure" >&2
  cat "$CASE_OUT" >&2
  exit 1
fi

if [[ "$(cat "$CASE_HISTORY/previous")" != "oldcommit" ]]; then
  echo "FAIL: previous pointer was unexpectedly swapped after rollback deploy failure" >&2
  cat "$CASE_OUT" >&2
  exit 1
fi

echo "OK: canary stage rollback failure handling test passed."
