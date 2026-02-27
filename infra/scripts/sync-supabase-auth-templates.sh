#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="${BOMINAL_ROOT_DIR:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
source "$SCRIPT_DIR/lib/env_utils.sh"

TEMPLATE_DIR="$ROOT_DIR/infra/supabase/auth-templates"
SUBJECTS_FILE="$TEMPLATE_DIR/subjects.json"
CONFIRM_TEMPLATE="$TEMPLATE_DIR/confirm-signup.html"
RECOVERY_TEMPLATE="$TEMPLATE_DIR/reset-password.html"
PROD_API_ENV="$ROOT_DIR/infra/env/prod/api.env"
PROD_WEB_ENV="$ROOT_DIR/infra/env/prod/web.env"
PROD_CADDY_ENV="$ROOT_DIR/infra/env/prod/caddy.env"

apply_mode=false
project_ref_override=""

usage() {
  cat <<'USAGE'
Usage: bash infra/scripts/sync-supabase-auth-templates.sh [--dry-run] [--apply] [--project-ref <ref>]

Sync bominal auth email templates to Supabase project auth config.

Modes:
  --dry-run              Build payload and validate files (default)
  --apply                Execute Supabase Management API PATCH request
  --project-ref <ref>    Explicit Supabase project ref (otherwise auto-detected)

Project ref auto-detection order:
  1) --project-ref
  2) SUPABASE_PROJECT_REF env var
  3) SUPABASE_URL env var
  4) SUPABASE_URL in infra/env/prod/api.env

Auth token for --apply:
  SUPABASE_MANAGEMENT_API_TOKEN (preferred), or SUPABASE_ACCESS_TOKEN
USAGE
}

