#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PY_SCRIPT="$ROOT_DIR/infra/scripts/setup-gsm-master-key.py"
SH_SCRIPT="$ROOT_DIR/infra/scripts/setup-gsm-master-key.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p "$TMP_DIR/bin" "$TMP_DIR/repo/infra/env/prod"
GCLOUD_LOG="$TMP_DIR/gcloud.log"

cat >"$TMP_DIR/bin/gcloud" <<'GCLOUD'
#!/usr/bin/env bash
set -euo pipefail

if [[ -n "${GCLOUD_LOG_FILE:-}" ]]; then
  echo "$*" >>"$GCLOUD_LOG_FILE"
fi

if [[ "${1:-}" == "services" && "${2:-}" == "enable" ]]; then
  exit 0
fi

if [[ "${1:-}" == "secrets" && "${2:-}" == "describe" ]]; then
  # Simulate missing secret so create path is covered.
  exit 1
fi

if [[ "${1:-}" == "secrets" && "${2:-}" == "create" ]]; then
  exit 0
fi

if [[ "${1:-}" == "secrets" && "${2:-}" == "versions" && "${3:-}" == "add" ]]; then
  cat >/dev/null
  exit 0
fi

if [[ "${1:-}" == "secrets" && "${2:-}" == "versions" && "${3:-}" == "list" ]]; then
  echo "projects/test-project/secrets/bominal-master-key/versions/7"
  exit 0
fi

if [[ "${1:-}" == "secrets" && "${2:-}" == "versions" && "${3:-}" == "describe" ]]; then
  exit 0
fi

if [[ "${1:-}" == "secrets" && "${2:-}" == "add-iam-policy-binding" ]]; then
  exit 0
fi

if [[ "${1:-}" == "pubsub" && "${2:-}" == "subscriptions" && "${3:-}" == "add-iam-policy-binding" ]]; then
  exit 0
fi

echo "unexpected gcloud invocation: $*" >&2
exit 1
GCLOUD
chmod +x "$TMP_DIR/bin/gcloud"

# 1) Python entrypoint with MASTER_KEY -> create secret/version and update api.env.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_ENV'
MASTER_KEY=MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY=
EOF_ENV

env PATH="$TMP_DIR/bin:$PATH" GCLOUD_LOG_FILE="$GCLOUD_LOG" \
  python3 "$PY_SCRIPT" \
    --root-dir "$TMP_DIR/repo" \
    --project-id test-project \
    --secret-id bominal-master-key \
    --runtime-service-account-email vm-sa@test-project.iam.gserviceaccount.com \
    --no-backup >/dev/null

API_ENV="$TMP_DIR/repo/infra/env/prod/api.env"
grep -q '^GCP_PROJECT_ID=test-project$' "$API_ENV"
grep -q '^GSM_MASTER_KEY_ENABLED=true$' "$API_ENV"
grep -q '^GSM_MASTER_KEY_PROJECT_ID=test-project$' "$API_ENV"
grep -q '^GSM_MASTER_KEY_SECRET_ID=bominal-master-key$' "$API_ENV"
grep -q '^GSM_MASTER_KEY_VERSION=7$' "$API_ENV"
grep -q '^GSM_MASTER_KEY_ALLOW_ENV_FALLBACK=false$' "$API_ENV"
grep -q '^MASTER_KEY=MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY=$' "$API_ENV"

grep -q 'services enable secretmanager.googleapis.com --project test-project' "$GCLOUD_LOG"
grep -q 'secrets create bominal-master-key --replication-policy=automatic --project test-project' "$GCLOUD_LOG"
grep -q 'secrets versions add bominal-master-key --data-file=- --project test-project' "$GCLOUD_LOG"
grep -q 'secrets add-iam-policy-binding bominal-master-key --project test-project --member serviceAccount:vm-sa@test-project.iam.gserviceaccount.com --role roles/secretmanager.secretAccessor' "$GCLOUD_LOG"
grep -q 'pubsub subscriptions add-iam-policy-binding bominal-deploy-requests-vm --project test-project --member serviceAccount:vm-sa@test-project.iam.gserviceaccount.com --role roles/pubsub.subscriber' "$GCLOUD_LOG"

# 2) Shell wrapper path with pinned version should avoid MASTER_KEY requirement.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_ENV2'
GCP_PROJECT_ID=test-project
MASTER_KEY=
EOF_ENV2

env PATH="$TMP_DIR/bin:$PATH" GCLOUD_LOG_FILE="$GCLOUD_LOG" \
  bash "$SH_SCRIPT" \
    --root-dir "$TMP_DIR/repo" \
    --project-id test-project \
    --secret-id bominal-master-key \
    --pin-version 9 \
    --skip-enable-api \
    --skip-secret-iam-binding \
    --skip-pubsub-binding \
    --no-backup >/dev/null

grep -q '^GSM_MASTER_KEY_VERSION=9$' "$API_ENV"

# 3) Invalid master key should fail fast.
cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF_ENV3'
MASTER_KEY=invalid
EOF_ENV3

if env PATH="$TMP_DIR/bin:$PATH" GCLOUD_LOG_FILE="$GCLOUD_LOG" \
  python3 "$PY_SCRIPT" \
    --root-dir "$TMP_DIR/repo" \
    --project-id test-project \
    --secret-id bominal-master-key \
    --skip-enable-api \
    --skip-secret-iam-binding \
    --skip-pubsub-binding \
    --no-backup >/dev/null 2>&1; then
  echo "FAIL: expected invalid master key setup to fail" >&2
  exit 1
fi

echo "OK: setup-gsm-master-key script tests passed."
