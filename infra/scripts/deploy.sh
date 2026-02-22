#!/usr/bin/env bash
# ==============================================================================
# Zero-Downtime Deployment Script for bominal (CI/CD Version)
# ==============================================================================
# This script pulls pre-built images from Google Artifact Registry and deploys
# them with zero downtime. Images are built on GitHub Actions runners instead
# of the e2-micro VM to prevent OOM issues.
#
# Usage:
#   ./deploy.sh              # Deploy latest images
#   ./deploy.sh <commit>     # Deploy specific commit SHA
#   ./deploy.sh --rollback   # Rollback to previous deployment
#   ./deploy.sh --status     # Show deployment status
#
# Environment:
#   GCP_PROJECT_ID         - Google Cloud project ID (required)
#   API_IMAGE              - Legacy override for all API/worker images
#   API_GATEWAY_IMAGE      - Override API gateway image URL
#   API_TRAIN_IMAGE        - Override API train image URL
#   API_RESTAURANT_IMAGE   - Override API restaurant image URL
#   WORKER_TRAIN_IMAGE     - Override train worker image URL
#   WORKER_RESTAURANT_IMAGE - Override restaurant worker image URL
#   WEB_IMAGE              - Override web image URL
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
DEPLOY_API_GATEWAY_CHANGED="true"
DEPLOY_API_TRAIN_CHANGED="true"
DEPLOY_API_RESTAURANT_CHANGED="true"
DEPLOY_WORKER_TRAIN_CHANGED="true"
DEPLOY_WORKER_RESTAURANT_CHANGED="true"
DEPLOY_WEB_CHANGED="true"

# Google Cloud configuration
GCP_REGION="${GCP_REGION:-us-central1}"
REGISTRY="${REGISTRY:-${GCP_REGION}-docker.pkg.dev}"

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

  # Fallback for minimal environments where flock is unavailable.
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

# Create deployment history directory
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

image_revision_label() {
  local image_ref="$1"
  normalize_inspect_value "$(docker inspect "$image_ref" --format='{{index .Config.Labels "org.opencontainers.image.revision"}}' 2>/dev/null || true)"
}

set_split_images_from_legacy_api_image() {
  if [[ -z "${API_IMAGE:-}" ]]; then
    return 0
  fi

  API_GATEWAY_IMAGE="${API_GATEWAY_IMAGE:-$API_IMAGE}"
  API_TRAIN_IMAGE="${API_TRAIN_IMAGE:-$API_IMAGE}"
  API_RESTAURANT_IMAGE="${API_RESTAURANT_IMAGE:-$API_IMAGE}"
  WORKER_TRAIN_IMAGE="${WORKER_TRAIN_IMAGE:-$API_IMAGE}"
  WORKER_RESTAURANT_IMAGE="${WORKER_RESTAURANT_IMAGE:-$API_IMAGE}"
}

