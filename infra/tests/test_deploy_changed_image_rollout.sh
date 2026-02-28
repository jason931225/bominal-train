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

cat >"$TMP_DIR/bin/curl" <<'CURL'
#!/usr/bin/env bash
set -euo pipefail
exit 0
CURL
chmod +x "$TMP_DIR/bin/curl"

cat >"$TMP_DIR/bin/predeploy-check.sh" <<'PREFLIGHT'
#!/usr/bin/env bash
set -euo pipefail
exit 0
PREFLIGHT
chmod +x "$TMP_DIR/bin/predeploy-check.sh"

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
    echo "deadbeef"
    exit 0
  fi

  if [[ "$*" == *".RepoDigests"* ]]; then
    echo "${2}@sha256:aaaaaaaa"
    exit 0
  fi

  if [[ "$*" == *"--format={{.Id}}"* ]]; then
    case "${2:-}" in
      */api:*) echo "sha256:api-new" ;;
      */web:*) echo "sha256:web-new" ;;
      *) echo "sha256:unknown-new" ;;
    esac
    exit 0
  fi

  if [[ "$*" == *"--format={{.Image}}"* ]]; then
    case "${2:-}" in
      bominal-api)
        if [[ "${SIM_API_CHANGED:-0}" == "1" ]]; then
          echo "sha256:api-old"
        else
          echo "sha256:api-new"
        fi
        exit 0
        ;;
      bominal-worker)
        if [[ "${SIM_WORKER_PRESENT:-1}" == "0" ]]; then
          exit 0
        fi
        if [[ "${SIM_WORKER_CHANGED:-0}" == "1" ]]; then
          echo "sha256:api-old"
        else
          echo "sha256:api-new"
        fi
        exit 0
        ;;
      bominal-web)
        if [[ "${SIM_WEB_CHANGED:-0}" == "1" ]]; then
          echo "sha256:web-old"
        else
          echo "sha256:web-new"
        fi
        exit 0
        ;;
      *)
        exit 0
        ;;
    esac
  fi

  if [[ "$*" == *"--format={{.State.Running}}"* ]]; then
    case "${2:-}" in
      bominal-worker)
        if [[ "${SIM_WORKER_PRESENT:-1}" == "0" ]]; then
          echo "false"
        else
          echo "true"
        fi
        ;;
      *)
        echo "true"
        ;;
    esac
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

run_case() {
  local api_changed="$1"
  local worker_changed="$2"
  local web_changed="$3"
  local worker_present="${4:-1}"
  local calls_file="$5"
  local out_file="$6"

  PATH="$TMP_DIR/bin:$PATH" \
    REPO_DIR="$ROOT_DIR" \
    DEPLOY_HISTORY_DIR="$TMP_DIR/history-$api_changed-$worker_changed-$web_changed-$worker_present" \
    DEPLOY_LOCK_FILE="$TMP_DIR/lock-$api_changed-$worker_changed-$web_changed-$worker_present" \
    PREDEPLOY_CHECK_SCRIPT="$TMP_DIR/bin/predeploy-check.sh" \
    DOCKER_CALLS_FILE="$calls_file" \
    SIM_API_CHANGED="$api_changed" \
    SIM_WORKER_CHANGED="$worker_changed" \
    SIM_WEB_CHANGED="$web_changed" \
    SIM_WORKER_PRESENT="$worker_present" \
    GCP_PROJECT_ID="bominal" \
    SMOKE_MAX_ATTEMPTS=1 \
    SMOKE_RETRY_DELAY_SECONDS=0 \
    bash "$SCRIPT" >"$out_file" 2>&1
}

matches_pattern() {
  local pattern="$1"
  local file="$2"
  if command -v rg >/dev/null 2>&1; then
    rg -q -- "$pattern" "$file"
    return $?
  fi
  grep -Eq -- "$pattern" "$file"
}

UNCHANGED_CALLS="$TMP_DIR/unchanged.calls"
UNCHANGED_OUT="$TMP_DIR/unchanged.out"
API_ONLY_CALLS="$TMP_DIR/api-only.calls"
API_ONLY_OUT="$TMP_DIR/api-only.out"
MISSING_WORKER_CALLS="$TMP_DIR/missing-worker.calls"
MISSING_WORKER_OUT="$TMP_DIR/missing-worker.out"

run_case 0 0 0 1 "$UNCHANGED_CALLS" "$UNCHANGED_OUT"
run_case 1 0 0 1 "$API_ONLY_CALLS" "$API_ONLY_OUT"
run_case 0 0 0 0 "$MISSING_WORKER_CALLS" "$MISSING_WORKER_OUT"

if ! matches_pattern "up -d --wait( --remove-orphans)? redis$" "$UNCHANGED_CALLS"; then
  echo "FAIL: unchanged-image case did not run base redis stage" >&2
  cat "$UNCHANGED_CALLS" >&2
  exit 1
fi

if matches_pattern "up -d --wait( --remove-orphans)? --no-deps api|up -d --wait( --remove-orphans)? --no-deps worker|up -d --wait( --remove-orphans)? --no-deps web" "$UNCHANGED_CALLS"; then
  echo "FAIL: unchanged-image case unexpectedly rolled services" >&2
  cat "$UNCHANGED_CALLS" >&2
  exit 1
fi

if ! matches_pattern "up -d --wait( --remove-orphans)? --no-deps caddy" "$UNCHANGED_CALLS"; then
  echo "FAIL: unchanged-image case did not run isolated caddy reconciliation" >&2
  cat "$UNCHANGED_CALLS" >&2
  exit 1
fi

if ! matches_pattern "up -d --wait( --remove-orphans)? --no-deps api" "$API_ONLY_CALLS"; then
  echo "FAIL: api-only case did not roll api service" >&2
  cat "$API_ONLY_CALLS" >&2
  exit 1
fi

if matches_pattern "up -d --wait( --remove-orphans)? --no-deps worker|up -d --wait( --remove-orphans)? --no-deps web" "$API_ONLY_CALLS"; then
  echo "FAIL: api-only case rolled unrelated services" >&2
  cat "$API_ONLY_CALLS" >&2
  exit 1
fi

if ! matches_pattern "up -d --wait( --remove-orphans)? --no-deps worker" "$MISSING_WORKER_CALLS"; then
  echo "FAIL: missing-worker case did not force worker rollout" >&2
  cat "$MISSING_WORKER_CALLS" >&2
  exit 1
fi

if matches_pattern "up -d --wait( --remove-orphans)? --no-deps api|up -d --wait( --remove-orphans)? --no-deps web" "$MISSING_WORKER_CALLS"; then
  echo "FAIL: missing-worker case rolled unrelated services" >&2
  cat "$MISSING_WORKER_CALLS" >&2
  exit 1
fi

echo "OK: changed-image rollout path test passed."
