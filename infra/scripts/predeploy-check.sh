#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# predeploy-check.sh — Validate production environment before deployment
# -----------------------------------------------------------------------------
# Checks:
#   - Required env files exist (api.env, web.env, caddy.env)
#   - No unresolved CHANGE_ME placeholders
#   - Required API/deploy/auth/email settings are set
#
# Usage:
#   ./infra/scripts/predeploy-check.sh
#
# Exit codes:
#   0 - All checks passed
#   1 - One or more checks failed
# -----------------------------------------------------------------------------
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/lib/env_utils.sh"

ROOT_DIR="${BOMINAL_ROOT_DIR:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
cd "$ROOT_DIR"

required_files=(
  "infra/env/prod/api.env"
  "infra/env/prod/web.env"
  "infra/env/prod/caddy.env"
)

require_running_services=0
skip_smoke_tests=0
min_total_memory_mb="${PREDEPLOY_MIN_TOTAL_MEMORY_MB:-0}"
min_total_swap_mb="${PREDEPLOY_MIN_TOTAL_SWAP_MB:-0}"
allow_deprecation_bypass="${PREDEPLOY_ALLOW_DEPRECATION_BYPASS:-false}"
allow_policy_gates_bypass="${PREDEPLOY_ALLOW_POLICY_GATES_BYPASS:-false}"
deprecation_registry_path="${PREDEPLOY_DEPRECATION_REGISTRY_PATH:-$ROOT_DIR/docs/deprecations/registry.json}"
deprecation_guard_script="${PREDEPLOY_DEPRECATION_GUARD_SCRIPT:-$ROOT_DIR/infra/scripts/deprecation_guard.py}"

require_nonnegative_integer() {
  local value="$1"
  local flag_name="$2"
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    log_error "$flag_name expects a non-negative integer, got: $value"
    exit 1
  fi
}

get_total_memory_mb() {
  free -m | awk '/^Mem:/ {print $2}'
}

get_total_swap_mb() {
  local swap_bytes
  if command -v swapon >/dev/null 2>&1; then
    swap_bytes=$(swapon --show=SIZE --bytes --noheadings 2>/dev/null | awk '{sum += $1} END {print sum + 0}')
    if [[ "$swap_bytes" =~ ^[0-9]+$ ]] && [[ "$swap_bytes" -gt 0 ]]; then
      echo $((swap_bytes / 1024 / 1024))
      return 0
    fi
  fi
  free -m | awk '/^Swap:/ {print $2}'
}

run_resource_gate() {
  if [[ "$min_total_memory_mb" -le 0 ]] && [[ "$min_total_swap_mb" -le 0 ]]; then
    return 0
  fi

  local total_memory_mb total_swap_mb
  total_memory_mb="$(get_total_memory_mb)"
  total_swap_mb="$(get_total_swap_mb)"

  if [[ ! "$total_memory_mb" =~ ^[0-9]+$ ]]; then
    log_error "Could not determine total memory in MB from 'free -m'."
    exit 1
  fi
  if [[ ! "$total_swap_mb" =~ ^[0-9]+$ ]]; then
    log_error "Could not determine total swap in MB."
    exit 1
  fi

  echo "==> Checking resource profile (total memory=${total_memory_mb}MB, total swap=${total_swap_mb}MB)"

  if [[ "$min_total_memory_mb" -gt 0 ]] && [[ "$total_memory_mb" -lt "$min_total_memory_mb" ]]; then
    log_error "Insufficient total memory: ${total_memory_mb}MB < required ${min_total_memory_mb}MB."
    exit 1
  fi

  if [[ "$min_total_swap_mb" -gt 0 ]] && [[ "$total_swap_mb" -lt "$min_total_swap_mb" ]]; then
    log_error "Insufficient total swap: ${total_swap_mb}MB < required ${min_total_swap_mb}MB."
    exit 1
  fi
}

require_boolean_like() {
  local value="$1"
  local name="$2"
  case "$value" in
    true|false|1|0|yes|no)
      ;;
    *)
      log_error "$name expects one of: true|false|1|0|yes|no (got: $value)"
      exit 1
      ;;
  esac
}

is_truthy() {
  local value="$1"
  case "$value" in
    true|1|yes)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

is_placeholder_value() {
  local value="$1"
  case "$value" in
    *CHANGE_ME*|*REPLACE_ME*|*TODO*|*"<no value>"*)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

