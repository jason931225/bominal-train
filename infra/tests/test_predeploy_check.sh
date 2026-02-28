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
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
AUTH_MODE=supabase
SUPABASE_URL=https://test-ref.supabase.co
SUPABASE_JWT_ISSUER=https://test-ref.supabase.co/auth/v1
SUPABASE_AUTH_ENABLED=true
SUPABASE_AUTH_API_KEY=anon-key
SUPABASE_AUTH_TIMEOUT_SECONDS=12
SUPABASE_STORAGE_ENABLED=true
SUPABASE_SERVICE_ROLE_KEY=service-role-key
EMAIL_PROVIDER=disabled
INTERNAL_API_KEY=abc123
INTERNAL_API_KEY_SECRET_ID=
INTERNAL_API_KEY_SECRET_VERSION=
MASTER_KEY=base64-secret
EOF
  cat >"$TMP_DIR/repo/infra/env/prod/web.env" <<'EOF'
NEXT_PUBLIC_API_BASE_URL=https://example.com
API_SERVER_URL=http://api:8000
NEXT_PUBLIC_SUPABASE_DIRECT_AUTH_ENABLED=true
NEXT_PUBLIC_SUPABASE_REALTIME_ENABLED=true
NEXT_PUBLIC_SUPABASE_URL=https://test-ref.supabase.co
NEXT_PUBLIC_SUPABASE_ANON_KEY=anon-key
EOF
  cat >"$TMP_DIR/repo/infra/env/prod/pay.env" <<'EOF'
CARDNUMBER=4111111111111111
EXPIRYMM=12
EXPIRYYY=99
DOB=19900101
NN=12
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

# GSM-enabled deploys should pass without MASTER_KEY in api.env.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
AUTH_MODE=supabase
SUPABASE_URL=https://test-ref.supabase.co
SUPABASE_JWT_ISSUER=https://test-ref.supabase.co/auth/v1
SUPABASE_AUTH_ENABLED=true
SUPABASE_AUTH_API_KEY=anon-key
SUPABASE_AUTH_TIMEOUT_SECONDS=12
SUPABASE_STORAGE_ENABLED=true
SUPABASE_SERVICE_ROLE_KEY=service-role-key
EMAIL_PROVIDER=disabled
INTERNAL_API_KEY=abc123
GSM_MASTER_KEY_ENABLED=true
GSM_MASTER_KEY_SECRET_ID=bominal-master-key
GSM_MASTER_KEY_VERSION=3
GSM_MASTER_KEY_ALLOW_ENV_FALLBACK=false
EOF
PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests >/dev/null

