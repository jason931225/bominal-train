#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/prod/_env_lib.sh
source "${SCRIPT_DIR}/_env_lib.sh"

require_cmd docker
require_cmd python3
require_cmd mktemp

require_var BOMINAL_API_IMAGE
require_var BOMINAL_WORKER_IMAGE
require_var BOMINAL_POSTGRES_HOST
require_var BOMINAL_POSTGRES_PORT
require_var BOMINAL_POSTGRES_DB
require_var BOMINAL_POSTGRES_USER
require_var BOMINAL_VM_SECRET_ENV_FILE
require_var BOMINAL_RUNTIME_ENV_PATH
require_var BOMINAL_COMPOSE_FILE
require_var BOMINAL_API_SERVICE
require_var BOMINAL_WORKER_SERVICE

if [ ! -f "${BOMINAL_VM_SECRET_ENV_FILE}" ]; then
  fail "secret env file not found: ${BOMINAL_VM_SECRET_ENV_FILE}"
fi

set -a
# shellcheck disable=SC1090
source "${BOMINAL_VM_SECRET_ENV_FILE}"
set +a

database_url="${BOMINAL_DATABASE_URL:-}"
if [ -z "${database_url}" ]; then
  require_var BOMINAL_POSTGRES_PASSWORD
  encoded_password="$(python3 -c 'import sys, urllib.parse; print(urllib.parse.quote(sys.argv[1], safe=""))' "${BOMINAL_POSTGRES_PASSWORD}")"
  database_url="postgresql+asyncpg://${BOMINAL_POSTGRES_USER}:${encoded_password}@${BOMINAL_POSTGRES_HOST}:${BOMINAL_POSTGRES_PORT}/${BOMINAL_POSTGRES_DB}"
fi

if [ -n "${BOMINAL_VM_BASELINE_SCRIPT:-}" ]; then
  baseline_script="${BOMINAL_VM_BASELINE_SCRIPT}"
else
  baseline_script="${SCRIPT_DIR}/ensure-vm-baseline.sh"
fi
if [ ! -f "${baseline_script}" ]; then
  fail "baseline script not found: ${baseline_script}"
fi
if [ ! -x "${baseline_script}" ]; then
  fail "baseline script is not executable: ${baseline_script}"
fi
"${baseline_script}"

if [ ! -f "${BOMINAL_RUNTIME_ENV_PATH}" ]; then
  fail "runtime env file not found: ${BOMINAL_RUNTIME_ENV_PATH}"
fi
if [ ! -f "${BOMINAL_COMPOSE_FILE}" ]; then
  fail "compose file not found: ${BOMINAL_COMPOSE_FILE}"
fi

rollback_state_path="${BOMINAL_ROLLBACK_STATE_PATH:-${PWD}/.deploy/rollback.env}"
mkdir -p "$(dirname "${rollback_state_path}")"

prev_api_image="$(read_env_key "${BOMINAL_RUNTIME_ENV_PATH}" "BOMINAL_API_IMAGE")"
prev_worker_image="$(read_env_key "${BOMINAL_RUNTIME_ENV_PATH}" "BOMINAL_WORKER_IMAGE")"
prev_database_url="$(read_env_key "${BOMINAL_RUNTIME_ENV_PATH}" "DATABASE_URL")"

umask 077
{
  printf 'BOMINAL_PREV_API_IMAGE=%q\n' "${prev_api_image}"
  printf 'BOMINAL_PREV_WORKER_IMAGE=%q\n' "${prev_worker_image}"
  printf 'BOMINAL_PREV_DATABASE_URL=%q\n' "${prev_database_url}"
} > "${rollback_state_path}"

set_env_key "${BOMINAL_RUNTIME_ENV_PATH}" "BOMINAL_API_IMAGE" "${BOMINAL_API_IMAGE}"
set_env_key "${BOMINAL_RUNTIME_ENV_PATH}" "BOMINAL_WORKER_IMAGE" "${BOMINAL_WORKER_IMAGE}"
set_env_key "${BOMINAL_RUNTIME_ENV_PATH}" "DATABASE_URL" "${database_url}"

log "pulling deploy images"
compose_cmd "${BOMINAL_RUNTIME_ENV_PATH}" "${BOMINAL_COMPOSE_FILE}" pull \
  "${BOMINAL_API_SERVICE}" "${BOMINAL_WORKER_SERVICE}"

log "starting updated services"
compose_cmd "${BOMINAL_RUNTIME_ENV_PATH}" "${BOMINAL_COMPOSE_FILE}" up -d --no-build \
  "${BOMINAL_API_SERVICE}" "${BOMINAL_WORKER_SERVICE}"

log "deploy script completed"
