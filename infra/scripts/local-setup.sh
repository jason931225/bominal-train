#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# local-setup.sh — One-time local development environment setup
# -----------------------------------------------------------------------------
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
source "$SCRIPT_DIR/lib/env_utils.sh"

echo "=== bominal local setup ==="

# Check prerequisites
command -v docker >/dev/null 2>&1 || { log_error "docker is required"; exit 1; }
detect_compose_cmd

cd "$REPO_ROOT"

# Initialize git submodules (third_party/srtgo)
echo "→ Initializing git submodules..."
git submodule update --init --recursive

# Create local env files from examples if they don't exist
echo "→ Setting up environment files..."
ENV_DEV_DIR="$REPO_ROOT/infra/env/dev"

if ! copy_env_from_examples "$ENV_DEV_DIR"; then
    echo "  No new env files created from examples."
fi

required_dev_files=(
  "$ENV_DEV_DIR/api.env"
  "$ENV_DEV_DIR/web.env"
  "$ENV_DEV_DIR/postgres.env"
)

for file in "${required_dev_files[@]}"; do
    require_nonempty_file "$file"
done

# Build Docker images
echo "→ Building Docker images..."
"${COMPOSE_CMD[@]}" -f infra/docker-compose.yml build

echo ""
echo "=== Setup complete ==="
echo ""
echo "Next steps:"
echo "  1. Edit infra/env/dev/*.env files with your local config"
echo "  2. Run: ./infra/scripts/local-run.sh"
echo "  3. Open Mailpit inbox at http://localhost:8025 for email testing"
echo ""
