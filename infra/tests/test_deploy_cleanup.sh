#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/infra/scripts/deploy.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p "$TMP_DIR/bin"
mkdir -p "$TMP_DIR/repo"

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
    echo "cleanup-commit"
    exit 0
  fi
  if [[ "$*" == *".RepoDigests"* ]]; then
    echo "${2}@sha256:dddddddd"
    exit 0
  fi
  exit 0
fi

if [[ "${1:-}" == "image" && "${2:-}" == "prune" ]]; then
  exit 0
fi

if [[ "${1:-}" == "image" && "${2:-}" == "ls" ]]; then
  printf '%s\n' \
    "ghcr.io/jason931225/bominal/api:newest" \
    "ghcr.io/jason931225/bominal/web:newest" \
    "ghcr.io/jason931225/bominal/api:old1" \
    "ghcr.io/jason931225/bominal/web:old1"
  exit 0
fi

if [[ "${1:-}" == "image" && "${2:-}" == "rm" ]]; then
  exit 0
fi

if [[ "${1:-}" == "builder" && "${2:-}" == "prune" ]]; then
  exit 0
fi

if [[ "${1:-}" == "system" && "${2:-}" == "df" ]]; then
  echo "TYPE TOTAL ACTIVE SIZE RECLAIMABLE"
  exit 0
fi

if [[ "${1:-}" == "ps" ]]; then
  if [[ "${2:-}" == "--format" ]]; then
    echo "ghcr.io/jason931225/bominal/api:newest"
    exit 0
  fi
  echo "NAMES"
  exit 0
fi

exit 0
DOCKER
chmod +x "$TMP_DIR/bin/docker"

run_case() {
  local calls_file="$1"
  local out_file="$2"
  local history_dir="$3"

  set +e
  PATH="$TMP_DIR/bin:$PATH" \
    REPO_DIR="$TMP_DIR/repo" \
    DEPLOY_HISTORY_DIR="$history_dir" \
    DEPLOY_LOCK_FILE="$TMP_DIR/lock-$(basename "$history_dir")" \
    PREDEPLOY_CHECK_SCRIPT="$TMP_DIR/bin/predeploy-check.sh" \
    DOCKER_CALLS_FILE="$calls_file" \
    GCP_PROJECT_ID="bominal" \
    SMOKE_MAX_ATTEMPTS=1 \
    SMOKE_RETRY_DELAY_SECONDS=0 \
    bash "$SCRIPT" >"$out_file" 2>&1
  local status=$?
  set -e
  echo "$status"
}

DEFAULT_CALLS="$TMP_DIR/default.calls"
DEFAULT_OUT="$TMP_DIR/default.out"
DEFAULT_HISTORY="$TMP_DIR/history-default"

default_status="$(run_case "$DEFAULT_CALLS" "$DEFAULT_OUT" "$DEFAULT_HISTORY")"
if [[ "$default_status" -ne 0 ]]; then
  echo "FAIL: default deploy case should succeed" >&2
  cat "$DEFAULT_OUT" >&2
  exit 1
fi

if ! rg -q "^image prune -a -f$" "$DEFAULT_CALLS"; then
  echo "FAIL: default deploy did not run unused image prune" >&2
  cat "$DEFAULT_CALLS" >&2
  exit 1
fi

if ! rg -q "^builder prune -a -f$" "$DEFAULT_CALLS"; then
  echo "FAIL: default deploy did not run build cache prune" >&2
  cat "$DEFAULT_CALLS" >&2
  exit 1
fi

DISABLED_CALLS="$TMP_DIR/disabled.calls"
DISABLED_OUT="$TMP_DIR/disabled.out"
DISABLED_HISTORY="$TMP_DIR/history-disabled"

