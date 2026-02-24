#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/infra/scripts/predeploy-check.sh"
GUARD_SCRIPT="$ROOT_DIR/infra/scripts/deprecation_guard.py"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p \
  "$TMP_DIR/bin" \
  "$TMP_DIR/repo/infra/env/prod" \
  "$TMP_DIR/repo/infra/scripts" \
  "$TMP_DIR/repo/.github/workflows" \
  "$TMP_DIR/repo/docs/deprecations"

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
  cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:6543/postgres?sslmode=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:6543/postgres?sslmode=require
AUTH_MODE=legacy
EMAIL_PROVIDER=disabled
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
EOF
  cat >"$TMP_DIR/repo/infra/env/prod/web.env" <<'EOF'
NEXT_PUBLIC_API_BASE_URL=https://example.com
API_SERVER_URL=http://api:8000
EOF
  cat >"$TMP_DIR/repo/infra/env/prod/caddy.env" <<'EOF'
CADDY_SITE_ADDRESS=example.com
CADDY_ACME_EMAIL=ops@example.com
EOF
}

make_valid_registry() {
  cat >"$TMP_DIR/repo/docs/deprecations/registry.json" <<'EOF'
{
  "schema_version": 1,
  "generated_at": "2026-02-14",
  "deprecations": [
    {
      "id": "DEP-TEST-PREDEPLOY",
      "surface": "runtime",
      "scope": "production",
      "artifact": "infra/docker-compose.deploy.yml.deprecated",
      "replacement": "infra/docker-compose.prod.yml",
      "owner": "Infra / Deployment",
      "status": "removed",
      "deprecated_on": "2026-01-01",
      "remove_after": "2026-02-01",
      "removed_on": "2026-02-10",
      "removal_commit": "5039127",
      "window_policy": "prod30_github14_local2",
      "callers_scan_paths": [
        "infra/scripts",
        ".github/workflows"
      ],
      "notes": "Test fixture"
    }
  ]
}
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
make_valid_registry

# Valid env files with skip smoke checks should pass.
PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests >/dev/null

# Missing required env file should fail.
rm -f "$TMP_DIR/repo/infra/env/prod/caddy.env"
assert_fails "missing env file" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests
make_valid_envs

# Placeholder should fail.
echo "INTERNAL_API_KEY=CHANGE_ME" >"$TMP_DIR/repo/infra/env/prod/api.env"
echo "MASTER_KEY=abc" >>"$TMP_DIR/repo/infra/env/prod/api.env"
assert_fails "placeholder value" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests
make_valid_envs
make_valid_registry

# Non-Supabase DB URLs should fail.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://bominal:strong-password@postgres:5432/bominal
SYNC_DATABASE_URL=postgresql+psycopg://bominal:strong-password@postgres:5432/bominal
AUTH_MODE=legacy
EMAIL_PROVIDER=disabled
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
EOF
assert_fails "database urls must target supabase" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests
make_valid_envs
make_valid_registry

# Missing required security key should fail.
echo "MASTER_KEY=abc" >"$TMP_DIR/repo/infra/env/prod/api.env"
assert_fails "missing INTERNAL_API_KEY" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# Supabase auth mode without required issuer/url should fail.
make_valid_envs
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:6543/postgres?sslmode=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:6543/postgres?sslmode=require
AUTH_MODE=supabase
SUPABASE_URL=
SUPABASE_JWT_ISSUER=
EMAIL_PROVIDER=disabled
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
EOF
assert_fails "supabase mode requires issuer/url" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# Supabase URLs must be HTTPS.
make_valid_envs
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:6543/postgres?sslmode=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:6543/postgres?sslmode=require
AUTH_MODE=supabase
SUPABASE_URL=http://supabase.local
SUPABASE_JWT_ISSUER=http://supabase.local/auth/v1
EMAIL_PROVIDER=disabled
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
EOF
assert_fails "supabase urls must be https" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# Supabase auth enabled requires auth API key (or service role key).
make_valid_envs
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:6543/postgres?sslmode=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:6543/postgres?sslmode=require
AUTH_MODE=supabase
SUPABASE_URL=https://test-ref.supabase.co
SUPABASE_JWT_ISSUER=https://test-ref.supabase.co/auth/v1
SUPABASE_AUTH_ENABLED=true
SUPABASE_AUTH_API_KEY=
SUPABASE_SERVICE_ROLE_KEY=
SUPABASE_AUTH_TIMEOUT_SECONDS=12
EMAIL_PROVIDER=disabled
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
EOF
assert_fails "supabase auth enabled requires auth key" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# Resend provider without API key should fail.
make_valid_envs
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:6543/postgres?sslmode=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:6543/postgres?sslmode=require
AUTH_MODE=legacy
EMAIL_PROVIDER=resend
RESEND_API_KEY=
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
EOF
assert_fails "resend requires api key" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# Public web API base URL must be HTTPS when set.
make_valid_envs
cat >"$TMP_DIR/repo/infra/env/prod/web.env" <<'EOF'
NEXT_PUBLIC_API_BASE_URL=http://example.com
API_SERVER_URL=http://api:8000
EOF
assert_fails "web public api base must be https" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# CORS origins must be HTTPS in production.
make_valid_envs
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:6543/postgres?sslmode=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:6543/postgres?sslmode=require
AUTH_MODE=legacy
CORS_ORIGINS=http://example.com
EMAIL_PROVIDER=disabled
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
EOF
assert_fails "cors origins must be https" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# Resend API base URL must be HTTPS when configured.
make_valid_envs
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:6543/postgres?sslmode=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:6543/postgres?sslmode=require
AUTH_MODE=legacy
EMAIL_PROVIDER=disabled
RESEND_API_BASE_URL=http://api.resend.com
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
EOF
assert_fails "resend api base url must be https" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# Overdue production deprecation with active references should fail deploy gate.
make_valid_envs
cat >"$TMP_DIR/repo/docs/deprecations/registry.json" <<'EOF'
{
  "schema_version": 1,
  "generated_at": "2026-02-14",
  "deprecations": [
    {
      "id": "DEP-TEST-OVERDUE",
      "surface": "runtime",
      "scope": "production",
      "artifact": "infra/scripts/legacy-deploy.sh",
      "replacement": "infra/scripts/deploy.sh",
      "owner": "Infra / Deployment",
      "status": "deprecated",
      "deprecated_on": "2025-01-01",
      "remove_after": "2025-02-01",
      "window_policy": "prod30_github14_local2",
      "callers_scan_paths": [
        "infra/scripts"
      ],
      "notes": "Past due reference should block predeploy"
    }
  ]
}
EOF
cat >"$TMP_DIR/repo/infra/scripts/runtime-wrapper.sh" <<'EOF'
#!/usr/bin/env bash
echo "infra/scripts/legacy-deploy.sh"
EOF
assert_fails "overdue deprecation should block predeploy" \
  env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# Explicit bypass should allow predeploy to continue.
PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" PREDEPLOY_ALLOW_DEPRECATION_BYPASS=true BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests >/dev/null

echo "OK: predeploy-check env validation tests passed."
