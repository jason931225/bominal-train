#!/usr/bin/env bash
# ==============================================================================
# Quick Restart Script for bominal
# ==============================================================================
# Use this script after a GCE VM reset or when containers need to be restarted
# WITHOUT rebuilding images. This is much faster than a full deploy.
#
# Usage:
#   ./quick-restart.sh           # Restart all containers
#   ./quick-restart.sh api # Restart specific service
#   ./quick-restart.sh --status  # Show container status only
#
# When to use:
#   - After GCE VM abrupt reset/reboot
#   - When containers crash but code hasn't changed
#   - To apply env file changes without rebuild
#
# When NOT to use (use deploy.sh instead):
#   - When code has changed (need to rebuild images)
#   - For production deployments
# ==============================================================================
set -euo pipefail

# Configuration
REPO_DIR="${REPO_DIR:-/opt/bominal/repo}"
COMPOSE_FILE="infra/docker-compose.prod.yml"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/lib/env_utils.sh"

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

detect_compose_cmd

# Show container status
show_status() {
  echo ""
  log_info "Container status:"
  "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" ps
  echo ""
  log_info "Container health:"
  docker ps --format "table {{.Names}}\t{{.Status}}" | grep bominal || true
}

# Wait for container to be healthy
wait_healthy() {
  local service="$1"
  local max_attempts=30
  local attempt=1
  
  while [[ $attempt -le $max_attempts ]]; do
    local status
    status=$("${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" ps "$service" --format "{{.Health}}" 2>/dev/null || echo "unknown")
    
    if [[ "$status" == "healthy" ]]; then
      return 0
    fi
    
    sleep 2
    ((attempt++))
  done
  
  return 1
}

# Restart all services in order
restart_all() {
  log_info "Starting containers (using existing images)..."
  
  # Start database layer first
  log_info "Starting database services..."
  "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait postgres redis
  
  # Start API + worker services
  log_info "Starting API and worker services..."
  "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait --no-deps api worker
  
  # Start web
  log_info "Starting Web service..."
  "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait --no-deps web
  
  # Start Caddy
  log_info "Starting Caddy..."
  "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --wait caddy
}

# Restart specific service
restart_service() {
  local service="$1"
  log_info "Restarting $service..."
  "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" restart "$service"
  
  log_info "Waiting for $service to be healthy..."
  if wait_healthy "$service"; then
    log_ok "$service is healthy"
  else
    log_warn "$service health check timed out (may still be starting)"
  fi
}

# Verify health
verify_health() {
  log_info "Verifying system health..."
  
  # Check API
  if curl -fsS --max-time 5 http://127.0.0.1:8000/health >/dev/null 2>&1; then
    log_ok "API health check passed"
  else
    log_error "API health check failed"
    return 1
  fi
  
  # Check web via Caddy
  if curl -fsS --max-time 5 -I http://127.0.0.1/ >/dev/null 2>&1; then
    log_ok "Web health check passed"
  else
    log_error "Web health check failed"
    return 1
  fi
  
  log_ok "All services healthy!"
}

# Main
main() {
  local target="${1:-}"
  
  case "${target}" in
    --status|-s)
      show_status
      exit 0
      ;;
    --help|-h)
      echo "Usage: $0 [service|--status]"
      echo ""
      echo "Commands:"
      echo "  (no args)    Restart all containers in order"
      echo "  <service>    Restart specific service (api, worker, web, caddy, etc.)"
      echo "  --status     Show current container status"
      echo ""
      echo "Examples:"
      echo "  $0           # Restart everything after VM reboot"
      echo "  $0 api       # Restart API container"
      echo "  $0 worker    # Restart worker container"
      exit 0
      ;;
    "")
      # No args - restart all
      restart_all
      verify_health
      show_status
      ;;
    *)
      # Specific service
      restart_service "$target"
      show_status
      ;;
  esac
}

main "$@"
