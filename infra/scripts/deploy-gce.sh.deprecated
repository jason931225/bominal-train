#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

if [[ (-z "${GHCR_USERNAME:-}" || -z "${GHCR_TOKEN:-}") && -f "infra/env/prod/deploy.env" ]]; then
  echo "==> Loading registry credentials from infra/env/prod/deploy.env"
  set -a
  # shellcheck disable=SC1091
  source "infra/env/prod/deploy.env"
  set +a
fi

if [[ -z "${BOMINAL_IMAGE_PREFIX:-}" ]]; then
  echo "BOMINAL_IMAGE_PREFIX is required (example: ghcr.io/your-org/bominal)."
  exit 1
fi

BOMINAL_IMAGE_TAG="${BOMINAL_IMAGE_TAG:-latest}"

required_files=(
  "infra/env/prod/postgres.env"
  "infra/env/prod/api.env"
  "infra/env/prod/web.env"
  "infra/env/prod/caddy.env"
)

echo "==> Checking required production env files"
for file in "${required_files[@]}"; do
  if [[ ! -f "$file" ]]; then
    echo "Missing required file: $file"
    exit 1
  fi
done

echo "==> Checking for unresolved placeholder values"
if grep -RIn "CHANGE_ME" infra/env/prod/*.env >/tmp/bominal-prod-placeholders.txt 2>/dev/null; then
  echo "Found unresolved placeholders in production env files:"
  cat /tmp/bominal-prod-placeholders.txt
  exit 1
fi

echo "==> Checking required API security settings"
required_api_keys=(
  "INTERNAL_API_KEY"
)
for key in "${required_api_keys[@]}"; do
  if ! grep -Eq "^${key}=.+" infra/env/prod/api.env; then
    echo "Missing or empty ${key} in infra/env/prod/api.env"
    exit 1
  fi
done

if [[ -n "${GHCR_USERNAME:-}" && -n "${GHCR_TOKEN:-}" ]]; then
  echo "==> Logging into ghcr.io"
  echo "$GHCR_TOKEN" | docker login ghcr.io -u "$GHCR_USERNAME" --password-stdin
fi

compose() {
  if docker compose version >/dev/null 2>&1; then
    docker compose "$@"
  else
    docker-compose "$@"
  fi
}

echo "==> Pulling images (${BOMINAL_IMAGE_PREFIX}, tag ${BOMINAL_IMAGE_TAG})"
compose -f infra/docker-compose.deploy.yml pull api worker web caddy

echo "==> Applying deployment"
compose -f infra/docker-compose.deploy.yml up -d --remove-orphans

echo "==> Waiting for API health"
attempt=0
until curl -fsS http://localhost:8000/health >/dev/null 2>&1; do
  attempt=$((attempt + 1))
  if [[ "$attempt" -ge 30 ]]; then
    echo "API did not become healthy in time."
    compose -f infra/docker-compose.deploy.yml logs --tail=150 caddy api worker web
    exit 1
  fi
  sleep 2
done

echo "==> Deployment status"
compose -f infra/docker-compose.deploy.yml ps
