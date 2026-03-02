#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

API_ENV_FILE="${EVERVAULT_TEST_API_ENV_FILE:-$ROOT_DIR/infra/env/prod/api.env}"
WEB_ENV_FILE="${EVERVAULT_TEST_WEB_ENV_FILE:-$ROOT_DIR/infra/env/prod/web.env}"
CADDY_ENV_FILE="${EVERVAULT_TEST_CADDY_ENV_FILE:-$ROOT_DIR/infra/env/prod/caddy.env}"
IMAGE_TAG="${EVERVAULT_TEST_IMAGE_TAG:-evervault-test:local}"
CONTAINER_NAME="${EVERVAULT_TEST_CONTAINER_NAME:-evervault-test}"
HOST_PORT="${EVERVAULT_TEST_PORT:-8787}"

for file in "$API_ENV_FILE" "$WEB_ENV_FILE" "$CADDY_ENV_FILE"; do
  if [[ ! -f "$file" ]]; then
    echo "ERROR: missing env file: $file" >&2
    exit 1
  fi
done

set -a
# shellcheck source=/dev/null
source "$API_ENV_FILE"
# shellcheck source=/dev/null
source "$WEB_ENV_FILE"
# shellcheck source=/dev/null
source "$CADDY_ENV_FILE"
set +a

export EVERVAULT_TEAM_ID="${EVERVAULT_TEAM_ID:-${NEXT_PUBLIC_EVERVAULT_TEAM_ID:-}}"
export EVERVAULT_APP_ID="${EVERVAULT_APP_ID:-${NEXT_PUBLIC_EVERVAULT_APP_ID:-}}"

resolve_api_key_from_running_api_container() {
  if ! command -v docker >/dev/null 2>&1; then
    return 1
  fi
  if ! docker ps --format '{{.Names}}' | grep -qx 'bominal-api'; then
    return 1
  fi

  docker inspect --format '{{range .Config.Env}}{{println .}}{{end}}' bominal-api \
    | awk -F= '/^EVERVAULT_API_KEY=/{print substr($0, index($0, "=") + 1)}' \
    | tail -n1
}

if [[ -z "${EVERVAULT_API_KEY:-}" ]]; then
  candidate="$(resolve_api_key_from_running_api_container || true)"
  if [[ -n "$candidate" ]]; then
    export EVERVAULT_API_KEY="$candidate"
  fi
fi

if [[ -z "${EVERVAULT_API_KEY:-}" ]]; then
  if [[ -n "${EVERVAULT_API_KEY_SECRET_ID:-}" && -n "${EVERVAULT_API_KEY_SECRET_VERSION:-}" && -n "${GCP_PROJECT_ID:-}" ]]; then
    if ! command -v gcloud >/dev/null 2>&1; then
      echo "ERROR: gcloud CLI required to resolve EVERVAULT_API_KEY from GSM" >&2
      exit 1
    fi

    export EVERVAULT_API_KEY="$(gcloud secrets versions access "$EVERVAULT_API_KEY_SECRET_VERSION" --secret="$EVERVAULT_API_KEY_SECRET_ID" --project="$GCP_PROJECT_ID")"
  fi
fi

if [[ -z "${EV_TEST_DESTINATION_DOMAIN:-}" ]]; then
  site="${CADDY_SITE_ADDRESS:-www.bominal.com}"
  site="${site#http://}"
  site="${site#https://}"
  site="${site%%/*}"
  export EV_TEST_DESTINATION_DOMAIN="$site"
fi

if [[ -z "${EV_TEST_SHARED_SECRET:-}" ]]; then
  if command -v openssl >/dev/null 2>&1; then
    export EV_TEST_SHARED_SECRET="$(openssl rand -hex 24)"
  else
    export EV_TEST_SHARED_SECRET="evervault-test-$(date +%s)-$$"
  fi
fi

if [[ -z "${EVERVAULT_TEAM_ID:-}" || -z "${EVERVAULT_APP_ID:-}" || -z "${EVERVAULT_API_KEY:-}" ]]; then
  echo "ERROR: missing required Evervault credentials after env resolution" >&2
  echo "Required: EVERVAULT_TEAM_ID, EVERVAULT_APP_ID, EVERVAULT_API_KEY" >&2
  exit 1
fi

echo "INFO: building $IMAGE_TAG"
docker build -t "$IMAGE_TAG" "$ROOT_DIR/evervault-test"

if docker ps -a --format '{{.Names}}' | grep -qx "$CONTAINER_NAME"; then
  echo "INFO: removing existing container $CONTAINER_NAME"
  docker rm -f "$CONTAINER_NAME" >/dev/null
fi

echo "INFO: starting container $CONTAINER_NAME on host port $HOST_PORT"
docker run -d \
  --name "$CONTAINER_NAME" \
  --restart unless-stopped \
  -p "${HOST_PORT}:8787" \
  -e PORT=8787 \
  -e EVERVAULT_TEAM_ID="$EVERVAULT_TEAM_ID" \
  -e EVERVAULT_APP_ID="$EVERVAULT_APP_ID" \
  -e EVERVAULT_API_KEY="$EVERVAULT_API_KEY" \
  -e EVERVAULT_API_BASE_URL="${EVERVAULT_API_BASE_URL:-https://api.evervault.com}" \
  -e EV_TEST_DESTINATION_DOMAIN="$EV_TEST_DESTINATION_DOMAIN" \
  -e EV_TEST_LISTENER_PATH="${EV_TEST_LISTENER_PATH:-/evervault-test/relay-listener}" \
  -e EV_TEST_CARD_LISTENER_PATH="${EV_TEST_CARD_LISTENER_PATH:-/evervault-test/relay-listener-card}" \
  -e EV_TEST_SRT_LISTENER_PATH="${EV_TEST_SRT_LISTENER_PATH:-/evervault-test/srt-listener}" \
  -e EV_TEST_SHARED_SECRET="$EV_TEST_SHARED_SECRET" \
  -e EV_TEST_RESULT_TTL_SECONDS="${EV_TEST_RESULT_TTL_SECONDS:-120}" \
  -e EV_TEST_POLL_TIMEOUT_SECONDS="${EV_TEST_POLL_TIMEOUT_SECONDS:-20}" \
  "$IMAGE_TAG" >/dev/null

cat <<SUMMARY
OK: container is running.
- Container: $CONTAINER_NAME
- Image: $IMAGE_TAG
- URL: http://127.0.0.1:$HOST_PORT

Next:
1) Ensure Caddy routes /evervault-test/* to host.docker.internal:$HOST_PORT
2) Open https://$EV_TEST_DESTINATION_DOMAIN/evervault-test/
SUMMARY
