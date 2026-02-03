#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

required_files=(
  "infra/env/prod/postgres.env"
  "infra/env/prod/api.env"
  "infra/env/prod/web.env"
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

echo "==> Validating production compose configuration"
docker-compose -f infra/docker-compose.prod.yml config >/tmp/bominal-prod-compose.txt

echo "==> Running backend smoke tests"
docker-compose -f infra/docker-compose.yml exec -T api sh -lc 'cd /app && PYTHONPATH=/app pytest -q tests/test_auth_flow.py tests/test_train_provider_crud.py'

echo "==> Running frontend type check"
docker-compose -f infra/docker-compose.yml exec -T web sh -lc 'cd /app && npx tsc --noEmit'

echo "Pre-deploy checks passed."

