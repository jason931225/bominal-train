#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/infra/scripts/free_tier_status_report.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p "$TMP_DIR/bin" "$TMP_DIR/repo/infra/env/prod"

cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'ENV'
GCP_PROJECT_ID=test-project
GSM_MASTER_KEY_SECRET_ID=bominal-master-key
EVERVAULT_APP_ID_SECRET_ID=bominal-evervault-app-id
EVERVAULT_API_KEY_SECRET_ID=bominal-evervault-api-key
INTERNAL_API_KEY_SECRET_ID=bominal-internal-api-key
RESEND_API_KEY_SECRET_ID=bominal-resend-api-key
SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID=bominal-supabase-management-api-token
ENV

cat >"$TMP_DIR/bin/gcloud" <<'GCLOUD'
#!/usr/bin/env bash
set -euo pipefail

if [[ "$*" == *"scheduler jobs list"* ]]; then
  printf 'daily-drift\n'
  printf 'daily-budget\n'
  printf 'hourly-smoke\n'
  exit 0
fi

if [[ "$*" == *"secrets versions list"* ]]; then
  # Return 2 enabled versions for each secret.
  printf '1\n2\n'
  exit 0
fi

exit 0
GCLOUD
chmod +x "$TMP_DIR/bin/gcloud"

REPORT_PATH="$TMP_DIR/report.md"
PATH="$TMP_DIR/bin:$PATH" BOMINAL_ROOT_DIR="$TMP_DIR/repo" bash "$SCRIPT" --output "$REPORT_PATH" >/dev/null

if [[ ! -s "$REPORT_PATH" ]]; then
  echo "FAIL: report file was not generated" >&2
  exit 1
fi

if ! rg -q '^# Free-Tier Status Report' "$REPORT_PATH"; then
  echo "FAIL: report header missing" >&2
  cat "$REPORT_PATH" >&2
  exit 1
fi

if ! rg -q 'Cloud Scheduler jobs \(limit 3\):' "$REPORT_PATH"; then
  echo "FAIL: scheduler summary missing" >&2
  cat "$REPORT_PATH" >&2
  exit 1
fi

if ! rg -q 'count=3, percent=100\.00, status=HARD_ALERT' "$REPORT_PATH"; then
  echo "FAIL: scheduler status computation mismatch" >&2
  cat "$REPORT_PATH" >&2
  exit 1
fi

if ! rg -q '\| bominal-master-key \| 2 \| 33\.33 \| OK \|' "$REPORT_PATH"; then
  echo "FAIL: secret version table row missing/incorrect" >&2
  cat "$REPORT_PATH" >&2
  exit 1
fi

echo "OK: free_tier_status_report script tests passed."
