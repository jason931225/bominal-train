#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# predeploy-check.sh — Validate production environment before deployment
# -----------------------------------------------------------------------------
# Checks:
#   - Required env files exist (postgres.env, api.env, web.env, caddy.env)
#   - No unresolved CHANGE_ME placeholders
#   - Required API security keys are set
#
# Usage:
#   ./infra/scripts/predeploy-check.sh
#
# Exit codes:
#   0 - All checks passed
#   1 - One or more checks failed
# -----------------------------------------------------------------------------
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/lib/env_utils.sh"

ROOT_DIR="${BOMINAL_ROOT_DIR:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
cd "$ROOT_DIR"

required_files=(
  "infra/env/prod/postgres.env"
  "infra/env/prod/api.env"
  "infra/env/prod/web.env"
  "infra/env/prod/caddy.env"
)

require_running_services=0
skip_smoke_tests=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --require-running-services)
      require_running_services=1
      shift
      ;;
    --skip-smoke-tests)
      skip_smoke_tests=1
      shift
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: ./infra/scripts/predeploy-check.sh [options]

Options:
  --skip-smoke-tests         Skip compose exec smoke checks.
  --require-running-services Fail if api/web containers are not currently running.
  --help                     Show this help.
USAGE
      exit 0
      ;;
    *)
      log_error "Unknown argument: $1"
      exit 1
      ;;
  esac
done

detect_compose_cmd

echo "==> Checking required production env files"
for file in "${required_files[@]}"; do
  require_nonempty_file "$file"
done

echo "==> Checking for unresolved placeholder values"
for file in "${required_files[@]}"; do
  require_no_env_placeholders "$file"
done

echo "==> Checking required API security settings"
required_api_keys=(
  "INTERNAL_API_KEY"
  "MASTER_KEY"
)
for key in "${required_api_keys[@]}"; do
  require_env_key_nonempty "infra/env/prod/api.env" "$key"
done

echo "==> Validating production compose configuration"
"${COMPOSE_CMD[@]}" -f infra/docker-compose.prod.yml config >/tmp/bominal-prod-compose.txt

service_is_running() {
  local service="$1"
  "${COMPOSE_CMD[@]}" -f infra/docker-compose.yml ps --services --filter status=running 2>/dev/null | grep -Fxq "$service"
}

if [[ "$skip_smoke_tests" -eq 1 ]]; then
  echo "==> Skipping smoke tests (--skip-smoke-tests)"
  echo "Pre-deploy checks passed."
  exit 0
fi

if ! service_is_running "api" || ! service_is_running "web"; then
  if [[ "$require_running_services" -eq 1 ]]; then
    log_error "Required local services are not running (api/web). Start stack or use --skip-smoke-tests."
    exit 1
  fi
  log_warn "Skipping smoke tests because api/web are not running. Use --require-running-services to enforce."
  echo "Pre-deploy checks passed (env + compose validation only)."
  exit 0
fi

echo "==> Running backend smoke tests"
"${COMPOSE_CMD[@]}" -f infra/docker-compose.yml exec -T api sh -lc 'cd /app && PYTHONPATH=/app pytest -q tests/test_auth_flow.py tests/test_train_provider_crud.py'

echo "==> Running frontend type check"
"${COMPOSE_CMD[@]}" -f infra/docker-compose.yml exec -T web sh -lc 'cd /app && npx tsc --noEmit'

echo "Pre-deploy checks passed."
