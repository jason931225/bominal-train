#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/infra/scripts/deploy.sh"
REAL_PREFLIGHT="$ROOT_DIR/infra/scripts/predeploy-check.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p "$TMP_DIR/bin" "$TMP_DIR/repo/infra/env/prod"

cat >"$TMP_DIR/repo/infra/env/prod/postgres.env" <<'EOF_ENV'
POSTGRES_DB=bominal
POSTGRES_USER=bominal
POSTGRES_PASSWORD=strong-password
EOF_ENV

cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_ENV'
GCP_PROJECT_ID=bominal
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
EOF_ENV

cat >"$TMP_DIR/repo/infra/env/prod/web.env" <<'EOF_ENV'
NEXT_PUBLIC_API_BASE_URL=https://example.com
EOF_ENV

cat >"$TMP_DIR/repo/infra/env/prod/caddy.env" <<'EOF_ENV'
CADDY_SITE_ADDRESS=example.com
EOF_ENV

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

cat >"$TMP_DIR/bin/free" <<'FREE'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${SIM_RESOURCE_PROFILE:-low}" == "low" ]]; then
  cat <<'OUT'
              total        used        free      shared  buff/cache   available
Mem:            512         250          80           1         182         200
Swap:             0           0           0
OUT
else
  cat <<'OUT'
              total        used        free      shared  buff/cache   available
Mem:           1024         400         200           1         424         500
Swap:          1024          12        1012
OUT
fi
FREE
chmod +x "$TMP_DIR/bin/free"

cat >"$TMP_DIR/bin/swapon" <<'SWAPON'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == "--show=SIZE" ]]; then
  if [[ "${SIM_RESOURCE_PROFILE:-low}" == "low" ]]; then
    exit 0
  fi
  echo "1073741824"
  exit 0
fi
exit 0
SWAPON
chmod +x "$TMP_DIR/bin/swapon"

cat >"$TMP_DIR/bin/docker" <<'DOCKER'
#!/usr/bin/env bash
set -euo pipefail

printf '%s\n' "$*" >> "${DOCKER_CALLS_FILE:?}"

if [[ "${1:-}" == "compose" && "${2:-}" == "version" ]]; then
  exit 0
fi

if [[ "${1:-}" == "compose" ]]; then
  if [[ "$*" == *"config"* ]]; then
    echo "ok"
    exit 0
  fi
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

run_case() {
  local profile="$1"
  local calls_file="$2"
  local out_file="$3"

  set +e
  PATH="$TMP_DIR/bin:$PATH" \
    REPO_DIR="$TMP_DIR/repo" \
    BOMINAL_ROOT_DIR="$TMP_DIR/repo" \
    DEPLOY_HISTORY_DIR="$TMP_DIR/history-$profile" \
    DEPLOY_LOCK_FILE="$TMP_DIR/lock-$profile" \
    PREDEPLOY_CHECK_SCRIPT="$REAL_PREFLIGHT" \
    DOCKER_CALLS_FILE="$calls_file" \
    SIM_RESOURCE_PROFILE="$profile" \
    DEPLOY_MIN_TOTAL_MEMORY_MB=900 \
    DEPLOY_MIN_TOTAL_SWAP_MB=900 \
    GCP_PROJECT_ID="bominal" \
    SMOKE_MAX_ATTEMPTS=1 \
    SMOKE_RETRY_DELAY_SECONDS=0 \
    bash "$SCRIPT" >"$out_file" 2>&1
  local status=$?
  set -e
  echo "$status"
}

LOW_CALLS="$TMP_DIR/low.calls"
HIGH_CALLS="$TMP_DIR/high.calls"
LOW_OUT="$TMP_DIR/low.out"
HIGH_OUT="$TMP_DIR/high.out"

low_status="$(run_case low "$LOW_CALLS" "$LOW_OUT")"
if [[ "$low_status" -eq 0 ]]; then
  echo "FAIL: low-resource preflight should fail" >&2
  cat "$LOW_OUT" >&2
  exit 1
fi

if rg -q "^pull " "$LOW_CALLS"; then
  echo "FAIL: low-resource preflight ran docker pull unexpectedly" >&2
  cat "$LOW_CALLS" >&2
  exit 1
fi

if rg -q " up -d " "$LOW_CALLS"; then
  echo "FAIL: low-resource preflight ran deploy commands unexpectedly" >&2
  cat "$LOW_CALLS" >&2
  exit 1
fi

high_status="$(run_case high "$HIGH_CALLS" "$HIGH_OUT")"
if [[ "$high_status" -ne 0 ]]; then
  echo "FAIL: high-resource preflight should pass" >&2
  cat "$HIGH_OUT" >&2
  exit 1
fi

if ! rg -q "^pull " "$HIGH_CALLS"; then
  echo "FAIL: high-resource deploy did not pull images" >&2
  cat "$HIGH_CALLS" >&2
  exit 1
fi

if ! rg -q " up -d " "$HIGH_CALLS"; then
  echo "FAIL: high-resource deploy did not run compose up" >&2
  cat "$HIGH_CALLS" >&2
  exit 1
fi

echo "OK: deploy preflight resource gate test passed."
