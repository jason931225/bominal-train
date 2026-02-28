#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
POLICY_FILE="$ROOT_DIR/docs/governance/SECRETS_RESIDENCY_POLICY.md"
API_ENV_EXAMPLE="$ROOT_DIR/infra/env/prod/api.env.example"
API_ENV_FILE="$ROOT_DIR/infra/env/prod/api.env"
PREDEPLOY_FILE="$ROOT_DIR/infra/scripts/predeploy-check.sh"

assert_contains() {
  local pattern="$1"
  local file="$2"
  local message="$3"
  if ! rg -n -- "$pattern" "$file" >/dev/null; then
    echo "FAIL: $message ($file)" >&2
    exit 1
  fi
}

env_value() {
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

has_meaningful_value() {
  local value="$1"
  local trimmed
  trimmed="${value#"${value%%[![:space:]]*}"}"
  trimmed="${trimmed%"${trimmed##*[![:space:]]}"}"
  [[ -z "$trimmed" ]] && return 1
  case "$trimmed" in
    *CHANGE_ME*|*REPLACE_ME*|*TODO*|*"<no value>"*) return 1 ;;
  esac
  return 0
}

if [[ ! -f "$POLICY_FILE" ]]; then
  echo "FAIL: missing policy doc: $POLICY_FILE" >&2
  exit 1
fi
if [[ ! -f "$API_ENV_EXAMPLE" ]]; then
  echo "FAIL: missing api env example: $API_ENV_EXAMPLE" >&2
  exit 1
fi

assert_contains "INTERNAL_API_KEY_SECRET_ID" "$POLICY_FILE" "policy must define INTERNAL_API_KEY GSM residency"
assert_contains "RESEND_API_KEY_SECRET_ID" "$POLICY_FILE" "policy must define RESEND_API_KEY GSM residency"
assert_contains "SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID" "$POLICY_FILE" "policy must define Supabase management token GSM residency"
assert_contains "SUPABASE_SERVICE_ROLE_KEY" "$POLICY_FILE" "policy must document bootstrap secret exceptions"

assert_contains '^INTERNAL_API_KEY_SECRET_ID=' "$API_ENV_EXAMPLE" "api.env.example must include INTERNAL_API_KEY secret id key"
assert_contains '^INTERNAL_API_KEY_SECRET_VERSION=' "$API_ENV_EXAMPLE" "api.env.example must include INTERNAL_API_KEY secret version key"
assert_contains '^RESEND_API_KEY_SECRET_ID=' "$API_ENV_EXAMPLE" "api.env.example must include RESEND_API_KEY secret id key"
assert_contains '^RESEND_API_KEY_SECRET_VERSION=' "$API_ENV_EXAMPLE" "api.env.example must include RESEND_API_KEY secret version key"
assert_contains '^SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID=' "$API_ENV_EXAMPLE" "api.env.example must include supabase management token secret id"
assert_contains '^SUPABASE_MANAGEMENT_API_TOKEN_SECRET_VERSION=' "$API_ENV_EXAMPLE" "api.env.example must include supabase management token secret version"
assert_contains '^SUPABASE_MANAGEMENT_API_TOKEN_PROJECT_ID=' "$API_ENV_EXAMPLE" "api.env.example must include supabase management token project id"

assert_contains 'INTERNAL_API_KEY and INTERNAL_API_KEY_SECRET_ID cannot both be set' "$PREDEPLOY_FILE" "predeploy must enforce internal key source exclusivity"
assert_contains 'RESEND API key source is ambiguous' "$PREDEPLOY_FILE" "predeploy must enforce resend source exclusivity"
assert_contains 'SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID' "$PREDEPLOY_FILE" "predeploy must enforce supabase management token gsm source"

if [[ -f "$API_ENV_FILE" ]]; then
  internal_key="$(env_value "$API_ENV_FILE" "INTERNAL_API_KEY")"
  internal_secret_id="$(env_value "$API_ENV_FILE" "INTERNAL_API_KEY_SECRET_ID")"
  internal_secret_version="$(env_value "$API_ENV_FILE" "INTERNAL_API_KEY_SECRET_VERSION")"

  resend_key="$(env_value "$API_ENV_FILE" "RESEND_API_KEY")"
  resend_secret_id="$(env_value "$API_ENV_FILE" "RESEND_API_KEY_SECRET_ID")"
  resend_secret_version="$(env_value "$API_ENV_FILE" "RESEND_API_KEY_SECRET_VERSION")"
  resend_vault_name="$(env_value "$API_ENV_FILE" "RESEND_API_KEY_VAULT_NAME")"
  gcp_project_id="$(env_value "$API_ENV_FILE" "GCP_PROJECT_ID")"
  supabase_management_api_token="$(env_value "$API_ENV_FILE" "SUPABASE_MANAGEMENT_API_TOKEN")"
  supabase_access_token="$(env_value "$API_ENV_FILE" "SUPABASE_ACCESS_TOKEN")"
  supabase_management_token_secret_id="$(env_value "$API_ENV_FILE" "SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID")"
  supabase_management_token_secret_version="$(env_value "$API_ENV_FILE" "SUPABASE_MANAGEMENT_API_TOKEN_SECRET_VERSION")"
  supabase_management_token_project_id="$(env_value "$API_ENV_FILE" "SUPABASE_MANAGEMENT_API_TOKEN_PROJECT_ID")"

  edge_enabled="$(env_value "$API_ENV_FILE" "EDGE_TASK_NOTIFY_ENABLED")"
  supabase_vault_enabled="$(env_value "$API_ENV_FILE" "SUPABASE_VAULT_ENABLED")"
  email_from_address="$(env_value "$API_ENV_FILE" "EMAIL_FROM_ADDRESS")"

  if has_meaningful_value "$internal_key" && has_meaningful_value "$internal_secret_id"; then
    echo "FAIL: infra/env/prod/api.env has ambiguous INTERNAL_API_KEY source" >&2
    exit 1
  fi

  if has_meaningful_value "$internal_secret_id"; then
    if ! has_meaningful_value "$internal_secret_version"; then
      echo "FAIL: INTERNAL_API_KEY_SECRET_ID requires INTERNAL_API_KEY_SECRET_VERSION" >&2
      exit 1
    fi
    if [[ "$(printf '%s' "$internal_secret_version" | tr '[:upper:]' '[:lower:]')" == "latest" ]]; then
      echo "FAIL: INTERNAL_API_KEY_SECRET_VERSION must be pinned (latest is not allowed)" >&2
      exit 1
    fi
  fi

  resend_sources=0
  has_meaningful_value "$resend_key" && resend_sources=$((resend_sources + 1))
  has_meaningful_value "$resend_secret_id" && resend_sources=$((resend_sources + 1))
  has_meaningful_value "$resend_vault_name" && resend_sources=$((resend_sources + 1))
  if [[ "$resend_sources" -gt 1 ]]; then
    echo "FAIL: infra/env/prod/api.env has ambiguous RESEND source configuration" >&2
    exit 1
  fi

  if has_meaningful_value "$resend_secret_id"; then
    if ! has_meaningful_value "$resend_secret_version"; then
      echo "FAIL: RESEND_API_KEY_SECRET_ID requires RESEND_API_KEY_SECRET_VERSION" >&2
      exit 1
    fi
    if [[ "$(printf '%s' "$resend_secret_version" | tr '[:upper:]' '[:lower:]')" == "latest" ]]; then
      echo "FAIL: RESEND_API_KEY_SECRET_VERSION must be pinned (latest is not allowed)" >&2
      exit 1
    fi
  fi

  if is_truthy "$edge_enabled"; then
    edge_resend_sources=0
    has_meaningful_value "$resend_key" && edge_resend_sources=$((edge_resend_sources + 1))
    has_meaningful_value "$resend_secret_id" && edge_resend_sources=$((edge_resend_sources + 1))
    has_meaningful_value "$resend_vault_name" && edge_resend_sources=$((edge_resend_sources + 1))

    if [[ "$edge_resend_sources" -eq 0 ]]; then
      echo "FAIL: EDGE_TASK_NOTIFY_ENABLED=true requires a RESEND key source" >&2
      exit 1
    fi
    if ! has_meaningful_value "$email_from_address"; then
      echo "FAIL: EDGE_TASK_NOTIFY_ENABLED=true requires EMAIL_FROM_ADDRESS" >&2
      exit 1
    fi
  fi

  if has_meaningful_value "$resend_vault_name" && ! is_truthy "$supabase_vault_enabled"; then
    echo "FAIL: RESEND_API_KEY_VAULT_NAME requires SUPABASE_VAULT_ENABLED=true" >&2
    exit 1
  fi

  if has_meaningful_value "$supabase_management_api_token"; then
    echo "FAIL: SUPABASE_MANAGEMENT_API_TOKEN must not be plaintext in infra/env/prod/api.env" >&2
    exit 1
  fi
  if has_meaningful_value "$supabase_access_token"; then
    echo "FAIL: SUPABASE_ACCESS_TOKEN must not be plaintext in infra/env/prod/api.env" >&2
    exit 1
  fi

  if has_meaningful_value "$supabase_management_token_secret_id"; then
    if ! has_meaningful_value "$supabase_management_token_secret_version"; then
      echo "FAIL: SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID requires SUPABASE_MANAGEMENT_API_TOKEN_SECRET_VERSION" >&2
      exit 1
    fi
    if [[ "$(printf '%s' "$supabase_management_token_secret_version" | tr '[:upper:]' '[:lower:]')" == "latest" ]]; then
      echo "FAIL: SUPABASE_MANAGEMENT_API_TOKEN_SECRET_VERSION must be pinned (latest is not allowed)" >&2
      exit 1
    fi
    if ! has_meaningful_value "$supabase_management_token_project_id" && ! has_meaningful_value "$gcp_project_id"; then
      echo "FAIL: SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID requires SUPABASE_MANAGEMENT_API_TOKEN_PROJECT_ID or GCP_PROJECT_ID" >&2
      exit 1
    fi
  fi
fi

echo "OK: secret residency contract checks passed."
