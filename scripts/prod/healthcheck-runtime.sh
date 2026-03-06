#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/prod/_env_lib.sh
source "${SCRIPT_DIR}/_env_lib.sh"

require_cmd curl
require_cmd docker

require_var BOMINAL_RUNTIME_ENV_PATH
require_var BOMINAL_COMPOSE_FILE
require_var BOMINAL_WORKER_SERVICE
require_var BOMINAL_HEALTHCHECK_LIVE_URL
require_var BOMINAL_HEALTHCHECK_READY_URL

if [ ! -f "${BOMINAL_RUNTIME_ENV_PATH}" ]; then
  fail "runtime env file not found: ${BOMINAL_RUNTIME_ENV_PATH}"
fi
if [ ! -f "${BOMINAL_COMPOSE_FILE}" ]; then
  fail "compose file not found: ${BOMINAL_COMPOSE_FILE}"
fi

check_endpoint() {
  local endpoint="$1"
  local retries="${BOMINAL_HEALTHCHECK_RETRIES:-20}"
  local delay_seconds="${BOMINAL_HEALTHCHECK_DELAY_SECONDS:-3}"
  local try=1

  if ! [[ "${retries}" =~ ^[0-9]+$ ]] || [ "${retries}" -lt 1 ]; then
    fail "BOMINAL_HEALTHCHECK_RETRIES must be an integer >= 1"
  fi
  if ! [[ "${delay_seconds}" =~ ^[0-9]+$ ]]; then
    fail "BOMINAL_HEALTHCHECK_DELAY_SECONDS must be an integer >= 0"
  fi

  while [ "${try}" -le "${retries}" ]; do
    if curl -fsS --max-time 5 "${endpoint}" >/dev/null 2>&1; then
      return 0
    fi
    sleep "${delay_seconds}"
    try=$((try + 1))
  done

  return 1
}

log "checking live endpoint: ${BOMINAL_HEALTHCHECK_LIVE_URL}"
check_endpoint "${BOMINAL_HEALTHCHECK_LIVE_URL}" || fail "live health check failed"

log "checking ready endpoint: ${BOMINAL_HEALTHCHECK_READY_URL}"
check_endpoint "${BOMINAL_HEALTHCHECK_READY_URL}" || fail "ready health check failed"

running_services="$(
  compose_cmd "${BOMINAL_RUNTIME_ENV_PATH}" "${BOMINAL_COMPOSE_FILE}" ps \
    --services --status running | tr -d '\r'
)"

printf '%s\n' "${running_services}" | grep -qx "${BOMINAL_WORKER_SERVICE}" || fail "worker service not running: ${BOMINAL_WORKER_SERVICE}"

log "health checks completed"
