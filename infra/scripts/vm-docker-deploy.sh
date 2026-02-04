#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

if docker compose version >/dev/null 2>&1; then
  COMPOSE_CMD=(docker compose)
else
  COMPOSE_CMD=(docker-compose)
fi

COMPOSE_FILE="infra/docker-compose.deploy.yml"
TAG="${1:-latest}"
export BOMINAL_IMAGE_TAG="$TAG"

required_files=(
  "infra/env/prod/postgres.env"
  "infra/env/prod/api.env"
  "infra/env/prod/web.env"
)

for file in "${required_files[@]}"; do
  if [[ ! -f "$file" ]]; then
    echo "Missing required file: $file"
    exit 1
  fi
done

echo "==> Pulling API/Web/Worker images (tag: $BOMINAL_IMAGE_TAG)"
"${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" pull api worker web

echo "==> Updating stack"
"${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --remove-orphans postgres redis api worker web caddy

echo "==> Waiting for health"
for _ in $(seq 1 60); do
  if curl -fsS --max-time 4 http://127.0.0.1/health >/dev/null \
    && curl -fsS --max-time 6 http://127.0.0.1/login >/dev/null; then
    break
  fi
  sleep 2
done

curl -fsS http://127.0.0.1/health >/dev/null
curl -fsS -I http://127.0.0.1/login >/dev/null

echo "==> Current services"
"${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" ps
