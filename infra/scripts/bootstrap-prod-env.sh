#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="${BOMINAL_ROOT_DIR:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
ENV_DIR="$ROOT_DIR/infra/env/prod"
source "$SCRIPT_DIR/lib/env_utils.sh"

API_EXAMPLE="$ENV_DIR/api.env.example"
WEB_EXAMPLE="$ENV_DIR/web.env.example"
CADDY_EXAMPLE="$ENV_DIR/caddy.env.example"
DEPLOY_EXAMPLE="$ENV_DIR/deploy.env.example"
PAY_EXAMPLE="$ENV_DIR/pay.env.example"

API_ENV="$ENV_DIR/api.env"
WEB_ENV="$ENV_DIR/web.env"
CADDY_ENV="$ENV_DIR/caddy.env"
DEPLOY_ENV="$ENV_DIR/deploy.env"
PAY_ENV="$ENV_DIR/pay.env"

usage() {
  cat <<'USAGE'
Usage: bash infra/scripts/bootstrap-prod-env.sh

Interactive production env bootstrap for:
- infra/env/prod/api.env
- infra/env/prod/pay.env
- infra/env/prod/web.env
- infra/env/prod/caddy.env
- optional infra/env/prod/deploy.env

The script:
- prompts for required sensitive and operational values
- writes env files from *.example templates
- backs up existing files before overwrite
- validates critical values and unresolved placeholders
USAGE
}

ensure_file_exists() {
  local file="$1"
  if [[ ! -f "$file" ]]; then
    log_error "Required file not found: $file"
    exit 1
  fi
}

trim() {
  local v="$1"
  v="${v#${v%%[![:space:]]*}}"
  v="${v%${v##*[![:space:]]}}"
  printf '%s' "$v"
}

prompt_value() {
  local prompt="$1"
  local default_value="$2"
  local required="$3"
  local secret="${4:-false}"
  local input=""

  while true; do
    if [[ "$secret" == "true" ]]; then
      if [[ -n "$default_value" ]]; then
        read -r -s -p "$prompt [hidden, press Enter to keep current]: " input
      else
        read -r -s -p "$prompt [hidden]: " input
      fi
      echo
    else
      if [[ -n "$default_value" ]]; then
        read -r -p "$prompt [$default_value]: " input
      else
        read -r -p "$prompt: " input
      fi
    fi

    input="$(trim "$input")"
    if [[ -z "$input" ]]; then
      input="$default_value"
    fi

    if [[ "$required" == "true" && -z "$input" ]]; then
      log_warn "Value is required."
      continue
    fi

    printf '%s' "$input"
    return 0
  done
}

confirm_yes_no() {
  local prompt="$1"
  local default_yes="${2:-false}"
  local input

  while true; do
    if [[ "$default_yes" == "true" ]]; then
      read -r -p "$prompt [Y/n]: " input
      input="${input:-y}"
    else
      read -r -p "$prompt [y/N]: " input
      input="${input:-n}"
    fi

    case "${input,,}" in
      y|yes) return 0 ;;
      n|no) return 1 ;;
      *) log_warn "Please answer y or n." ;;
    esac
  done
}

backup_and_copy_example() {
  local example="$1"
  local target="$2"

  if [[ -f "$target" ]]; then
    if ! confirm_yes_no "${target#$ROOT_DIR/} already exists. Overwrite?" false; then
      log_error "Aborted by user (existing file kept): ${target#$ROOT_DIR/}"
      exit 1
    fi
    local ts
    ts="$(date +%Y%m%d%H%M%S)"
    cp "$target" "${target}.bak.${ts}"
    log_info "Backed up existing file to ${target#$ROOT_DIR/}.bak.${ts}"
  fi

  cp "$example" "$target"
  chmod 600 "$target"
  log_info "Created ${target#$ROOT_DIR/} from example"
}

set_env_key() {
  local file="$1"
  local key="$2"
  local value="$3"
  local tmp
  tmp="$(mktemp)"

  awk -v key="$key" -v value="$value" '
    BEGIN { done=0 }
    {
      if ($0 ~ "^" key "=") {
        print key "=" value
        done=1
      } else {
        print $0
      }
    }
    END {
      if (!done) {
        print key "=" value
      }
    }
  ' "$file" > "$tmp"

  mv "$tmp" "$file"
}