# GSM-enabled production must pin secret version (latest not allowed).
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
AUTH_MODE=supabase
SUPABASE_URL=https://test-ref.supabase.co
SUPABASE_JWT_ISSUER=https://test-ref.supabase.co/auth/v1
SUPABASE_AUTH_ENABLED=true
SUPABASE_AUTH_API_KEY=anon-key
SUPABASE_AUTH_TIMEOUT_SECONDS=12
SUPABASE_STORAGE_ENABLED=true
SUPABASE_SERVICE_ROLE_KEY=service-role-key
EMAIL_PROVIDER=disabled
INTERNAL_API_KEY=abc123
GSM_MASTER_KEY_ENABLED=true
GSM_MASTER_KEY_SECRET_ID=bominal-master-key
GSM_MASTER_KEY_VERSION=latest
GSM_MASTER_KEY_ALLOW_ENV_FALLBACK=false
EOF
assert_fails "gsm latest version must fail" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# GSM-enabled production must disable env fallback.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
AUTH_MODE=supabase
SUPABASE_URL=https://test-ref.supabase.co
SUPABASE_JWT_ISSUER=https://test-ref.supabase.co/auth/v1
SUPABASE_AUTH_ENABLED=true
SUPABASE_AUTH_API_KEY=anon-key
SUPABASE_AUTH_TIMEOUT_SECONDS=12
SUPABASE_STORAGE_ENABLED=true
SUPABASE_SERVICE_ROLE_KEY=service-role-key
EMAIL_PROVIDER=disabled
INTERNAL_API_KEY=abc123
GSM_MASTER_KEY_ENABLED=true
GSM_MASTER_KEY_SECRET_ID=bominal-master-key
GSM_MASTER_KEY_VERSION=3
GSM_MASTER_KEY_ALLOW_ENV_FALLBACK=true
EOF
assert_fails "gsm env fallback true must fail" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# GSM-enabled production requires project id resolution.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
AUTH_MODE=supabase
SUPABASE_URL=https://test-ref.supabase.co
SUPABASE_JWT_ISSUER=https://test-ref.supabase.co/auth/v1
SUPABASE_AUTH_ENABLED=true
SUPABASE_AUTH_API_KEY=anon-key
SUPABASE_AUTH_TIMEOUT_SECONDS=12
SUPABASE_STORAGE_ENABLED=true
SUPABASE_SERVICE_ROLE_KEY=service-role-key
EMAIL_PROVIDER=disabled
INTERNAL_API_KEY=abc123
GSM_MASTER_KEY_ENABLED=true
GSM_MASTER_KEY_SECRET_ID=bominal-master-key
GSM_MASTER_KEY_VERSION=3
GSM_MASTER_KEY_ALLOW_ENV_FALLBACK=false
EOF
assert_fails "gsm missing project id must fail" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests
make_valid_envs
make_valid_registry

# INTERNAL_API_KEY can be sourced from GSM secret reference.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
AUTH_MODE=supabase
SUPABASE_URL=https://test-ref.supabase.co
SUPABASE_JWT_ISSUER=https://test-ref.supabase.co/auth/v1
SUPABASE_AUTH_ENABLED=true
SUPABASE_AUTH_API_KEY=anon-key
SUPABASE_AUTH_TIMEOUT_SECONDS=12
SUPABASE_STORAGE_ENABLED=true
SUPABASE_SERVICE_ROLE_KEY=service-role-key
EMAIL_PROVIDER=disabled
INTERNAL_API_KEY=
INTERNAL_API_KEY_SECRET_ID=bominal-internal-api-key
INTERNAL_API_KEY_SECRET_VERSION=4
MASTER_KEY=base64-secret
EOF
PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests >/dev/null

# INTERNAL_API_KEY source ambiguity should fail.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
AUTH_MODE=supabase
SUPABASE_URL=https://test-ref.supabase.co
SUPABASE_JWT_ISSUER=https://test-ref.supabase.co/auth/v1
SUPABASE_AUTH_ENABLED=true
SUPABASE_AUTH_API_KEY=anon-key
SUPABASE_AUTH_TIMEOUT_SECONDS=12
SUPABASE_STORAGE_ENABLED=true
SUPABASE_SERVICE_ROLE_KEY=service-role-key
EMAIL_PROVIDER=disabled
INTERNAL_API_KEY=abc123
INTERNAL_API_KEY_SECRET_ID=bominal-internal-api-key
INTERNAL_API_KEY_SECRET_VERSION=4
MASTER_KEY=base64-secret
EOF
assert_fails "internal api key source ambiguity must fail" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests
make_valid_envs
make_valid_registry

# INTERNAL_API_KEY GSM source requires pinned version.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
AUTH_MODE=supabase
SUPABASE_URL=https://test-ref.supabase.co
SUPABASE_JWT_ISSUER=https://test-ref.supabase.co/auth/v1
SUPABASE_AUTH_ENABLED=true
SUPABASE_AUTH_API_KEY=anon-key
SUPABASE_AUTH_TIMEOUT_SECONDS=12
SUPABASE_STORAGE_ENABLED=true
SUPABASE_SERVICE_ROLE_KEY=service-role-key
EMAIL_PROVIDER=disabled
INTERNAL_API_KEY=
INTERNAL_API_KEY_SECRET_ID=bominal-internal-api-key
INTERNAL_API_KEY_SECRET_VERSION=latest
MASTER_KEY=base64-secret
EOF
assert_fails "internal api key gsm latest version must fail" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests
make_valid_envs
make_valid_registry

