#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
ENV_FILE="${REPO_ROOT}/env/prod/cloudrun-api.env"
OUTPUT_PATH=""
DRY_RUN=0

usage() {
  cat <<EOF
Usage: ./runtime/cloudrun/api/bootstrap.sh [options]

Renders a future Cloud Run API service manifest from env/prod/cloudrun-api.env.
This script does not deploy anything; it only materializes the YAML needed for a later cutover.

Options:
  --env-file PATH        Env file to load (default: env/prod/cloudrun-api.env)
  --output PATH          Output YAML path (default: runtime/cloudrun/api/rendered/<service>.yaml)
  --dry-run              Print rendered YAML to stdout, do not write a file.
  --help                 Show this help.
EOF
}

die() {
  printf 'bootstrap.sh: %s\n' "$*" >&2
  exit 1
}

require_var() {
  local key="$1"
  if [ -z "${!key:-}" ]; then
    die "missing required value: ${key}"
  fi
}

load_env_file() {
  [ -f "${ENV_FILE}" ] || die "missing env file: ${ENV_FILE}"
  set -a
  # shellcheck disable=SC1090
  source "${ENV_FILE}"
  set +a
}

count_gsm_secrets() {
  local keys=(
    GSM_DATABASE_URL_SECRET
    GSM_SESSION_SECRET_SECRET
    GSM_INTERNAL_IDENTITY_SECRET_SECRET
    GSM_MASTER_KEY_SECRET
    GSM_RESEND_API_KEY_SECRET
  )
  local key
  local value
  local other_key
  local other_value
  local unique_count=0

  for key in "${keys[@]}"; do
    value="${!key:-}"
    [ -n "${value}" ] || die "missing required GSM secret name: ${key}"
    unique_count=$((unique_count + 1))
    for other_key in "${keys[@]}"; do
      [ "${other_key}" = "${key}" ] && continue
      other_value="${!other_key:-}"
      if [ -n "${other_value}" ] && [ "${other_value}" = "${value}" ]; then
        die "duplicate GSM secret name '${value}' reused by ${key} and ${other_key}"
      fi
    done
  done

  if [ "${unique_count}" -gt 5 ]; then
    die "configured ${unique_count} GSM secrets; policy limit is 5"
  fi
}

build_network_interfaces() {
  if [ -n "${CLOUDRUN_API_VPC_NETWORK_TAGS:-}" ]; then
    printf '[{"network":"%s","subnetwork":"%s","tags":"%s"}]' \
      "${CLOUDRUN_API_VPC_NETWORK}" \
      "${CLOUDRUN_API_VPC_SUBNET}" \
      "${CLOUDRUN_API_VPC_NETWORK_TAGS}"
  else
    printf '[{"network":"%s","subnetwork":"%s"}]' \
      "${CLOUDRUN_API_VPC_NETWORK}" \
      "${CLOUDRUN_API_VPC_SUBNET}"
  fi
}

render_template() {
  local template
  local network_interfaces

  template="$(cat "${SCRIPT_DIR}/service.yaml")"
  network_interfaces="$(build_network_interfaces)"

  template="${template//__CLOUDRUN_API_SERVICE__/${CLOUDRUN_API_SERVICE}}"
  template="${template//__CLOUDRUN_API_MIN_INSTANCES__/${CLOUDRUN_API_MIN_INSTANCES}}"
  template="${template//__CLOUDRUN_API_MAX_INSTANCES__/${CLOUDRUN_API_MAX_INSTANCES}}"
  template="${template//__CLOUDRUN_API_STARTUP_CPU_BOOST__/${CLOUDRUN_API_STARTUP_CPU_BOOST}}"
  template="${template//__CLOUDRUN_NETWORK_INTERFACES__/${network_interfaces}}"
  template="${template//__CLOUDRUN_API_CONTAINER_CONCURRENCY__/${CLOUDRUN_API_CONTAINER_CONCURRENCY}}"
  template="${template//__CLOUDRUN_API_TIMEOUT_SECONDS__/${CLOUDRUN_API_TIMEOUT_SECONDS}}"
  template="${template//__CLOUDRUN_API_SERVICE_ACCOUNT__/${CLOUDRUN_API_SERVICE_ACCOUNT}}"
  template="${template//__CLOUDRUN_API_IMAGE__/${CLOUDRUN_API_IMAGE}}"
  template="${template//__CLOUDRUN_API_CPU__/${CLOUDRUN_API_CPU}}"
  template="${template//__CLOUDRUN_API_MEMORY__/${CLOUDRUN_API_MEMORY}}"
  template="${template//__USER_APP_HOST__/${USER_APP_HOST}}"
  template="${template//__ADMIN_APP_HOST__/${ADMIN_APP_HOST}}"
  template="${template//__SESSION_COOKIE_DOMAIN__/${SESSION_COOKIE_DOMAIN}}"
  template="${template//__INVITE_BASE_URL__/${INVITE_BASE_URL}}"
  template="${template//__EMAIL_FROM_ADDRESS__/${EMAIL_FROM_ADDRESS}}"
  template="${template//__WEBAUTHN_RP_ID__/${WEBAUTHN_RP_ID}}"
  template="${template//__WEBAUTHN_RP_ORIGIN__/${WEBAUTHN_RP_ORIGIN}}"
  template="${template//__REDIS_URL__/${REDIS_URL}}"
  template="${template//__HTTP_REQUEST_TIMEOUT_SECONDS__/${HTTP_REQUEST_TIMEOUT_SECONDS}}"
  template="${template//__HTTP_REQUEST_BODY_LIMIT_BYTES__/${HTTP_REQUEST_BODY_LIMIT_BYTES}}"
  template="${template//__HTTP_CONCURRENCY_LIMIT__/${HTTP_CONCURRENCY_LIMIT}}"
  template="${template//__PASSWORD_HASH_CONCURRENCY__/${PASSWORD_HASH_CONCURRENCY}}"
  template="${template//__API_DB_POOL_MAX_CONNECTIONS__/${API_DB_POOL_MAX_CONNECTIONS}}"
  template="${template//__DB_POOL_ACQUIRE_TIMEOUT_SECONDS__/${DB_POOL_ACQUIRE_TIMEOUT_SECONDS}}"
  template="${template//__DB_POOL_IDLE_TIMEOUT_SECONDS__/${DB_POOL_IDLE_TIMEOUT_SECONDS}}"
  template="${template//__DB_POOL_MAX_LIFETIME_SECONDS__/${DB_POOL_MAX_LIFETIME_SECONDS}}"
  template="${template//__GSM_DATABASE_URL_SECRET__/${GSM_DATABASE_URL_SECRET}}"
  template="${template//__GSM_SESSION_SECRET_SECRET__/${GSM_SESSION_SECRET_SECRET}}"
  template="${template//__GSM_INTERNAL_IDENTITY_SECRET_SECRET__/${GSM_INTERNAL_IDENTITY_SECRET_SECRET}}"
  template="${template//__GSM_MASTER_KEY_SECRET__/${GSM_MASTER_KEY_SECRET}}"
  template="${template//__GSM_RESEND_API_KEY_SECRET__/${GSM_RESEND_API_KEY_SECRET}}"

  printf '%s\n' "${template}"
}

