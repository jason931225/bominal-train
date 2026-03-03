#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/prod/_env_lib.sh
source "${SCRIPT_DIR}/_env_lib.sh"

require_cmd docker
require_cmd mktemp
require_var BOMINAL_RUNTIME_ENV_PATH
require_var BOMINAL_COMPOSE_FILE
require_var BOMINAL_API_SERVICE
require_var BOMINAL_WORKER_SERVICE

if [ ! -f "${BOMINAL_RUNTIME_ENV_PATH}" ]; then
  fail "runtime env file not found: ${BOMINAL_RUNTIME_ENV_PATH}"
fi
if [ ! -f "${BOMINAL_COMPOSE_FILE}" ]; then
  fail "compose file not found: ${BOMINAL_COMPOSE_FILE}"
fi

rollback_state_path="${BOMINAL_ROLLBACK_STATE_PATH:-${PWD}/.deploy/rollback.env}"
if [ ! -f "${rollback_state_path}" ]; then
  fail "rollback state file not found: ${rollback_state_path}"
fi

set -a
# shellcheck disable=SC1090
source "${rollback_state_path}"
set +a

set_env_key "${BOMINAL_RUNTIME_ENV_PATH}" "BOMINAL_API_IMAGE" "${BOMINAL_PREV_API_IMAGE:-}"
set_env_key "${BOMINAL_RUNTIME_ENV_PATH}" "BOMINAL_WORKER_IMAGE" "${BOMINAL_PREV_WORKER_IMAGE:-}"
set_env_key "${BOMINAL_RUNTIME_ENV_PATH}" "DATABASE_URL" "${BOMINAL_PREV_DATABASE_URL:-}"

log "rolling back services"
compose_cmd "${BOMINAL_RUNTIME_ENV_PATH}" "${BOMINAL_COMPOSE_FILE}" up -d --no-build \
  "${BOMINAL_API_SERVICE}" "${BOMINAL_WORKER_SERVICE}"

log "rollback script completed"
