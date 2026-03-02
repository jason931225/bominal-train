#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

API_ENV_FILE="${EVERVAULT_TEST_API_ENV_FILE:-$ROOT_DIR/infra/env/prod/api.env}"
WEB_ENV_FILE="${EVERVAULT_TEST_WEB_ENV_FILE:-$ROOT_DIR/infra/env/prod/web.env}"
CADDY_ENV_FILE="${EVERVAULT_TEST_CADDY_ENV_FILE:-$ROOT_DIR/infra/env/prod/caddy.env}"

if [[ ! -f "$API_ENV_FILE" ]]; then
  echo "ERROR: missing API env file: $API_ENV_FILE" >&2
  exit 1
fi

if [[ ! -f "$WEB_ENV_FILE" ]]; then
  echo "ERROR: missing web env file: $WEB_ENV_FILE" >&2
  exit 1
fi

if [[ ! -f "$CADDY_ENV_FILE" ]]; then
  echo "ERROR: missing caddy env file: $CADDY_ENV_FILE" >&2
  exit 1
fi

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

if [[ -z "${EVERVAULT_API_KEY:-}" ]]; then
  if [[ -n "${EVERVAULT_API_KEY_SECRET_ID:-}" && -n "${EVERVAULT_API_KEY_SECRET_VERSION:-}" && -n "${GCP_PROJECT_ID:-}" ]]; then
    if ! command -v gcloud >/dev/null 2>&1; then
      echo "ERROR: gcloud CLI required to resolve EVERVAULT_API_KEY from GSM" >&2
      exit 1
    fi

    EVERVAULT_API_KEY="$(gcloud secrets versions access "$EVERVAULT_API_KEY_SECRET_VERSION" --secret="$EVERVAULT_API_KEY_SECRET_ID" --project="$GCP_PROJECT_ID")"
    export EVERVAULT_API_KEY
  fi
fi

if [[ -z "${EV_TEST_DESTINATION_DOMAIN:-}" ]]; then
  site="${CADDY_SITE_ADDRESS:-www.bominal.com}"
  site="${site#http://}"
  site="${site#https://}"
  site="${site%%/*}"
  export EV_TEST_DESTINATION_DOMAIN="$site"
fi

export EV_TEST_CARD_LISTENER_PATH="${EV_TEST_CARD_LISTENER_PATH:-/evervault-test/relay-listener-card}"
export EV_TEST_SRT_LISTENER_PATH="${EV_TEST_SRT_LISTENER_PATH:-/evervault-test/srt-listener}"

if [[ -z "${EV_TEST_SHARED_SECRET:-}" ]]; then
  if command -v openssl >/dev/null 2>&1; then
    export EV_TEST_SHARED_SECRET="$(openssl rand -hex 24)"
  else
    export EV_TEST_SHARED_SECRET="evervault-test-$(date +%s)-$$"
  fi
fi

if [[ -z "${EVERVAULT_TEAM_ID:-}" || -z "${EVERVAULT_APP_ID:-}" || -z "${EVERVAULT_API_KEY:-}" ]]; then
  echo "ERROR: missing required Evervault credentials after sourcing prod env files" >&2
  echo "Required: EVERVAULT_TEAM_ID, EVERVAULT_APP_ID, EVERVAULT_API_KEY" >&2
  exit 1
fi

cd "$ROOT_DIR/evervault-test"
exec npm start