parse_args() {
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --env-file)
        [ "$#" -ge 2 ] || die "--env-file requires a path"
        ENV_FILE="$2"
        shift
        ;;
      --output)
        [ "$#" -ge 2 ] || die "--output requires a path"
        OUTPUT_PATH="$2"
        shift
        ;;
      --dry-run)
        DRY_RUN=1
        ;;
      --help|-h)
        usage
        exit 0
        ;;
      *)
        die "unknown option: $1"
        ;;
    esac
    shift
  done
}

main() {
  parse_args "$@"
  load_env_file

  require_var CLOUDRUN_API_SERVICE
  require_var CLOUDRUN_API_REGION
  require_var CLOUDRUN_API_IMAGE
  require_var CLOUDRUN_API_SERVICE_ACCOUNT
  require_var CLOUDRUN_API_VPC_NETWORK
  require_var CLOUDRUN_API_VPC_SUBNET
  require_var CLOUDRUN_API_MIN_INSTANCES
  require_var CLOUDRUN_API_MAX_INSTANCES
  require_var CLOUDRUN_API_CONTAINER_CONCURRENCY
  require_var CLOUDRUN_API_TIMEOUT_SECONDS
  require_var CLOUDRUN_API_CPU
  require_var CLOUDRUN_API_MEMORY
  require_var CLOUDRUN_API_STARTUP_CPU_BOOST
  require_var USER_APP_HOST
  require_var ADMIN_APP_HOST
  require_var SESSION_COOKIE_DOMAIN
  require_var INVITE_BASE_URL
  require_var EMAIL_FROM_ADDRESS
  require_var WEBAUTHN_RP_ID
  require_var WEBAUTHN_RP_ORIGIN
  require_var REDIS_URL
  require_var HTTP_REQUEST_TIMEOUT_SECONDS
  require_var HTTP_REQUEST_BODY_LIMIT_BYTES
  require_var HTTP_CONCURRENCY_LIMIT
  require_var PASSWORD_HASH_CONCURRENCY
  require_var API_DB_POOL_MAX_CONNECTIONS
  require_var DB_POOL_ACQUIRE_TIMEOUT_SECONDS
  require_var DB_POOL_IDLE_TIMEOUT_SECONDS
  require_var DB_POOL_MAX_LIFETIME_SECONDS
  count_gsm_secrets

  if [ -z "${OUTPUT_PATH}" ]; then
    OUTPUT_PATH="${SCRIPT_DIR}/rendered/${CLOUDRUN_API_SERVICE}.yaml"
  fi

  if [ "${DRY_RUN}" = "1" ]; then
    render_template
  else
    mkdir -p "$(dirname "${OUTPUT_PATH}")"
    render_template > "${OUTPUT_PATH}"
    printf 'rendered %s\n' "${OUTPUT_PATH}"
  fi

  printf 'region=%s\n' "${CLOUDRUN_API_REGION}"
  printf 'gsm_secrets=%s,%s,%s,%s,%s\n' \
    "${GSM_DATABASE_URL_SECRET}" \
    "${GSM_SESSION_SECRET_SECRET}" \
    "${GSM_INTERNAL_IDENTITY_SECRET_SECRET}" \
    "${GSM_MASTER_KEY_SECRET}" \
    "${GSM_RESEND_API_KEY_SECRET}"
  printf 'plain_env=REDIS_URL,USER_APP_HOST,ADMIN_APP_HOST,SESSION_COOKIE_DOMAIN,INVITE_BASE_URL,EMAIL_FROM_ADDRESS,WEBAUTHN_RP_ID,WEBAUTHN_RP_ORIGIN,HTTP_REQUEST_TIMEOUT_SECONDS,HTTP_REQUEST_BODY_LIMIT_BYTES,HTTP_CONCURRENCY_LIMIT,PASSWORD_HASH_CONCURRENCY,API_DB_POOL_MAX_CONNECTIONS,DB_POOL_ACQUIRE_TIMEOUT_SECONDS,DB_POOL_IDLE_TIMEOUT_SECONDS,DB_POOL_MAX_LIFETIME_SECONDS\n'
  printf 'next_step=gcloud run services replace %s --region=%s\n' "${OUTPUT_PATH}" "${CLOUDRUN_API_REGION}"
}

main "$@"
