#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# local-run.sh — Start local development environment
# -----------------------------------------------------------------------------
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$REPO_ROOT"

echo "=== Starting bominal development environment ==="

DC=()
if docker compose version >/dev/null 2>&1; then
    DC=(docker compose)
elif command -v docker-compose >/dev/null 2>&1; then
    DC=(docker-compose)
else
    echo "Error: docker compose (v2) or docker-compose (v1) is required"
    exit 1
fi

PROJECT_NAME="bominal"
existing_project="$(docker inspect --format '{{ index .Config.Labels "com.docker.compose.project" }}' bominal-postgres 2>/dev/null || true)"
if [[ -n "$existing_project" && "$existing_project" != "<no value>" ]]; then
    PROJECT_NAME="$existing_project"
fi

# Check if setup has been run
if [[ ! -f "infra/env/dev/api.env" ]]; then
    echo "Error: Environment files not found. Run ./infra/scripts/local-setup.sh first."
    exit 1
fi

# Start services
echo "→ Starting Docker Compose services..."
"${DC[@]}" -p "$PROJECT_NAME" -f infra/docker-compose.yml up --build "$@"