# Leading-zero month values should be treated as decimal and avoid bash octal warnings.
cat >"$TMP_DIR/repo/infra/env/prod/pay.env" <<'EOF'
CARDNUMBER=4111111111111111
EXPIRYMM=08
EXPIRYYY=99
DOB=19900101
NN=12
EOF
predeploy_out="$(
  env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" \
    "$SCRIPT" --skip-smoke-tests 2>&1
)"
if grep -q "value too great for base" <<<"$predeploy_out"; then
  echo "FAIL: predeploy check emitted bash octal parse warning for EXPIRYMM=08" >&2
  echo "$predeploy_out" >&2
  exit 1
fi
make_valid_envs

# Policy-gate bypass should skip deprecation/resource gates.
PATH="$TMP_DIR/bin:$PATH" \
  PREDEPLOY_ALLOW_POLICY_GATES_BYPASS=true \
  PREDEPLOY_MIN_TOTAL_MEMORY_MB=999999 \
  PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" \
  BOMINAL_ROOT_DIR="$TMP_DIR/repo" \
  "$SCRIPT" --skip-smoke-tests >/dev/null

# PAYMENT_PROVIDER=evervault requires browser-side Evervault env vars.
cat >>"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
PAYMENT_PROVIDER=evervault
PAYMENT_EVERVAULT_ENFORCE=true
AUTOPAY_REQUIRE_USER_WALLET=true
AUTOPAY_ALLOW_SERVER_FALLBACK=false
EOF
assert_fails "evervault payment mode requires NEXT_PUBLIC_EVERVAULT_TEAM_ID and NEXT_PUBLIC_EVERVAULT_APP_ID" \
  env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" \
  "$SCRIPT" --skip-smoke-tests
make_valid_envs
make_valid_registry

cat >>"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
PAYMENT_PROVIDER=evervault
PAYMENT_EVERVAULT_ENFORCE=true
AUTOPAY_REQUIRE_USER_WALLET=true
AUTOPAY_ALLOW_SERVER_FALLBACK=false
EOF
cat >>"$TMP_DIR/repo/infra/env/prod/web.env" <<'EOF'
NEXT_PUBLIC_EVERVAULT_TEAM_ID=team_test_123
NEXT_PUBLIC_EVERVAULT_APP_ID=app_test_123
EOF
PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" \
  "$SCRIPT" --skip-smoke-tests >/dev/null
make_valid_envs
make_valid_registry

# PAYMENT_PROVIDER=evervault must reject server fallback in production.
cat >>"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
PAYMENT_PROVIDER=evervault
PAYMENT_EVERVAULT_ENFORCE=true
AUTOPAY_REQUIRE_USER_WALLET=true
AUTOPAY_ALLOW_SERVER_FALLBACK=true
EOF
cat >>"$TMP_DIR/repo/infra/env/prod/web.env" <<'EOF'
NEXT_PUBLIC_EVERVAULT_TEAM_ID=team_test_123
NEXT_PUBLIC_EVERVAULT_APP_ID=app_test_123
EOF
assert_fails "evervault payment mode must disable server fallback" \
  env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" \
  "$SCRIPT" --skip-smoke-tests
make_valid_envs
make_valid_registry

