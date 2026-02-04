#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# local-setup.sh — One-time local development environment setup
# -----------------------------------------------------------------------------
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== bominal local setup ==="

# Check prerequisites
command -v docker >/dev/null 2>&1 || { echo "Error: docker is required"; exit 1; }
command -v docker-compose >/dev/null 2>&1 || command -v "docker compose" >/dev/null 2>&1 || { echo "Error: docker-compose is required"; exit 1; }

cd "$REPO_ROOT"

# Initialize git submodules (third_party/srtgo)
echo "→ Initializing git submodules..."
git submodule update --init --recursive

# Create local env files from examples if they don't exist
echo "→ Setting up environment files..."
ENV_DEV_DIR="$REPO_ROOT/infra/env/dev"

for example_file in "$ENV_DEV_DIR"/*.env.example; do
    if [[ -f "$example_file" ]]; then
        target_file="${example_file%.example}"
        if [[ ! -f "$target_file" ]]; then
            cp "$example_file" "$target_file"
            echo "  Created: $(basename "$target_file")"
        else
            echo "  Exists: $(basename "$target_file")"
        fi
    fi
done

# Build Docker images
echo "→ Building Docker images..."
docker-compose -f infra/docker-compose.yml build

echo ""
echo "=== Setup complete ==="
echo ""
echo "Next steps:"
echo "  1. Edit infra/env/dev/*.env files with your local config"
echo "  2. Run: ./infra/scripts/local-run.sh"
echo ""
