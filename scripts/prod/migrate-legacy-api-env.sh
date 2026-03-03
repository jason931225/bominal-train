#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <legacy_api_env_path> <runtime_env_output_path>" >&2
  exit 1
fi

legacy_env_path="$1"
runtime_env_output_path="$2"

if [ ! -f "${legacy_env_path}" ]; then
  echo "legacy env file not found: ${legacy_env_path}" >&2
  exit 1
fi

require_cmd() {
  local cmd="$1"
  if ! command -v "${cmd}" >/dev/null 2>&1; then
    echo "missing required command: ${cmd}" >&2
    exit 1
  fi
}

require_non_empty() {
  local key="$1"
  local value="$2"
  if [ -z "${value}" ]; then
    echo "missing required key after migration mapping: ${key}" >&2
    exit 1
  fi
}

trim_outer_quotes() {
  local value="$1"
  local first
  local last

  if [ "${#value}" -lt 2 ]; then
    printf '%s' "${value}"
    return
  fi

  first="${value:0:1}"
  last="${value: -1}"
  if { [ "${first}" = '"' ] && [ "${last}" = '"' ]; } || { [ "${first}" = "'" ] && [ "${last}" = "'" ]; }; then
    printf '%s' "${value:1:${#value}-2}"
  else
    printf '%s' "${value}"
  fi
}

read_legacy_key() {
  local key="$1"
  local raw
  raw="$(
    awk -v target="${key}" '
      /^[[:space:]]*#/ { next }
      /^[[:space:]]*$/ { next }
      {
        pos = index($0, "=")
        if (pos == 0) next
        name = substr($0, 1, pos - 1)
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", name)
        if (name == target) {
          print substr($0, pos + 1)
          exit
        }
      }
    ' "${legacy_env_path}"
  )"
  trim_outer_quotes "${raw}"
}

first_non_empty() {
  local value
  for value in "$@"; do
    if [ -n "${value}" ]; then
      printf '%s' "${value}"
      return
    fi
  done
  printf ''
}