# Production must enforce AUTH_MODE=supabase.
make_valid_envs
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
AUTH_MODE=legacy
SUPABASE_URL=https://test-ref.supabase.co
SUPABASE_JWT_ISSUER=https://test-ref.supabase.co/auth/v1
SUPABASE_AUTH_ENABLED=true
SUPABASE_AUTH_API_KEY=anon-key
SUPABASE_AUTH_TIMEOUT_SECONDS=12
SUPABASE_STORAGE_ENABLED=true
SUPABASE_SERVICE_ROLE_KEY=service-role-key
EMAIL_PROVIDER=disabled
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
EOF
assert_fails "production auth mode must be supabase" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests
make_valid_envs
make_valid_registry

# Production must enforce Supabase auth integration.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
AUTH_MODE=supabase
SUPABASE_URL=https://test-ref.supabase.co
SUPABASE_JWT_ISSUER=https://test-ref.supabase.co/auth/v1
SUPABASE_AUTH_ENABLED=false
SUPABASE_STORAGE_ENABLED=true
SUPABASE_SERVICE_ROLE_KEY=service-role-key
EMAIL_PROVIDER=disabled
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
EOF
assert_fails "supabase auth must be enabled in production" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests
make_valid_envs
make_valid_registry

# Production must enforce Supabase storage integration.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
AUTH_MODE=supabase
SUPABASE_URL=https://test-ref.supabase.co
SUPABASE_JWT_ISSUER=https://test-ref.supabase.co/auth/v1
SUPABASE_AUTH_ENABLED=true
SUPABASE_AUTH_API_KEY=anon-key
SUPABASE_AUTH_TIMEOUT_SECONDS=12
SUPABASE_STORAGE_ENABLED=false
SUPABASE_SERVICE_ROLE_KEY=
EMAIL_PROVIDER=disabled
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
EOF
assert_fails "supabase storage must be enabled in production" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests
make_valid_envs
make_valid_registry

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
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
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
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
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
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
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
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
AUTH_MODE=legacy
EMAIL_PROVIDER=resend
EMAIL_FROM_ADDRESS=no-reply@example.com
RESEND_API_KEY=
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
EOF
assert_fails "resend requires api key" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# Resend provider can use Supabase Vault secret name in place of RESEND_API_KEY.
make_valid_envs
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
AUTH_MODE=supabase
SUPABASE_URL=https://test-ref.supabase.co
SUPABASE_JWT_ISSUER=https://test-ref.supabase.co/auth/v1
SUPABASE_AUTH_ENABLED=true
SUPABASE_AUTH_API_KEY=anon-key
SUPABASE_AUTH_TIMEOUT_SECONDS=12
SUPABASE_STORAGE_ENABLED=true
SUPABASE_SERVICE_ROLE_KEY=service-role-key
EDGE_TASK_NOTIFY_ENABLED=true
SUPABASE_VAULT_ENABLED=true
EMAIL_PROVIDER=resend
EMAIL_FROM_ADDRESS=no-reply@example.com
RESEND_API_KEY=
RESEND_API_KEY_VAULT_NAME=resend_api_key
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
EOF
PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests >/dev/null

