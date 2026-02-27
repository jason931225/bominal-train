#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="$(basename "$0")"
ROOT_DIR="${BOMINAL_ROOT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
API_ENV_FILE="$ROOT_DIR/infra/env/prod/api.env"

MODE="dry-run"
PROJECT_REF="${SUPABASE_PROJECT_REF:-}"
GCP_PROJECT_ID="${GCP_PROJECT_ID:-}"
SECRET_ID=""
SECRET_VERSION=""
REQUIRE_EDGE_NOTIFY="true"

usage() {
  cat <<USAGE
Usage: $SCRIPT_NAME [--dry-run|--apply] [options]

Sync GSM-authoritative RESEND API key into Supabase Edge Function secrets.

Options:
  --dry-run                      Show planned actions (default)
  --apply                        Fetch GSM secret and update Supabase Edge secrets
  --project-ref <ref>            Supabase project ref (fallback: SUPABASE_PROJECT_REF or api.env SUPABASE_URL)
  --gcp-project-id <id>          GCP project id (fallback: GCP_PROJECT_ID env/api.env)
  --secret-id <id>               GSM secret id (fallback: RESEND_API_KEY_SECRET_ID in api.env)
  --secret-version <ver>         GSM secret version (fallback: RESEND_API_KEY_SECRET_VERSION in api.env)
  --allow-edge-disabled          Allow execution even if EDGE_TASK_NOTIFY_ENABLED is not true in api.env
  --help                         Show this help

Environment (apply mode):
  SUPABASE_ACCESS_TOKEN (or logged-in supabase CLI session)
USAGE
}

log_info() { echo "[INFO] $*"; }
log_ok() { echo "[OK] $*"; }
log_warn() { echo "[WARN] $*" >&2; }
log_error() { echo "[ERROR] $*" >&2; }

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    log_error "Required command not found: $cmd"
    exit 1
  fi
}

env_key_value() {
  local file="$1"
  local key="$2"
  awk -F'=' -v key="$key" '
    /^[[:space:]]*#/ {next}
    /^[[:space:]]*$/ {next}
    {
      k=$1
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", k)
      if (k == key) {
        v=$0
        sub(/^[^=]*=/, "", v)
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", v)
        gsub(/^"/, "", v)
        gsub(/"$/, "", v)
        print v
        exit
      }
    }
  ' "$file"
}

is_truthy() {
  case "$(printf '%s' "$1" | tr '[:upper:]' '[:lower:]')" in
    true|1|yes) return 0 ;;
    *) return 1 ;;
  esac
}

parse_project_ref_from_supabase_url() {
  local url="$1"
  local host
  host="${url#https://}"
  host="${host#http://}"
  host="${host%%/*}"
  if [[ "$host" == *.supabase.co ]]; then
    echo "${host%%.supabase.co}"
  fi
}

require_pinned_version() {
  local version="$1"
  if [[ -z "$version" ]]; then
    log_error "RESEND_API_KEY_SECRET_VERSION is required"
    exit 1
  fi
  if [[ "$(printf '%s' "$version" | tr '[:upper:]' '[:lower:]')" == "latest" ]]; then
    log_error "RESEND_API_KEY_SECRET_VERSION must be pinned (latest is not allowed)."
    exit 1
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      MODE="dry-run"
      shift
      ;;
    --apply)
      MODE="apply"
      shift
      ;;
    --project-ref)
      PROJECT_REF="$2"
      shift 2
      ;;
    --gcp-project-id)
      GCP_PROJECT_ID="$2"
      shift 2
      ;;
    --secret-id)
      SECRET_ID="$2"
      shift 2
      ;;
    --secret-version)
      SECRET_VERSION="$2"
      shift 2
      ;;
    --allow-edge-disabled)
      REQUIRE_EDGE_NOTIFY="false"
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      log_error "Unknown argument: $1"
      usage
      exit 1
      ;;
  esac
done

if [[ ! -f "$API_ENV_FILE" ]]; then
  log_error "Missing api env file: $API_ENV_FILE"
  exit 1
