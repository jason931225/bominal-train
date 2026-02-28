#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="$(basename "$0")"
ROOT_DIR="${BOMINAL_ROOT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
API_ENV_FILE="$ROOT_DIR/infra/env/prod/api.env"

OUTPUT_FILE=""
WARNING_THRESHOLD="70"
HARD_THRESHOLD="85"
SCHEDULER_LIMIT="3"
GSM_VERSION_LIMIT="6"

usage() {
  cat <<USAGE
Usage: $SCRIPT_NAME [options]

Generate a free-tier status report (Markdown) for weekly governance audits.

Options:
  --output <path>              Write report to file (default: stdout)
  --warning-threshold <pct>    Warning threshold percent (default: 70)
  --hard-threshold <pct>       Hard threshold percent (default: 85)
  --scheduler-limit <count>    Planned Cloud Scheduler free-tier budget (default: 3)
  --gsm-version-limit <count>  Active GSM version limit per secret family (default: 6)
  --help                       Show this help
USAGE
}

log_error() { echo "[ERROR] $*" >&2; }
log_warn() { echo "[WARN] $*" >&2; }

require_nonnegative_int() {
  local value="$1"
  local name="$2"
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    log_error "$name must be a non-negative integer (got: $value)"
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

has_meaningful_value() {
  local value="$1"
  local trimmed
  trimmed="${value#"${value%%[![:space:]]*}"}"
  trimmed="${trimmed%"${trimmed##*[![:space:]]}"}"
  [[ -z "$trimmed" ]] && return 1
  case "$trimmed" in
    *CHANGE_ME*|*REPLACE_ME*|*TODO*) return 1 ;;
  esac
  return 0
}

status_from_percent() {
  local pct="$1"
  local warning="$2"
  local hard="$3"
  awk -v p="$pct" -v w="$warning" -v h="$hard" 'BEGIN {
    if (p >= h) {
      print "HARD_ALERT"
    } else if (p >= w) {
      print "WARNING"
    } else {
      print "OK"
    }
  }'
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output)
      OUTPUT_FILE="$2"
      shift 2
      ;;
    --warning-threshold)
      WARNING_THRESHOLD="$2"
      shift 2
      ;;
    --hard-threshold)
      HARD_THRESHOLD="$2"
      shift 2
      ;;
    --scheduler-limit)
      SCHEDULER_LIMIT="$2"
      shift 2
      ;;
    --gsm-version-limit)
      GSM_VERSION_LIMIT="$2"
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

require_nonnegative_int "$WARNING_THRESHOLD" "--warning-threshold"
require_nonnegative_int "$HARD_THRESHOLD" "--hard-threshold"
require_nonnegative_int "$SCHEDULER_LIMIT" "--scheduler-limit"
require_nonnegative_int "$GSM_VERSION_LIMIT" "--gsm-version-limit"

if [[ "$HARD_THRESHOLD" -lt "$WARNING_THRESHOLD" ]]; then
  log_error "--hard-threshold must be >= --warning-threshold"
  exit 1
fi

utc_now="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
report_path=""
if [[ -n "$OUTPUT_FILE" ]]; then
  report_path="$OUTPUT_FILE"
else
  report_path="/dev/stdout"
fi

gcp_project_id=""
if [[ -f "$API_ENV_FILE" ]]; then
  gcp_project_id="$(env_key_value "$API_ENV_FILE" "GCP_PROJECT_ID")"
fi
if [[ -z "$gcp_project_id" ]]; then
  gcp_project_id="${GCP_PROJECT_ID:-}"
fi

scheduler_job_count="N/A"
scheduler_percent="N/A"
scheduler_status="MANUAL"

if command -v gcloud >/dev/null 2>&1 && has_meaningful_value "$gcp_project_id"; then
  scheduler_job_count_raw="$(gcloud scheduler jobs list --project "$gcp_project_id" --format='value(name)' 2>/dev/null | sed '/^[[:space:]]*$/d' | wc -l | tr -d ' ' || true)"
  if [[ "$scheduler_job_count_raw" =~ ^[0-9]+$ ]]; then
    scheduler_job_count="$scheduler_job_count_raw"
    if [[ "$SCHEDULER_LIMIT" -gt 0 ]]; then
      scheduler_percent="$(awk -v n="$scheduler_job_count" -v d="$SCHEDULER_LIMIT" 'BEGIN {printf "%.2f", (n/d)*100}')"
      scheduler_status="$(status_from_percent "$scheduler_percent" "$WARNING_THRESHOLD" "$HARD_THRESHOLD")"
    else
      scheduler_percent="N/A"
      scheduler_status="MANUAL"
    fi
  fi