extract_project_ref_from_url() {
  local supabase_url="$1"
  if [[ "$supabase_url" =~ ^https://([a-zA-Z0-9-]+)\.supabase\.co/?$ ]]; then
    printf '%s' "${BASH_REMATCH[1]}"
    return 0
  fi
  printf '%s' ""
  return 1
}

json_read_subject() {
  local json_file="$1"
  local key="$2"
  python3 - "$json_file" "$key" <<'PY'
import json
import sys
from pathlib import Path

json_path = Path(sys.argv[1])
key = sys.argv[2]

try:
    payload = json.loads(json_path.read_text(encoding="utf-8"))
except Exception as exc:
    print(f"Failed to parse JSON at {json_path}: {exc}", file=sys.stderr)
    raise SystemExit(1)

value = payload.get(key)
if not isinstance(value, str) or not value.strip():
    print(f"Missing non-empty string key '{key}' in {json_path}", file=sys.stderr)
    raise SystemExit(2)

print(value)
PY
}

build_payload_json() {
  local confirm_html="$1"
  local recovery_html="$2"
  local confirmation_subject="$3"
  local recovery_subject="$4"
  local site_url="$5"
  local uri_allow_list="$6"

  python3 - "$confirm_html" "$recovery_html" "$confirmation_subject" "$recovery_subject" "$site_url" "$uri_allow_list" <<'PY'
import json
import sys
from pathlib import Path

confirm_path = Path(sys.argv[1])
recovery_path = Path(sys.argv[2])
confirmation_subject = sys.argv[3]
recovery_subject = sys.argv[4]
site_url = sys.argv[5]
uri_allow_list = sys.argv[6]

payload = {
    "mailer_subjects_confirmation": confirmation_subject,
    "mailer_subjects_recovery": recovery_subject,
    "mailer_templates_confirmation_content": confirm_path.read_text(encoding="utf-8"),
    "mailer_templates_recovery_content": recovery_path.read_text(encoding="utf-8"),
    "site_url": site_url,
    "uri_allow_list": uri_allow_list,
}
print(json.dumps(payload, ensure_ascii=False))
PY
}

normalize_https_url() {
  local value="$1"
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  value="${value%/}"
  printf '%s' "$value"
}

host_from_url() {
  local url="$1"
  local host
  host="${url#https://}"
  host="${host%%/*}"
  host="${host%%:*}"
  printf '%s' "$host" | tr '[:upper:]' '[:lower:]'
}

is_local_host_url() {
  local host
  host="$(host_from_url "$1")"
  case "$host" in
    localhost|127.0.0.1|0.0.0.0|::1)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

resolve_site_url() {
  local value="${SUPABASE_AUTH_SITE_URL:-}"
  if [[ -z "$value" && -f "$PROD_WEB_ENV" ]]; then
    value="$(env_key_value "$PROD_WEB_ENV" "NEXT_PUBLIC_API_BASE_URL")"
  fi
  if [[ -z "$value" && -f "$PROD_CADDY_ENV" ]]; then
    local caddy_site
    caddy_site="$(env_key_value "$PROD_CADDY_ENV" "CADDY_SITE_ADDRESS")"
    caddy_site="${caddy_site#"${caddy_site%%[![:space:]]*}"}"
    caddy_site="${caddy_site%"${caddy_site##*[![:space:]]}"}"
    if [[ -n "$caddy_site" ]]; then
      if [[ "$caddy_site" =~ ^https?:// ]]; then
        value="$caddy_site"
      else
        value="https://$caddy_site"
      fi
    fi
  fi

  value="$(normalize_https_url "$value")"
  if [[ -z "$value" ]]; then
    log_error "Unable to resolve Supabase site URL. Set SUPABASE_AUTH_SITE_URL or NEXT_PUBLIC_API_BASE_URL."
    return 1
  fi
  if [[ ! "$value" =~ ^https:// ]]; then
    log_error "Supabase site URL must start with https:// (got: $value)"
    return 1
  fi
  if is_local_host_url "$value"; then
    log_error "Supabase site URL must not use localhost/loopback in production (got: $value)"
    return 1
  fi

  printf '%s' "$value"
}

resolve_uri_allow_list() {
  local site_url="$1"
  local raw="${SUPABASE_AUTH_REDIRECT_URLS:-}"
  if [[ -z "$raw" ]]; then
    raw="${site_url}/auth/callback,${site_url}/reset-password,${site_url}/login"
  fi

  local urls=()
  local item
  IFS=',' read -ra items <<<"$raw"
  for item in "${items[@]}"; do
    item="$(normalize_https_url "$item")"
    [[ -z "$item" ]] && continue
    if [[ ! "$item" =~ ^https:// ]]; then
      log_error "Supabase redirect URL must start with https:// (got: $item)"
      return 1
    fi
    if is_local_host_url "$item"; then
      log_error "Supabase redirect URL must not use localhost/loopback in production (got: $item)"
      return 1
    fi
    urls+=("$item")
  done

  if [[ "${#urls[@]}" -eq 0 ]]; then
    log_error "Supabase redirect URL allow-list is empty."
    return 1
  fi

  local joined
  joined="$(IFS=, ; echo "${urls[*]}")"
  printf '%s' "$joined"
}

resolve_project_ref() {
  local resolved="${project_ref_override:-}"
  if [[ -z "$resolved" && -n "${SUPABASE_PROJECT_REF:-}" ]]; then
    resolved="${SUPABASE_PROJECT_REF}"
  fi

  local supabase_url="${SUPABASE_URL:-}"
  if [[ -z "$supabase_url" && -f "$PROD_API_ENV" ]]; then
    supabase_url="$(env_key_value "$PROD_API_ENV" "SUPABASE_URL")"
  fi

  if [[ -z "$resolved" && -n "$supabase_url" ]]; then
    resolved="$(extract_project_ref_from_url "$supabase_url" || true)"
  fi

  if [[ -z "$resolved" ]]; then
    log_error "Unable to resolve project ref. Set --project-ref, SUPABASE_PROJECT_REF, or SUPABASE_URL."
    return 1
  fi
  if [[ ! "$resolved" =~ ^[a-zA-Z0-9-]+$ ]]; then
    log_error "Invalid Supabase project ref: $resolved"
    return 1
  fi

  printf '%s' "$resolved"
}

resolve_management_token() {
  local token="${SUPABASE_MANAGEMENT_API_TOKEN:-${SUPABASE_ACCESS_TOKEN:-}}"
  if [[ -z "$token" ]]; then
    log_error "--apply requires SUPABASE_MANAGEMENT_API_TOKEN (or SUPABASE_ACCESS_TOKEN)."
    return 1
  fi
  printf '%s' "$token"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --apply)
      apply_mode=true
      shift
      ;;
    --dry-run)
      apply_mode=false
      shift
      ;;
    --project-ref)
      if [[ $# -lt 2 ]]; then
        log_error "--project-ref requires a value"
        exit 1
      fi
      project_ref_override="$2"
      shift 2
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

command -v python3 >/dev/null 2>&1 || {
  log_error "python3 is required"
  exit 1
}

require_nonempty_file "$SUBJECTS_FILE"
require_nonempty_file "$CONFIRM_TEMPLATE"
require_nonempty_file "$RECOVERY_TEMPLATE"

confirmation_subject="$(json_read_subject "$SUBJECTS_FILE" "confirmation")"
recovery_subject="$(json_read_subject "$SUBJECTS_FILE" "recovery")"
project_ref="$(resolve_project_ref)"
site_url="$(resolve_site_url)"
uri_allow_list="$(resolve_uri_allow_list "$site_url")"

payload_file="$(mktemp)"
response_file="$(mktemp)"
auth_header_file="$(mktemp)"
cleanup() {
  rm -f "$payload_file" "$response_file" "$auth_header_file"
}
trap cleanup EXIT

build_payload_json \
  "$CONFIRM_TEMPLATE" \
  "$RECOVERY_TEMPLATE" \
  "$confirmation_subject" \
  "$recovery_subject" \
  "$site_url" \
  "$uri_allow_list" \
  >"$payload_file"

confirm_bytes="$(wc -c <"$CONFIRM_TEMPLATE" | tr -d ' ')"
recovery_bytes="$(wc -c <"$RECOVERY_TEMPLATE" | tr -d ' ')"

log_info "Prepared Supabase auth template payload for project '${project_ref}'"
log_info "Template sizes: confirmation=${confirm_bytes}B recovery=${recovery_bytes}B"
log_info "Resolved Supabase site URL: ${site_url}"
log_info "Resolved Supabase redirect allow-list: ${uri_allow_list}"

if [[ "$apply_mode" != "true" ]]; then
  log_info "Dry run complete. Re-run with --apply to sync templates."
  exit 0
fi

command -v curl >/dev/null 2>&1 || {
  log_error "curl is required for --apply"
  exit 1
}

token="$(resolve_management_token)"
printf 'Authorization: Bearer %s\n' "$token" >"$auth_header_file"
chmod 600 "$auth_header_file"

api_url="https://api.supabase.com/v1/projects/${project_ref}/config/auth"
http_code="$(
  curl -sS -o "$response_file" -w "%{http_code}" \
    -X PATCH "$api_url" \
    -H "@$auth_header_file" \
    -H "Content-Type: application/json" \
    --data-binary "@$payload_file"
)"

if [[ ! "$http_code" =~ ^2[0-9][0-9]$ ]]; then
  response_preview="$(head -c 400 "$response_file" | tr '\n' ' ' | tr '\r' ' ')"
  log_error "Supabase template sync failed (HTTP ${http_code})."
  if [[ -n "$response_preview" ]]; then
    log_error "Response preview: $response_preview"
  fi
  exit 1
fi

log_info "Supabase auth templates synced successfully for project '${project_ref}'."
