#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# local-cleanup.sh — Cleanup local Docker artifacts for bominal dev workflows
# -----------------------------------------------------------------------------
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
source "$SCRIPT_DIR/lib/env_utils.sh"

cleanup_dev=true
cleanup_prod=true
aggressive=false
prune_volumes=false

usage() {
  cat <<'USAGE'
Usage: ./infra/scripts/local-cleanup.sh [options]

Options:
  --dev-only      cleanup only dev compose stack
  --prod-only     cleanup only prod compose stack
  --aggressive    use full image/build-cache prune (-a)
  --volumes       also remove compose volumes and run docker volume prune
  -h, --help      show this help
USAGE
}

for arg in "$@"; do
  case "$arg" in
    --dev-only)
      cleanup_dev=true
      cleanup_prod=false
      ;;
    --prod-only)
      cleanup_dev=false
      cleanup_prod=true
      ;;
    --aggressive)
      aggressive=true
      ;;
    --volumes)
      prune_volumes=true
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Error: unknown argument '$arg'"
      usage
      exit 1
      ;;
  esac
done

cd "$REPO_ROOT"

command -v docker >/dev/null 2>&1 || { echo "Error: docker is required"; exit 1; }
detect_compose_cmd

echo "=== bominal local cleanup ==="

down_stack() {
  local compose_file="$1"
  local label="$2"
  if [[ "$prune_volumes" == "true" ]]; then
    echo "→ Stopping ${label} stack (${compose_file}) with volumes..."
    "${COMPOSE_CMD[@]}" -f "$compose_file" down --remove-orphans -v >/dev/null 2>&1 || true
  else
    echo "→ Stopping ${label} stack (${compose_file})..."
    "${COMPOSE_CMD[@]}" -f "$compose_file" down --remove-orphans >/dev/null 2>&1 || true
  fi
}

if [[ "$cleanup_dev" == "true" ]]; then
  down_stack "infra/docker-compose.yml" "dev"
fi
if [[ "$cleanup_prod" == "true" ]]; then
  down_stack "infra/docker-compose.prod.yml" "prod"
fi

echo "→ Pruning stopped containers..."
docker container prune -f >/dev/null 2>&1 || true

echo "→ Pruning unused networks..."
docker network prune -f >/dev/null 2>&1 || true

if [[ "$aggressive" == "true" ]]; then
  echo "→ Pruning unused images (aggressive)..."
  docker image prune -a -f >/dev/null 2>&1 || true
  echo "→ Pruning build cache (aggressive)..."
  docker builder prune -a -f >/dev/null 2>&1 || true
else
  echo "→ Pruning dangling images..."
  docker image prune -f >/dev/null 2>&1 || true
  echo "→ Pruning dangling build cache..."
  docker builder prune -f >/dev/null 2>&1 || true
fi

if [[ "$prune_volumes" == "true" ]]; then
  echo "→ Pruning dangling volumes..."
  docker volume prune -f >/dev/null 2>&1 || true
fi

echo "→ Docker disk usage snapshot:"
docker system df || true

echo ""
echo "=== Local cleanup complete ==="
