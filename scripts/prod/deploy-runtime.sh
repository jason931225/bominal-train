#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/prod/_env_lib.sh
source "${SCRIPT_DIR}/_env_lib.sh"

require_cmd docker
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
require_var BOMINAL_MIGRATIONS_DIR
require_var BOMINAL_API_SERVICE
require_var BOMINAL_WORKER_SERVICE

url_encode() {
  local raw="$1"
  local encoded=""
  local i
  local ch
  local hex

  for ((i = 0; i < ${#raw}; i++)); do
    ch="${raw:i:1}"
    case "${ch}" in
      [a-zA-Z0-9.~_-])
        encoded+="${ch}"
        ;;
      *)
        printf -v hex '%02X' "'${ch}"
        encoded+="%${hex}"
        ;;
    esac
  done

  printf '%s' "${encoded}"
}

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
if [ ! -d "${BOMINAL_MIGRATIONS_DIR}" ]; then
  fail "migrations directory not found: ${BOMINAL_MIGRATIONS_DIR}"
fi

secret_dir="$(dirname "${BOMINAL_VM_SECRET_ENV_FILE}")"
mkdir -p "${secret_dir}"
if [ ! -f "${BOMINAL_VM_SECRET_ENV_FILE}" ]; then
  umask 077
  : > "${BOMINAL_VM_SECRET_ENV_FILE}"
  log "created missing vm secret env file: ${BOMINAL_VM_SECRET_ENV_FILE}"
fi
chmod 600 "${BOMINAL_VM_SECRET_ENV_FILE}"

database_url="${BOMINAL_DATABASE_URL:-}"
if [ -z "${database_url}" ]; then
  database_url="$(read_env_key "${BOMINAL_VM_SECRET_ENV_FILE}" "BOMINAL_DATABASE_URL")"
fi
if [ -z "${database_url}" ]; then
  database_url="$(read_env_key "${BOMINAL_RUNTIME_ENV_PATH}" "DATABASE_URL")"
fi

if [ -z "${database_url}" ]; then
  postgres_password="$(read_env_key "${BOMINAL_VM_SECRET_ENV_FILE}" "BOMINAL_POSTGRES_PASSWORD")"
  if [ -z "${postgres_password}" ]; then
    postgres_password="${BOMINAL_POSTGRES_PASSWORD:-}"
  fi
  if [ -z "${postgres_password}" ]; then
    fail "unable to bootstrap database URL: set BOMINAL_DATABASE_URL or BOMINAL_POSTGRES_PASSWORD in VM secret env"
  fi

  encoded_password="$(url_encode "${postgres_password}")"
  database_url="postgresql+asyncpg://${BOMINAL_POSTGRES_USER}:${encoded_password}@${BOMINAL_POSTGRES_HOST}:${BOMINAL_POSTGRES_PORT}/${BOMINAL_POSTGRES_DB}"
fi

set_env_key "${BOMINAL_VM_SECRET_ENV_FILE}" "BOMINAL_DATABASE_URL" "${database_url}"
chmod 600 "${BOMINAL_VM_SECRET_ENV_FILE}"

migrations_script="${SCRIPT_DIR}/apply-migrations.sh"
if [ ! -f "${migrations_script}" ]; then
  fail "migrations script not found: ${migrations_script}"
fi
if [ ! -x "${migrations_script}" ]; then
  fail "migrations script is not executable: ${migrations_script}"
fi

export BOMINAL_DATABASE_URL="${database_url}"
"${migrations_script}"

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
