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
    printf 'api-gateway\nweb\n'
    exit 0
  fi
  exit 0
fi

if [[ "${1:-}" == "pull" ]]; then
  exit 0
fi

if [[ "${1:-}" == "inspect" ]]; then
  # deploy commit label lookup
  if [[ "$*" == *"org.opencontainers.image.revision"* ]]; then
    if [[ "$2" == *"/api-train:"* ]]; then
      echo "traincommit"
    else
      echo "deadbeef"
    fi
    exit 0
  fi

  # rollback digest lookup
  if [[ "$*" == *".RepoDigests"* ]]; then
    echo "${2}@sha256:aaaaaaaa"
    exit 0
  fi

  # target image IDs
  if [[ "$*" == *"--format={{.Id}}"* ]]; then
    case "${2:-}" in
      */api-gateway:*)
        echo "sha256:api-gateway-new"
        ;;
      */api-train:*)
        echo "sha256:api-train-new"
        ;;
      */api-restaurant:*)
        echo "sha256:api-restaurant-new"
        ;;
      */web:*)
        echo "sha256:web-new"
        ;;
      *)
        echo "sha256:unknown-new"
        ;;
    esac
    exit 0
  fi

  # running container image IDs
  if [[ "$*" == *"--format={{.Image}}"* ]]; then
    case "${2:-}" in
      bominal-api-gateway)
        if [[ "${SIM_API_GATEWAY_CHANGED:-0}" == "1" ]]; then
          echo "sha256:api-gateway-old"
        else
          echo "sha256:api-gateway-new"
        fi
        exit 0
        ;;
      bominal-api-train)
        if [[ "${SIM_API_TRAIN_CHANGED:-0}" == "1" ]]; then
          echo "sha256:api-train-old"
        else
          echo "sha256:api-train-new"
        fi
        exit 0
        ;;
      bominal-api-restaurant)
        if [[ "${SIM_API_RESTAURANT_CHANGED:-0}" == "1" ]]; then
          echo "sha256:api-restaurant-old"
        else
          echo "sha256:api-restaurant-new"
        fi
        exit 0
        ;;
      bominal-worker-train)
        if [[ "${SIM_WORKER_TRAIN_PRESENT:-1}" == "0" ]]; then
          exit 0
        fi
        if [[ "${SIM_WORKER_TRAIN_CHANGED:-0}" == "1" ]]; then
          echo "sha256:api-train-old"
        else
          echo "sha256:api-train-new"
        fi
        exit 0
        ;;
      bominal-worker-restaurant)
        if [[ "${SIM_WORKER_RESTAURANT_CHANGED:-0}" == "1" ]]; then
          echo "sha256:api-restaurant-old"
        else
          echo "sha256:api-restaurant-new"
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
  local api_gateway_changed="$1"
  local api_train_changed="$2"
  local api_restaurant_changed="$3"
  local worker_train_changed="$4"
  local worker_restaurant_changed="$5"
  local web_changed="$6"
  local worker_train_present="${7:-1}"
  local calls_file="$8"
  local out_file="$9"

  PATH="$TMP_DIR/bin:$PATH" \
    REPO_DIR="$ROOT_DIR" \
    DEPLOY_HISTORY_DIR="$TMP_DIR/history-$api_gateway_changed-$api_train_changed-$api_restaurant_changed-$worker_train_changed-$worker_restaurant_changed-$web_changed" \
    DEPLOY_LOCK_FILE="$TMP_DIR/lock-$api_gateway_changed-$api_train_changed-$api_restaurant_changed-$worker_train_changed-$worker_restaurant_changed-$web_changed" \
    PREDEPLOY_CHECK_SCRIPT="$TMP_DIR/bin/predeploy-check.sh" \
    DOCKER_CALLS_FILE="$calls_file" \
    SIM_API_GATEWAY_CHANGED="$api_gateway_changed" \
    SIM_API_TRAIN_CHANGED="$api_train_changed" \
    SIM_API_RESTAURANT_CHANGED="$api_restaurant_changed" \
    SIM_WORKER_TRAIN_CHANGED="$worker_train_changed" \
    SIM_WORKER_RESTAURANT_CHANGED="$worker_restaurant_changed" \
    SIM_WEB_CHANGED="$web_changed" \
    SIM_WORKER_TRAIN_PRESENT="$worker_train_present" \
    GCP_PROJECT_ID="bominal" \
    SMOKE_MAX_ATTEMPTS=1 \
    SMOKE_RETRY_DELAY_SECONDS=0 \
    bash "$SCRIPT" >"$out_file" 2>&1
}

