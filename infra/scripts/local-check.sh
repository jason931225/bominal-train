#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# local-check.sh — Repeatable local smoothness check (full stack)
# -----------------------------------------------------------------------------
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$REPO_ROOT"

echo "=== bominal local smoothness check ==="

command -v docker >/dev/null 2>&1 || { echo "Error: docker is required"; exit 1; }
command -v curl >/dev/null 2>&1 || { echo "Error: curl is required"; exit 1; }

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

COMPOSE=("${DC[@]}" -p "$PROJECT_NAME" -f infra/docker-compose.yml)

debug_dump() {
    echo ""
    echo "=== Debug: docker compose ps ==="
    "${COMPOSE[@]}" ps || true
    echo ""
    echo "=== Debug: docker compose logs (tail 200) ==="
    "${COMPOSE[@]}" logs --tail=200 api worker web || true
}

trap debug_dump ERR

echo "→ Initializing git submodules..."
git submodule update --init --recursive

ENV_DEV_DIR="$REPO_ROOT/infra/env/dev"
if [[ ! -f "$ENV_DEV_DIR/api.env" || ! -f "$ENV_DEV_DIR/postgres.env" || ! -f "$ENV_DEV_DIR/web.env" ]]; then
    echo "→ Environment files missing. Running local setup..."
    "$REPO_ROOT/infra/scripts/local-setup.sh"
fi

echo "→ Starting dev stack..."
"${COMPOSE[@]}" up -d --build

wait_for_container_healthy() {
    local name="$1"
    local timeout_s="$2"
    local start
    start="$(date +%s)"

    while true; do
        local status
        status="$(docker inspect --format '{{.State.Health.Status}}' "$name" 2>/dev/null || true)"

        if [[ "$status" == "healthy" ]]; then
            echo "  $name: healthy"
            return 0
        fi
        if [[ "$status" == "unhealthy" ]]; then
            echo "  $name: unhealthy"
            return 1
        fi
        if (( "$(date +%s)" - start >= timeout_s )); then
            echo "  $name: timed out waiting for health"
            return 1
        fi

        sleep 2
    done
}

wait_for_url() {
    local url="$1"
    local timeout_s="$2"
    local start
    start="$(date +%s)"

    while true; do
        if curl -fsS "$url" >/dev/null 2>&1; then
            echo "  $url: ok"
            return 0
        fi
        if (( "$(date +%s)" - start >= timeout_s )); then
            echo "  $url: timed out waiting for ok"
            return 1
        fi

        sleep 2
    done
}

echo "→ Waiting for dependencies..."
wait_for_container_healthy "bominal-postgres" 60
wait_for_container_healthy "bominal-redis" 60
wait_for_url "http://localhost:8000/health" 60

echo "→ Running backend tests..."
"${COMPOSE[@]}" exec -T api pytest -q

echo "→ Running web typecheck..."
"${COMPOSE[@]}" exec -T web npx tsc --noEmit

echo ""
echo "=== local-check: ok ==="
