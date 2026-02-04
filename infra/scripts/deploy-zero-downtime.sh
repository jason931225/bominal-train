#!/usr/bin/env bash
# ==============================================================================
# Zero-Downtime Deployment Script for bominal
# ==============================================================================
# This script performs a zero-downtime deployment using Docker Compose health
# checks and the --wait flag. New containers must be healthy before old ones
# are removed.
#
# Usage:
#   ./deploy-zero-downtime.sh              # Deploy current HEAD
#   ./deploy-zero-downtime.sh <commit>     # Deploy specific commit
#   ./deploy-zero-downtime.sh --rollback   # Rollback to previous deployment
#   ./deploy-zero-downtime.sh --skip-build # Deploy without rebuilding images
#   ./deploy-zero-downtime.sh --status     # Show deployment status
#
# For quick restarts after VM reboot (no rebuild), use:
#   ./quick-restart.sh
#
# The script maintains version history in /opt/bominal/deployments/ for
# rollback capability.
# ==============================================================================
set -euo pipefail

# Configuration
REPO_DIR="${REPO_DIR:-/opt/bominal/repo}"
COMPOSE_FILE="infra/docker-compose.prod.yml"
DEPLOY_HISTORY_DIR="/opt/bominal/deployments"
MAX_HISTORY=10  # Keep last N deployment records
SKIP_BUILD=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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
  
  # Save to history
  echo "$commit" > "$DEPLOY_HISTORY_DIR/$timestamp"
  echo "$commit" > "$DEPLOY_HISTORY_DIR/current"
  
  # Update previous pointer
  if [[ -f "$DEPLOY_HISTORY_DIR/current" ]]; then
    local prev_commit
    prev_commit=$(get_current_version)
    if [[ "$prev_commit" != "$commit" && "$prev_commit" != "unknown" ]]; then
      echo "$prev_commit" > "$DEPLOY_HISTORY_DIR/previous"
    fi
  fi
  
  # Cleanup old history (keep last N)
  cd "$DEPLOY_HISTORY_DIR"
  # shellcheck disable=SC2012
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
  
  # Swap current and previous
  local current_commit
  current_commit=$(get_current_version)
  
  # Checkout previous commit
  git fetch origin
  git checkout "$prev_commit"
  
  # Deploy
  deploy_services
  
  # Update pointers (swap current and previous)
  echo "$prev_commit" > "$DEPLOY_HISTORY_DIR/current"
  echo "$current_commit" > "$DEPLOY_HISTORY_DIR/previous"
  
  log_ok "Rollback complete to $prev_commit"
}

# Build images (this doesn't cause downtime)
build_images() {
  log_info "Building new images (no downtime)..."
  "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" build --parallel api worker web
}

# Deploy services with zero downtime
deploy_services() {
  log_info "Deploying with zero-downtime strategy..."
  
  # The --wait flag ensures Docker waits for containers to be healthy
  # before considering the deployment complete. Combined with health checks,
  # this means the old container stays running until the new one is ready.
  
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
  docker ps --format "table {{.Names}}\t{{.Status}}" | grep bominal || true
  echo ""
  log_info "Current version: $(get_current_version)"
  if [[ -f "$DEPLOY_HISTORY_DIR/previous" ]]; then
    log_info "Previous version: $(cat "$DEPLOY_HISTORY_DIR/previous")"
  fi
}

# Main
main() {
  local target_commit="${1:-}"
  
  # Handle special commands
  case "${target_commit}" in
    --rollback|-r)
      do_rollback
      verify_deployment
      show_status
      exit 0
      ;;
    --status|-s)
      show_status
      exit 0
      ;;
    --skip-build|-S)
      SKIP_BUILD=true
      shift || true
      target_commit="${1:-}"
      ;;
    --help|-h)
      echo "Usage: $0 [options] [commit]"
      echo ""
      echo "Options:"
      echo "  <commit>       Deploy specific commit (default: current HEAD)"
      echo "  --rollback     Rollback to previous deployment"
      echo "  --skip-build   Deploy without rebuilding images (faster)"
      echo "  --status       Show current deployment status"
      echo ""
      echo "For quick restarts after VM reboot, use: ./quick-restart.sh"
      exit 0
      ;;
  esac
  
  # Get current version for record
  local prev_version
  prev_version=$(get_current_version)
  
  # If commit specified, checkout that commit
  if [[ -n "$target_commit" ]]; then
    log_info "Fetching and checking out $target_commit..."
    git fetch origin
    git checkout "$target_commit"
  else
    log_info "Pulling latest changes..."
    git pull origin main
  fi
  
  # Get the commit we're deploying
  local deploy_commit
  deploy_commit=$(git rev-parse --short HEAD)
  log_info "Deploying commit: $deploy_commit"
  
  # Check if already deployed
  if [[ "$deploy_commit" == "$prev_version" ]]; then
    log_warn "Commit $deploy_commit is already deployed. Rebuilding anyway..."
  fi
  
  # Save previous version for rollback
  if [[ "$prev_version" != "unknown" && "$prev_version" != "$deploy_commit" ]]; then
    echo "$prev_version" > "$DEPLOY_HISTORY_DIR/previous"
  fi
  
  # Build and deploy
  if [[ "$SKIP_BUILD" == "true" ]]; then
    log_info "Skipping build (--skip-build specified)"
  else
    build_images
  fi
  deploy_services
  
  # Verify
  if verify_deployment; then
    save_deployment "$deploy_commit"
    log_ok "Deployment of $deploy_commit complete!"
    show_status
  else
    log_error "Deployment verification failed!"
    log_warn "Consider rolling back with: $0 --rollback"
    exit 1
  fi
}

main "$@"
