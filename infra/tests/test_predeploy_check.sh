#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/infra/scripts/predeploy-check.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p "$TMP_DIR/bin" "$TMP_DIR/repo/infra/env/prod"

cat >"$TMP_DIR/bin/docker" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == "compose" && "${2:-}" == "version" ]]; then
  exit 0
fi
if [[ "${1:-}" == "compose" ]]; then
  if [[ "$*" == *"config"* ]]; then
    echo "ok"
    exit 0
  fi
  if [[ "$*" == *"ps --services --filter status=running"* ]]; then
    exit 0
  fi
  exit 0
fi
exit 0
EOF
chmod +x "$TMP_DIR/bin/docker"

make_valid_envs() {
  cat >"$TMP_DIR/repo/infra/env/prod/postgres.env" <<'EOF'
POSTGRES_DB=bominal
POSTGRES_USER=bominal
POSTGRES_PASSWORD=strong-password
EOF
  cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
EOF
  cat >"$TMP_DIR/repo/infra/env/prod/web.env" <<'EOF'
NEXT_PUBLIC_API_BASE_URL=https://example.com
EOF
  cat >"$TMP_DIR/repo/infra/env/prod/caddy.env" <<'EOF'
CADDY_SITE_ADDRESS=example.com
EOF
}

assert_fails() {
  local msg="$1"
  shift
  if "$@" >/dev/null 2>&1; then
    echo "FAIL: expected failure - $msg" >&2
    exit 1
  fi
}

make_valid_envs

# Valid env files with skip smoke checks should pass.
PATH="$TMP_DIR/bin:$PATH" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests >/dev/null

# Missing required env file should fail.
rm -f "$TMP_DIR/repo/infra/env/prod/caddy.env"
assert_fails "missing env file" env PATH="$TMP_DIR/bin:$PATH" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests
make_valid_envs

# Placeholder should fail.
echo "INTERNAL_API_KEY=CHANGE_ME" >"$TMP_DIR/repo/infra/env/prod/api.env"
echo "MASTER_KEY=abc" >>"$TMP_DIR/repo/infra/env/prod/api.env"
assert_fails "placeholder value" env PATH="$TMP_DIR/bin:$PATH" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests
make_valid_envs

# Missing required security key should fail.
echo "MASTER_KEY=abc" >"$TMP_DIR/repo/infra/env/prod/api.env"
assert_fails "missing INTERNAL_API_KEY" env PATH="$TMP_DIR/bin:$PATH" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

echo "OK: predeploy-check env validation tests passed."