# Vault-based resend secret reference requires SUPABASE_VAULT_ENABLED=true.
make_valid_envs
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
AUTH_MODE=supabase
SUPABASE_URL=https://test-ref.supabase.co
SUPABASE_JWT_ISSUER=https://test-ref.supabase.co/auth/v1
SUPABASE_AUTH_ENABLED=true
SUPABASE_AUTH_API_KEY=anon-key
SUPABASE_AUTH_TIMEOUT_SECONDS=12
SUPABASE_STORAGE_ENABLED=true
SUPABASE_SERVICE_ROLE_KEY=service-role-key
EDGE_TASK_NOTIFY_ENABLED=true
SUPABASE_VAULT_ENABLED=false
EMAIL_PROVIDER=resend
EMAIL_FROM_ADDRESS=no-reply@example.com
RESEND_API_KEY=
RESEND_API_KEY_VAULT_NAME=resend_api_key
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
EOF
assert_fails "vault resend secret requires supabase vault enabled" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# Vault-based resend secret reference requires edge notify mode.
make_valid_envs
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
AUTH_MODE=supabase
SUPABASE_URL=https://test-ref.supabase.co
SUPABASE_JWT_ISSUER=https://test-ref.supabase.co/auth/v1
SUPABASE_AUTH_ENABLED=true
SUPABASE_AUTH_API_KEY=anon-key
SUPABASE_AUTH_TIMEOUT_SECONDS=12
SUPABASE_STORAGE_ENABLED=true
SUPABASE_SERVICE_ROLE_KEY=service-role-key
EDGE_TASK_NOTIFY_ENABLED=false
SUPABASE_VAULT_ENABLED=true
EMAIL_PROVIDER=resend
EMAIL_FROM_ADDRESS=no-reply@example.com
RESEND_API_KEY=
RESEND_API_KEY_VAULT_NAME=resend_api_key
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
EOF
assert_fails "vault resend secret requires edge notify mode" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# Resend provider can use GSM secret reference.
make_valid_envs
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
AUTH_MODE=supabase
SUPABASE_URL=https://test-ref.supabase.co
SUPABASE_JWT_ISSUER=https://test-ref.supabase.co/auth/v1
SUPABASE_AUTH_ENABLED=true
SUPABASE_AUTH_API_KEY=anon-key
SUPABASE_AUTH_TIMEOUT_SECONDS=12
SUPABASE_STORAGE_ENABLED=true
SUPABASE_SERVICE_ROLE_KEY=service-role-key
EMAIL_PROVIDER=resend
EMAIL_FROM_ADDRESS=no-reply@example.com
RESEND_API_KEY=
RESEND_API_KEY_SECRET_ID=bominal-resend-api-key
RESEND_API_KEY_SECRET_VERSION=3
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
EOF
PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests >/dev/null