looks_like_database_url() {
  local value="$1"
  case "${value}" in
    postgres://*|postgresql://*|postgresql+asyncpg://*|postgresql+psycopg://*|postgresql+psycopg2://*)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

to_seconds_or_default() {
  local raw_ms="$1"
  local default_seconds="$2"
  if [[ "${raw_ms}" =~ ^[0-9]+$ ]]; then
    local seconds=$((raw_ms / 1000))
    if [ "${seconds}" -lt 1 ]; then
      seconds=1
    fi
    printf '%s' "${seconds}"
  else
    printf '%s' "${default_seconds}"
  fi
}

require_cmd mktemp
require_cmd openssl

legacy_database_url_target="$(read_legacy_key DATABASE_URL_TARGET)"
legacy_database_url="$(read_legacy_key DATABASE_URL)"

if looks_like_database_url "${legacy_database_url_target}"; then
  database_url="${legacy_database_url_target}"
else
  database_url="${legacy_database_url}"
fi

app_env="$(first_non_empty "$(read_legacy_key APP_ENV)" "production")"
log_json="$(first_non_empty "$(read_legacy_key LOG_JSON)" "true")"
frontend_assets_dir="$(first_non_empty "$(read_legacy_key FRONTEND_ASSETS_DIR)" "/app/frontend/dist")"
redis_url="$(first_non_empty "$(read_legacy_key REDIS_URL_NON_CDE)" "$(read_legacy_key REDIS_URL)")"
invite_base_url="$(first_non_empty "$(read_legacy_key INVITE_BASE_URL)" "$(read_legacy_key APP_PUBLIC_BASE_URL)")"
session_secret="$(read_legacy_key SESSION_SECRET)"
webauthn_rp_id="$(read_legacy_key PASSKEY_RP_ID)"
webauthn_rp_origin="$(first_non_empty "$(read_legacy_key PASSKEY_ORIGIN)" "$(read_legacy_key APP_PUBLIC_BASE_URL)")"
webauthn_rp_name="$(first_non_empty "$(read_legacy_key WEBAUTHN_RP_NAME)" "bominal")"
webauthn_ttl_seconds="$(to_seconds_or_default "$(read_legacy_key PASSKEY_TIMEOUT_MS)" "300")"
internal_identity_secret="$(first_non_empty "$(read_legacy_key INTERNAL_IDENTITY_SECRET)" "$(read_legacy_key INTERNAL_API_KEY)")"
internal_identity_issuer="$(first_non_empty "$(read_legacy_key INTERNAL_IDENTITY_ISSUER)" "bominal-internal")"
kek_version="$(first_non_empty "$(read_legacy_key KEK_VERSION)" "1")"
master_key="$(read_legacy_key MASTER_KEY)"
master_key_override="$(read_legacy_key MASTER_KEY_OVERRIDE)"
email_from_address="$(read_legacy_key EMAIL_FROM_ADDRESS)"
resend_api_key="$(read_legacy_key RESEND_API_KEY)"
resend_base_url="$(first_non_empty "$(read_legacy_key RESEND_BASE_URL)" "$(read_legacy_key RESEND_API_BASE_URL)" "https://api.resend.com")"
evervault_relay_base_url="$(first_non_empty "$(read_legacy_key EVERVAULT_RELAY_BASE_URL)" "https://relay.evervault.com")"
evervault_app_id="$(read_legacy_key EVERVAULT_APP_ID)"
worker_poll_seconds="$(first_non_empty "$(read_legacy_key WORKER_POLL_SECONDS)" "3")"
worker_reconcile_seconds="$(first_non_empty "$(read_legacy_key WORKER_RECONCILE_SECONDS)" "30")"
worker_watch_seconds="$(first_non_empty "$(read_legacy_key WORKER_WATCH_SECONDS)" "5")"
key_rotation_seconds="$(first_non_empty "$(read_legacy_key KEY_ROTATION_SECONDS)" "3600")"
password_hash_concurrency="$(first_non_empty "$(read_legacy_key PASSWORD_HASH_CONCURRENCY)" "1")"
rqueue_key="$(first_non_empty "$(read_legacy_key RUNTIME_QUEUE_KEY)" "train:queue")"
rqueue_dlq_key="$(first_non_empty "$(read_legacy_key RUNTIME_QUEUE_DLQ_KEY)" "train:queue:dlq")"
rlease_prefix="$(first_non_empty "$(read_legacy_key RUNTIME_LEASE_PREFIX)" "train:lease")"
rrate_prefix="$(first_non_empty "$(read_legacy_key RUNTIME_RATE_LIMIT_PREFIX)" "rate_limit")"

if [ -z "${session_secret}" ]; then
  session_secret="$(openssl rand -hex 32)"
fi

require_non_empty "DATABASE_URL" "${database_url}"
require_non_empty "REDIS_URL" "${redis_url}"
require_non_empty "INVITE_BASE_URL" "${invite_base_url}"
require_non_empty "INTERNAL_IDENTITY_SECRET" "${internal_identity_secret}"
require_non_empty "MASTER_KEY" "${master_key}"
require_non_empty "EMAIL_FROM_ADDRESS" "${email_from_address}"
require_non_empty "RESEND_API_KEY" "${resend_api_key}"
require_non_empty "WEBAUTHN_RP_ID" "${webauthn_rp_id}"
require_non_empty "WEBAUTHN_RP_ORIGIN" "${webauthn_rp_origin}"

tmp_file="$(mktemp)"
cat > "${tmp_file}" <<EOF
# Generated by scripts/prod/migrate-legacy-api-env.sh from ${legacy_env_path}
APP_ENV=${app_env}
LOG_JSON=${log_json}
FRONTEND_ASSETS_DIR=${frontend_assets_dir}
INVITE_BASE_URL=${invite_base_url}
SESSION_SECRET=${session_secret}
DATABASE_URL=${database_url}
REDIS_URL=${redis_url}
RUNTIME_QUEUE_KEY=${rqueue_key}
RUNTIME_QUEUE_DLQ_KEY=${rqueue_dlq_key}
RUNTIME_LEASE_PREFIX=${rlease_prefix}
RUNTIME_RATE_LIMIT_PREFIX=${rrate_prefix}
INTERNAL_IDENTITY_SECRET=${internal_identity_secret}
INTERNAL_IDENTITY_ISSUER=${internal_identity_issuer}
PASSKEY_PROVIDER=server_webauthn
WEBAUTHN_RP_ID=${webauthn_rp_id}
WEBAUTHN_RP_ORIGIN=${webauthn_rp_origin}
WEBAUTHN_RP_NAME=${webauthn_rp_name}
WEBAUTHN_CHALLENGE_TTL_SECONDS=${webauthn_ttl_seconds}
KEK_VERSION=${kek_version}
MASTER_KEY=${master_key}
MASTER_KEY_OVERRIDE=${master_key_override}
EMAIL_FROM_ADDRESS=${email_from_address}
RESEND_API_KEY=${resend_api_key}
RESEND_BASE_URL=${resend_base_url}
EVERVAULT_RELAY_BASE_URL=${evervault_relay_base_url}
EVERVAULT_APP_ID=${evervault_app_id}
WORKER_POLL_SECONDS=${worker_poll_seconds}
WORKER_RECONCILE_SECONDS=${worker_reconcile_seconds}
WORKER_WATCH_SECONDS=${worker_watch_seconds}
KEY_ROTATION_SECONDS=${key_rotation_seconds}
PASSWORD_HASH_CONCURRENCY=${password_hash_concurrency}
EOF

umask 077
mkdir -p "$(dirname "${runtime_env_output_path}")"
mv "${tmp_file}" "${runtime_env_output_path}"
echo "wrote ${runtime_env_output_path}"
