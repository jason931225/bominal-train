#!/usr/bin/env bash
# Simple deploy script for e2-micro VM (no CI, source build on VM)
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

if docker compose version >/dev/null 2>&1; then
  COMPOSE_CMD=(docker compose)
else
  COMPOSE_CMD=(docker-compose)
fi

COMPOSE_FILE="infra/docker-compose.prod.yml"

# Ensure required env files exist
required_files=(
  "infra/env/prod/postgres.env"
  "infra/env/prod/api.env"
  "infra/env/prod/web.env"
  "infra/env/prod/caddy.env"
)

for file in "${required_files[@]}"; do
  if [[ ! -f "$file" ]]; then
    echo "ERROR: Missing required file: $file"
    echo "Copy from $file.example and fill in values."
    exit 1
  fi
done

# Check for CHANGE_ME placeholders
if grep -rq "CHANGE_ME" infra/env/prod/*.env 2>/dev/null; then
  echo "ERROR: Found CHANGE_ME placeholders in env files:"
  grep -l "CHANGE_ME" infra/env/prod/*.env
  exit 1
fi

echo "==> Pulling latest code"
git pull --ff-only origin main || true
git submodule update --init --recursive

echo "==> Building images (this may take a few minutes on e2-micro)"
"${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" build --pull

echo "==> Starting services"
"${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --remove-orphans

echo "==> Waiting for services to be healthy..."
sleep 10

for _ in $(seq 1 30); do
  if curl -fsS --max-time 5 http://127.0.0.1:8000/health >/dev/null 2>&1; then
    echo "API is healthy"
    break
  fi
  echo "Waiting for API..."
  sleep 5
done

echo "==> Service status"
"${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" ps

echo "==> Recent logs (last 20 lines per service)"
"${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" logs --tail=20

echo ""
echo "Deploy complete! Check https://www.bominal.com"