set +e
PATH="$TMP_DIR/bin:$PATH" \
  REPO_DIR="$TMP_DIR/repo" \
  DEPLOY_HISTORY_DIR="$DISABLED_HISTORY" \
  DEPLOY_LOCK_FILE="$TMP_DIR/lock-disabled" \
  PREDEPLOY_CHECK_SCRIPT="$TMP_DIR/bin/predeploy-check.sh" \
  DOCKER_CALLS_FILE="$DISABLED_CALLS" \
  GCP_PROJECT_ID="bominal" \
  SMOKE_MAX_ATTEMPTS=1 \
  SMOKE_RETRY_DELAY_SECONDS=0 \
  DEPLOY_DOCKER_PRUNE_UNUSED_IMAGES=false \
  DEPLOY_DOCKER_PRUNE_BUILD_CACHE=false \
  bash "$SCRIPT" >"$DISABLED_OUT" 2>&1
disabled_status=$?
set -e

if [[ "$disabled_status" -ne 0 ]]; then
  echo "FAIL: disabled cleanup case should succeed" >&2
  cat "$DISABLED_OUT" >&2
  exit 1
fi

if rg -q "^image prune -a -f$" "$DISABLED_CALLS"; then
  echo "FAIL: disabled cleanup case still ran image prune" >&2
  cat "$DISABLED_CALLS" >&2
  exit 1
fi

if rg -q "^builder prune -a -f$" "$DISABLED_CALLS"; then
  echo "FAIL: disabled cleanup case still ran build cache prune" >&2
  cat "$DISABLED_CALLS" >&2
  exit 1
fi

RETENTION_CALLS="$TMP_DIR/retention.calls"
RETENTION_OUT="$TMP_DIR/retention.out"
RETENTION_HISTORY="$TMP_DIR/history-retention"

set +e
PATH="$TMP_DIR/bin:$PATH" \
  REPO_DIR="$TMP_DIR/repo" \
  DEPLOY_HISTORY_DIR="$RETENTION_HISTORY" \
  DEPLOY_LOCK_FILE="$TMP_DIR/lock-retention" \
  PREDEPLOY_CHECK_SCRIPT="$TMP_DIR/bin/predeploy-check.sh" \
  DOCKER_CALLS_FILE="$RETENTION_CALLS" \
  GCP_PROJECT_ID="bominal" \
  SMOKE_MAX_ATTEMPTS=1 \
  SMOKE_RETRY_DELAY_SECONDS=0 \
  DEPLOY_DOCKER_KEEP_BOMINAL_IMAGES=2 \
  bash "$SCRIPT" >"$RETENTION_OUT" 2>&1
retention_status=$?
set -e

if [[ "$retention_status" -ne 0 ]]; then
  echo "FAIL: retention cleanup case should succeed" >&2
  cat "$RETENTION_OUT" >&2
  exit 1
fi

if rg -q "^image prune -a -f$" "$RETENTION_CALLS"; then
  echo "FAIL: retention cleanup case unexpectedly ran full image prune -a" >&2
  cat "$RETENTION_CALLS" >&2
  exit 1
fi

if ! rg -q "^image prune -f$" "$RETENTION_CALLS"; then
  echo "FAIL: retention cleanup case did not run dangling image prune" >&2
  cat "$RETENTION_OUT" >&2
  cat "$RETENTION_CALLS" >&2
  exit 1
fi

if ! rg -q "^image rm ghcr.io/jason931225/bominal/api:old1$" "$RETENTION_CALLS"; then
  echo "FAIL: retention cleanup case did not remove old api tag" >&2
  cat "$RETENTION_OUT" >&2
  cat "$RETENTION_CALLS" >&2
  exit 1
fi

if ! rg -q "^image rm ghcr.io/jason931225/bominal/web:old1$" "$RETENTION_CALLS"; then
  echo "FAIL: retention cleanup case did not remove old web tag" >&2
  cat "$RETENTION_OUT" >&2
  cat "$RETENTION_CALLS" >&2
  exit 1
fi

echo "OK: deploy docker cleanup controls test passed."
