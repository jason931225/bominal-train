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

cat >"$TMP_DIR/bin/docker" <<'DOCKER'
#!/usr/bin/env bash
set -euo pipefail

printf '%s\n' "$*" >> "${DOCKER_CALLS_FILE:?}"

if [[ "${1:-}" == "compose" && "${2:-}" == "version" ]]; then
  exit 0
fi

if [[ "${1:-}" == "compose" ]]; then
  if [[ "$*" == *"ps --services --filter status=running"* ]]; then
    if [[ "${SIM_RUNNING_STACK:-0}" == "1" ]]; then
      printf 'api\nweb\n'
    fi
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

cat >"$TMP_DIR/bin/predeploy-check.sh" <<'PREFLIGHT'
#!/usr/bin/env bash
set -euo pipefail
exit 0
PREFLIGHT
chmod +x "$TMP_DIR/bin/predeploy-check.sh"

run_case() {
  local running="$1"
  local output_file="$2"
  local calls_file="$3"

  PATH="$TMP_DIR/bin:$PATH" \
    REPO_DIR="$ROOT_DIR" \
    DEPLOY_HISTORY_DIR="$TMP_DIR/history-$running" \
    DEPLOY_LOCK_FILE="$TMP_DIR/lock-$running" \
    PREDEPLOY_CHECK_SCRIPT="$TMP_DIR/bin/predeploy-check.sh" \
    DOCKER_CALLS_FILE="$calls_file" \
    SIM_RUNNING_STACK="$running" \
    GCP_PROJECT_ID="bominal" \
    SMOKE_MAX_ATTEMPTS=1 \
    SMOKE_RETRY_DELAY_SECONDS=0 \
    bash "$SCRIPT" >"$output_file" 2>&1
}

FIRST_CALLS="$TMP_DIR/docker-calls-first.txt"
UPDATE_CALLS="$TMP_DIR/docker-calls-update.txt"
FIRST_OUT="$TMP_DIR/first.out"
UPDATE_OUT="$TMP_DIR/update.out"

run_case 0 "$FIRST_OUT" "$FIRST_CALLS"
run_case 1 "$UPDATE_OUT" "$UPDATE_CALLS"

if ! rg -q "up -d --wait redis api worker web caddy" "$FIRST_CALLS"; then
  echo "FAIL: first deploy did not use bootstrap-safe path" >&2
  cat "$FIRST_CALLS" >&2
  exit 1
fi

if ! rg -q "up -d --wait redis$" "$UPDATE_CALLS"; then
  echo "FAIL: rolling deploy did not run base redis step" >&2
  cat "$UPDATE_CALLS" >&2
  exit 1
fi

if ! rg -q "up -d --wait --no-deps api" "$UPDATE_CALLS"; then
  echo "FAIL: rolling deploy did not use no-deps api update step" >&2
  cat "$UPDATE_CALLS" >&2
  exit 1
fi

if ! rg -q "up -d --wait --no-deps worker" "$UPDATE_CALLS"; then
  echo "FAIL: rolling deploy did not use no-deps worker update step" >&2
  cat "$UPDATE_CALLS" >&2
  exit 1
fi

if ! rg -q "up -d --wait --no-deps web" "$UPDATE_CALLS"; then
  echo "FAIL: rolling deploy did not roll web service when changed" >&2
  cat "$UPDATE_CALLS" >&2
  exit 1
fi

if rg -q "up -d --wait redis api worker web caddy" "$UPDATE_CALLS"; then
  echo "FAIL: running-stack deploy unexpectedly used bootstrap path" >&2
  cat "$UPDATE_CALLS" >&2
  exit 1
fi

echo "OK: running-container detection path test passed."
