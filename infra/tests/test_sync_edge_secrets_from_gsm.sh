#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/infra/scripts/sync-edge-secrets-from-gsm.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p "$TMP_DIR/bin" "$TMP_DIR/repo/infra/env/prod"

cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'ENV'
GCP_PROJECT_ID=test-project
SUPABASE_URL=https://test-ref.supabase.co
EDGE_TASK_NOTIFY_ENABLED=true
EMAIL_FROM_ADDRESS=no-reply@example.com
EMAIL_FROM_NAME=bominal
RESEND_API_KEY_SECRET_ID=bominal-resend-api-key
RESEND_API_KEY_SECRET_VERSION=3
ENV

cat >"$TMP_DIR/bin/gcloud" <<'GCLOUD'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "${GCLOUD_CALLS_FILE:?}"
if [[ "$*" == *"secrets versions access"* ]]; then
  printf 're_test_secret_value'
  exit 0
fi
exit 0
GCLOUD
chmod +x "$TMP_DIR/bin/gcloud"

cat >"$TMP_DIR/bin/supabase" <<'SUPABASE'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "${SUPABASE_CALLS_FILE:?}"
if [[ "$*" == *"secrets set"* ]]; then
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --env-file)
        cp "$2" "${SYNC_ENV_CAPTURE_FILE:?}"
        shift 2
        ;;
      *)
        shift
        ;;
    esac
  done
  exit 0
fi
exit 0
SUPABASE
chmod +x "$TMP_DIR/bin/supabase"

# Dry-run should not call gcloud/supabase.
GCLOUD_CALLS_FILE="$TMP_DIR/gcloud.calls" \
SUPABASE_CALLS_FILE="$TMP_DIR/supabase.calls" \
PATH="$TMP_DIR/bin:$PATH" \
BOMINAL_ROOT_DIR="$TMP_DIR/repo" \
bash "$SCRIPT" --dry-run >/dev/null

if [[ -s "$TMP_DIR/gcloud.calls" ]]; then
  echo "FAIL: dry-run unexpectedly called gcloud" >&2
  cat "$TMP_DIR/gcloud.calls" >&2
  exit 1
fi
if [[ -s "$TMP_DIR/supabase.calls" ]]; then
  echo "FAIL: dry-run unexpectedly called supabase" >&2
  cat "$TMP_DIR/supabase.calls" >&2
  exit 1
fi

# Apply should call both commands and sync expected env payload.
GCLOUD_CALLS_FILE="$TMP_DIR/gcloud.calls" \
SUPABASE_CALLS_FILE="$TMP_DIR/supabase.calls" \
SYNC_ENV_CAPTURE_FILE="$TMP_DIR/synced.env" \
PATH="$TMP_DIR/bin:$PATH" \
BOMINAL_ROOT_DIR="$TMP_DIR/repo" \
bash "$SCRIPT" --apply >/dev/null

if ! rg -q 'secrets versions access' "$TMP_DIR/gcloud.calls"; then
  echo "FAIL: apply did not fetch secret from gcloud" >&2
  cat "$TMP_DIR/gcloud.calls" >&2
  exit 1
fi

if ! rg -q 'secrets set --project-ref test-ref --env-file' "$TMP_DIR/supabase.calls"; then
  echo "FAIL: apply did not call supabase secrets set with project ref/env file" >&2
  cat "$TMP_DIR/supabase.calls" >&2
  exit 1
fi

if ! rg -q '^RESEND_API_KEY=re_test_secret_value$' "$TMP_DIR/synced.env"; then
  echo "FAIL: synced env missing RESEND_API_KEY" >&2
  cat "$TMP_DIR/synced.env" >&2
  exit 1
fi
if ! rg -q '^EMAIL_FROM_ADDRESS=no-reply@example.com$' "$TMP_DIR/synced.env"; then
  echo "FAIL: synced env missing EMAIL_FROM_ADDRESS" >&2
  cat "$TMP_DIR/synced.env" >&2
  exit 1
fi
if ! rg -q '^EMAIL_FROM_NAME=bominal$' "$TMP_DIR/synced.env"; then
  echo "FAIL: synced env missing EMAIL_FROM_NAME" >&2
  cat "$TMP_DIR/synced.env" >&2
  exit 1
fi

# latest secret version should fail.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'ENV'
GCP_PROJECT_ID=test-project
SUPABASE_URL=https://test-ref.supabase.co
EDGE_TASK_NOTIFY_ENABLED=true
EMAIL_FROM_ADDRESS=no-reply@example.com
RESEND_API_KEY_SECRET_ID=bominal-resend-api-key
RESEND_API_KEY_SECRET_VERSION=latest
ENV

if PATH="$TMP_DIR/bin:$PATH" BOMINAL_ROOT_DIR="$TMP_DIR/repo" bash "$SCRIPT" --dry-run >/dev/null 2>&1; then
  echo "FAIL: latest secret version should be rejected" >&2
  exit 1
fi

echo "OK: sync-edge-secrets-from-gsm script tests passed."
