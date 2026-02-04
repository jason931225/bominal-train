#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# local-run.sh — Start local development environment
# -----------------------------------------------------------------------------
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$REPO_ROOT"

echo "=== Starting bominal development environment ==="

# Check if setup has been run
if [[ ! -f "infra/env/dev/api.env" ]]; then
    echo "Error: Environment files not found. Run ./infra/scripts/local-setup.sh first."
    exit 1
fi

# Start services
echo "→ Starting Docker Compose services..."
docker-compose -f infra/docker-compose.yml up --build "$@"