UNCHANGED_CALLS="$TMP_DIR/unchanged.calls"
UNCHANGED_OUT="$TMP_DIR/unchanged.out"
TRAIN_ONLY_CALLS="$TMP_DIR/train-only.calls"
TRAIN_ONLY_OUT="$TMP_DIR/train-only.out"
MISSING_WORKER_CALLS="$TMP_DIR/missing-worker.calls"
MISSING_WORKER_OUT="$TMP_DIR/missing-worker.out"

run_case 0 0 0 0 0 0 1 "$UNCHANGED_CALLS" "$UNCHANGED_OUT"
run_case 0 1 0 1 0 0 1 "$TRAIN_ONLY_CALLS" "$TRAIN_ONLY_OUT"
run_case 0 0 0 0 0 0 0 "$MISSING_WORKER_CALLS" "$MISSING_WORKER_OUT"

if ! rg -q "up -d --wait postgres redis$" "$UNCHANGED_CALLS"; then
  echo "FAIL: unchanged-image case did not run base db/redis stage" >&2
  cat "$UNCHANGED_CALLS" >&2
  exit 1
fi

if rg -q "up -d --wait --no-deps api-gateway|up -d --wait --no-deps api-train|up -d --wait --no-deps api-restaurant|up -d --wait --no-deps worker-train|up -d --wait --no-deps worker-restaurant|up -d --wait --no-deps web" "$UNCHANGED_CALLS"; then
  echo "FAIL: unchanged-image case unexpectedly rolled services" >&2
  cat "$UNCHANGED_CALLS" >&2
  exit 1
fi

if ! rg -q "up -d --wait --no-deps api-train" "$TRAIN_ONLY_CALLS"; then
  echo "FAIL: train-only case did not roll api-train service" >&2
  cat "$TRAIN_ONLY_CALLS" >&2
  exit 1
fi

if ! rg -q "up -d --wait --no-deps worker-train" "$TRAIN_ONLY_CALLS"; then
  echo "FAIL: train-only case did not roll worker-train service" >&2
  cat "$TRAIN_ONLY_CALLS" >&2
  exit 1
fi

if rg -q "up -d --wait --no-deps api-gateway|up -d --wait --no-deps api-restaurant|up -d --wait --no-deps worker-restaurant|up -d --wait --no-deps web" "$TRAIN_ONLY_CALLS"; then
  echo "FAIL: train-only case rolled unrelated services" >&2
  cat "$TRAIN_ONLY_CALLS" >&2
  exit 1
fi

if ! rg -q "up -d --wait --no-deps worker-train" "$MISSING_WORKER_CALLS"; then
  echo "FAIL: missing-worker case did not force worker-train rollout" >&2
  cat "$MISSING_WORKER_CALLS" >&2
  exit 1
fi

if rg -q "up -d --wait --no-deps api-gateway|up -d --wait --no-deps api-train|up -d --wait --no-deps api-restaurant|up -d --wait --no-deps worker-restaurant|up -d --wait --no-deps web" "$MISSING_WORKER_CALLS"; then
  echo "FAIL: missing-worker case rolled unrelated services" >&2
  cat "$MISSING_WORKER_CALLS" >&2
  exit 1
fi

echo "OK: changed-image rollout path test passed."
