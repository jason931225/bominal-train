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
#   GCP_PROJECT_ID   - Google Cloud project ID (required)
#   API_IMAGE        - Override API image URL
#   WEB_IMAGE        - Override web image URL
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
  local api_digest="${2:-}"
  local web_digest="${3:-}"
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
    printf 'api_image=%q\n' "$API_IMAGE"
    printf 'web_image=%q\n' "$WEB_IMAGE"
    printf 'api_digest=%q\n' "$api_digest"
    printf 'web_digest=%q\n' "$web_digest"
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

    # Malformed: would fail the script's `source` usage (syntax errors / unbound vars / etc).
    if ! bash -c 'set -euo pipefail; commit=""; timestamp=""; api_digest=""; web_digest=""; deployed_by=""; previous=""; api_image=""; web_image=""; source "$1"' bash "$path" >/dev/null 2>&1; then
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
    api_digest=""
    web_digest=""
    api_image=""
    web_image=""
    if source "$DEPLOY_HISTORY_DIR/$prev_record" >/dev/null 2>&1; then
      # Prefer immutable digests (deterministic rollback), fall back to commit tags.
      if [[ -n "${api_digest:-}" && -n "${web_digest:-}" ]]; then
        export API_IMAGE="$api_digest"
        export WEB_IMAGE="$web_digest"
        log_info "Using previous digests:"
        log_info "  API: $API_IMAGE"
        log_info "  Web: $WEB_IMAGE"
      else
        log_warn "Deployment record missing digests; falling back to commit tag"
        require_gcp_project_id
        export API_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/api:${prev_commit}"
        export WEB_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/web:${prev_commit}"
      fi
    else
      log_warn "Failed to load deployment record; falling back to commit tag"
      require_gcp_project_id
      export API_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/api:${prev_commit}"
      export WEB_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/web:${prev_commit}"
    fi
  else
    log_warn "No detailed record found, using commit tag"
    require_gcp_project_id
    export API_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/api:${prev_commit}"
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
  log_info "Pulling images from Artifact Registry..."
  log_info "  API: $API_IMAGE"
  log_info "  Web: $WEB_IMAGE"
  
  # Pull images
  if ! docker pull "$API_IMAGE"; then
    log_error "Failed to pull API image: $API_IMAGE"
    log_error "Make sure the image exists and you have access"
    return 1
  fi
  
  if ! docker pull "$WEB_IMAGE"; then
    log_error "Failed to pull web image: $WEB_IMAGE"
    log_error "Make sure the image exists and you have access"
    return 1
  fi
  
  log_ok "Images pulled successfully"
}

# Deploy services with zero downtime
deploy_services() {
  log_info "Deploying with zero-downtime strategy..."
  
  # Export image URLs for docker-compose
  export API_IMAGE
  export WEB_IMAGE
  export GCP_PROJECT_ID

  if stack_has_running_containers; then
    log_info "Running stack detected. Using rolling-update path."

    # Deploy database layer first (usually no changes).
    log_info "Ensuring database services are healthy..."
    if ! "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait postgres redis; then
      log_error "Failed to deploy database services"
      return 1
    fi

    # Deploy API (backend must be ready before web).
    log_info "Deploying API service..."
    if ! "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait --no-deps api; then
      log_error "Failed to deploy API service"
      return 1
    fi

    # Deploy workers (can run alongside API).
    log_info "Deploying Worker services..."
    if ! "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait --no-deps worker worker-restaurant; then
      log_error "Failed to deploy worker services"
      return 1
    fi

    # Deploy web (depends on API being healthy).
    log_info "Deploying Web service..."
    if ! "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait --no-deps web; then
      log_error "Failed to deploy web service"
      return 1
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
  if ! "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait postgres redis api worker worker-restaurant web caddy; then
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
      commit=""
      timestamp=""
      api_digest=""
      web_digest=""
      deployed_by=""
      if source "$DEPLOY_HISTORY_DIR/$record" >/dev/null 2>&1; then
        echo "  - $timestamp: $commit (by ${deployed_by:-unknown})"
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
      echo "  API_IMAGE        Override API image URL"
      echo "  WEB_IMAGE        Override web image URL"
      echo "  DEPLOY_HISTORY_DIR Override deployment history dir (default: /opt/bominal/deployments)"
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
    export API_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/api:${target_commit}"
    export WEB_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/web:${target_commit}"
  else
    log_info "Deploying latest images"
    export API_IMAGE="${API_IMAGE:-${REGISTRY}/${GCP_PROJECT_ID}/bominal/api:latest}"
    export WEB_IMAGE="${WEB_IMAGE:-${REGISTRY}/${GCP_PROJECT_ID}/bominal/web:latest}"
  fi
  
  # Get the commit SHA from the image we're deploying
  local deploy_commit
  if [[ -n "$target_commit" ]]; then
    deploy_commit="$target_commit"
  else
    # Pull to inspect the image
    docker pull "$API_IMAGE" >/dev/null 2>&1 || true
    deploy_commit=$(docker inspect "$API_IMAGE" --format='{{index .Config.Labels "org.opencontainers.image.revision"}}' 2>/dev/null || true)
    if [[ -z "$deploy_commit" || "$deploy_commit" == "<no value>" ]]; then
      log_error "Could not determine image revision from label: org.opencontainers.image.revision"
      log_error "This would corrupt /opt/bominal/deployments/* tracking."
      log_error "Fix: ensure CI sets this label, or deploy a specific commit tag: $0 <commit>"
      exit 1
    fi
  fi

  log_info "Deploying commit: $deploy_commit"
  log_info "  API: $API_IMAGE"
  log_info "  Web: $WEB_IMAGE"
  
  # Check if already deployed
  if [[ "$deploy_commit" == "$prev_version" ]]; then
    log_warn "Commit $deploy_commit is already deployed. Deploying anyway..."
  fi
  
  # Pull and deploy
  pull_images

  # Resolve immutable repo digests after pull (used for deterministic rollback)
  local api_digest web_digest
  api_digest=$(docker inspect "$API_IMAGE" --format='{{if .RepoDigests}}{{index .RepoDigests 0}}{{end}}' 2>/dev/null || true)
  web_digest=$(docker inspect "$WEB_IMAGE" --format='{{if .RepoDigests}}{{index .RepoDigests 0}}{{end}}' 2>/dev/null || true)

  if [[ "$api_digest" == "<no value>" ]]; then api_digest=""; fi
  if [[ "$web_digest" == "<no value>" ]]; then web_digest=""; fi

  if [[ -n "$api_digest" && "$api_digest" != *@sha256:* ]]; then api_digest=""; fi
  if [[ -n "$web_digest" && "$web_digest" != *@sha256:* ]]; then web_digest=""; fi

  if [[ -n "$api_digest" && -n "$web_digest" ]]; then
    log_info "Resolved image digests for rollback:"
    log_info "  API: $api_digest"
    log_info "  Web: $web_digest"
  else
    log_warn "Could not resolve one or both repo digests; rollback may fall back to commit tags"
  fi

  deploy_services

  # Verify
  if verify_deployment; then
    save_deployment "$deploy_commit" "$api_digest" "$web_digest"
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