require_https_url() {
  local value="$1"
  local name="$2"
  if [[ ! "$value" =~ ^https:// ]]; then
    log_error "$name must start with https://"
    exit 1
  fi
}

require_https_csv() {
  local value="$1"
  local name="$2"
  local item
  IFS=',' read -ra items <<<"$value"
  for item in "${items[@]}"; do
    item="$(trim "$item")"
    [[ -z "$item" ]] && continue
    require_https_url "$item" "$name"
  done
}

validate_master_key() {
  local key="$1"
  local decoded_len
  decoded_len="$(python3 - <<'PY' "$key"
import base64
import sys
key = sys.argv[1]
try:
    raw = base64.b64decode(key)
except Exception:
    print("ERR")
    raise SystemExit(0)
print(len(raw))
PY
)"
  if [[ "$decoded_len" != "32" ]]; then
    log_error "MASTER_KEY must decode from base64 to exactly 32 bytes."
    exit 1
  fi
}

extract_project_ref() {
  local supabase_url="$1"
  if [[ "$supabase_url" =~ ^https://([a-zA-Z0-9-]+)\.supabase\.co/?$ ]]; then
    printf '%s' "${BASH_REMATCH[1]}"
    return 0
  fi
  printf '%s' ""
  return 1
}

validate_no_placeholders() {
  require_no_env_placeholders "$API_ENV"
  require_no_env_placeholders "$PAY_ENV"
  require_no_env_placeholders "$WEB_ENV"
  require_no_env_placeholders "$CADDY_ENV"
  if [[ -f "$DEPLOY_ENV" ]]; then
    require_no_env_placeholders "$DEPLOY_ENV"
  fi
}

main() {
  if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
    usage
    exit 0
  fi

  ensure_file_exists "$API_EXAMPLE"
  ensure_file_exists "$WEB_EXAMPLE"
  ensure_file_exists "$CADDY_EXAMPLE"
  ensure_file_exists "$DEPLOY_EXAMPLE"
  ensure_file_exists "$PAY_EXAMPLE"

  log_info "Bootstrapping production env files in ${ENV_DIR#$ROOT_DIR/}"
  log_warn "Sensitive values are prompted interactively and not echoed."

  backup_and_copy_example "$API_EXAMPLE" "$API_ENV"
  backup_and_copy_example "$PAY_EXAMPLE" "$PAY_ENV"
  backup_and_copy_example "$WEB_EXAMPLE" "$WEB_ENV"
  backup_and_copy_example "$CADDY_EXAMPLE" "$CADDY_ENV"

  local domain
  domain="$(prompt_value "Primary production domain (no scheme)" "www.bominal.com" true false)"

  local caddy_email
  caddy_email="$(prompt_value "ACME notification email" "" true false)"

  local cors_origins
  cors_origins="$(prompt_value "CORS origins (comma-separated https://...)" "https://${domain}" true false)"
  require_https_csv "$cors_origins" "CORS_ORIGINS"

  local ghcr_namespace
  ghcr_namespace="$(prompt_value "GHCR namespace" "ghcr.io/jason931225/bominal" true false)"

  local supabase_url
  supabase_url="$(prompt_value "Supabase project URL" "" true false)"
  require_https_url "$supabase_url" "SUPABASE_URL"

  local project_ref
  project_ref="$(extract_project_ref "$supabase_url" || true)"
  if [[ -z "$project_ref" ]]; then
    project_ref="$(prompt_value "Supabase project ref (from <ref>.supabase.co)" "" true false)"
  fi

  local db_password
  db_password="$(prompt_value "Supabase DB password (pooler user: postgres.<ref>)" "" true true)"
  local db_password_encoded
  db_password_encoded="$(python3 - <<'PY' "$db_password"
import sys
from urllib.parse import quote
print(quote(sys.argv[1], safe=''))
PY
)"

  local supabase_jwt_issuer_default="https://${project_ref}.supabase.co/auth/v1"
  local supabase_jwt_issuer
  supabase_jwt_issuer="$(prompt_value "Supabase JWT issuer" "$supabase_jwt_issuer_default" true false)"
  require_https_url "$supabase_jwt_issuer" "SUPABASE_JWT_ISSUER"

  local supabase_auth_api_key
  supabase_auth_api_key="$(prompt_value "SUPABASE_AUTH_API_KEY" "" true true)"

  local supabase_service_role_key
  supabase_service_role_key="$(prompt_value "SUPABASE_SERVICE_ROLE_KEY" "" true true)"

  local internal_api_key
  internal_api_key="$(prompt_value "INTERNAL_API_KEY (leave empty to auto-generate)" "" false true)"
  if [[ -z "$internal_api_key" ]]; then
    if command -v openssl >/dev/null 2>&1; then
      internal_api_key="$(openssl rand -hex 32)"
      log_info "Generated INTERNAL_API_KEY using openssl."
    else
      log_error "OpenSSL is required to auto-generate INTERNAL_API_KEY; provide it manually."
      exit 1
    fi
  fi

  local master_key
  master_key="$(prompt_value "MASTER_KEY base64(32-byte key)" "" true true)"
  validate_master_key "$master_key"

  local passkey_rp_id
  passkey_rp_id="$(prompt_value "PASSKEY_RP_ID" "$domain" true false)"

  local passkey_origin
  passkey_origin="$(prompt_value "PASSKEY_ORIGIN" "https://${domain}" true false)"
  require_https_url "$passkey_origin" "PASSKEY_ORIGIN"

  local email_from_domain
  email_from_domain="$(prompt_value "Resend sender domain (verified)" "$domain" true false)"

  local email_from_address
  email_from_address="$(prompt_value "EMAIL_FROM_ADDRESS" "no-reply@${email_from_domain}" true false)"

  local email_reply_to
  email_reply_to="$(prompt_value "EMAIL_REPLY_TO" "support@${domain}" true false)"

  local resend_api_key
  resend_api_key="$(prompt_value "RESEND_API_KEY" "" true true)"

  local pay_cardnumber
  pay_cardnumber="$(prompt_value "CARDNUMBER (backend auto-pay)" "" true true)"
  local pay_expirymm
  pay_expirymm="$(prompt_value "EXPIRYMM (MM)" "" true false)"
  local pay_expiryyy
  pay_expiryyy="$(prompt_value "EXPIRYYY (YY)" "" true false)"
  local pay_dob
  pay_dob="$(prompt_value "DOB (YYYYMMDD)" "" true false)"
  local pay_nn
  pay_nn="$(prompt_value "NN (PIN 2-digit)" "" true true)"

  local next_public_api_base_url
  next_public_api_base_url="$(prompt_value "NEXT_PUBLIC_API_BASE_URL (blank for same-origin)" "" false false)"
  if [[ -n "$next_public_api_base_url" ]]; then
    require_https_url "$next_public_api_base_url" "NEXT_PUBLIC_API_BASE_URL"
  fi

  local api_server_url
  api_server_url="$(prompt_value "API_SERVER_URL (web server-side URL)" "http://api:8000" true false)"

  local font_base
  font_base="$(prompt_value "NEXT_PUBLIC_FONT_BASE_URL" "https://github.com/jason931225/bominal.github.io/raw/refs/heads/main/public/font" false false)"
  if [[ -n "$font_base" ]]; then
    require_https_url "$font_base" "NEXT_PUBLIC_FONT_BASE_URL"
  fi

  local db_async_url="postgresql+asyncpg://postgres.${project_ref}:${db_password_encoded}@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require"
  local db_sync_url="postgresql+psycopg://postgres.${project_ref}:${db_password_encoded}@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require"

  set_env_key "$API_ENV" "APP_ENV" "production"
  set_env_key "$API_ENV" "GHCR_NAMESPACE" "$ghcr_namespace"
  set_env_key "$API_ENV" "DATABASE_URL" "$db_async_url"
  set_env_key "$API_ENV" "SYNC_DATABASE_URL" "$db_sync_url"
  set_env_key "$API_ENV" "CORS_ORIGINS" "$cors_origins"
  set_env_key "$API_ENV" "AUTH_MODE" "supabase"
  set_env_key "$API_ENV" "SUPABASE_URL" "$supabase_url"
  set_env_key "$API_ENV" "SUPABASE_JWT_ISSUER" "$supabase_jwt_issuer"
  set_env_key "$API_ENV" "SUPABASE_AUTH_ENABLED" "true"
  set_env_key "$API_ENV" "SUPABASE_AUTH_API_KEY" "$supabase_auth_api_key"
  set_env_key "$API_ENV" "SUPABASE_STORAGE_ENABLED" "true"
  set_env_key "$API_ENV" "SUPABASE_SERVICE_ROLE_KEY" "$supabase_service_role_key"
  set_env_key "$API_ENV" "INTERNAL_API_KEY" "$internal_api_key"
  set_env_key "$API_ENV" "MASTER_KEY" "$master_key"
  set_env_key "$API_ENV" "RESTAURANT_MODULE_ENABLED" "false"
  set_env_key "$API_ENV" "TRAIN_POLL_FORCE_MAX_RATE" "false"
  set_env_key "$API_ENV" "TRAIN_PERSIST_ALL_ATTEMPTS" "false"
  set_env_key "$API_ENV" "PAYMENT_ENABLED" "true"
  set_env_key "$API_ENV" "PASSKEY_ENABLED" "true"
  set_env_key "$API_ENV" "PASSKEY_RP_ID" "$passkey_rp_id"
  set_env_key "$API_ENV" "PASSKEY_ORIGIN" "$passkey_origin"
  set_env_key "$API_ENV" "EMAIL_PROVIDER" "resend"
  set_env_key "$API_ENV" "EMAIL_FROM_NAME" "bominal"
  set_env_key "$API_ENV" "EMAIL_FROM_ADDRESS" "$email_from_address"
  set_env_key "$API_ENV" "EMAIL_REPLY_TO" "$email_reply_to"
  set_env_key "$API_ENV" "RESEND_API_KEY" "$resend_api_key"
  set_env_key "$API_ENV" "RESEND_API_BASE_URL" "https://api.resend.com"

  set_env_key "$PAY_ENV" "CARDNUMBER" "$pay_cardnumber"
  set_env_key "$PAY_ENV" "EXPIRYMM" "$pay_expirymm"
  set_env_key "$PAY_ENV" "EXPIRYYY" "$pay_expiryyy"
  set_env_key "$PAY_ENV" "DOB" "$pay_dob"
  set_env_key "$PAY_ENV" "NN" "$pay_nn"

  set_env_key "$WEB_ENV" "NEXT_PUBLIC_API_BASE_URL" "$next_public_api_base_url"
  set_env_key "$WEB_ENV" "API_SERVER_URL" "$api_server_url"
  set_env_key "$WEB_ENV" "NEXT_PUBLIC_TRAIN_AUTO_PAY_ENABLED" "true"
  set_env_key "$WEB_ENV" "NEXT_PUBLIC_RESTAURANT_MODULE_ENABLED" "false"
  set_env_key "$WEB_ENV" "NEXT_PUBLIC_FONT_BASE_URL" "$font_base"
  set_env_key "$WEB_ENV" "NODE_ENV" "production"

  set_env_key "$CADDY_ENV" "CADDY_SITE_ADDRESS" "$domain"
  set_env_key "$CADDY_ENV" "CADDY_ACME_EMAIL" "$caddy_email"

  if confirm_yes_no "Create/update infra/env/prod/deploy.env with GHCR credentials?" false; then
    backup_and_copy_example "$DEPLOY_EXAMPLE" "$DEPLOY_ENV"
    local ghcr_username ghcr_token
    ghcr_username="$(prompt_value "GHCR_USERNAME" "" true false)"
    ghcr_token="$(prompt_value "GHCR_TOKEN" "" true true)"
    set_env_key "$DEPLOY_ENV" "GHCR_NAMESPACE" "$ghcr_namespace"
    set_env_key "$DEPLOY_ENV" "GHCR_USERNAME" "$ghcr_username"
    set_env_key "$DEPLOY_ENV" "GHCR_TOKEN" "$ghcr_token"
  fi

  validate_no_placeholders

  log_info "Production env bootstrap complete."
  log_info "Generated files:"
  log_info "- ${API_ENV#$ROOT_DIR/}"
  log_info "- ${PAY_ENV#$ROOT_DIR/}"
  log_info "- ${WEB_ENV#$ROOT_DIR/}"
  log_info "- ${CADDY_ENV#$ROOT_DIR/}"
  if [[ -f "$DEPLOY_ENV" ]]; then
    log_info "- ${DEPLOY_ENV#$ROOT_DIR/}"
  fi

  log_info "Next steps:"
  log_info "1) Review values carefully (do not commit secret files)."
  log_info "2) Run: bash infra/scripts/predeploy-check.sh --min-total-memory-mb 900 --min-total-swap-mb 900"
  log_info "3) Deploy: sudo -u bominal infra/scripts/deploy.sh"
}

main "$@"
