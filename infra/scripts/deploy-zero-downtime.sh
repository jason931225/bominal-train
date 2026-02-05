#!/usr/bin/env bash
# ==============================================================================
# Zero-Downtime Deployment Script for bominal (CI/CD Version)
# ==============================================================================
# This script pulls pre-built images from Google Artifact Registry and deploys
# them with zero downtime. Images are built on GitHub Actions runners instead
# of the e2-micro VM to prevent OOM issues.
#
# Usage:
#   ./deploy-zero-downtime.sh              # Deploy latest images
#   ./deploy-zero-downtime.sh <commit>     # Deploy specific commit SHA
#   ./deploy-zero-downtime.sh --rollback   # Rollback to previous deployment
#   ./deploy-zero-downtime.sh --status     # Show deployment status
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
DEPLOY_HISTORY_DIR="/opt/bominal/deployments"
MAX_HISTORY=10

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

# Load GCP_PROJECT_ID from env file if not set
if [[ -z "${GCP_PROJECT_ID:-}" ]]; then
  if [[ -f "infra/env/prod/api.env" ]]; then
    # Try to extract from env file
    GCP_PROJECT_ID=$(grep -E '^GCP_PROJECT_ID=' infra/env/prod/api.env | cut -d'=' -f2- | tr -d '"' || echo "")
  fi
  
  if [[ -z "${GCP_PROJECT_ID:-}" ]]; then
    log_error "GCP_PROJECT_ID not set. Please set it in your environment or infra/env/prod/api.env"
    exit 1
  fi
fi

# Set default image URLs
export API_IMAGE="${API_IMAGE:-${REGISTRY}/${GCP_PROJECT_ID}/bominal/api:latest}"
export WEB_IMAGE="${WEB_IMAGE:-${REGISTRY}/${GCP_PROJECT_ID}/bominal/web:latest}"
export GCP_PROJECT_ID

# Get current deployed version
get_current_version() {
  if [[ -f "$DEPLOY_HISTORY_DIR/current" ]]; then
    cat "$DEPLOY_HISTORY_DIR/current"
  else
    echo "unknown"
  fi
}

# Save deployment record
save_deployment() {
  local commit="$1"
  local timestamp
  timestamp=$(date -u +"%Y%m%d_%H%M%S")
  
  # Get previous version before updating
  local prev_commit
  prev_commit=$(get_current_version)
  
  # Save to history with metadata
  {
    echo "commit=$commit"
    echo "timestamp=$timestamp"
    echo "api_image=$API_IMAGE"
    echo "web_image=$WEB_IMAGE"
    echo "deployed_by=${USER:-unknown}"
    echo "previous=$prev_commit"
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
    cat "$DEPLOY_HISTORY_DIR/previous"
  else
    log_error "No previous deployment found for rollback"
    exit 1
  fi
}

# Rollback to previous deployment
do_rollback() {
  local prev_commit
  prev_commit=$(get_previous_version)
  log_warn "Rolling back to previous deployment: $prev_commit"
  
  # Load previous deployment metadata if available
  local prev_record
  prev_record=$(ls -t "$DEPLOY_HISTORY_DIR" | grep -E '^[0-9]{8}_[0-9]{6}$' | while read -r record; do
    if grep -q "commit=$prev_commit" "$DEPLOY_HISTORY_DIR/$record" 2>/dev/null; then
      echo "$record"
      break
    fi
  done)
  
  if [[ -n "$prev_record" ]]; then
    log_info "Found previous deployment record: $prev_record"
    source "$DEPLOY_HISTORY_DIR/$prev_record"
    
    # Override image URLs from previous deployment
    export API_IMAGE="$api_image"
    export WEB_IMAGE="$web_image"
    log_info "Using previous images:"
    log_info "  API: $API_IMAGE"
    log_info "  Web: $WEB_IMAGE"
  else
    log_warn "No detailed record found, using commit tag"
    export API_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/api:${prev_commit}"
    export WEB_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/web:${prev_commit}"
  fi
  
  # Swap current and previous
  local current_commit
  current_commit=$(get_current_version)
  
  # Pull and deploy
  pull_images
  deploy_services
  
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
    exit 1
  fi
  
  if ! docker pull "$WEB_IMAGE"; then
    log_error "Failed to pull web image: $WEB_IMAGE"
    log_error "Make sure the image exists and you have access"
    exit 1
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
  
  # Deploy database layer first (usually no changes)
  log_info "Ensuring database services are healthy..."
  "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait postgres redis
  
  # Deploy API (backend must be ready before web)
  log_info "Deploying API service..."
  "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait --no-deps api
  
  # Deploy worker (can run alongside API)
  log_info "Deploying Worker service..."
  "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait --no-deps worker
  
  # Deploy web (depends on API being healthy)
  log_info "Deploying Web service..."
  "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait --no-deps web
  
  # Reload Caddy (if Caddyfile changed, it auto-reloads)
  log_info "Ensuring Caddy is healthy..."
  "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait caddy
}

# Verify deployment health
verify_deployment() {
  log_info "Verifying deployment health..."
  
  local max_attempts=30
  local attempt=1
  
  # Check API health
  while [[ $attempt -le $max_attempts ]]; do
    if curl -fsS --max-time 5 http://127.0.0.1:8000/health >/dev/null 2>&1; then
      log_ok "API health check passed"
      break
    fi
    log_warn "Waiting for API... (attempt $attempt/$max_attempts)"
    sleep 2
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
    sleep 2
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
  ls -t "$DEPLOY_HISTORY_DIR" | grep -E '^[0-9]{8}_[0-9]{6}$' | head -5 | while read -r record; do
    if [[ -f "$DEPLOY_HISTORY_DIR/$record" ]]; then
      source "$DEPLOY_HISTORY_DIR/$record"
      echo "  - $timestamp: $commit (by ${deployed_by:-unknown})"
    fi
  done
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
  
  # Handle special commands
  case "${target_commit}" in
    --rollback|-r)
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
      echo ""
      echo "Environment variables:"
      echo "  GCP_PROJECT_ID   Google Cloud project ID (required)"
      echo "  API_IMAGE        Override API image URL"
      echo "  WEB_IMAGE        Override web image URL"
      echo ""
      echo "Examples:"
      echo "  $0                           # Deploy latest"
      echo "  $0 abc123                    # Deploy commit abc123"
      echo "  $0 --rollback                # Rollback to previous"
      exit 0
      ;;
  esac
  
  # Configure Docker authentication
  configure_docker_auth
  
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
    export API_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/api:latest"
    export WEB_IMAGE="${REGISTRY}/${GCP_PROJECT_ID}/bominal/web:latest"
  fi
  
  # Get the commit SHA from the image we're deploying
  local deploy_commit
  if [[ -n "$target_commit" ]]; then
    deploy_commit="$target_commit"
  else
    # Pull to inspect the image
    docker pull "$API_IMAGE" >/dev/null 2>&1 || true
    deploy_commit=$(docker inspect "$API_IMAGE" --format='{{.Config.Labels.org.opencontainers.image.revision}}' 2>/dev/null || echo "latest")
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
  deploy_services
  
  # Verify
  if verify_deployment; then
    save_deployment "$deploy_commit"
    cleanup_docker
    log_ok "Deployment of $deploy_commit complete!"
    show_status
  else
    log_error "Deployment verification failed!"
    log_warn "Consider rolling back with: $0 --rollback"
    exit 1
  fi
}

main "$@"
