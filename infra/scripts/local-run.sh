#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# local-run.sh — Start local development environment
# -----------------------------------------------------------------------------
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
source "$SCRIPT_DIR/lib/env_utils.sh"

cd "$REPO_ROOT"

echo "=== Starting bominal development environment ==="

detect_compose_cmd

APP_VERSION="$(resolve_dev_app_version "$REPO_ROOT")"
BUILD_VERSION="$(resolve_dev_build_version "$REPO_ROOT")"
export APP_VERSION BUILD_VERSION
echo "→ Using APP_VERSION=$APP_VERSION BUILD_VERSION=$BUILD_VERSION"

# Check if setup has been run
required_dev_files=(
  "infra/env/dev/api.env"
  "infra/env/dev/web.env"
  "infra/env/dev/postgres.env"
)
for file in "${required_dev_files[@]}"; do
  if ! require_nonempty_file "$file"; then
    echo "Error: environment setup incomplete. Run ./infra/scripts/local-setup.sh first."
    exit 1
  fi
done

# Start services
echo "→ Starting Docker Compose services..."
"${COMPOSE_CMD[@]}" -f infra/docker-compose.yml up --build "$@"

echo ""
echo "Mailpit dev inbox: http://localhost:8025"
echo "Mailpit SMTP endpoint: localhost:1025"
