#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# evervault-relay-probe.sh — optional Evervault auth/provider relay probes
# -----------------------------------------------------------------------------
# Default behavior: auth-only management probe (no provider relay POST).
# Provider relay probes can consume relay credits and are opt-in via:
#   --include-provider-probes  OR EVERVAULT_RELAY_PROVIDER_PROBES_ENABLED=true
# -----------------------------------------------------------------------------
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="${BOMINAL_ROOT_DIR:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
API_ENV_FILE="${EVERVAULT_RELAY_PROBE_API_ENV_FILE:-$ROOT_DIR/infra/env/prod/api.env}"
EVERVAULT_API_BASE_URL="${EVERVAULT_API_BASE_URL:-https://api.evervault.com}"

include_provider_probes="${EVERVAULT_RELAY_PROVIDER_PROBES_ENABLED:-false}"

usage() {
  cat <<USAGE
Usage: $0 [--include-provider-probes]

Options:
  --include-provider-probes  Send probe POSTs to configured KTX/SRT relay domains.
                             WARNING: these provider probes can consume relay credits.
  --help                     Show this help.
USAGE
}

is_truthy() {
  local value="$1"
  case "$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')" in
    true|1|yes|on) return 0 ;;
    *) return 1 ;;
  esac
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

resolve_nonempty() {
  local preferred="$1"
  local fallback="$2"
  if [[ -n "${preferred//[[:space:]]/}" ]]; then
    printf '%s' "$preferred"
    return 0
  fi
  printf '%s' "$fallback"
}

for arg in "$@"; do
  case "$arg" in
    --include-provider-probes)
      include_provider_probes="true"
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "ERROR: unknown argument '$arg'" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ ! -f "$API_ENV_FILE" ]]; then
  echo "ERROR: missing api env file: $API_ENV_FILE" >&2
  exit 1
fi

app_id_env="${EVERVAULT_APP_ID:-}"
api_key_env="${EVERVAULT_API_KEY:-}"
app_id_file="$(env_key_value "$API_ENV_FILE" "EVERVAULT_APP_ID")"
api_key_file="$(env_key_value "$API_ENV_FILE" "EVERVAULT_API_KEY")"

app_id="$(resolve_nonempty "$app_id_env" "$app_id_file")"
api_key="$(resolve_nonempty "$api_key_env" "$api_key_file")"

if [[ -z "${api_key//[[:space:]]/}" ]]; then
  api_key_secret_id="$(env_key_value "$API_ENV_FILE" "EVERVAULT_API_KEY_SECRET_ID")"
  api_key_secret_version="$(env_key_value "$API_ENV_FILE" "EVERVAULT_API_KEY_SECRET_VERSION")"
  gcp_project_id="$(env_key_value "$API_ENV_FILE" "GCP_PROJECT_ID")"
  if [[ -n "${api_key_secret_id//[[:space:]]/}" ]]; then
    if [[ -z "${gcp_project_id//[[:space:]]/}" ]]; then
      echo "ERROR: GCP_PROJECT_ID is required for GSM-backed Evervault API key probe resolution" >&2
      exit 1
    fi
    if [[ -z "${api_key_secret_version//[[:space:]]/}" ]]; then
      echo "ERROR: EVERVAULT_API_KEY_SECRET_VERSION is required for GSM-backed Evervault API key probe resolution" >&2
      exit 1
    fi
    if ! command -v gcloud >/dev/null 2>&1; then
      echo "ERROR: gcloud CLI is required for GSM-backed Evervault API key probe resolution" >&2
      exit 1
    fi
    api_key="$(gcloud secrets versions access "$api_key_secret_version" --secret="$api_key_secret_id" --project="$gcp_project_id" 2>/dev/null || true)"
  fi
fi

if [[ -z "${app_id//[[:space:]]/}" || -z "${api_key//[[:space:]]/}" ]]; then
  echo "ERROR: missing Evervault runtime probe credentials (app id and api key are required)" >&2
  exit 1
fi

echo "INFO: running Evervault management auth probe"
management_status="$(curl -sS -o /dev/null -w '%{http_code}' -u "${app_id}:${api_key}" "${EVERVAULT_API_BASE_URL%/}/relays" || true)"
if [[ "$management_status" != "200" ]]; then
  echo "ERROR: Evervault management auth probe failed (status=${management_status:-curl_error})" >&2
  exit 1
fi
echo "OK: Evervault management auth probe returned 200"

if ! is_truthy "$include_provider_probes"; then
  echo "INFO: provider relay probes skipped (auth-only mode; no relay credit usage)"
  exit 0
fi

echo "WARN: provider relay probes enabled; these probe POSTs can consume relay credits"

ktx_domain="$(env_key_value "$API_ENV_FILE" "EVERVAULT_KTX_PAYMENT_RELAY_DOMAIN")"
srt_domain="$(env_key_value "$API_ENV_FILE" "EVERVAULT_SRT_PAYMENT_RELAY_DOMAIN")"

probe_provider() {
  local label="$1"
  local domain="$2"
  local path="$3"

  if [[ -z "${domain//[[:space:]]/}" ]]; then
    echo "INFO: ${label} relay domain not configured; skipping ${label} provider probe"
    return 0
  fi

  local url="https://${domain}${path}"
  local status
  status="$(curl -sS -o /dev/null -w '%{http_code}' --max-time 20 -X POST "$url" \
    -H "X-Evervault-App-Id: ${app_id}" \
    -H "X-Evervault-Api-Key: ${api_key}" \
    -H 'Content-Type: application/x-www-form-urlencoded' \
    --data 'probe=1' || true)"

  if [[ "$status" == "401" || "$status" == "403" || -z "$status" || "$status" == "000" ]]; then
    echo "ERROR: ${label} relay auth probe failed (status=${status:-curl_error})" >&2
    return 1
  fi

  echo "OK: ${label} relay probe status=${status}"
}

probe_provider "KTX" "$ktx_domain" "/classes/com.korail.mobile.payment.ReservationPayment"
probe_provider "SRT" "$srt_domain" "/ata/selectListAta09036_n.do"

echo "OK: Evervault relay provider probes completed"