calculate_rollout_changes() {
  local target_api_gateway_id target_api_train_id target_api_restaurant_id
  local target_worker_train_id target_worker_restaurant_id target_web_id
  local current_api_gateway_id current_api_train_id current_api_restaurant_id
  local current_worker_train_id current_worker_restaurant_id current_web_id

  DEPLOY_API_GATEWAY_CHANGED="true"
  DEPLOY_API_TRAIN_CHANGED="true"
  DEPLOY_API_RESTAURANT_CHANGED="true"
  DEPLOY_WORKER_TRAIN_CHANGED="true"
  DEPLOY_WORKER_RESTAURANT_CHANGED="true"
  DEPLOY_WEB_CHANGED="true"

  target_api_gateway_id="$(image_id_for_ref "$API_GATEWAY_IMAGE")"
  target_api_train_id="$(image_id_for_ref "$API_TRAIN_IMAGE")"
  target_api_restaurant_id="$(image_id_for_ref "$API_RESTAURANT_IMAGE")"
  target_worker_train_id="$(image_id_for_ref "$WORKER_TRAIN_IMAGE")"
  target_worker_restaurant_id="$(image_id_for_ref "$WORKER_RESTAURANT_IMAGE")"
  target_web_id="$(image_id_for_ref "$WEB_IMAGE")"

  current_api_gateway_id="$(container_image_id "bominal-api-gateway")"
  current_api_train_id="$(container_image_id "bominal-api-train")"
  current_api_restaurant_id="$(container_image_id "bominal-api-restaurant")"
  current_worker_train_id="$(container_image_id "bominal-worker-train")"
  current_worker_restaurant_id="$(container_image_id "bominal-worker-restaurant")"
  current_web_id="$(container_image_id "bominal-web")"

  if [[ -n "$target_api_gateway_id" && -n "$current_api_gateway_id" && "$target_api_gateway_id" == "$current_api_gateway_id" ]]; then
    DEPLOY_API_GATEWAY_CHANGED="false"
  fi
  if [[ -n "$target_api_train_id" && -n "$current_api_train_id" && "$target_api_train_id" == "$current_api_train_id" ]]; then
    DEPLOY_API_TRAIN_CHANGED="false"
  fi
  if [[ -n "$target_api_restaurant_id" && -n "$current_api_restaurant_id" && "$target_api_restaurant_id" == "$current_api_restaurant_id" ]]; then
    DEPLOY_API_RESTAURANT_CHANGED="false"
  fi
  if [[ -n "$target_worker_train_id" && -n "$current_worker_train_id" && "$target_worker_train_id" == "$current_worker_train_id" ]]; then
    DEPLOY_WORKER_TRAIN_CHANGED="false"
  fi
  if [[ -n "$target_worker_restaurant_id" && -n "$current_worker_restaurant_id" && "$target_worker_restaurant_id" == "$current_worker_restaurant_id" ]]; then
    DEPLOY_WORKER_RESTAURANT_CHANGED="false"
  fi
  if [[ -n "$target_web_id" && -n "$current_web_id" && "$target_web_id" == "$current_web_id" ]]; then
    DEPLOY_WEB_CHANGED="false"
  fi

  log_info "Rollout plan:"
  log_info "  api-gateway_changed=${DEPLOY_API_GATEWAY_CHANGED}"
  log_info "  api-train_changed=${DEPLOY_API_TRAIN_CHANGED}"
  log_info "  api-restaurant_changed=${DEPLOY_API_RESTAURANT_CHANGED}"
  log_info "  worker-train_changed=${DEPLOY_WORKER_TRAIN_CHANGED}"
  log_info "  worker-restaurant_changed=${DEPLOY_WORKER_RESTAURANT_CHANGED}"
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

require_gcp_project_id() {
  if [[ -n "${GCP_PROJECT_ID:-}" ]]; then
    export GCP_PROJECT_ID
    return 0
  fi

  if [[ -f "infra/env/prod/api.env" ]]; then
    # Try to extract from env file
    GCP_PROJECT_ID=$(grep -E '^GCP_PROJECT_ID=' infra/env/prod/api.env | cut -d'=' -f2- | tr -d '"' || echo "")
  fi

  if [[ -z "${GCP_PROJECT_ID:-}" ]]; then
    log_error "GCP_PROJECT_ID not set. Please set it in your environment or infra/env/prod/api.env"
    exit 1
  fi

  export GCP_PROJECT_ID
}

# Get current deployed version
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

# Save deployment record
save_deployment() {
  local commit="$1"
  local api_gateway_digest="${2:-}"
  local api_train_digest="${3:-}"
  local api_restaurant_digest="${4:-}"
  local worker_train_digest="${5:-}"
  local worker_restaurant_digest="${6:-}"
  local web_digest="${7:-}"
  local timestamp
  timestamp=$(date -u +"%Y%m%d_%H%M%S")
  
  # Get previous version before updating
  local prev_commit
  prev_commit=$(get_current_version)
  
  # Save to history with metadata.
  # Use shell-escaped values so records can be safely `source`'d even if values
  # contain spaces or quotes (e.g. OS Login usernames, future image refs).
  {
    printf 'commit=%q\n' "$commit"
    printf 'timestamp=%q\n' "$timestamp"
    printf 'api_gateway_image=%q\n' "$API_GATEWAY_IMAGE"
    printf 'api_train_image=%q\n' "$API_TRAIN_IMAGE"
    printf 'api_restaurant_image=%q\n' "$API_RESTAURANT_IMAGE"
    printf 'worker_train_image=%q\n' "$WORKER_TRAIN_IMAGE"
    printf 'worker_restaurant_image=%q\n' "$WORKER_RESTAURANT_IMAGE"
    printf 'web_image=%q\n' "$WEB_IMAGE"
    printf 'api_gateway_digest=%q\n' "$api_gateway_digest"
    printf 'api_train_digest=%q\n' "$api_train_digest"
    printf 'api_restaurant_digest=%q\n' "$api_restaurant_digest"
    printf 'worker_train_digest=%q\n' "$worker_train_digest"
    printf 'worker_restaurant_digest=%q\n' "$worker_restaurant_digest"
    printf 'web_digest=%q\n' "$web_digest"
    # Backward-compatibility aliases for older tooling
    printf 'api_image=%q\n' "$API_GATEWAY_IMAGE"
    printf 'api_digest=%q\n' "$api_gateway_digest"
    printf 'deployed_by=%q\n' "${USER:-unknown}"
    printf 'previous=%q\n' "$prev_commit"
  } > "$DEPLOY_HISTORY_DIR/$timestamp"
  
  # Update current pointer
  echo "$commit" > "$DEPLOY_HISTORY_DIR/current"
  
  # Update previous pointer
  if [[ "$prev_commit" != "$commit" && "$prev_commit" != "unknown" ]]; then
    echo "$prev_commit" > "$DEPLOY_HISTORY_DIR/previous"
  fi
  
  # Cleanup old history (keep last N)
  cd "$DEPLOY_HISTORY_DIR"
  ls -t | grep -E '^[0-9]{8}_[0-9]{6}$' | tail -n +$((MAX_HISTORY + 1)) | xargs -r rm -f
  cd "$REPO_DIR"
}

# Parse deployment record values without shell execution.
# We intentionally treat records as plain key/value data and never source them.
decode_record_value() {
  local value="$1"

  # Trim surrounding whitespace.
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"

  # Strip optional wrapping quotes.
  if [[ "$value" == \"*\" && "$value" == *\" ]]; then
    value="${value:1:${#value}-2}"
  elif [[ "$value" == \'*\' && "$value" == *\' ]]; then
    value="${value:1:${#value}-2}"
  fi

  # Decode simple backslash-escaped characters produced by printf %q.
  value="$(printf '%s' "$value" | sed 's/\\\(.\)/\1/g')"
  printf '%s' "$value"
}

record_commit=""
record_timestamp=""
record_api_gateway_image=""
record_api_train_image=""
record_api_restaurant_image=""
record_worker_train_image=""
record_worker_restaurant_image=""
record_web_image=""
record_api_gateway_digest=""
record_api_train_digest=""
record_api_restaurant_digest=""
record_worker_train_digest=""
record_worker_restaurant_digest=""
record_web_digest=""
record_deployed_by=""
record_previous=""

load_deployment_record() {
  local path="$1"
  local line key raw

  record_commit=""
  record_timestamp=""
  record_api_gateway_image=""
  record_api_train_image=""
  record_api_restaurant_image=""
  record_worker_train_image=""
  record_worker_restaurant_image=""
  record_web_image=""
  record_api_gateway_digest=""
  record_api_train_digest=""
  record_api_restaurant_digest=""
  record_worker_train_digest=""
  record_worker_restaurant_digest=""
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
      commit)
        record_commit="$(decode_record_value "$raw")"
        ;;
      timestamp)
        record_timestamp="$(decode_record_value "$raw")"
        ;;
      api_gateway_image)
        record_api_gateway_image="$(decode_record_value "$raw")"
        ;;
      api_train_image)
        record_api_train_image="$(decode_record_value "$raw")"
        ;;
      api_restaurant_image)
        record_api_restaurant_image="$(decode_record_value "$raw")"
        ;;
      worker_train_image)
        record_worker_train_image="$(decode_record_value "$raw")"
        ;;
      worker_restaurant_image)
        record_worker_restaurant_image="$(decode_record_value "$raw")"
        ;;
      web_image)
        record_web_image="$(decode_record_value "$raw")"
        ;;
      api_gateway_digest)
        record_api_gateway_digest="$(decode_record_value "$raw")"
        ;;
      api_train_digest)
        record_api_train_digest="$(decode_record_value "$raw")"
        ;;
      api_restaurant_digest)
        record_api_restaurant_digest="$(decode_record_value "$raw")"
        ;;
      worker_train_digest)
        record_worker_train_digest="$(decode_record_value "$raw")"
        ;;
      worker_restaurant_digest)
        record_worker_restaurant_digest="$(decode_record_value "$raw")"
        ;;
      web_digest)
        record_web_digest="$(decode_record_value "$raw")"
        ;;
      api_image)
        # Backward compatibility: monolithic API image record.
        raw="$(decode_record_value "$raw")"
        if [[ -z "$record_api_gateway_image" ]]; then record_api_gateway_image="$raw"; fi
        if [[ -z "$record_api_train_image" ]]; then record_api_train_image="$raw"; fi
        if [[ -z "$record_api_restaurant_image" ]]; then record_api_restaurant_image="$raw"; fi
        if [[ -z "$record_worker_train_image" ]]; then record_worker_train_image="$raw"; fi
        if [[ -z "$record_worker_restaurant_image" ]]; then record_worker_restaurant_image="$raw"; fi
        ;;
      api_digest)
        # Backward compatibility: monolithic API digest record.
        raw="$(decode_record_value "$raw")"
        if [[ -z "$record_api_gateway_digest" ]]; then record_api_gateway_digest="$raw"; fi
        if [[ -z "$record_api_train_digest" ]]; then record_api_train_digest="$raw"; fi
        if [[ -z "$record_api_restaurant_digest" ]]; then record_api_restaurant_digest="$raw"; fi
        if [[ -z "$record_worker_train_digest" ]]; then record_worker_train_digest="$raw"; fi
        if [[ -z "$record_worker_restaurant_digest" ]]; then record_worker_restaurant_digest="$raw"; fi
        ;;
      deployed_by)
        record_deployed_by="$(decode_record_value "$raw")"
        ;;
      previous)
        record_previous="$(decode_record_value "$raw")"
        ;;
      *)
        # Ignore unknown keys for forward compatibility.
        ;;
    esac
  done <"$path"

  [[ -n "$record_commit" ]] || return 1
  [[ -n "$record_timestamp" ]] || return 1
  return 0
}