has_meaningful_value() {
  local value="$1"
  local normalized
  normalized="${value#"${value%%[![:space:]]*}"}"
  normalized="${normalized%"${normalized##*[![:space:]]}"}"
  if [[ -z "$normalized" ]]; then
    return 1
  fi
  if is_placeholder_value "$normalized"; then
    return 1
  fi
  return 0
}

require_positive_number() {
  local value="$1"
  local name="$2"
  if ! [[ "$value" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
    log_error "$name must be a positive number (got: ${value:-<empty>})"
    exit 1
  fi
  awk -v v="$value" 'BEGIN { if (v <= 0) exit 1 }' || {
    log_error "$name must be > 0 (got: ${value})"
    exit 1
  }
}

require_positive_integer() {
  local value="$1"
  local name="$2"
  if ! [[ "$value" =~ ^[0-9]+$ ]]; then
    log_error "$name must be a positive integer (got: ${value:-<empty>})"
    exit 1
  fi
  if [[ "$value" -le 0 ]]; then
    log_error "$name must be > 0 (got: ${value})"
    exit 1
  fi
}

require_pinned_secret_version() {
  local value="$1"
  local name="$2"
  if [[ -z "$value" ]]; then
    log_error "$name is required when corresponding *_SECRET_ID is configured."
    exit 1
  fi
  if [[ "$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')" == "latest" ]]; then
    log_error "$name must be pinned in production (latest is not allowed)."
    exit 1
  fi
}

require_https_url() {
  local value="$1"
  local name="$2"
  if [[ ! "$value" =~ ^https:// ]]; then
    log_error "$name must start with https:// in production (got: ${value:-<empty>})"
    exit 1
  fi
}

require_https_url_or_empty() {
  local value="$1"
  local name="$2"
  if [[ -z "$value" ]]; then
    return 0
  fi
  require_https_url "$value" "$name"
}

url_host() {
  local value="$1"
  local host
  host="${value#https://}"
  host="${host#http://}"
  host="${host%%/*}"
  host="${host%%:*}"
  printf '%s' "$host" | tr '[:upper:]' '[:lower:]'
}

require_non_local_https_url() {
  local value="$1"
  local name="$2"
  require_https_url "$value" "$name"
  case "$(url_host "$value")" in
    localhost|127.0.0.1|0.0.0.0|::1)
      log_error "$name must not use localhost/loopback in production (got: $value)"
      exit 1
      ;;
    *)
      ;;
  esac
}

require_non_local_https_csv_urls() {
  local value="$1"
  local name="$2"
  local item
  IFS=',' read -ra items <<<"$value"
  for item in "${items[@]}"; do
    item="${item#"${item%%[![:space:]]*}"}"
    item="${item%"${item##*[![:space:]]}"}"
    [[ -z "$item" ]] && continue
    require_non_local_https_url "$item" "$name"
  done
}

require_csv_contains_verify_path() {
  local value="$1"
  local name="$2"
  local found=0
  local item
  IFS=',' read -ra items <<<"$value"
  for item in "${items[@]}"; do
    item="${item#"${item%%[![:space:]]*}"}"
    item="${item%"${item##*[![:space:]]}"}"
    [[ -z "$item" ]] && continue
    local without_scheme="${item#https://}"
    without_scheme="${without_scheme#http://}"
    local path="/"
    if [[ "$without_scheme" == */* ]]; then
      path="/${without_scheme#*/}"
    fi
    if [[ "$path" == "/auth/verify" ]] || [[ "$path" == "/auth/verify/"* ]]; then
      found=1
      break
    fi
  done
  if [[ "$found" -ne 1 ]]; then
    log_error "$name must include an /auth/verify URL."
    exit 1
  fi
}

require_supabase_database_url() {
  local value="$1"
  local name="$2"

  if [[ "$value" == *"@postgres:"* ]] || [[ "$value" == *"@localhost:"* ]] || [[ "$value" == *"@127.0.0.1:"* ]]; then
    log_error "$name must target managed Supabase Postgres, not local/container hostnames."
    exit 1
  fi

  if [[ "$value" != *".supabase.co"* ]]; then
    log_error "$name must point to a Supabase Postgres endpoint (*.supabase.co)."
    exit 1
  fi

  if [[ "$value" != *"sslmode=require"* ]] && [[ "$value" != *"ssl=require"* ]] && [[ "$value" != *"ssl=true"* ]]; then
    log_error "$name must require TLS (set sslmode=require or equivalent ssl=require/ssl=true)."
    exit 1
  fi
}

require_https_csv_origins() {
  local value="$1"
  local name="$2"
  local origin
  IFS=',' read -ra origins <<<"$value"
  for origin in "${origins[@]}"; do
    origin="${origin#"${origin%%[![:space:]]*}"}"
    origin="${origin%"${origin##*[![:space:]]}"}"
    [[ -z "$origin" ]] && continue
    require_https_url "$origin" "$name"
  done
}

run_deprecation_gate() {
  require_boolean_like "$allow_deprecation_bypass" "PREDEPLOY_ALLOW_DEPRECATION_BYPASS"

  if is_truthy "$allow_deprecation_bypass"; then
    log_warn "Skipping deprecation deploy gate (PREDEPLOY_ALLOW_DEPRECATION_BYPASS=true)."
    return 0
  fi

  require_file "$deprecation_registry_path"
  require_file "$deprecation_guard_script"
  if ! command -v python3 >/dev/null 2>&1; then
    log_error "python3 is required for deprecation gate checks."
    exit 1
  fi

  echo "==> Validating deprecation registry policy"
  if ! python3 "$deprecation_guard_script" validate \
    --root "$ROOT_DIR" \
    --registry "$deprecation_registry_path"; then
    log_error "Deprecation registry validation failed."
    exit 1
  fi

  echo "==> Enforcing production deprecation gate"
  if ! python3 "$deprecation_guard_script" enforce-deploy \
    --root "$ROOT_DIR" \
    --registry "$deprecation_registry_path"; then
    log_error "Deprecation deploy gate failed."
    exit 1
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --require-running-services)
      require_running_services=1
      shift
      ;;
    --skip-smoke-tests)
      skip_smoke_tests=1
      shift
      ;;
    --min-total-memory-mb)
      if [[ $# -lt 2 ]]; then
        log_error "--min-total-memory-mb requires a value"
        exit 1
      fi
      min_total_memory_mb="$2"
      require_nonnegative_integer "$min_total_memory_mb" "--min-total-memory-mb"
      shift 2
      ;;
    --min-total-swap-mb)
      if [[ $# -lt 2 ]]; then
        log_error "--min-total-swap-mb requires a value"
        exit 1
      fi
      min_total_swap_mb="$2"
      require_nonnegative_integer "$min_total_swap_mb" "--min-total-swap-mb"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: ./infra/scripts/predeploy-check.sh [options]

Options:
  --skip-smoke-tests         Skip compose exec smoke checks.
  --require-running-services Fail if api/web containers are not currently running.
  --min-total-memory-mb N    Require at least N MB total system memory (0 disables gate).
  --min-total-swap-mb N      Require at least N MB total system swap (0 disables gate).
  --help                     Show this help.

Environment:
  PREDEPLOY_ALLOW_DEPRECATION_BYPASS=true
      Emergency override for deprecation gate (approval required).
  PREDEPLOY_ALLOW_POLICY_GATES_BYPASS=true
      Temporary override for deploy policy gates (deprecation + resource checks).
  PREDEPLOY_DEPRECATION_REGISTRY_PATH=<path>
      Override default registry path (default: docs/deprecations/registry.json).
  PREDEPLOY_DEPRECATION_GUARD_SCRIPT=<path>
      Override guard script (default: infra/scripts/deprecation_guard.py).
USAGE
      exit 0
      ;;
    *)
      log_error "Unknown argument: $1"
      exit 1
      ;;
  esac
done

detect_compose_cmd
require_boolean_like "$allow_policy_gates_bypass" "PREDEPLOY_ALLOW_POLICY_GATES_BYPASS"

echo "==> Checking required production env files"
for file in "${required_files[@]}"; do
  require_nonempty_file "$file"
done

echo "==> Checking for unresolved placeholder values"
for file in "${required_files[@]}"; do
  require_no_env_placeholders "$file"
done

echo "==> Checking required API/deploy settings"
required_api_keys=(
  "DATABASE_URL"
  "SYNC_DATABASE_URL"
  "AUTH_MODE"
  "EMAIL_PROVIDER"
)
for key in "${required_api_keys[@]}"; do
  require_env_key_nonempty "infra/env/prod/api.env" "$key"
done

internal_api_key="$(env_key_value "infra/env/prod/api.env" "INTERNAL_API_KEY")"
internal_api_key_secret_id="$(env_key_value "infra/env/prod/api.env" "INTERNAL_API_KEY_SECRET_ID")"
internal_api_key_secret_version="$(env_key_value "infra/env/prod/api.env" "INTERNAL_API_KEY_SECRET_VERSION")"
gcp_project_id="$(env_key_value "infra/env/prod/api.env" "GCP_PROJECT_ID")"

internal_api_key_set=0
internal_api_key_secret_set=0
if has_meaningful_value "$internal_api_key"; then
  internal_api_key_set=1
fi
if has_meaningful_value "$internal_api_key_secret_id"; then
  internal_api_key_secret_set=1
fi

if [[ "$internal_api_key_set" -eq 1 && "$internal_api_key_secret_set" -eq 1 ]]; then
  log_error "INTERNAL_API_KEY and INTERNAL_API_KEY_SECRET_ID cannot both be set."
  exit 1
fi
if [[ "$internal_api_key_set" -ne 1 && "$internal_api_key_secret_set" -ne 1 ]]; then
  log_error "Set INTERNAL_API_KEY or INTERNAL_API_KEY_SECRET_ID."
  exit 1
fi
if [[ "$internal_api_key_secret_set" -eq 1 ]]; then
  if [[ -z "$gcp_project_id" ]]; then
    log_error "INTERNAL_API_KEY_SECRET_ID requires GCP_PROJECT_ID."
    exit 1
  fi
  require_pinned_secret_version "${internal_api_key_secret_version:-}" "INTERNAL_API_KEY_SECRET_VERSION"
fi

gsm_master_key_enabled="$(env_key_value "infra/env/prod/api.env" "GSM_MASTER_KEY_ENABLED" | tr '[:upper:]' '[:lower:]')"
if [[ -z "$gsm_master_key_enabled" ]]; then
  gsm_master_key_enabled="false"
fi

if is_truthy "$gsm_master_key_enabled"; then
  gsm_master_key_project_id="$(env_key_value "infra/env/prod/api.env" "GSM_MASTER_KEY_PROJECT_ID")"
  if [[ -z "$gsm_master_key_project_id" ]]; then
    gsm_master_key_project_id="$(env_key_value "infra/env/prod/api.env" "GCP_PROJECT_ID")"
  fi
  gsm_master_key_secret_id="$(env_key_value "infra/env/prod/api.env" "GSM_MASTER_KEY_SECRET_ID")"
  gsm_master_key_version="$(env_key_value "infra/env/prod/api.env" "GSM_MASTER_KEY_VERSION")"
  gsm_master_key_allow_env_fallback="$(env_key_value "infra/env/prod/api.env" "GSM_MASTER_KEY_ALLOW_ENV_FALLBACK" | tr '[:upper:]' '[:lower:]')"

  if [[ -z "$gsm_master_key_project_id" ]]; then
    log_error "GSM_MASTER_KEY_ENABLED=true requires GSM_MASTER_KEY_PROJECT_ID or GCP_PROJECT_ID."
    exit 1
  fi
  if [[ -z "$gsm_master_key_secret_id" ]]; then
    log_error "GSM_MASTER_KEY_ENABLED=true requires GSM_MASTER_KEY_SECRET_ID."
    exit 1
  fi
  if [[ -z "$gsm_master_key_version" ]]; then
    log_error "GSM_MASTER_KEY_ENABLED=true requires GSM_MASTER_KEY_VERSION."
    exit 1
  fi
  if [[ "$(printf '%s' "$gsm_master_key_version" | tr '[:upper:]' '[:lower:]')" == "latest" ]]; then
    log_error "GSM_MASTER_KEY_VERSION must be pinned in production (latest is not allowed)."
    exit 1
  fi
  require_boolean_like "${gsm_master_key_allow_env_fallback:-}" "GSM_MASTER_KEY_ALLOW_ENV_FALLBACK"
  if is_truthy "$gsm_master_key_allow_env_fallback"; then
    log_error "GSM_MASTER_KEY_ALLOW_ENV_FALLBACK must be false in production."
    exit 1
  fi
else
  require_env_key_nonempty "infra/env/prod/api.env" "MASTER_KEY"
fi

evervault_app_id="$(env_key_value "infra/env/prod/api.env" "EVERVAULT_APP_ID")"
evervault_api_key="$(env_key_value "infra/env/prod/api.env" "EVERVAULT_API_KEY")"
evervault_app_id_secret_id="$(env_key_value "infra/env/prod/api.env" "EVERVAULT_APP_ID_SECRET_ID")"
evervault_api_key_secret_id="$(env_key_value "infra/env/prod/api.env" "EVERVAULT_API_KEY_SECRET_ID")"
evervault_app_id_secret_version="$(env_key_value "infra/env/prod/api.env" "EVERVAULT_APP_ID_SECRET_VERSION")"
evervault_api_key_secret_version="$(env_key_value "infra/env/prod/api.env" "EVERVAULT_API_KEY_SECRET_VERSION")"
evervault_config_present=0
if [[ -n "$evervault_app_id" || -n "$evervault_api_key" || -n "$evervault_app_id_secret_id" || -n "$evervault_api_key_secret_id" ]]; then
  evervault_config_present=1
fi
if [[ "$evervault_config_present" -eq 1 ]]; then
  if [[ -z "$evervault_app_id" && -z "$evervault_app_id_secret_id" ]]; then
    log_error "Evervault config requires EVERVAULT_APP_ID or EVERVAULT_APP_ID_SECRET_ID."
    exit 1
  fi
  if [[ -z "$evervault_api_key" && -z "$evervault_api_key_secret_id" ]]; then
    log_error "Evervault config requires EVERVAULT_API_KEY or EVERVAULT_API_KEY_SECRET_ID."
    exit 1
  fi
  if [[ -n "$evervault_app_id_secret_id" || -n "$evervault_api_key_secret_id" ]]; then
    require_env_key_nonempty "infra/env/prod/api.env" "GCP_PROJECT_ID"
  fi
  if [[ -n "$evervault_app_id_secret_id" ]]; then
    require_pinned_secret_version "${evervault_app_id_secret_version:-}" "EVERVAULT_APP_ID_SECRET_VERSION"
  fi
  if [[ -n "$evervault_api_key_secret_id" ]]; then
    require_pinned_secret_version "${evervault_api_key_secret_version:-}" "EVERVAULT_API_KEY_SECRET_VERSION"
  fi
fi

payment_provider_mode="$(env_key_value "infra/env/prod/api.env" "PAYMENT_PROVIDER" | tr '[:upper:]' '[:lower:]')"

database_url="$(env_key_value "infra/env/prod/api.env" "DATABASE_URL")"
sync_database_url="$(env_key_value "infra/env/prod/api.env" "SYNC_DATABASE_URL")"
require_supabase_database_url "$database_url" "DATABASE_URL"
require_supabase_database_url "$sync_database_url" "SYNC_DATABASE_URL"
if [[ "$database_url" == postgresql+asyncpg://* ]] && [[ "$database_url" == *"sslmode="* ]]; then
  log_error "DATABASE_URL uses asyncpg but contains sslmode=. Use ssl=require for asyncpg URLs."
  exit 1
fi

api_auth_mode="$(env_key_value "infra/env/prod/api.env" "AUTH_MODE" | tr '[:upper:]' '[:lower:]')"
if [[ "$api_auth_mode" != "supabase" ]]; then
  log_error "AUTH_MODE must be 'supabase' in production (got: ${api_auth_mode:-<empty>})"
  exit 1
fi

supabase_url="$(env_key_value "infra/env/prod/api.env" "SUPABASE_URL")"
supabase_jwt_issuer="$(env_key_value "infra/env/prod/api.env" "SUPABASE_JWT_ISSUER")"
if [[ -z "$supabase_url" ]]; then
  log_error "Missing required key in infra/env/prod/api.env: SUPABASE_URL"
  exit 1
fi
if [[ -z "$supabase_jwt_issuer" ]]; then
  log_error "Missing required key in infra/env/prod/api.env: SUPABASE_JWT_ISSUER"
  exit 1
fi
require_https_url "$supabase_url" "SUPABASE_URL"
require_https_url "$supabase_jwt_issuer" "SUPABASE_JWT_ISSUER"

supabase_auth_enabled="$(env_key_value "infra/env/prod/api.env" "SUPABASE_AUTH_ENABLED" | tr '[:upper:]' '[:lower:]')"
if ! is_truthy "$supabase_auth_enabled"; then
  log_error "SUPABASE_AUTH_ENABLED must be true in production."
  exit 1
fi
supabase_auth_api_key="$(env_key_value "infra/env/prod/api.env" "SUPABASE_AUTH_API_KEY")"
supabase_service_role_key="$(env_key_value "infra/env/prod/api.env" "SUPABASE_SERVICE_ROLE_KEY")"
supabase_auth_timeout_seconds="$(env_key_value "infra/env/prod/api.env" "SUPABASE_AUTH_TIMEOUT_SECONDS")"
if [[ -z "$supabase_auth_api_key" && -z "$supabase_service_role_key" ]]; then
  log_error "SUPABASE_AUTH_ENABLED=true requires SUPABASE_AUTH_API_KEY or SUPABASE_SERVICE_ROLE_KEY"
  exit 1
fi
require_positive_number "${supabase_auth_timeout_seconds:-0}" "SUPABASE_AUTH_TIMEOUT_SECONDS"

supabase_storage_enabled="$(env_key_value "infra/env/prod/api.env" "SUPABASE_STORAGE_ENABLED" | tr '[:upper:]' '[:lower:]')"
if ! is_truthy "$supabase_storage_enabled"; then
  log_error "SUPABASE_STORAGE_ENABLED must be true in production."
  exit 1
fi
require_env_key_nonempty "infra/env/prod/api.env" "SUPABASE_SERVICE_ROLE_KEY"

edge_task_notify_enabled="$(env_key_value "infra/env/prod/api.env" "EDGE_TASK_NOTIFY_ENABLED")"
if [[ -n "$edge_task_notify_enabled" ]]; then
  require_boolean_like "$edge_task_notify_enabled" "EDGE_TASK_NOTIFY_ENABLED"
fi
if is_truthy "$edge_task_notify_enabled"; then
  require_env_key_nonempty "infra/env/prod/api.env" "SUPABASE_SERVICE_ROLE_KEY"
  supabase_edge_functions_base_url="$(env_key_value "infra/env/prod/api.env" "SUPABASE_EDGE_FUNCTIONS_BASE_URL")"
  if [[ -n "$supabase_edge_functions_base_url" ]]; then
    require_https_url "$supabase_edge_functions_base_url" "SUPABASE_EDGE_FUNCTIONS_BASE_URL"
  fi
  supabase_edge_timeout_seconds="$(env_key_value "infra/env/prod/api.env" "SUPABASE_EDGE_TIMEOUT_SECONDS")"
  if [[ -n "$supabase_edge_timeout_seconds" ]]; then
    require_positive_number "$supabase_edge_timeout_seconds" "SUPABASE_EDGE_TIMEOUT_SECONDS"
  fi
fi

email_provider="$(env_key_value "infra/env/prod/api.env" "EMAIL_PROVIDER" | tr '[:upper:]' '[:lower:]')"
case "$email_provider" in
  smtp|resend|log|disabled)
    ;;
  *)
    log_error "EMAIL_PROVIDER must be one of: smtp|resend|log|disabled (got: ${email_provider:-<empty>})"
    exit 1
    ;;
esac

if [[ "$email_provider" == "resend" ]]; then
  resend_api_key="$(env_key_value "infra/env/prod/api.env" "RESEND_API_KEY")"
  resend_api_key_secret_id="$(env_key_value "infra/env/prod/api.env" "RESEND_API_KEY_SECRET_ID")"
  resend_api_key_secret_version="$(env_key_value "infra/env/prod/api.env" "RESEND_API_KEY_SECRET_VERSION")"
  resend_api_key_vault_name="$(env_key_value "infra/env/prod/api.env" "RESEND_API_KEY_VAULT_NAME")"
  resend_api_key_set=0
  resend_api_key_secret_set=0
  resend_api_key_vault_set=0
  if has_meaningful_value "$resend_api_key"; then
    resend_api_key_set=1
  fi
  if has_meaningful_value "$resend_api_key_secret_id"; then
    resend_api_key_secret_set=1
  fi
  if has_meaningful_value "$resend_api_key_vault_name"; then
    resend_api_key_vault_set=1
  fi

  configured_sources=$((resend_api_key_set + resend_api_key_secret_set + resend_api_key_vault_set))
  if [[ "$configured_sources" -eq 0 ]]; then
    log_error "EMAIL_PROVIDER=resend requires exactly one source: RESEND_API_KEY, RESEND_API_KEY_SECRET_ID, or RESEND_API_KEY_VAULT_NAME."
    exit 1
  fi
  if [[ "$configured_sources" -gt 1 ]]; then
    log_error "RESEND API key source is ambiguous; set only one of RESEND_API_KEY, RESEND_API_KEY_SECRET_ID, RESEND_API_KEY_VAULT_NAME."
    exit 1
  fi
  if [[ "$resend_api_key_secret_set" -eq 1 ]]; then
    if [[ -z "$gcp_project_id" ]]; then
      log_error "RESEND_API_KEY_SECRET_ID requires GCP_PROJECT_ID."
      exit 1
    fi
    require_pinned_secret_version "${resend_api_key_secret_version:-}" "RESEND_API_KEY_SECRET_VERSION"
  fi
  if [[ "$resend_api_key_vault_set" -eq 1 ]]; then
    supabase_vault_enabled="$(env_key_value "infra/env/prod/api.env" "SUPABASE_VAULT_ENABLED" | tr '[:upper:]' '[:lower:]')"
    if ! is_truthy "$supabase_vault_enabled"; then
      log_error "RESEND_API_KEY_VAULT_NAME requires SUPABASE_VAULT_ENABLED=true."
      exit 1
    fi
    if ! is_truthy "$edge_task_notify_enabled"; then
      log_error "RESEND_API_KEY_VAULT_NAME is allowed only when EDGE_TASK_NOTIFY_ENABLED=true."
      exit 1
    fi
    require_env_key_nonempty "infra/env/prod/api.env" "SUPABASE_URL"
  fi
  require_env_key_nonempty "infra/env/prod/api.env" "EMAIL_FROM_ADDRESS"
fi

if [[ "$email_provider" == "smtp" ]]; then
  require_env_key_nonempty "infra/env/prod/api.env" "SMTP_HOST"
  require_env_key_nonempty "infra/env/prod/api.env" "SMTP_PORT"
fi

cors_origins="$(env_key_value "infra/env/prod/api.env" "CORS_ORIGINS")"
if [[ -n "$cors_origins" ]]; then
  require_https_csv_origins "$cors_origins" "CORS_ORIGINS"
fi

resend_api_base_url="$(env_key_value "infra/env/prod/api.env" "RESEND_API_BASE_URL")"
if [[ -n "$resend_api_base_url" ]]; then
  require_https_url "$resend_api_base_url" "RESEND_API_BASE_URL"
fi

payment_enabled="$(env_key_value "infra/env/prod/api.env" "PAYMENT_ENABLED" | tr '[:upper:]' '[:lower:]')"
if [[ -z "$payment_enabled" ]]; then
  payment_enabled="true"
fi

for legacy_key in CARDNUMBER EXPIRYMM EXPIRYYY DOB NN; do
  legacy_value="$(env_key_value "infra/env/prod/api.env" "$legacy_key")"
  if has_meaningful_value "$legacy_value"; then
    log_error "Legacy backend card alias '$legacy_key' is forbidden in infra/env/prod/api.env."
    exit 1
  fi
done

worker_max_jobs="$(env_key_value "infra/env/prod/api.env" "WORKER_MAX_JOBS")"
if [[ -n "$worker_max_jobs" ]]; then
  require_positive_integer "$worker_max_jobs" "WORKER_MAX_JOBS"
  if [[ "$worker_max_jobs" -gt 50 ]]; then
    log_error "WORKER_MAX_JOBS must be <= 50 in production (got: $worker_max_jobs)"
    exit 1
  fi
fi

if is_truthy "$payment_enabled"; then
  payment_provider_mode="${payment_provider_mode:-evervault}"
  payment_evervault_enforce="$(env_key_value "infra/env/prod/api.env" "PAYMENT_EVERVAULT_ENFORCE" | tr '[:upper:]' '[:lower:]')"
  autopay_require_user_wallet="$(env_key_value "infra/env/prod/api.env" "AUTOPAY_REQUIRE_USER_WALLET" | tr '[:upper:]' '[:lower:]')"
  autopay_allow_server_fallback="$(env_key_value "infra/env/prod/api.env" "AUTOPAY_ALLOW_SERVER_FALLBACK" | tr '[:upper:]' '[:lower:]')"

  if [[ "$payment_provider_mode" != "evervault" ]]; then
    log_error "PAYMENT_ENABLED=true requires PAYMENT_PROVIDER=evervault in production."
    exit 1
  fi

  require_boolean_like "${payment_evervault_enforce:-}" "PAYMENT_EVERVAULT_ENFORCE"
  require_boolean_like "${autopay_require_user_wallet:-}" "AUTOPAY_REQUIRE_USER_WALLET"
  require_boolean_like "${autopay_allow_server_fallback:-}" "AUTOPAY_ALLOW_SERVER_FALLBACK"
  if ! is_truthy "$payment_evervault_enforce"; then
    log_error "PAYMENT_ENABLED=true requires PAYMENT_EVERVAULT_ENFORCE=true."
    exit 1
  fi
  if ! is_truthy "$autopay_require_user_wallet"; then
    log_error "PAYMENT_ENABLED=true requires AUTOPAY_REQUIRE_USER_WALLET=true."
    exit 1
  fi
  if is_truthy "$autopay_allow_server_fallback"; then
    log_error "PAYMENT_ENABLED=true requires AUTOPAY_ALLOW_SERVER_FALLBACK=false."
    exit 1
  fi

  if [[ -z "$evervault_app_id" && -z "$evervault_app_id_secret_id" ]]; then
    log_error "PAYMENT_ENABLED=true requires EVERVAULT_APP_ID or EVERVAULT_APP_ID_SECRET_ID."
    exit 1
  fi
  if [[ -z "$evervault_api_key" && -z "$evervault_api_key_secret_id" ]]; then
    log_error "PAYMENT_ENABLED=true requires EVERVAULT_API_KEY or EVERVAULT_API_KEY_SECRET_ID."
    exit 1
  fi

  require_env_key_nonempty "infra/env/prod/web.env" "NEXT_PUBLIC_EVERVAULT_TEAM_ID"
  require_env_key_nonempty "infra/env/prod/web.env" "NEXT_PUBLIC_EVERVAULT_APP_ID"
fi

echo "==> Checking required Web settings"
required_web_keys=(
  "NEXT_PUBLIC_API_BASE_URL"
  "API_SERVER_URL"
)
for key in "${required_web_keys[@]}"; do
  require_env_key_nonempty "infra/env/prod/web.env" "$key"
done

next_public_api_base_url="$(env_key_value "infra/env/prod/web.env" "NEXT_PUBLIC_API_BASE_URL")"
require_https_url_or_empty "$next_public_api_base_url" "NEXT_PUBLIC_API_BASE_URL"

supabase_auth_site_url="$(env_key_value "infra/env/prod/api.env" "SUPABASE_AUTH_SITE_URL")"
if [[ -z "$supabase_auth_site_url" ]]; then
  supabase_auth_site_url="$next_public_api_base_url"
fi
if [[ -z "$supabase_auth_site_url" ]]; then
  caddy_site_address="$(env_key_value "infra/env/prod/caddy.env" "CADDY_SITE_ADDRESS")"
  caddy_site_address="${caddy_site_address#"${caddy_site_address%%[![:space:]]*}"}"
  caddy_site_address="${caddy_site_address%"${caddy_site_address##*[![:space:]]}"}"
  if [[ -n "$caddy_site_address" ]]; then
    if [[ "$caddy_site_address" =~ ^https?:// ]]; then
      supabase_auth_site_url="$caddy_site_address"
    else
      supabase_auth_site_url="https://${caddy_site_address}"
    fi
  fi
fi
if [[ -z "$supabase_auth_site_url" ]]; then
  log_error "Could not resolve Supabase auth site URL from SUPABASE_AUTH_SITE_URL, NEXT_PUBLIC_API_BASE_URL, or CADDY_SITE_ADDRESS."
  exit 1
fi
require_non_local_https_url "$supabase_auth_site_url" "Supabase auth site URL"

supabase_auth_redirect_urls="$(env_key_value "infra/env/prod/api.env" "SUPABASE_AUTH_REDIRECT_URLS")"
if [[ -z "$supabase_auth_redirect_urls" ]]; then
  supabase_auth_redirect_urls="${supabase_auth_site_url%/}/auth/verify,${supabase_auth_site_url%/}/auth/confirm,${supabase_auth_site_url%/}/reset-password,${supabase_auth_site_url%/}/login"
fi
require_non_local_https_csv_urls "$supabase_auth_redirect_urls" "Supabase auth redirect URL"
require_csv_contains_verify_path "$supabase_auth_redirect_urls" "Supabase auth redirect URL"

next_public_supabase_direct_auth_enabled="$(env_key_value "infra/env/prod/web.env" "NEXT_PUBLIC_SUPABASE_DIRECT_AUTH_ENABLED")"
next_public_supabase_realtime_enabled="$(env_key_value "infra/env/prod/web.env" "NEXT_PUBLIC_SUPABASE_REALTIME_ENABLED")"
next_public_supabase_realtime_delta_read_enabled="$(env_key_value "infra/env/prod/web.env" "NEXT_PUBLIC_SUPABASE_REALTIME_DELTA_READ_ENABLED")"
next_public_train_reads_via_data_api="$(env_key_value "infra/env/prod/web.env" "NEXT_PUBLIC_TRAIN_READS_VIA_DATA_API")"
next_public_train_detail_via_graphql="$(env_key_value "infra/env/prod/web.env" "NEXT_PUBLIC_TRAIN_DETAIL_VIA_GRAPHQL")"
if [[ -n "$next_public_supabase_direct_auth_enabled" ]]; then
  require_boolean_like "$next_public_supabase_direct_auth_enabled" "NEXT_PUBLIC_SUPABASE_DIRECT_AUTH_ENABLED"
fi
if [[ -n "$next_public_supabase_realtime_enabled" ]]; then
  require_boolean_like "$next_public_supabase_realtime_enabled" "NEXT_PUBLIC_SUPABASE_REALTIME_ENABLED"
fi
if [[ -n "$next_public_supabase_realtime_delta_read_enabled" ]]; then
  require_boolean_like "$next_public_supabase_realtime_delta_read_enabled" "NEXT_PUBLIC_SUPABASE_REALTIME_DELTA_READ_ENABLED"
fi
if [[ -n "$next_public_train_reads_via_data_api" ]]; then
  require_boolean_like "$next_public_train_reads_via_data_api" "NEXT_PUBLIC_TRAIN_READS_VIA_DATA_API"
fi
if [[ -n "$next_public_train_detail_via_graphql" ]]; then
  require_boolean_like "$next_public_train_detail_via_graphql" "NEXT_PUBLIC_TRAIN_DETAIL_VIA_GRAPHQL"
fi

if is_truthy "$next_public_supabase_direct_auth_enabled" || is_truthy "$next_public_supabase_realtime_enabled" || is_truthy "$next_public_supabase_realtime_delta_read_enabled" || is_truthy "$next_public_train_reads_via_data_api" || is_truthy "$next_public_train_detail_via_graphql"; then
  require_env_key_nonempty "infra/env/prod/web.env" "NEXT_PUBLIC_SUPABASE_URL"
  require_env_key_nonempty "infra/env/prod/web.env" "NEXT_PUBLIC_SUPABASE_ANON_KEY"
  next_public_supabase_url="$(env_key_value "infra/env/prod/web.env" "NEXT_PUBLIC_SUPABASE_URL")"
  require_https_url "$next_public_supabase_url" "NEXT_PUBLIC_SUPABASE_URL"
fi

next_public_font_base_url="$(env_key_value "infra/env/prod/web.env" "NEXT_PUBLIC_FONT_BASE_URL")"
if [[ -n "$next_public_font_base_url" ]]; then
  require_https_url "$next_public_font_base_url" "NEXT_PUBLIC_FONT_BASE_URL"
fi

api_server_url="$(env_key_value "infra/env/prod/web.env" "API_SERVER_URL")"
if ! [[ "$api_server_url" =~ ^https?:// ]]; then
  log_error "API_SERVER_URL must be an absolute http(s) URL in infra/env/prod/web.env"
  exit 1
fi

echo "==> Checking required Caddy settings"
required_caddy_keys=(
  "CADDY_SITE_ADDRESS"
  "CADDY_ACME_EMAIL"
)
for key in "${required_caddy_keys[@]}"; do
  require_env_key_nonempty "infra/env/prod/caddy.env" "$key"
done

echo "==> Validating production compose configuration"
"${COMPOSE_CMD[@]}" -f infra/docker-compose.prod.yml config >/tmp/bominal-prod-compose.txt

if is_truthy "$allow_policy_gates_bypass"; then
  log_warn "Skipping policy gates (PREDEPLOY_ALLOW_POLICY_GATES_BYPASS=true)."
else
  run_deprecation_gate
  run_resource_gate
fi

service_is_running() {
  local service="$1"
  "${COMPOSE_CMD[@]}" -f infra/docker-compose.yml ps --services --filter status=running 2>/dev/null | grep -Fxq "$service"
}

if [[ "$skip_smoke_tests" -eq 1 ]]; then
  echo "==> Skipping smoke tests (--skip-smoke-tests)"
  echo "Pre-deploy checks passed."
  exit 0
fi

api_service="$(first_running_compose_service infra/docker-compose.yml api || true)"
if [[ -z "$api_service" ]] || ! service_is_running "web"; then
  if [[ "$require_running_services" -eq 1 ]]; then
    log_error "Required local services are not running (api/web). Start stack or use --skip-smoke-tests."
    exit 1
  fi
  log_warn "Skipping smoke tests because api/web are not running. Use --require-running-services to enforce."
  echo "Pre-deploy checks passed (env + compose validation only)."
  exit 0
fi

echo "==> Running backend smoke tests"
"${COMPOSE_CMD[@]}" -f infra/docker-compose.yml exec -T "$api_service" sh -lc 'cd /app && PYTHONPATH=/app pytest -q tests/test_auth_flow.py tests/test_train_provider_crud.py'

echo "==> Running frontend type check"
"${COMPOSE_CMD[@]}" -f infra/docker-compose.yml exec -T web sh -lc 'cd /app && npx tsc --noEmit'

echo "Pre-deploy checks passed."