# Resend GSM secret reference requires pinned version.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
AUTH_MODE=supabase
SUPABASE_URL=https://test-ref.supabase.co
SUPABASE_JWT_ISSUER=https://test-ref.supabase.co/auth/v1
SUPABASE_AUTH_ENABLED=true
SUPABASE_AUTH_API_KEY=anon-key
SUPABASE_AUTH_TIMEOUT_SECONDS=12
SUPABASE_STORAGE_ENABLED=true
SUPABASE_SERVICE_ROLE_KEY=service-role-key
EMAIL_PROVIDER=resend
EMAIL_FROM_ADDRESS=no-reply@example.com
RESEND_API_KEY=
RESEND_API_KEY_SECRET_ID=bominal-resend-api-key
RESEND_API_KEY_SECRET_VERSION=latest
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
EOF
assert_fails "resend gsm latest version must fail" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# Resend source ambiguity should fail.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
AUTH_MODE=supabase
SUPABASE_URL=https://test-ref.supabase.co
SUPABASE_JWT_ISSUER=https://test-ref.supabase.co/auth/v1
SUPABASE_AUTH_ENABLED=true
SUPABASE_AUTH_API_KEY=anon-key
SUPABASE_AUTH_TIMEOUT_SECONDS=12
SUPABASE_STORAGE_ENABLED=true
SUPABASE_SERVICE_ROLE_KEY=service-role-key
EMAIL_PROVIDER=resend
EMAIL_FROM_ADDRESS=no-reply@example.com
RESEND_API_KEY=re_test_key
RESEND_API_KEY_SECRET_ID=bominal-resend-api-key
RESEND_API_KEY_SECRET_VERSION=2
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
EOF
assert_fails "resend key source ambiguity must fail" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# Public web API base URL must be HTTPS when set.
make_valid_envs
cat >"$TMP_DIR/repo/infra/env/prod/web.env" <<'EOF'
NEXT_PUBLIC_API_BASE_URL=http://example.com
API_SERVER_URL=http://api:8000
EOF
assert_fails "web public api base must be https" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# Supabase auth redirect/site URL sources must not point to localhost in production.
make_valid_envs
cat >"$TMP_DIR/repo/infra/env/prod/web.env" <<'EOF'
NEXT_PUBLIC_API_BASE_URL=https://localhost:3000
API_SERVER_URL=http://api:8000
EOF
assert_fails "supabase auth site url must not target localhost" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# Supabase auth redirect URL list must include verify path.
make_valid_envs
cat >>"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
SUPABASE_AUTH_REDIRECT_URLS=https://example.com/reset-password,https://example.com/login
EOF
assert_fails "supabase auth redirect URLs must include verify path" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# Web Supabase direct auth/realtime enabled requires browser URL + anon key.
make_valid_envs
cat >"$TMP_DIR/repo/infra/env/prod/web.env" <<'EOF'
NEXT_PUBLIC_API_BASE_URL=https://example.com
API_SERVER_URL=http://api:8000
NEXT_PUBLIC_SUPABASE_DIRECT_AUTH_ENABLED=true
NEXT_PUBLIC_SUPABASE_REALTIME_ENABLED=true
NEXT_PUBLIC_SUPABASE_URL=
NEXT_PUBLIC_SUPABASE_ANON_KEY=
EOF
assert_fails "web supabase realtime/auth requires supabase browser config" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# Web Supabase browser URL must be HTTPS when direct auth/realtime enabled.
make_valid_envs
cat >"$TMP_DIR/repo/infra/env/prod/web.env" <<'EOF'
NEXT_PUBLIC_API_BASE_URL=https://example.com
API_SERVER_URL=http://api:8000
NEXT_PUBLIC_SUPABASE_DIRECT_AUTH_ENABLED=true
NEXT_PUBLIC_SUPABASE_REALTIME_ENABLED=true
NEXT_PUBLIC_SUPABASE_URL=http://supabase.local
NEXT_PUBLIC_SUPABASE_ANON_KEY=anon-key
EOF
assert_fails "web supabase browser url must be https" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# Data API/GraphQL read flags also require browser Supabase config.
make_valid_envs
cat >"$TMP_DIR/repo/infra/env/prod/web.env" <<'EOF'
NEXT_PUBLIC_API_BASE_URL=https://example.com
API_SERVER_URL=http://api:8000
NEXT_PUBLIC_TRAIN_READS_VIA_DATA_API=true
NEXT_PUBLIC_TRAIN_DETAIL_VIA_GRAPHQL=true
NEXT_PUBLIC_SUPABASE_URL=
NEXT_PUBLIC_SUPABASE_ANON_KEY=
EOF
assert_fails "web supabase data api/graphql flags require supabase browser config" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# Edge notify timeout must be positive when enabled.
make_valid_envs
cat >>"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
EDGE_TASK_NOTIFY_ENABLED=true
SUPABASE_EDGE_TIMEOUT_SECONDS=0
EOF
assert_fails "edge notify timeout must be positive when enabled" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# Edge notify base URL override must be HTTPS when set.
make_valid_envs
cat >>"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
EDGE_TASK_NOTIFY_ENABLED=true
SUPABASE_EDGE_FUNCTIONS_BASE_URL=http://edge.local/functions/v1
EOF
assert_fails "edge notify base url must be https when configured" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# CORS origins must be HTTPS in production.
make_valid_envs
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
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
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@aws-0-us-central1.pooler.supabase.co:5432/postgres?sslmode=require
AUTH_MODE=legacy
EMAIL_PROVIDER=disabled
RESEND_API_BASE_URL=http://api.resend.com
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
EOF
assert_fails "resend api base url must be https" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

# Worker max jobs must be a bounded positive integer when set.
make_valid_envs
cat >>"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
WORKER_MAX_JOBS=0
EOF
assert_fails "worker max jobs must be positive" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

make_valid_envs
cat >>"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
WORKER_MAX_JOBS=51
EOF
assert_fails "worker max jobs must be bounded" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests

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
