#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# local-check.sh — Local dev "doctor": bring up stack, verify health, run checks
# -----------------------------------------------------------------------------
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$REPO_ROOT"

DOWN_AFTER=0
if [[ "${1:-}" == "--down" ]]; then
  DOWN_AFTER=1
fi

# Detect docker compose command (v2 preferred)
if docker compose version >/dev/null 2>&1; then
  COMPOSE_CMD=(docker compose)
elif command -v docker-compose >/dev/null 2>&1; then
  COMPOSE_CMD=(docker-compose)
else
  echo "Error: docker compose (v2) or docker-compose (v1) is required"
  exit 1
fi

COMPOSE_FILE="infra/docker-compose.yml"

echo "=== bominal local check ==="

if [[ ! -f "infra/env/dev/api.env" ]]; then
  echo "Error: missing env files. Run ./infra/scripts/local-setup.sh first."
  exit 1
fi

cleanup() {
  if [[ "$DOWN_AFTER" -eq 1 ]]; then
    echo "→ Stopping stack (--down)..."
    "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" down
  fi
}
trap cleanup EXIT

echo "→ Starting stack (detached)..."
"${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" up -d --build

echo "→ Waiting for API healthcheck..."
deadline=$((SECONDS + 90))
while true; do
  if [[ $SECONDS -gt $deadline ]]; then
    echo "Error: API healthcheck did not become ready in time"
    "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" logs --tail=200 api worker web || true
    exit 1
  fi

  health_json="$(curl -fsS http://localhost:8000/health 2>/dev/null || true)"
  if [[ -n "$health_json" ]]; then
    ok="$(python3 - <<'PY' "$health_json"
import json, sys
payload = json.loads(sys.argv[1])
print("1" if payload.get("db") is True and payload.get("redis") is True else "0")
PY
)"
    if [[ "$ok" == "1" ]]; then
      break
    fi
  fi

  sleep 2
done

echo "→ Waiting for web (localhost:3000)..."
deadline=$((SECONDS + 90))
while true; do
  if [[ $SECONDS -gt $deadline ]]; then
    echo "Error: web did not become ready in time"
    "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" logs --tail=200 api worker web || true
    exit 1
  fi

  code="$(curl -s -o /dev/null -w '%{http_code}' http://localhost:3000 2>/dev/null || true)"
  if [[ "$code" == "200" || "$code" == "302" || "$code" == "307" ]]; then
    break
  fi
  sleep 2
done

echo "→ Running backend tests..."
if ! "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" exec -T api pytest -q; then
  echo "Error: backend tests failed"
  "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" logs --tail=200 api worker web || true
  exit 1
fi

echo "→ Running web typecheck..."
if ! "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" exec -T web npx tsc --noEmit; then
  echo "Error: web typecheck failed"
  "${COMPOSE_CMD[@]}" -f "$COMPOSE_FILE" logs --tail=200 api worker web || true
  exit 1
fi

echo "OK: local stack is healthy"