fi

secret_ids=()
if [[ -f "$API_ENV_FILE" ]]; then
  for key in \
    GSM_MASTER_KEY_SECRET_ID \
    EVERVAULT_API_KEY_SECRET_ID \
    INTERNAL_API_KEY_SECRET_ID \
    RESEND_API_KEY_SECRET_ID \
    SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID; do
    value="$(env_key_value "$API_ENV_FILE" "$key")"
    if has_meaningful_value "$value"; then
      already="false"
      for existing in "${secret_ids[@]:-}"; do
        if [[ "$existing" == "$value" ]]; then
          already="true"
          break
        fi
      done
      if [[ "$already" == "false" ]]; then
        secret_ids+=("$value")
      fi
    fi
  done
fi

secret_rows=""
if command -v gcloud >/dev/null 2>&1 && has_meaningful_value "$gcp_project_id" && [[ "${#secret_ids[@]}" -gt 0 ]]; then
  for secret_id in "${secret_ids[@]}"; do
    active_versions="$(gcloud secrets versions list "$secret_id" --project "$gcp_project_id" --filter='state=enabled' --format='value(name)' 2>/dev/null | sed '/^[[:space:]]*$/d' | wc -l | tr -d ' ' || true)"
    if [[ ! "$active_versions" =~ ^[0-9]+$ ]]; then
      active_versions="N/A"
      pct="N/A"
      status="MANUAL"
    else
      if [[ "$GSM_VERSION_LIMIT" -gt 0 ]]; then
        pct="$(awk -v n="$active_versions" -v d="$GSM_VERSION_LIMIT" 'BEGIN {printf "%.2f", (n/d)*100}')"
        status="$(status_from_percent "$pct" "$WARNING_THRESHOLD" "$HARD_THRESHOLD")"
      else
        pct="N/A"
        status="MANUAL"
      fi
    fi
    secret_rows+="| ${secret_id} | ${active_versions} | ${pct} | ${status} |"$'\n'
  done
else
  secret_rows="| N/A | N/A | N/A | MANUAL |"$'\n'
fi

{
  cat <<MARKDOWN
# Free-Tier Status Report

- Generated at (UTC): \
  - ${utc_now}
- Warning threshold: \
  - ${WARNING_THRESHOLD}%
- Hard alert threshold: \
  - ${HARD_THRESHOLD}%

## GCP Snapshot

- Project: \
  - ${gcp_project_id:-N/A}
- Cloud Scheduler jobs (limit ${SCHEDULER_LIMIT}): \
  - count=${scheduler_job_count}, percent=${scheduler_percent}, status=${scheduler_status}

### Secret Manager Active Versions (limit ${GSM_VERSION_LIMIT} per secret)

| Secret ID | Active Versions | Percent of Limit | Status |
|---|---:|---:|---|
${secret_rows}

## Supabase Snapshot (Manual Evidence Required)

- Capture and attach these in weekly ops evidence:
  - Usage dashboard screenshot (with date)
  - Billing/usage controls screenshot (with date)
  - Current project ref and enabled edge functions summary

- Live source references:
  - https://supabase.com/docs/guides/platform/billing-on-supabase
  - https://supabase.com/docs/guides/platform/manage-your-usage
  - https://cloud.google.com/free/docs/free-cloud-features
  - https://cloud.google.com/scheduler/pricing

## Operator Notes

- If any status is WARNING or HARD_ALERT, open a follow-up action in ops backlog.
- HARD_ALERT requires immediate mitigation: reduce optional offloads or rotate/prune secret versions.
MARKDOWN
} > "$report_path"

if [[ -n "$OUTPUT_FILE" ]]; then
  echo "[OK] Report written: $OUTPUT_FILE"
fi