fi

if [[ -z "$SECRET_ID" ]]; then
  SECRET_ID="$(env_key_value "$API_ENV_FILE" "RESEND_API_KEY_SECRET_ID")"
fi
if [[ -z "$SECRET_VERSION" ]]; then
  SECRET_VERSION="$(env_key_value "$API_ENV_FILE" "RESEND_API_KEY_SECRET_VERSION")"
fi
if [[ -z "$GCP_PROJECT_ID" ]]; then
  GCP_PROJECT_ID="$(env_key_value "$API_ENV_FILE" "GCP_PROJECT_ID")"
fi
if [[ -z "$PROJECT_REF" ]]; then
  PROJECT_REF="$(parse_project_ref_from_supabase_url "$(env_key_value "$API_ENV_FILE" "SUPABASE_URL")")"
fi

edge_notify_enabled="$(env_key_value "$API_ENV_FILE" "EDGE_TASK_NOTIFY_ENABLED")"
email_from_address="$(env_key_value "$API_ENV_FILE" "EMAIL_FROM_ADDRESS")"
email_from_name="$(env_key_value "$API_ENV_FILE" "EMAIL_FROM_NAME")"

if [[ -z "$SECRET_ID" ]]; then
  log_error "RESEND_API_KEY_SECRET_ID is required (set in api.env or pass --secret-id)."
  exit 1
fi
require_pinned_version "$SECRET_VERSION"

if [[ -z "$GCP_PROJECT_ID" ]]; then
  log_error "GCP_PROJECT_ID is required (set in api.env or pass --gcp-project-id)."
  exit 1
fi

if [[ -z "$PROJECT_REF" ]]; then
  log_error "Supabase project ref is required (set --project-ref, SUPABASE_PROJECT_REF, or SUPABASE_URL in api.env)."
  exit 1
fi

if [[ -z "$email_from_address" ]]; then
  log_error "EMAIL_FROM_ADDRESS is required in api.env for edge notify sync."
  exit 1
fi

if [[ "$REQUIRE_EDGE_NOTIFY" == "true" ]] && ! is_truthy "$edge_notify_enabled"; then
  log_error "EDGE_TASK_NOTIFY_ENABLED must be true (or pass --allow-edge-disabled)."
  exit 1
fi

log_info "Mode: $MODE"
log_info "Supabase project ref: $PROJECT_REF"
log_info "GSM source: ${GCP_PROJECT_ID}/${SECRET_ID}@${SECRET_VERSION}"
log_info "Edge secrets to sync: RESEND_API_KEY, EMAIL_FROM_ADDRESS${email_from_name:+, EMAIL_FROM_NAME}"

if [[ "$MODE" == "dry-run" ]]; then
  log_ok "Dry run complete. No remote changes were made."
  exit 0
fi

require_cmd gcloud
require_cmd supabase

resend_api_key="$(gcloud secrets versions access "$SECRET_VERSION" --secret="$SECRET_ID" --project="$GCP_PROJECT_ID" 2>/dev/null || true)"
if [[ -z "$resend_api_key" ]]; then
  log_error "Failed to read GSM secret: ${GCP_PROJECT_ID}/${SECRET_ID}@${SECRET_VERSION}"
  exit 1
fi

tmp_env="$(mktemp)"
trap 'rm -f "$tmp_env"' EXIT
chmod 600 "$tmp_env"
{
  printf 'RESEND_API_KEY=%s\n' "$resend_api_key"
  printf 'EMAIL_FROM_ADDRESS=%s\n' "$email_from_address"
  if [[ -n "$email_from_name" ]]; then
    printf 'EMAIL_FROM_NAME=%s\n' "$email_from_name"
  fi
} > "$tmp_env"

supabase secrets set --project-ref "$PROJECT_REF" --env-file "$tmp_env" >/dev/null

log_ok "Supabase Edge secrets synced successfully for project '$PROJECT_REF'."
