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

cat >"$TMP_DIR/bin/docker" <<'DOCKER'
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
DOCKER
chmod +x "$TMP_DIR/bin/docker"

make_valid_registry() {
  cat >"$TMP_DIR/repo/docs/deprecations/registry.json" <<'EOF_REGISTRY'
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
      "callers_scan_paths": ["infra/scripts", ".github/workflows"],
      "notes": "Test fixture"
    }
  ]
}
EOF_REGISTRY
}

make_valid_envs() {
  cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_API'
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
MASTER_KEY=base64-secret
SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID=bominal-supabase-management-api-token
SUPABASE_MANAGEMENT_API_TOKEN_SECRET_VERSION=1
SUPABASE_MANAGEMENT_API_TOKEN_PROJECT_ID=test-project
PAYMENT_ENABLED=true
PAYMENT_PROVIDER=evervault
PAYMENT_EVERVAULT_ENFORCE=true
AUTOPAY_REQUIRE_USER_WALLET=true
AUTOPAY_ALLOW_SERVER_FALLBACK=false
EVERVAULT_APP_ID=app_test_123
EVERVAULT_API_KEY_SECRET_ID=bominal-evervault-api-key
EVERVAULT_API_KEY_SECRET_VERSION=4
EOF_API

  cat >"$TMP_DIR/repo/infra/env/prod/web.env" <<'EOF_WEB'
NEXT_PUBLIC_API_BASE_URL=https://example.com
API_SERVER_URL=http://api:8000
NEXT_PUBLIC_EVERVAULT_TEAM_ID=team_test_123
NEXT_PUBLIC_EVERVAULT_APP_ID=app_test_123
NEXT_PUBLIC_SUPABASE_DIRECT_AUTH_ENABLED=false
NEXT_PUBLIC_SUPABASE_REALTIME_ENABLED=false
NEXT_PUBLIC_SUPABASE_REALTIME_DELTA_READ_ENABLED=false
NEXT_PUBLIC_TRAIN_READS_VIA_DATA_API=false
NEXT_PUBLIC_TRAIN_DETAIL_VIA_GRAPHQL=false
EOF_WEB

  cat >"$TMP_DIR/repo/infra/env/prod/caddy.env" <<'EOF_CADDY'
CADDY_SITE_ADDRESS=example.com
CADDY_ACME_EMAIL=ops@example.com
EOF_CADDY
}

assert_fails() {
  local msg="$1"
  shift
  if "$@" >/dev/null 2>&1; then
    echo "FAIL: expected failure - $msg" >&2
    exit 1
  fi
}

run_predeploy() {
  env PATH="$TMP_DIR/bin:$PATH" \
    PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" \
    BOMINAL_ROOT_DIR="$TMP_DIR/repo" \
    "$SCRIPT" --skip-smoke-tests
}

make_valid_registry
make_valid_envs

# Baseline should pass without pay.env.
run_predeploy >/dev/null

# Supabase direct endpoints (*.supabase.com) should pass.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_API'
GCP_PROJECT_ID=test-project
DATABASE_URL=postgresql+asyncpg://postgres.test-ref:strong-password@db.test-ref.supabase.com:5432/postgres?ssl=require
SYNC_DATABASE_URL=postgresql+psycopg://postgres.test-ref:strong-password@db.test-ref.supabase.com:5432/postgres?sslmode=require
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
MASTER_KEY=base64-secret
SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID=bominal-supabase-management-api-token
SUPABASE_MANAGEMENT_API_TOKEN_SECRET_VERSION=1
SUPABASE_MANAGEMENT_API_TOKEN_PROJECT_ID=test-project
PAYMENT_ENABLED=true
PAYMENT_PROVIDER=evervault
PAYMENT_EVERVAULT_ENFORCE=true
AUTOPAY_REQUIRE_USER_WALLET=true
AUTOPAY_ALLOW_SERVER_FALLBACK=false
EVERVAULT_APP_ID=app_test_123
EVERVAULT_API_KEY_SECRET_ID=bominal-evervault-api-key
EVERVAULT_API_KEY_SECRET_VERSION=4
EOF_API
run_predeploy >/dev/null
make_valid_envs

# DATABASE_URL_TARGET=direct requires DATABASE_URL_DIRECT.
cat >>"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_API'
DATABASE_URL_TARGET=direct
EOF_API
assert_fails "direct target without direct URL must fail" run_predeploy
make_valid_envs

# DATABASE_URL_TARGET=direct passes with a valid direct URL.
cat >>"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_API'
DATABASE_URL_TARGET=direct
DATABASE_URL_DIRECT=postgresql+asyncpg://postgres.test-ref:strong-password@db.test-ref.supabase.com:5432/postgres?ssl=require
EOF_API
run_predeploy >/dev/null
make_valid_envs

