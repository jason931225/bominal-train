#!/usr/bin/env bash
# ==============================================================================
# Zero-Downtime Deployment Script for bominal (Monolithic Runtime)
# ==============================================================================
#
# Usage:
#   ./deploy.sh              # Deploy latest images
#   ./deploy.sh <commit>     # Deploy specific commit SHA
#   ./deploy.sh --rollback   # Roll back to previous deployment
#   ./deploy.sh --status     # Show deployment status
#
# Environment:
#   GHCR_NAMESPACE   - GHCR image namespace (default: ghcr.io/jason931225/bominal)
#   GHCR_USERNAME    - Optional GHCR username for docker login
#   GHCR_TOKEN       - Optional GHCR token/PAT for docker login
#   API_IMAGE        - Override monolithic API image URL
#   WORKER_IMAGE     - Override worker image URL
#   WEB_IMAGE        - Override web image URL
#
# Backward compatibility:
#   Older split-runtime env overrides (API_GATEWAY_IMAGE, API_TRAIN_IMAGE,
#   API_RESTAURANT_IMAGE, WORKER_TRAIN_IMAGE, WORKER_RESTAURANT_IMAGE) are
#   still accepted and mapped to monolithic API/worker image overrides.
# ==============================================================================
set -euo pipefail

# Configuration
REPO_DIR="${REPO_DIR:-/opt/bominal/repo}"
COMPOSE_FILE="infra/docker-compose.prod.yml"
DEPLOY_HISTORY_DIR="${DEPLOY_HISTORY_DIR:-/opt/bominal/deployments}"
DEPLOY_LOCK_FILE="${DEPLOY_LOCK_FILE:-/tmp/bominal-deploy.lock}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PREDEPLOY_CHECK_SCRIPT="${PREDEPLOY_CHECK_SCRIPT:-$SCRIPT_DIR/predeploy-check.sh}"
MAX_HISTORY=10
AUTO_ROLLBACK_ON_SMOKE_FAILURE="${AUTO_ROLLBACK_ON_SMOKE_FAILURE:-true}"
SMOKE_MAX_ATTEMPTS="${SMOKE_MAX_ATTEMPTS:-30}"
SMOKE_RETRY_DELAY_SECONDS="${SMOKE_RETRY_DELAY_SECONDS:-2}"
DEPLOY_MIN_TOTAL_MEMORY_MB="${DEPLOY_MIN_TOTAL_MEMORY_MB:-900}"
DEPLOY_MIN_TOTAL_SWAP_MB="${DEPLOY_MIN_TOTAL_SWAP_MB:-900}"
DEPLOY_LOCK_FALLBACK_DIR=""
DEPLOY_FAIL_ON_DIRTY_REPO="${DEPLOY_FAIL_ON_DIRTY_REPO:-false}"
DEPLOY_API_CHANGED="true"
DEPLOY_WORKER_CHANGED="true"
DEPLOY_WEB_CHANGED="true"
PROD_API_ENV_FILE="infra/env/prod/api.env"

# Registry configuration
GHCR_NAMESPACE="${GHCR_NAMESPACE:-ghcr.io/jason931225/bominal}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info()  { echo -e "${BLUE}[INFO]${NC} $*"; }
log_ok()    { echo -e "${GREEN}[OK]${NC} $*"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }

read_env_file_value() {
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

is_truthy_bool() {
  local value="$1"
  local normalized
  normalized="$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')"
  case "$normalized" in
    true|1|yes)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

is_falsey_bool() {
  local value="$1"
  local normalized
  normalized="$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')"
  case "$normalized" in
    false|0|no)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

validate_master_key_b64() {
  local candidate="$1"
  local source="$2"

  if ! command -v python3 >/dev/null 2>&1; then
    log_error "python3 is required to validate ${source} MASTER_KEY payload."
    return 1
  fi

  if ! python3 - <<'PY' "$candidate" "$source"; then
import base64
import sys

value = (sys.argv[1] or "").strip()
source = sys.argv[2]
if not value:
    print(f"{source} master key is empty", file=sys.stderr)
    raise SystemExit(1)
try:
    decoded = base64.b64decode(value, validate=True)
except Exception:
    print(f"{source} master key is not valid base64", file=sys.stderr)
    raise SystemExit(1)
if len(decoded) != 32:
    print(f"{source} master key must decode to 32 bytes", file=sys.stderr)
    raise SystemExit(1)
PY
    return 1
  fi

  return 0
}

resolve_master_key_override_from_gsm() {
  export MASTER_KEY_OVERRIDE=""

  if [[ ! -f "$PROD_API_ENV_FILE" ]]; then
    return 0
  fi

  local gsm_enabled
  gsm_enabled="$(read_env_file_value "$PROD_API_ENV_FILE" "GSM_MASTER_KEY_ENABLED" | tr '[:upper:]' '[:lower:]')"
  if ! is_truthy_bool "${gsm_enabled:-false}"; then
    return 0
  fi

  if ! command -v gcloud >/dev/null 2>&1; then
    log_error "GSM_MASTER_KEY_ENABLED=true but gcloud CLI is not available on deploy host."
    return 1
  fi

  local project_id secret_id secret_version allow_fallback secret_value
  project_id="$(read_env_file_value "$PROD_API_ENV_FILE" "GSM_MASTER_KEY_PROJECT_ID")"
  if [[ -z "$project_id" ]]; then
    project_id="$(read_env_file_value "$PROD_API_ENV_FILE" "GCP_PROJECT_ID")"
  fi
  secret_id="$(read_env_file_value "$PROD_API_ENV_FILE" "GSM_MASTER_KEY_SECRET_ID")"
  secret_version="$(read_env_file_value "$PROD_API_ENV_FILE" "GSM_MASTER_KEY_VERSION")"
  allow_fallback="$(read_env_file_value "$PROD_API_ENV_FILE" "GSM_MASTER_KEY_ALLOW_ENV_FALLBACK" | tr '[:upper:]' '[:lower:]')"

  if [[ -z "$project_id" ]]; then
    log_error "GSM_MASTER_KEY_ENABLED=true requires GSM_MASTER_KEY_PROJECT_ID or GCP_PROJECT_ID in $PROD_API_ENV_FILE"
    return 1
  fi
  if [[ -z "$secret_id" ]]; then
    log_error "GSM_MASTER_KEY_ENABLED=true requires GSM_MASTER_KEY_SECRET_ID in $PROD_API_ENV_FILE"
    return 1
  fi
  if [[ -z "$secret_version" ]]; then
    log_error "GSM_MASTER_KEY_ENABLED=true requires GSM_MASTER_KEY_VERSION in $PROD_API_ENV_FILE"
    return 1
  fi
  if [[ "$(printf '%s' "$secret_version" | tr '[:upper:]' '[:lower:]')" == "latest" ]]; then
    log_error "GSM_MASTER_KEY_VERSION must be pinned in production (latest is not allowed)."
    return 1
  fi
  if [[ -z "$allow_fallback" ]]; then
    log_error "GSM_MASTER_KEY_ALLOW_ENV_FALLBACK must be explicitly set to false in $PROD_API_ENV_FILE"
    return 1
  fi
  if ! is_truthy_bool "$allow_fallback" && ! is_falsey_bool "$allow_fallback"; then
    log_error "GSM_MASTER_KEY_ALLOW_ENV_FALLBACK must be one of true|false|1|0|yes|no."
    return 1
  fi
  if is_truthy_bool "$allow_fallback"; then
    log_error "GSM_MASTER_KEY_ALLOW_ENV_FALLBACK must be false in production."
    return 1
  fi

  log_info "Resolving runtime MASTER_KEY from Secret Manager (${project_id}/${secret_id}@${secret_version})..."
  secret_value="$(gcloud secrets versions access "$secret_version" \
    --secret="$secret_id" \
    --project="$project_id" 2>/dev/null || true)"
  if [[ -z "$secret_value" ]]; then
    log_error "Failed to fetch master key from Secret Manager (${project_id}/${secret_id}@${secret_version})."
    return 1
  fi

  if ! validate_master_key_b64 "$secret_value" "Secret Manager"; then
    log_error "Secret Manager payload did not contain a valid base64-encoded 32-byte MASTER_KEY."
    return 1
  fi

  export MASTER_KEY_OVERRIDE="$secret_value"
  log_ok "Runtime MASTER_KEY resolved from Secret Manager."
}

acquire_deploy_lock() {
  mkdir -p "$(dirname "$DEPLOY_LOCK_FILE")"
  if command -v flock >/dev/null 2>&1; then
    exec 9>"$DEPLOY_LOCK_FILE"
    if ! flock -n 9; then
      log_error "Another deployment is already running (lock: $DEPLOY_LOCK_FILE)"
      exit 1
    fi
    log_info "Deploy lock acquired via flock ($DEPLOY_LOCK_FILE)"
    return 0
  fi

  DEPLOY_LOCK_FALLBACK_DIR="${DEPLOY_LOCK_FILE}.d"
  if ! mkdir "$DEPLOY_LOCK_FALLBACK_DIR" 2>/dev/null; then
    log_error "Another deployment is already running (lock: $DEPLOY_LOCK_FILE)"
    exit 1
  fi
  trap 'if [[ -n "${DEPLOY_LOCK_FALLBACK_DIR:-}" ]]; then rm -rf "$DEPLOY_LOCK_FALLBACK_DIR"; fi' EXIT
  log_warn "flock not found; using mkdir-based lock fallback."
  log_info "Deploy lock acquired via mkdir fallback ($DEPLOY_LOCK_FILE)"
}

# Ensure we're in the repo directory
cd "$REPO_DIR"

# Detect docker compose command
if docker compose version >/dev/null 2>&1; then
  COMPOSE_CMD=(docker compose)
else
  COMPOSE_CMD=(docker-compose)
fi

mkdir -p "$DEPLOY_HISTORY_DIR"

stack_has_running_containers() {
  local running
  running="$("${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" ps --services --filter status=running 2>/dev/null || true)"
  running="${running//$'\n'/}"
  running="${running//[[:space:]]/}"
  [[ -n "$running" ]]
}

normalize_inspect_value() {
  local value="$1"
  value="${value//$'\n'/}"
  value="${value//[[:space:]]/}"
  if [[ "$value" == "<novalue>" ]]; then
    value=""
  fi
  printf '%s' "$value"
}

image_id_for_ref() {
  local image_ref="$1"
  normalize_inspect_value "$(docker inspect "$image_ref" --format='{{.Id}}' 2>/dev/null || true)"
}

container_image_id() {
  local container_name="$1"
  normalize_inspect_value "$(docker inspect "$container_name" --format='{{.Image}}' 2>/dev/null || true)"
}

container_running_state() {
  local container_name="$1"
  normalize_inspect_value "$(docker inspect "$container_name" --format='{{.State.Running}}' 2>/dev/null || true)"
}

image_revision_label() {
  local image_ref="$1"
  normalize_inspect_value "$(docker inspect "$image_ref" --format='{{index .Config.Labels "org.opencontainers.image.revision"}}' 2>/dev/null || true)"
}

set_images_from_legacy_overrides() {
  if [[ -z "${API_IMAGE:-}" ]]; then
    API_IMAGE="${API_GATEWAY_IMAGE:-${API_TRAIN_IMAGE:-${API_RESTAURANT_IMAGE:-}}}"
  fi

  if [[ -z "${WORKER_IMAGE:-}" ]]; then
    WORKER_IMAGE="${WORKER_IMAGE:-${WORKER_TRAIN_IMAGE:-${WORKER_RESTAURANT_IMAGE:-${API_IMAGE:-}}}}"
  fi
}

calculate_rollout_changes() {
  local target_api_id target_worker_id target_web_id
  local current_api_id current_worker_id current_web_id
  local api_running worker_running web_running

  DEPLOY_API_CHANGED="true"
  DEPLOY_WORKER_CHANGED="true"
  DEPLOY_WEB_CHANGED="true"

  target_api_id="$(image_id_for_ref "$API_IMAGE")"
  target_worker_id="$(image_id_for_ref "$WORKER_IMAGE")"
  target_web_id="$(image_id_for_ref "$WEB_IMAGE")"

  current_api_id="$(container_image_id "bominal-api")"
  current_worker_id="$(container_image_id "bominal-worker")"
  current_web_id="$(container_image_id "bominal-web")"

  if [[ -n "$target_api_id" && -n "$current_api_id" && "$target_api_id" == "$current_api_id" ]]; then
    DEPLOY_API_CHANGED="false"
  fi
  if [[ -n "$target_worker_id" && -n "$current_worker_id" && "$target_worker_id" == "$current_worker_id" ]]; then
    DEPLOY_WORKER_CHANGED="false"
  fi
  if [[ -n "$target_web_id" && -n "$current_web_id" && "$target_web_id" == "$current_web_id" ]]; then
    DEPLOY_WEB_CHANGED="false"
  fi

  api_running="$(container_running_state "bominal-api")"
  worker_running="$(container_running_state "bominal-worker")"
  web_running="$(container_running_state "bominal-web")"

  if [[ -z "$current_api_id" || "$api_running" == "false" ]]; then
    DEPLOY_API_CHANGED="true"
    log_warn "api container missing or stopped; forcing rollout"
  fi
  if [[ -z "$current_worker_id" || "$worker_running" == "false" ]]; then
    DEPLOY_WORKER_CHANGED="true"
    log_warn "worker container missing or stopped; forcing rollout"
  fi
  if [[ -z "$current_web_id" || "$web_running" == "false" ]]; then
    DEPLOY_WEB_CHANGED="true"
    log_warn "web container missing or stopped; forcing rollout"
  fi

  log_info "Rollout plan:"
  log_info "  api_changed=${DEPLOY_API_CHANGED}"
  log_info "  worker_changed=${DEPLOY_WORKER_CHANGED}"
  log_info "  web_changed=${DEPLOY_WEB_CHANGED}"
}

check_repo_state() {
  if ! command -v git >/dev/null 2>&1; then
    return 0
  fi
  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    return 0
  fi

  local dirty output_count
  dirty="$(git status --porcelain --untracked-files=no -- \
    . \
    ':(exclude)infra/env/prod/*.env' \
    ':(exclude)infra/env/prod/*.env.example' 2>/dev/null || true)"

  if [[ -z "${dirty//[[:space:]]/}" ]]; then
    log_ok "Repo working tree state: clean"
    return 0
  fi

  output_count="$(printf '%s\n' "$dirty" | sed '/^[[:space:]]*$/d' | wc -l | tr -d ' ')"
  log_warn "Repo working tree has ${output_count} tracked change(s) (prod env files excluded)."

  if [[ "$DEPLOY_FAIL_ON_DIRTY_REPO" == "true" ]]; then
    log_error "DEPLOY_FAIL_ON_DIRTY_REPO=true and tracked changes were detected."
    log_error "Commit/stash changes or set DEPLOY_FAIL_ON_DIRTY_REPO=false to continue."
    exit 1
  fi

  log_warn "Continuing deploy because runtime uses immutable registry images."
}

run_preflight_checks() {
  if [[ ! -f "$PREDEPLOY_CHECK_SCRIPT" ]]; then
    log_error "Predeploy check script not found: $PREDEPLOY_CHECK_SCRIPT"
    exit 1
  fi

  log_info "Running preflight checks (memory>=${DEPLOY_MIN_TOTAL_MEMORY_MB}MB, swap>=${DEPLOY_MIN_TOTAL_SWAP_MB}MB)..."
  if ! bash "$PREDEPLOY_CHECK_SCRIPT" \
    --skip-smoke-tests \
    --min-total-memory-mb "$DEPLOY_MIN_TOTAL_MEMORY_MB" \
    --min-total-swap-mb "$DEPLOY_MIN_TOTAL_SWAP_MB"; then
    log_error "Preflight checks failed. Deployment aborted before pull/deploy."
    exit 1
  fi
  log_ok "Preflight checks passed"
}

resolve_ghcr_namespace() {
  if [[ -n "${GHCR_NAMESPACE:-}" ]]; then
    export GHCR_NAMESPACE
    return 0
  fi

  if [[ -f "infra/env/prod/api.env" ]]; then
    GHCR_NAMESPACE="$(grep -E '^GHCR_NAMESPACE=' infra/env/prod/api.env | cut -d'=' -f2- | tr -d '"' || echo "")"
  fi

  GHCR_NAMESPACE="${GHCR_NAMESPACE:-ghcr.io/jason931225/bominal}"
  if [[ "$GHCR_NAMESPACE" != ghcr.io/* ]]; then
    log_error "GHCR_NAMESPACE must start with ghcr.io/ (got: $GHCR_NAMESPACE)"
    exit 1
  fi

  export GHCR_NAMESPACE
}

get_current_version() {
  if [[ -f "$DEPLOY_HISTORY_DIR/current" ]]; then
    local v
    v=$(cat "$DEPLOY_HISTORY_DIR/current" 2>/dev/null || true)
    v=${v//$'\n'/}
    if [[ -z "$v" || "$v" == "<no value>" ]]; then
      echo "unknown"
    else
      echo "$v"
    fi
  else
    echo "unknown"
  fi
}

save_deployment() {
  local commit="$1"
  local api_digest="${2:-}"
  local worker_digest="${3:-}"
  local web_digest="${4:-}"
  local timestamp prev_commit

  timestamp=$(date -u +"%Y%m%d_%H%M%S")
  prev_commit=$(get_current_version)

  {
    printf 'commit=%q\n' "$commit"
    printf 'timestamp=%q\n' "$timestamp"
    printf 'api_image=%q\n' "$API_IMAGE"
    printf 'worker_image=%q\n' "$WORKER_IMAGE"
    printf 'web_image=%q\n' "$WEB_IMAGE"
    printf 'api_digest=%q\n' "$api_digest"
    printf 'worker_digest=%q\n' "$worker_digest"
    printf 'web_digest=%q\n' "$web_digest"
    printf 'deployed_by=%q\n' "${USER:-unknown}"
    printf 'previous=%q\n' "$prev_commit"
  } > "$DEPLOY_HISTORY_DIR/$timestamp"

  echo "$commit" > "$DEPLOY_HISTORY_DIR/current"
  if [[ "$prev_commit" != "$commit" && "$prev_commit" != "unknown" ]]; then
    echo "$prev_commit" > "$DEPLOY_HISTORY_DIR/previous"
  fi

  cd "$DEPLOY_HISTORY_DIR"
  ls -t | grep -E '^[0-9]{8}_[0-9]{6}$' | tail -n +$((MAX_HISTORY + 1)) | xargs -r rm -f
  cd "$REPO_DIR"
}

decode_record_value() {
  local value="$1"
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"

  if [[ "$value" == \"*\" && "$value" == *\" ]]; then
    value="${value:1:${#value}-2}"
  elif [[ "$value" == \'*\' && "$value" == *\' ]]; then
    value="${value:1:${#value}-2}"
  fi

  value="$(printf '%s' "$value" | sed 's/\\\(.\)/\1/g')"
  printf '%s' "$value"
}

record_commit=""
record_timestamp=""
record_api_image=""
record_worker_image=""
record_web_image=""
record_api_digest=""
record_worker_digest=""
record_web_digest=""
record_deployed_by=""
record_previous=""

load_deployment_record() {
  local path="$1"
  local line key raw

  record_commit=""
  record_timestamp=""
  record_api_image=""
  record_worker_image=""
  record_web_image=""
  record_api_digest=""
  record_worker_digest=""
  record_web_digest=""
  record_deployed_by=""
  record_previous=""

  [[ -f "$path" ]] || return 1

  while IFS= read -r line || [[ -n "$line" ]]; do
    [[ -z "${line//[[:space:]]/}" ]] && continue
    [[ "$line" =~ ^[[:space:]]*# ]] && continue
    [[ "$line" == *=* ]] || return 1

    key="${line%%=*}"
    raw="${line#*=}"
    key="${key//[[:space:]]/}"

    case "$key" in
      commit) record_commit="$(decode_record_value "$raw")" ;;
      timestamp) record_timestamp="$(decode_record_value "$raw")" ;;
      api_image) record_api_image="$(decode_record_value "$raw")" ;;
      worker_image) record_worker_image="$(decode_record_value "$raw")" ;;
      web_image) record_web_image="$(decode_record_value "$raw")" ;;
      api_digest) record_api_digest="$(decode_record_value "$raw")" ;;
      worker_digest) record_worker_digest="$(decode_record_value "$raw")" ;;
      web_digest) record_web_digest="$(decode_record_value "$raw")" ;;
      deployed_by) record_deployed_by="$(decode_record_value "$raw")" ;;
      previous) record_previous="$(decode_record_value "$raw")" ;;
      # Backward compatibility with split-runtime record keys.
      api_gateway_image|api_train_image|api_restaurant_image)
        raw="$(decode_record_value "$raw")"
        if [[ -z "$record_api_image" && -n "$raw" ]]; then record_api_image="$raw"; fi
        ;;
      worker_train_image|worker_restaurant_image)
        raw="$(decode_record_value "$raw")"
        if [[ -z "$record_worker_image" && -n "$raw" ]]; then record_worker_image="$raw"; fi
        ;;
      api_gateway_digest|api_train_digest|api_restaurant_digest)
        raw="$(decode_record_value "$raw")"
        if [[ -z "$record_api_digest" && -n "$raw" ]]; then record_api_digest="$raw"; fi
        ;;
      worker_train_digest|worker_restaurant_digest)
        raw="$(decode_record_value "$raw")"
        if [[ -z "$record_worker_digest" && -n "$raw" ]]; then record_worker_digest="$raw"; fi
        ;;
      *)
        ;;
    esac
  done <"$path"

  [[ -n "$record_commit" ]] || return 1
  [[ -n "$record_timestamp" ]] || return 1
  return 0
}

get_previous_version() {
  if [[ -f "$DEPLOY_HISTORY_DIR/previous" ]]; then
    local v
    v=$(cat "$DEPLOY_HISTORY_DIR/previous" 2>/dev/null || true)
    v=${v//$'\n'/}
    if [[ -z "$v" || "$v" == "<no value>" || "$v" == "unknown" ]]; then
      log_error "No previous deployment found for rollback"
      exit 1
    fi
    echo "$v"
  else
    log_error "No previous deployment found for rollback"
    exit 1
  fi
}

purge_legacy_records() {
  local dry_run="${1:-false}"
  local backup="${2:-false}"
  local -a to_delete=()

  log_info "Purging legacy/malformed deployment history records in: $DEPLOY_HISTORY_DIR"

  if [[ ! -d "$DEPLOY_HISTORY_DIR" ]]; then
    log_warn "History dir does not exist: $DEPLOY_HISTORY_DIR"
    return 0
  fi

  while IFS= read -r record; do
    [[ -z "$record" ]] && continue
    local path="$DEPLOY_HISTORY_DIR/$record"
    [[ -f "$path" ]] || continue

    local compact
    compact=$(tr -d '[:space:]' <"$path" 2>/dev/null || true)
    if [[ -n "$compact" && "$compact" =~ ^[0-9a-f]{7,40}$ ]] && ! grep -q '=' "$path" 2>/dev/null; then
      to_delete+=("$path")
      continue
    fi

    if ! load_deployment_record "$path" >/dev/null 2>&1; then
      to_delete+=("$path")
      continue
    fi
  done < <(ls -1 "$DEPLOY_HISTORY_DIR" 2>/dev/null | grep -E '^[0-9]{8}_[0-9]{6}$' || true)

  if [[ ${#to_delete[@]} -eq 0 ]]; then
    log_ok "No legacy/malformed records found."
    return 0
  fi

  log_info "Found ${#to_delete[@]} legacy/malformed record(s):"
  for p in "${to_delete[@]}"; do
    echo "  - $p"
  done

  if [[ "$dry_run" == "true" ]]; then
    log_ok "Dry run complete (no files deleted)."
    return 0
  fi

  if [[ "$backup" == "true" ]]; then
    local ts backup_path
    ts=$(date -u +"%Y%m%d_%H%M%S")
    backup_path="$DEPLOY_HISTORY_DIR/legacy-backup-$ts.tgz"
    log_info "Creating backup tarball: $backup_path"
    local -a rel_files=()
    for p in "${to_delete[@]}"; do
      rel_files+=("$(basename "$p")")
    done
    tar -czf "$backup_path" -C "$DEPLOY_HISTORY_DIR" "${rel_files[@]}"
    log_ok "Backup created."
  fi

  log_warn "Deleting legacy/malformed records..."
  for p in "${to_delete[@]}"; do
    rm -f "$p"
  done
  log_ok "Purge complete."
}

do_rollback() {
  local prev_commit prev_record current_commit
  prev_commit="$(get_previous_version)"
  log_warn "Rolling back to previous deployment: $prev_commit"

  prev_record=$( (ls -t "$DEPLOY_HISTORY_DIR" | grep -E '^[0-9]{8}_[0-9]{6}$' | while read -r record; do
    if grep -q "commit=$prev_commit" "$DEPLOY_HISTORY_DIR/$record" 2>/dev/null; then
      echo "$record"
      break
    fi
  done) || true)

  if [[ -n "$prev_record" ]] && load_deployment_record "$DEPLOY_HISTORY_DIR/$prev_record"; then
    if [[ -n "${record_api_digest:-}" && -n "${record_worker_digest:-}" && -n "${record_web_digest:-}" ]]; then
      export API_IMAGE="$record_api_digest"
      export WORKER_IMAGE="$record_worker_digest"
      export WEB_IMAGE="$record_web_digest"
      log_info "Using previous image digests:"
      log_info "  API: $API_IMAGE"
      log_info "  Worker: $WORKER_IMAGE"
      log_info "  Web: $WEB_IMAGE"
    else
      log_warn "Deployment record missing digests; falling back to commit tags"
      resolve_ghcr_namespace
      export API_IMAGE="${GHCR_NAMESPACE}/api:${prev_commit}"
      export WORKER_IMAGE="${GHCR_NAMESPACE}/api:${prev_commit}"
      export WEB_IMAGE="${GHCR_NAMESPACE}/web:${prev_commit}"
    fi
  else
    log_warn "No valid detailed record found; using commit tags"
    resolve_ghcr_namespace
    export API_IMAGE="${GHCR_NAMESPACE}/api:${prev_commit}"
    export WORKER_IMAGE="${GHCR_NAMESPACE}/api:${prev_commit}"
    export WEB_IMAGE="${GHCR_NAMESPACE}/web:${prev_commit}"
  fi

  current_commit="$(get_current_version)"

  pull_images || return 1
  deploy_services || return 1

  echo "$prev_commit" > "$DEPLOY_HISTORY_DIR/current"
  echo "$current_commit" > "$DEPLOY_HISTORY_DIR/previous"
  log_ok "Rollback complete to $prev_commit"
}

configure_docker_auth() {
  if [[ -n "${GHCR_USERNAME:-}" && -n "${GHCR_TOKEN:-}" ]]; then
    log_info "Logging into GHCR as ${GHCR_USERNAME}..."
    if ! printf '%s' "$GHCR_TOKEN" | docker login ghcr.io -u "$GHCR_USERNAME" --password-stdin >/dev/null 2>&1; then
      log_error "GHCR login failed for GHCR_USERNAME=${GHCR_USERNAME}"
      log_error "Fix GHCR credentials or unset GHCR_USERNAME/GHCR_TOKEN for anonymous pulls."
      exit 1
    fi
    log_ok "GHCR login successful"
    return 0
  fi
  log_warn "GHCR_USERNAME/GHCR_TOKEN not set; attempting anonymous image pulls."
}

pull_images() {
  local image
  local -a images_to_pull=("$API_IMAGE" "$WORKER_IMAGE" "$WEB_IMAGE")
  local seen_images=$'\n'

  log_info "Pulling images from GHCR..."
  log_info "  API: $API_IMAGE"
  log_info "  Worker: $WORKER_IMAGE"
  log_info "  Web: $WEB_IMAGE"

  for image in "${images_to_pull[@]}"; do
    [[ -z "$image" ]] && continue
    if [[ "$seen_images" == *$'\n'"$image"$'\n'* ]]; then
      continue
    fi
    seen_images+="$image"$'\n'
    if ! docker pull "$image"; then
      log_error "Failed to pull image: $image"
      log_error "Make sure the image exists and you have access"
      return 1
    fi
  done
  log_ok "Images pulled successfully"
}

deploy_services() {
  log_info "Deploying with zero-downtime strategy..."

  export API_IMAGE
  export WORKER_IMAGE
  export WEB_IMAGE

  if stack_has_running_containers; then
    log_info "Running stack detected. Using rolling-update path."
    calculate_rollout_changes

    log_info "Ensuring redis service is healthy..."
    if ! "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait redis; then
      log_error "Failed to deploy redis service"
      return 1
    fi

    if [[ "$DEPLOY_API_CHANGED" == "true" ]]; then
      log_info "Deploying API service..."
      if ! "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait --no-deps api; then
        log_error "Failed to deploy api service"
        return 1
      fi
    else
      log_info "Skipping API rollout (image unchanged)."
    fi

    if [[ "$DEPLOY_WORKER_CHANGED" == "true" ]]; then
      log_info "Deploying worker service..."
      if ! "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait --no-deps worker; then
        log_error "Failed to deploy worker service"
        return 1
      fi
    else
      log_info "Skipping worker rollout (image unchanged)."
    fi

    if [[ "$DEPLOY_WEB_CHANGED" == "true" ]]; then
      log_info "Deploying web service..."
      if ! "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait --no-deps web; then
        log_error "Failed to deploy web service"
        return 1
      fi
    else
      log_info "Skipping web rollout (image unchanged)."
    fi

    # Keep Caddy reconciliation isolated so unchanged dependencies are not
    # implicitly recreated during rolling updates.
    log_info "Ensuring Caddy is healthy..."
    if ! "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait --no-deps caddy; then
      log_error "Failed to deploy Caddy service"
      return 1
    fi
    return 0
  fi

  log_warn "No running bominal containers detected. Using first-deploy bootstrap path."
  if ! "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait redis api worker web caddy; then
    log_error "Failed to deploy services in bootstrap path"
    return 1
  fi
}

verify_deployment() {
  log_info "Verifying deployment health..."
  local max_attempts="${SMOKE_MAX_ATTEMPTS}"
  local retry_delay="${SMOKE_RETRY_DELAY_SECONDS}"
  local attempt=1

  while [[ $attempt -le $max_attempts ]]; do
    if curl -fsS --max-time 5 http://127.0.0.1:8000/health >/dev/null 2>&1; then
      log_ok "API health check passed"
      break
    fi
    log_warn "Waiting for API... (attempt $attempt/$max_attempts)"
    sleep "$retry_delay"
    ((attempt++))
  done

  if [[ $attempt -gt $max_attempts ]]; then
    log_error "API health check failed after $max_attempts attempts"
    return 1
  fi

  attempt=1
  while [[ $attempt -le $max_attempts ]]; do
    if curl -fsS --max-time 5 -I http://127.0.0.1/ >/dev/null 2>&1; then
      log_ok "Web health check passed (via Caddy)"
      break
    fi
    log_warn "Waiting for Web... (attempt $attempt/$max_attempts)"
    sleep "$retry_delay"
    ((attempt++))
  done

  if [[ $attempt -gt $max_attempts ]]; then
    log_error "Web health check failed after $max_attempts attempts"
    return 1
  fi

  log_ok "All health checks passed!"
}

show_status() {
  log_info "Current deployment status:"
  "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" ps
  echo ""
  log_info "Container health:"
  docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Image}}" | grep -E "(NAMES|bominal)" || true
  echo ""
  log_info "Current version: $(get_current_version)"
  if [[ -f "$DEPLOY_HISTORY_DIR/previous" ]]; then
    log_info "Previous version: $(cat "$DEPLOY_HISTORY_DIR/previous")"
  fi
  echo ""
  log_info "Recent deployments:"
  (ls -t "$DEPLOY_HISTORY_DIR" | grep -E '^[0-9]{8}_[0-9]{6}$' | head -5 | while read -r record; do
    if [[ -f "$DEPLOY_HISTORY_DIR/$record" ]]; then
      if load_deployment_record "$DEPLOY_HISTORY_DIR/$record"; then
        echo "  - $record_timestamp: $record_commit (by ${record_deployed_by:-unknown})"
      else
        log_warn "Could not load deployment record: $record"
      fi
    fi
  done) || true
}

cleanup_docker() {
  log_info "Cleaning up old Docker resources..."
  docker image prune -f >/dev/null 2>&1 || true
  docker images --format "{{.Repository}}:{{.Tag}}" | grep "bominal" | tail -n +4 | xargs -r docker rmi -f 2>/dev/null || true
  log_ok "Docker cleanup complete"
}

main() {
  local target_commit="${1:-}"
  local canary_stage=""

  acquire_deploy_lock
  check_repo_state

  # Optional no-op canary flag kept for compatibility with existing runbooks/tests.
  if [[ "$target_commit" == --canary-stage=* ]]; then
    canary_stage="${target_commit#--canary-stage=}"
    shift || true
    target_commit="${1:-}"
  elif [[ "$target_commit" == "--canary-stage" ]]; then
    canary_stage="${2:-}"
    shift 2 || true
    target_commit="${1:-}"
  fi
  if [[ -n "$canary_stage" ]]; then
    log_info "Canary stage requested: $canary_stage (compatibility mode; no staged mutation applied)"
  fi

  if [[ "${target_commit:-}" == "--purge-legacy-records" ]]; then
    shift || true
    local dry_run="false"
    local backup="false"
    while [[ $# -gt 0 ]]; do
      case "$1" in
        --dry-run) dry_run="true" ;;
        --backup) backup="true" ;;
        --help|-h|help)
          echo "Usage: $0 --purge-legacy-records [--dry-run] [--backup]"
          exit 0
          ;;
        *)
          log_error "Unknown flag for --purge-legacy-records: $1"
          exit 1
          ;;
      esac
      shift
    done
    purge_legacy_records "$dry_run" "$backup"
    exit 0
  fi

  case "${target_commit:-}" in
    --rollback|-r)
      run_preflight_checks
      resolve_master_key_override_from_gsm
      configure_docker_auth
      do_rollback
      verify_deployment
      show_status
      exit 0
      ;;
    --status|-s|status)
      show_status
      exit 0
      ;;
    --help|-h|help)
      echo "Usage: $0 [command] [commit]"
      echo ""
      echo "Commands:"
      echo "  deploy         Deploy latest images (default)"
      echo "  <commit>       Deploy specific commit SHA"
      echo "  --rollback     Rollback to previous deployment"
      echo "  --status       Show current deployment status"
      echo "  --purge-legacy-records [--dry-run] [--backup]"
      echo "                 Delete legacy/malformed historical record files (safe; does not touch current/previous)"
      echo ""
      echo "Environment variables:"
      echo "  GHCR_NAMESPACE   GHCR image namespace (default: ghcr.io/jason931225/bominal)"
      echo "  GHCR_USERNAME    Optional GHCR username for docker login"
      echo "  GHCR_TOKEN       Optional GHCR token/PAT for docker login"
      echo "  API_IMAGE        Override API image URL"
      echo "  WORKER_IMAGE     Override worker image URL"
      echo "  WEB_IMAGE        Override web image URL"
      echo "  MASTER_KEY_OVERRIDE Deploy-time runtime override for MASTER_KEY (set automatically when GSM is enabled)"
      echo "  DEPLOY_HISTORY_DIR Override deployment history dir (default: /opt/bominal/deployments)"
      echo "  DEPLOY_FAIL_ON_DIRTY_REPO=true  Block deploy when tracked repo changes are present"
      echo ""
      echo "Examples:"
      echo "  $0"
      echo "  $0 abc123"
      echo "  $0 --rollback"
      exit 0
      ;;
  esac

  run_preflight_checks
  resolve_master_key_override_from_gsm
  configure_docker_auth
  resolve_ghcr_namespace

  local prev_version deploy_commit
  prev_version="$(get_current_version)"

  if [[ -n "${target_commit:-}" ]]; then
    log_info "Deploying specific commit: $target_commit"
    export API_IMAGE="${GHCR_NAMESPACE}/api:${target_commit}"
    export WORKER_IMAGE="${GHCR_NAMESPACE}/api:${target_commit}"
    export WEB_IMAGE="${GHCR_NAMESPACE}/web:${target_commit}"
  else
    log_info "Deploying latest images"
    set_images_from_legacy_overrides
    export API_IMAGE="${API_IMAGE:-${GHCR_NAMESPACE}/api:latest}"
    export WORKER_IMAGE="${WORKER_IMAGE:-$API_IMAGE}"
    export WEB_IMAGE="${WEB_IMAGE:-${GHCR_NAMESPACE}/web:latest}"
  fi

  pull_images

  if [[ -n "${target_commit:-}" ]]; then
    deploy_commit="$target_commit"
  else
    local -a revisions=()
    local revision image repo_head="" seen_revisions=$'\n'
    for image in "$API_IMAGE" "$WORKER_IMAGE" "$WEB_IMAGE"; do
      revision="$(image_revision_label "$image")"
      if [[ -z "$revision" || "$revision" == "<no value>" ]]; then
        continue
      fi
      if [[ "$seen_revisions" != *$'\n'"$revision"$'\n'* ]]; then
        seen_revisions+="$revision"$'\n'
        revisions+=("$revision")
      fi
    done

    if command -v git >/dev/null 2>&1; then
      repo_head="$(normalize_inspect_value "$(git rev-parse HEAD 2>/dev/null || true)")"
    fi

    if [[ "${#revisions[@]}" -eq 1 ]]; then
      deploy_commit="${revisions[0]}"
    elif [[ "${#revisions[@]}" -gt 1 ]]; then
      if [[ -n "$repo_head" && "$seen_revisions" == *$'\n'"$repo_head"$'\n'* ]]; then
        deploy_commit="$repo_head"
      else
        deploy_commit="${revisions[0]}"
      fi
      log_warn "Detected mixed image revisions; recording deploy commit as $deploy_commit"
    elif [[ -n "$repo_head" ]]; then
      deploy_commit="$repo_head"
      log_warn "Image revision labels missing; using repository HEAD for deployment record: $deploy_commit"
    else
      log_error "Could not determine deployment commit from images or repository state."
      log_error "Fix: ensure CI sets org.opencontainers.image.revision, or deploy a specific commit tag: $0 <commit>"
      exit 1
    fi
  fi

  log_info "Deploying commit: $deploy_commit"
  log_info "  API: $API_IMAGE"
  log_info "  Worker: $WORKER_IMAGE"
  log_info "  Web: $WEB_IMAGE"

  if [[ "$deploy_commit" == "$prev_version" ]]; then
    log_warn "Commit $deploy_commit is already deployed. Deploying anyway..."
  fi

  local api_digest worker_digest web_digest
  api_digest=$(docker inspect "$API_IMAGE" --format='{{if .RepoDigests}}{{index .RepoDigests 0}}{{end}}' 2>/dev/null || true)
  worker_digest=$(docker inspect "$WORKER_IMAGE" --format='{{if .RepoDigests}}{{index .RepoDigests 0}}{{end}}' 2>/dev/null || true)
  web_digest=$(docker inspect "$WEB_IMAGE" --format='{{if .RepoDigests}}{{index .RepoDigests 0}}{{end}}' 2>/dev/null || true)

  if [[ "$api_digest" == "<no value>" || "$api_digest" != *@sha256:* ]]; then api_digest=""; fi
  if [[ "$worker_digest" == "<no value>" || "$worker_digest" != *@sha256:* ]]; then worker_digest=""; fi
  if [[ "$web_digest" == "<no value>" || "$web_digest" != *@sha256:* ]]; then web_digest=""; fi

  if [[ -n "$api_digest" && -n "$worker_digest" && -n "$web_digest" ]]; then
    log_info "Resolved image digests for rollback:"
    log_info "  API: $api_digest"
    log_info "  Worker: $worker_digest"
    log_info "  Web: $web_digest"
  else
    log_warn "Could not resolve one or more repo digests; rollback may fall back to commit tags"
  fi

  deploy_services

  if verify_deployment; then
    save_deployment "$deploy_commit" "$api_digest" "$worker_digest" "$web_digest"
    cleanup_docker
    log_ok "Deployment of $deploy_commit complete!"
    show_status
  else
    log_error "Deployment verification failed!"
    if [[ "$AUTO_ROLLBACK_ON_SMOKE_FAILURE" == "true" ]]; then
      log_warn "Auto rollback enabled; attempting rollback after smoke failure."
      if do_rollback; then
        if verify_deployment; then
          log_ok "Rollback health verification passed."
        else
          log_error "Rollback completed but health verification failed."
        fi
      else
        log_error "Auto rollback attempt failed."
      fi
    else
      log_warn "Auto rollback disabled (AUTO_ROLLBACK_ON_SMOKE_FAILURE=false)."
    fi
    log_warn "Manual rollback command: $0 --rollback"
    exit 1
  fi
}

main "$@"