# Get previous deployment for rollback
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

# Purge legacy/malformed deployment history records.
# This intentionally does NOT touch `current` / `previous` pointers.
purge_legacy_records() {
  local dry_run="${1:-false}"
  local backup="${2:-false}"

  log_info "Purging legacy/malformed deployment history records in: $DEPLOY_HISTORY_DIR"

  if [[ ! -d "$DEPLOY_HISTORY_DIR" ]]; then
    log_warn "History dir does not exist: $DEPLOY_HISTORY_DIR"
    return 0
  fi

  local -a to_delete=()

  # Only consider timestamped history record files.
  while IFS= read -r record; do
    [[ -z "$record" ]] && continue
    local path="$DEPLOY_HISTORY_DIR/$record"
    [[ -f "$path" ]] || continue

    # Legacy format: single-line short SHA, not sourceable and not useful for digests.
    local compact
    compact=$(tr -d '[:space:]' <"$path" 2>/dev/null || true)
    if [[ -n "$compact" && "$compact" =~ ^[0-9a-f]{7,40}$ ]] && ! grep -q '=' "$path" 2>/dev/null; then
      to_delete+=("$path")
      continue
    fi

    # Malformed: record cannot be parsed as key/value data.
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

# Rollback to previous deployment
do_rollback() {
  local prev_commit
  prev_commit=$(get_previous_version)
  log_warn "Rolling back to previous deployment: $prev_commit"
  
  # Load previous deployment metadata if available
  local prev_record
  prev_record=$( (ls -t "$DEPLOY_HISTORY_DIR" | grep -E '^[0-9]{8}_[0-9]{6}$' | while read -r record; do
    if grep -q "commit=$prev_commit" "$DEPLOY_HISTORY_DIR/$record" 2>/dev/null; then
      echo "$record"
      break
    fi
  done) || true)
  
  if [[ -n "$prev_record" ]]; then
    log_info "Found previous deployment record: $prev_record"
    if load_deployment_record "$DEPLOY_HISTORY_DIR/$prev_record"; then
      # Prefer immutable digests (deterministic rollback), fall back to commit tags.
      if [[ -n "${record_api_gateway_digest:-}" &&
            -n "${record_api_train_digest:-}" &&
            -n "${record_api_restaurant_digest:-}" &&
            -n "${record_worker_train_digest:-}" &&
            -n "${record_worker_restaurant_digest:-}" &&
            -n "${record_web_digest:-}" ]]; then
        export API_GATEWAY_IMAGE="$record_api_gateway_digest"
        export API_TRAIN_IMAGE="$record_api_train_digest"
        export API_RESTAURANT_IMAGE="$record_api_restaurant_digest"
        export WORKER_TRAIN_IMAGE="$record_worker_train_digest"
        export WORKER_RESTAURANT_IMAGE="$record_worker_restaurant_digest"
        export WEB_IMAGE="$record_web_digest"
        log_info "Using previous digests:"
        log_info "  API Gateway: $API_GATEWAY_IMAGE"
        log_info "  API Train: $API_TRAIN_IMAGE"
        log_info "  API Restaurant: $API_RESTAURANT_IMAGE"
        log_info "  Worker Train: $WORKER_TRAIN_IMAGE"
        log_info "  Worker Restaurant: $WORKER_RESTAURANT_IMAGE"
        log_info "  Web: $WEB_IMAGE"
      else
        log_warn "Deployment record missing digests; falling back to commit tag"
        require_gcp_project_id
        export API_GATEWAY_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/api-gateway:${prev_commit}"
        export API_TRAIN_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/api-train:${prev_commit}"
        export API_RESTAURANT_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/api-restaurant:${prev_commit}"
        export WORKER_TRAIN_IMAGE="$API_TRAIN_IMAGE"
        export WORKER_RESTAURANT_IMAGE="$API_RESTAURANT_IMAGE"
        export WEB_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/web:${prev_commit}"
      fi
    else
      log_warn "Failed to load deployment record; falling back to commit tag"
      require_gcp_project_id
      export API_GATEWAY_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/api-gateway:${prev_commit}"
      export API_TRAIN_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/api-train:${prev_commit}"
      export API_RESTAURANT_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/api-restaurant:${prev_commit}"
      export WORKER_TRAIN_IMAGE="$API_TRAIN_IMAGE"
      export WORKER_RESTAURANT_IMAGE="$API_RESTAURANT_IMAGE"
      export WEB_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/web:${prev_commit}"
    fi
  else
    log_warn "No detailed record found, using commit tag"
    require_gcp_project_id
    export API_GATEWAY_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/api-gateway:${prev_commit}"
    export API_TRAIN_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/api-train:${prev_commit}"
    export API_RESTAURANT_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/api-restaurant:${prev_commit}"
    export WORKER_TRAIN_IMAGE="$API_TRAIN_IMAGE"
    export WORKER_RESTAURANT_IMAGE="$API_RESTAURANT_IMAGE"
    export WEB_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/web:${prev_commit}"
  fi
  
  # Swap current and previous
  local current_commit
  current_commit=$(get_current_version)
  
  # Pull and deploy
  if ! pull_images; then
    return 1
  fi
  if ! deploy_services; then
    return 1
  fi
  
  # Update pointers (swap current and previous)
  echo "$prev_commit" > "$DEPLOY_HISTORY_DIR/current"
  echo "$current_commit" > "$DEPLOY_HISTORY_DIR/previous"
  
  log_ok "Rollback complete to $prev_commit"
}

# Configure Docker authentication for Artifact Registry
configure_docker_auth() {
  log_info "Configuring Docker authentication for Artifact Registry..."
  
  # Check if gcloud is available
  if ! command -v gcloud &> /dev/null; then
    log_error "gcloud CLI not found. Please install it:"
    log_error "  https://cloud.google.com/sdk/docs/install"
    exit 1
  fi
  
  # Configure Docker credential helper
  if ! gcloud auth configure-docker "${GCP_REGION}-docker.pkg.dev" --quiet 2>/dev/null; then
    log_warn "Could not configure docker authentication automatically"
    log_warn "You may need to run: gcloud auth login"
    log_warn "Or use a service account key"
  else
    log_ok "Docker authentication configured"
  fi
}

# Pull images from Artifact Registry
pull_images() {
  local image
  local -a images_to_pull=()
  local seen_images=$'\n'

  images_to_pull=(
    "$API_GATEWAY_IMAGE"
    "$API_TRAIN_IMAGE"
    "$API_RESTAURANT_IMAGE"
    "$WORKER_TRAIN_IMAGE"
    "$WORKER_RESTAURANT_IMAGE"
    "$WEB_IMAGE"
  )

  log_info "Pulling images from Artifact Registry..."
  log_info "  API Gateway: $API_GATEWAY_IMAGE"
  log_info "  API Train: $API_TRAIN_IMAGE"
  log_info "  API Restaurant: $API_RESTAURANT_IMAGE"
  log_info "  Worker Train: $WORKER_TRAIN_IMAGE"
  log_info "  Worker Restaurant: $WORKER_RESTAURANT_IMAGE"
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

# Deploy services with zero downtime
deploy_services() {
  log_info "Deploying with zero-downtime strategy..."
  
  # Export image URLs for docker-compose
  export API_GATEWAY_IMAGE
  export API_TRAIN_IMAGE
  export API_RESTAURANT_IMAGE
  export WORKER_TRAIN_IMAGE
  export WORKER_RESTAURANT_IMAGE
  export WEB_IMAGE
  export GCP_PROJECT_ID

  if stack_has_running_containers; then
    log_info "Running stack detected. Using rolling-update path."
    calculate_rollout_changes

    # Deploy database layer first (usually no changes).
    log_info "Ensuring database services are healthy..."
    if ! "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait postgres redis; then
      log_error "Failed to deploy database services"
      return 1
    fi

    local -a api_services_to_roll=()
    local -a worker_services_to_roll=()

    if [[ "$DEPLOY_API_GATEWAY_CHANGED" == "true" ]]; then
      api_services_to_roll+=("api-gateway")
    fi
    if [[ "$DEPLOY_API_TRAIN_CHANGED" == "true" ]]; then
      api_services_to_roll+=("api-train")
    fi
    if [[ "$DEPLOY_API_RESTAURANT_CHANGED" == "true" ]]; then
      api_services_to_roll+=("api-restaurant")
    fi
    if [[ "$DEPLOY_WORKER_TRAIN_CHANGED" == "true" ]]; then
      worker_services_to_roll+=("worker-train")
    fi
    if [[ "$DEPLOY_WORKER_RESTAURANT_CHANGED" == "true" ]]; then
      worker_services_to_roll+=("worker-restaurant")
    fi

    if [[ ${#api_services_to_roll[@]} -gt 0 ]]; then
      log_info "Deploying API services: ${api_services_to_roll[*]}"
      if ! "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait --no-deps "${api_services_to_roll[@]}"; then
        log_error "Failed to deploy API services (${api_services_to_roll[*]})"
        return 1
      fi
    else
      log_info "Skipping API service rollout (images unchanged)."
    fi

    if [[ ${#worker_services_to_roll[@]} -gt 0 ]]; then
      log_info "Deploying Worker services: ${worker_services_to_roll[*]}"
      if ! "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait --no-deps "${worker_services_to_roll[@]}"; then
        log_error "Failed to deploy worker services (${worker_services_to_roll[*]})"
        return 1
      fi
    else
      log_info "Skipping worker rollout (images unchanged)."
    fi

    if [[ "$DEPLOY_WEB_CHANGED" == "true" ]]; then
      # Deploy web (depends on API being healthy).
      log_info "Deploying Web service..."
      if ! "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait --no-deps web; then
        log_error "Failed to deploy web service"
        return 1
      fi
    else
      log_info "Skipping Web rollout (image unchanged)."
    fi

    # Reload Caddy (if Caddyfile changed, it auto-reloads).
    log_info "Ensuring Caddy is healthy..."
    if ! "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait caddy; then
      log_error "Failed to deploy Caddy service"
      return 1
    fi
    return 0
  fi

  log_warn "No running bominal containers detected. Using first-deploy bootstrap path."
  if ! "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait postgres redis api-gateway api-train api-restaurant worker-train worker-restaurant web caddy; then
    log_error "Failed to deploy services in bootstrap path"
    return 1
  fi
}

# Verify deployment health
verify_deployment() {
  log_info "Verifying deployment health..."
  
  local max_attempts="${SMOKE_MAX_ATTEMPTS}"
  local retry_delay="${SMOKE_RETRY_DELAY_SECONDS}"
  local attempt=1
  
  # Check API health
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
  
  # Check web via Caddy
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

# Show deployment status
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

# Cleanup old Docker resources
cleanup_docker() {
  log_info "Cleaning up old Docker resources..."
  
  # Remove dangling images
  docker image prune -f >/dev/null 2>&1 || true
  
  # Remove old bominal images (keep last 3)
  docker images --format "{{.Repository}}:{{.Tag}}" | grep "bominal" | tail -n +4 | xargs -r docker rmi -f 2>/dev/null || true
  
  log_ok "Docker cleanup complete"
}

# Main
main() {
  local target_commit="${1:-}"

  acquire_deploy_lock
  check_repo_state

  # Purge legacy/malformed history records (safe maintenance command).
  if [[ "${target_commit:-}" == "--purge-legacy-records" ]]; then
    shift || true
    local dry_run="false"
    local backup="false"
    while [[ $# -gt 0 ]]; do
      case "$1" in
        --dry-run)
          dry_run="true"
          ;;
        --backup)
          backup="true"
          ;;
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
  
  # Handle special commands
  case "${target_commit}" in
    --rollback|-r)
      run_preflight_checks
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
      echo "               Delete legacy/malformed historical record files (safe; does not touch current/previous)"
      echo ""
      echo "Environment variables:"
      echo "  GCP_PROJECT_ID   Google Cloud project ID (required)"
      echo "  API_IMAGE        Legacy override for all API/worker image URLs"
      echo "  API_GATEWAY_IMAGE Override API gateway image URL"
      echo "  API_TRAIN_IMAGE  Override API train image URL"
      echo "  API_RESTAURANT_IMAGE Override API restaurant image URL"
      echo "  WORKER_TRAIN_IMAGE Override worker-train image URL"
      echo "  WORKER_RESTAURANT_IMAGE Override worker-restaurant image URL"
      echo "  WEB_IMAGE        Override web image URL"
      echo "  DEPLOY_HISTORY_DIR Override deployment history dir (default: /opt/bominal/deployments)"
      echo "  DEPLOY_FAIL_ON_DIRTY_REPO=true  Block deploy when tracked repo changes are present"
      echo ""
      echo "Examples:"
      echo "  $0                           # Deploy latest"
      echo "  $0 abc123                    # Deploy commit abc123"
      echo "  $0 --rollback                # Rollback to previous"
      echo "  $0 --purge-legacy-records --dry-run"
      exit 0
      ;;
  esac
  
  run_preflight_checks

  # Configure Docker authentication
  configure_docker_auth

  require_gcp_project_id
  
  # Get current version for record
  local prev_version
  prev_version=$(get_current_version)
  
  # If commit specified, use specific image tags
  if [[ -n "$target_commit" ]]; then
    log_info "Deploying specific commit: $target_commit"
    export API_GATEWAY_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/api-gateway:${target_commit}"
    export API_TRAIN_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/api-train:${target_commit}"
    export API_RESTAURANT_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/api-restaurant:${target_commit}"
    export WORKER_TRAIN_IMAGE="${WORKER_TRAIN_IMAGE:-$API_TRAIN_IMAGE}"
    export WORKER_RESTAURANT_IMAGE="${WORKER_RESTAURANT_IMAGE:-$API_RESTAURANT_IMAGE}"
    export WEB_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/web:${target_commit}"
  else
    log_info "Deploying latest images"
    set_split_images_from_legacy_api_image
    export API_GATEWAY_IMAGE="${API_GATEWAY_IMAGE:-${REGISTRY}/${GCP_PROJECT_ID}/bominal/api-gateway:latest}"
    export API_TRAIN_IMAGE="${API_TRAIN_IMAGE:-${REGISTRY}/${GCP_PROJECT_ID}/bominal/api-train:latest}"
    export API_RESTAURANT_IMAGE="${API_RESTAURANT_IMAGE:-${REGISTRY}/${GCP_PROJECT_ID}/bominal/api-restaurant:latest}"
    export WORKER_TRAIN_IMAGE="${WORKER_TRAIN_IMAGE:-$API_TRAIN_IMAGE}"
    export WORKER_RESTAURANT_IMAGE="${WORKER_RESTAURANT_IMAGE:-$API_RESTAURANT_IMAGE}"
    export WEB_IMAGE="${WEB_IMAGE:-${REGISTRY}/${GCP_PROJECT_ID}/bominal/web:latest}"
  fi
  
  # Pull and deploy
  pull_images

  # Get the commit SHA from the images we're deploying.
  # On split-image rollouts, unchanged services may still carry older revisions,
  # so we prefer target_commit, otherwise use the single unique image revision
  # label when available, and fall back to repo HEAD if revisions differ.
  local deploy_commit
  if [[ -n "$target_commit" ]]; then
    deploy_commit="$target_commit"
  else
    local -a revisions=()
    local revision image
    local seen_revisions=$'\n'
    local repo_head=""

    for image in \
      "$API_GATEWAY_IMAGE" \
      "$API_TRAIN_IMAGE" \
      "$API_RESTAURANT_IMAGE" \
      "$WORKER_TRAIN_IMAGE" \
      "$WORKER_RESTAURANT_IMAGE" \
      "$WEB_IMAGE"; do
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
      log_warn "Detected mixed image revisions in split rollout; recording deploy commit as $deploy_commit"
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
  log_info "  API Gateway: $API_GATEWAY_IMAGE"
  log_info "  API Train: $API_TRAIN_IMAGE"
  log_info "  API Restaurant: $API_RESTAURANT_IMAGE"
  log_info "  Worker Train: $WORKER_TRAIN_IMAGE"
  log_info "  Worker Restaurant: $WORKER_RESTAURANT_IMAGE"
  log_info "  Web: $WEB_IMAGE"
  
  # Check if already deployed
  if [[ "$deploy_commit" == "$prev_version" ]]; then
    log_warn "Commit $deploy_commit is already deployed. Deploying anyway..."
  fi
  
  # Resolve immutable repo digests after pull (used for deterministic rollback)
  local api_gateway_digest api_train_digest api_restaurant_digest
  local worker_train_digest worker_restaurant_digest web_digest
  api_gateway_digest=$(docker inspect "$API_GATEWAY_IMAGE" --format='{{if .RepoDigests}}{{index .RepoDigests 0}}{{end}}' 2>/dev/null || true)
  api_train_digest=$(docker inspect "$API_TRAIN_IMAGE" --format='{{if .RepoDigests}}{{index .RepoDigests 0}}{{end}}' 2>/dev/null || true)
  api_restaurant_digest=$(docker inspect "$API_RESTAURANT_IMAGE" --format='{{if .RepoDigests}}{{index .RepoDigests 0}}{{end}}' 2>/dev/null || true)
  worker_train_digest=$(docker inspect "$WORKER_TRAIN_IMAGE" --format='{{if .RepoDigests}}{{index .RepoDigests 0}}{{end}}' 2>/dev/null || true)
  worker_restaurant_digest=$(docker inspect "$WORKER_RESTAURANT_IMAGE" --format='{{if .RepoDigests}}{{index .RepoDigests 0}}{{end}}' 2>/dev/null || true)
  web_digest=$(docker inspect "$WEB_IMAGE" --format='{{if .RepoDigests}}{{index .RepoDigests 0}}{{end}}' 2>/dev/null || true)

  if [[ "$api_gateway_digest" == "<no value>" ]]; then api_gateway_digest=""; fi
  if [[ "$api_train_digest" == "<no value>" ]]; then api_train_digest=""; fi
  if [[ "$api_restaurant_digest" == "<no value>" ]]; then api_restaurant_digest=""; fi
  if [[ "$worker_train_digest" == "<no value>" ]]; then worker_train_digest=""; fi
  if [[ "$worker_restaurant_digest" == "<no value>" ]]; then worker_restaurant_digest=""; fi
  if [[ "$web_digest" == "<no value>" ]]; then web_digest=""; fi

  if [[ -n "$api_gateway_digest" && "$api_gateway_digest" != *@sha256:* ]]; then api_gateway_digest=""; fi
  if [[ -n "$api_train_digest" && "$api_train_digest" != *@sha256:* ]]; then api_train_digest=""; fi
  if [[ -n "$api_restaurant_digest" && "$api_restaurant_digest" != *@sha256:* ]]; then api_restaurant_digest=""; fi
  if [[ -n "$worker_train_digest" && "$worker_train_digest" != *@sha256:* ]]; then worker_train_digest=""; fi
  if [[ -n "$worker_restaurant_digest" && "$worker_restaurant_digest" != *@sha256:* ]]; then worker_restaurant_digest=""; fi
  if [[ -n "$web_digest" && "$web_digest" != *@sha256:* ]]; then web_digest=""; fi

  if [[ -n "$api_gateway_digest" &&
        -n "$api_train_digest" &&
        -n "$api_restaurant_digest" &&
        -n "$worker_train_digest" &&
        -n "$worker_restaurant_digest" &&
        -n "$web_digest" ]]; then
    log_info "Resolved image digests for rollback:"
    log_info "  API Gateway: $api_gateway_digest"
    log_info "  API Train: $api_train_digest"
    log_info "  API Restaurant: $api_restaurant_digest"
    log_info "  Worker Train: $worker_train_digest"
    log_info "  Worker Restaurant: $worker_restaurant_digest"
    log_info "  Web: $web_digest"
  else
    log_warn "Could not resolve one or more repo digests; rollback may fall back to commit tags"
  fi

  deploy_services

  # Verify
  if verify_deployment; then
    save_deployment \
      "$deploy_commit" \
      "$api_gateway_digest" \
      "$api_train_digest" \
      "$api_restaurant_digest" \
      "$worker_train_digest" \
      "$worker_restaurant_digest" \
      "$web_digest"
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