# Missing required env file should fail.
rm -f "$TMP_DIR/repo/infra/env/prod/caddy.env"
assert_fails "missing caddy.env must fail" run_predeploy
make_valid_envs

# Placeholder should fail.
echo "INTERNAL_API_KEY=CHANGE_ME" >"$TMP_DIR/repo/infra/env/prod/api.env"
echo "MASTER_KEY=base64-secret" >>"$TMP_DIR/repo/infra/env/prod/api.env"
assert_fails "placeholder value must fail" run_predeploy
make_valid_envs

# AUTH_MODE must be supabase in production.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_API'
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
PAYMENT_ENABLED=false
EOF_API
assert_fails "legacy auth mode must fail" run_predeploy
make_valid_envs

# INTERNAL_API_KEY source ambiguity should fail.
cat >>"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_API'
INTERNAL_API_KEY_SECRET_ID=bominal-internal-api-key
INTERNAL_API_KEY_SECRET_VERSION=2
EOF_API
assert_fails "internal api key source ambiguity must fail" run_predeploy
make_valid_envs

# INTERNAL_API_KEY GSM source requires pinned version.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_API'
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
INTERNAL_API_KEY_SECRET_ID=bominal-internal-api-key
INTERNAL_API_KEY_SECRET_VERSION=latest
MASTER_KEY=base64-secret
PAYMENT_ENABLED=false
EOF_API
assert_fails "internal api key latest version must fail" run_predeploy
make_valid_envs

# Supabase management token source requires GSM secret id.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_API'
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
MASTER_KEY=base64-secret
SUPABASE_MANAGEMENT_API_TOKEN_SECRET_VERSION=1
SUPABASE_MANAGEMENT_API_TOKEN_PROJECT_ID=test-project
PAYMENT_ENABLED=false
EOF_API
assert_fails "missing supabase management token secret id must fail" run_predeploy
make_valid_envs

# Supabase management token secret source requires pinned version.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_API'
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
MASTER_KEY=base64-secret
SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID=bominal-supabase-management-api-token
SUPABASE_MANAGEMENT_API_TOKEN_SECRET_VERSION=latest
SUPABASE_MANAGEMENT_API_TOKEN_PROJECT_ID=test-project
PAYMENT_ENABLED=false
EOF_API
assert_fails "supabase management token latest version must fail" run_predeploy
make_valid_envs

# Supabase management token secret source requires project id fallback path.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_API'
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
MASTER_KEY=base64-secret
SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID=bominal-supabase-management-api-token
SUPABASE_MANAGEMENT_API_TOKEN_SECRET_VERSION=1
PAYMENT_ENABLED=false
EOF_API
assert_fails "supabase management token secret id without project id must fail" run_predeploy
make_valid_envs

# Supabase management plaintext tokens in api.env are forbidden.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_API'
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
MASTER_KEY=base64-secret
SUPABASE_MANAGEMENT_API_TOKEN=plain-token
SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID=bominal-supabase-management-api-token
SUPABASE_MANAGEMENT_API_TOKEN_SECRET_VERSION=1
SUPABASE_MANAGEMENT_API_TOKEN_PROJECT_ID=test-project
PAYMENT_ENABLED=false
EOF_API
assert_fails "plaintext supabase management token in api.env must fail" run_predeploy
make_valid_envs

cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_API'
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
MASTER_KEY=base64-secret
SUPABASE_ACCESS_TOKEN=plain-token
SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID=bominal-supabase-management-api-token
SUPABASE_MANAGEMENT_API_TOKEN_SECRET_VERSION=1
SUPABASE_MANAGEMENT_API_TOKEN_PROJECT_ID=test-project
PAYMENT_ENABLED=false
EOF_API
assert_fails "plaintext supabase access token in api.env must fail" run_predeploy
make_valid_envs

# Payment enabled requires evervault-only provider contract.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_API'
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
MASTER_KEY=base64-secret
PAYMENT_ENABLED=true
PAYMENT_PROVIDER=legacy
PAYMENT_EVERVAULT_ENFORCE=true
AUTOPAY_REQUIRE_USER_WALLET=true
AUTOPAY_ALLOW_SERVER_FALLBACK=false
EVERVAULT_APP_ID_SECRET_ID=bominal-evervault-app-id
EVERVAULT_APP_ID_SECRET_VERSION=3
EVERVAULT_API_KEY_SECRET_ID=bominal-evervault-api-key
EVERVAULT_API_KEY_SECRET_VERSION=4
EOF_API
assert_fails "payment enabled with legacy provider must fail" run_predeploy
make_valid_envs

cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_API'
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
MASTER_KEY=base64-secret
PAYMENT_ENABLED=true
PAYMENT_PROVIDER=evervault
PAYMENT_EVERVAULT_ENFORCE=true
AUTOPAY_REQUIRE_USER_WALLET=true
AUTOPAY_ALLOW_SERVER_FALLBACK=true
EVERVAULT_APP_ID_SECRET_ID=bominal-evervault-app-id
EVERVAULT_APP_ID_SECRET_VERSION=3
EVERVAULT_API_KEY_SECRET_ID=bominal-evervault-api-key
EVERVAULT_API_KEY_SECRET_VERSION=4
EOF_API
assert_fails "payment enabled with server fallback must fail" run_predeploy
make_valid_envs

cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_API'
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
MASTER_KEY=base64-secret
PAYMENT_ENABLED=true
PAYMENT_PROVIDER=evervault
PAYMENT_EVERVAULT_ENFORCE=true
AUTOPAY_REQUIRE_USER_WALLET=true
AUTOPAY_ALLOW_SERVER_FALLBACK=false
EVERVAULT_APP_ID_SECRET_ID=bominal-evervault-app-id
EVERVAULT_APP_ID_SECRET_VERSION=3
EVERVAULT_API_KEY_SECRET_ID=bominal-evervault-api-key
EVERVAULT_API_KEY_SECRET_VERSION=4
CARDNUMBER=4111111111111111
EOF_API
assert_fails "legacy CARDNUMBER alias in api.env must fail" run_predeploy
make_valid_envs

# Evervault secret versions must be pinned when using secret IDs.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_API'
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
MASTER_KEY=base64-secret
PAYMENT_ENABLED=true
PAYMENT_PROVIDER=evervault
PAYMENT_EVERVAULT_ENFORCE=true
AUTOPAY_REQUIRE_USER_WALLET=true
AUTOPAY_ALLOW_SERVER_FALLBACK=false
EVERVAULT_APP_ID_SECRET_ID=bominal-evervault-app-id
EVERVAULT_APP_ID_SECRET_VERSION=latest
EVERVAULT_API_KEY_SECRET_ID=bominal-evervault-api-key
EVERVAULT_API_KEY_SECRET_VERSION=latest
EOF_API
assert_fails "evervault latest secret versions must fail" run_predeploy
make_valid_envs

# Payment-disabled mode can bypass evervault-only gates.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_API'
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
MASTER_KEY=base64-secret
PAYMENT_ENABLED=false
PAYMENT_PROVIDER=legacy
SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID=bominal-supabase-management-api-token
SUPABASE_MANAGEMENT_API_TOKEN_SECRET_VERSION=1
SUPABASE_MANAGEMENT_API_TOKEN_PROJECT_ID=test-project
EOF_API
run_predeploy >/dev/null
make_valid_envs

# Resend source ambiguity should fail.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_API'
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
RESEND_API_KEY=re_live_key
RESEND_API_KEY_SECRET_ID=bominal-resend-api-key
RESEND_API_KEY_SECRET_VERSION=2
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
PAYMENT_ENABLED=false
EOF_API
assert_fails "resend source ambiguity must fail" run_predeploy
make_valid_envs

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
MASTER_KEY=base64-secret
PAYMENT_ENABLED=false
CORS_ORIGINS=http://example.com
EOF
assert_fails "cors origins must be https in production" env PATH="$TMP_DIR/bin:$PATH" PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --skip-smoke-tests
make_valid_envs

# Vault-based resend source requires edge mode and vault enabled.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_API'
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
RESEND_API_KEY_VAULT_NAME=resend_api_key
SUPABASE_VAULT_ENABLED=false
EDGE_TASK_NOTIFY_ENABLED=false
INTERNAL_API_KEY=abc123
MASTER_KEY=base64-secret
PAYMENT_ENABLED=false
EOF_API
assert_fails "vault resend source requires edge mode and supabase vault" run_predeploy
make_valid_envs

# Explicit policy-gate bypass should still allow passing valid inputs.
env PATH="$TMP_DIR/bin:$PATH" \
  PREDEPLOY_ALLOW_POLICY_GATES_BYPASS=true \
  PREDEPLOY_DEPRECATION_GUARD_SCRIPT="$GUARD_SCRIPT" \
  BOMINAL_ROOT_DIR="$TMP_DIR/repo" \
  "$SCRIPT" --skip-smoke-tests >/dev/null

echo "OK: predeploy-check env validation tests passed."
