#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# local-run.sh — Start local development environment
# -----------------------------------------------------------------------------
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$REPO_ROOT"

echo "=== Starting bominal development environment ==="

# Detect docker compose command (v2 plugin preferred)
if docker compose version >/dev/null 2>&1; then
    COMPOSE_CMD=(docker compose)
elif command -v docker-compose >/dev/null 2>&1; then
    COMPOSE_CMD=(docker-compose)
else
    echo "Error: docker compose (v2) or docker-compose (v1) is required"
    exit 1
fi

# Check if setup has been run
if [[ ! -f "infra/env/dev/api.env" ]]; then
    echo "Error: Environment files not found. Run ./infra/scripts/local-setup.sh first."
    exit 1
fi

# Start services
echo "→ Starting Docker Compose services..."
"${COMPOSE_CMD[@]}" -f infra/docker-compose.yml up --build "$@"
