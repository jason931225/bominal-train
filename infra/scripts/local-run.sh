#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# local-run.sh — Start local development environment
# -----------------------------------------------------------------------------
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
source "$SCRIPT_DIR/lib/env_utils.sh"

cd "$REPO_ROOT"

is_truthy_bool() {
  local value
  value="$(printf '%s' "$1" | tr '[:upper:]' '[:lower:]')"
  case "$value" in
    true|1|yes)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

is_falsey_bool() {
  local value
  value="$(printf '%s' "$1" | tr '[:upper:]' '[:lower:]')"
  case "$value" in
    false|0|no)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

compose_args=()
detach_mode=false
down_on_exit_override=""

for arg in "$@"; do
  case "$arg" in
    -d|--detach)
      detach_mode=true
      compose_args+=("$arg")
      ;;
    --down-on-exit)
      down_on_exit_override="true"
      ;;
    --keep-containers|--no-down-on-exit)
      down_on_exit_override="false"
      ;;
    *)
      compose_args+=("$arg")
      ;;
  esac
done

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

down_on_exit=true
if [[ -n "$down_on_exit_override" ]]; then
  if is_truthy_bool "$down_on_exit_override"; then
    down_on_exit=true
  elif is_falsey_bool "$down_on_exit_override"; then
    down_on_exit=false
  else
    echo "Error: invalid down-on-exit override: $down_on_exit_override"
    exit 1
  fi
elif [[ "$detach_mode" == "true" ]]; then
  down_on_exit=false
fi

cleanup() {
  if [[ "$down_on_exit" == "true" ]]; then
    echo ""
    echo "→ Stopping Docker Compose services..."
    "${COMPOSE_CMD[@]}" -f infra/docker-compose.yml down --remove-orphans >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

# Start services
echo "→ Starting Docker Compose services..."
if (( ${#compose_args[@]} > 0 )); then
  "${COMPOSE_CMD[@]}" -f infra/docker-compose.yml up --build --remove-orphans "${compose_args[@]}"
else
  "${COMPOSE_CMD[@]}" -f infra/docker-compose.yml up --build --remove-orphans
fi

echo ""
echo "Mailpit dev inbox: http://localhost:8025"
echo "Mailpit SMTP endpoint: localhost:1025"
