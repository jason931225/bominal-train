#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/infra/scripts/sync-supabase-auth-templates.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

assert_fails() {
  local msg="$1"
  shift
  if "$@" >/dev/null 2>&1; then
    echo "FAIL: expected failure - $msg" >&2
    exit 1
  fi
}

mkdir -p \
  "$TMP_DIR/bin" \
  "$TMP_DIR/repo/infra/supabase/auth-templates" \
  "$TMP_DIR/repo/infra/env/prod"

cat >"$TMP_DIR/repo/infra/supabase/auth-templates/confirm-signup.html" <<'EOF'
<a href="{{ .ConfirmationURL }}">Verify</a>
<p>{{ .Token }}</p>
EOF

cat >"$TMP_DIR/repo/infra/supabase/auth-templates/reset-password.html" <<'EOF'
<a href="{{ .ConfirmationURL }}">Reset</a>
<p>{{ .Token }}</p>
EOF

cat >"$TMP_DIR/repo/infra/supabase/auth-templates/subjects.json" <<'EOF'
{
  "confirmation": "Verify your email for bominal",
  "recovery": "Reset your bominal password"
}
EOF

cat >"$TMP_DIR/repo/infra/env/prod/api.env" <<'EOF'
SUPABASE_URL=https://test-project.supabase.co
EOF

cat >"$TMP_DIR/repo/infra/env/prod/web.env" <<'EOF'
NEXT_PUBLIC_API_BASE_URL=https://www.bominal.com
EOF

cat >"$TMP_DIR/repo/infra/env/prod/caddy.env" <<'EOF'
CADDY_SITE_ADDRESS=www.bominal.com
EOF

cat >"$TMP_DIR/bin/curl" <<'CURL'
#!/usr/bin/env bash
set -euo pipefail

touch "${TMP_TEST_ROOT:?}/curl.called"

out_file=""
write_format=""
method="GET"
url=""
data_arg=""
headers=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    -o)
      out_file="$2"
      shift 2
      ;;
    -w)
      write_format="$2"
      shift 2
      ;;
    -X)
      method="$2"
      shift 2
      ;;
    -H|--header)
      headers+=("$2")
      shift 2
      ;;
    --data-binary)
      data_arg="$2"
      shift 2
      ;;
    -s|-S|-sS)
      shift
      ;;
    http*)
      url="$1"
      shift
      ;;
    *)
      shift
      ;;
  esac
done

if [[ -n "${CURL_ARGS_FILE:-}" ]]; then
  {
    printf 'method=%s\n' "$method"
    printf 'url=%s\n' "$url"
    printf 'data=%s\n' "$data_arg"
    for header in "${headers[@]}"; do
      printf 'header=%s\n' "$header"
    done
  } >"$CURL_ARGS_FILE"
fi

if [[ -n "${CURL_HEADER_DUMP_FILE:-}" ]]; then
  : >"$CURL_HEADER_DUMP_FILE"
  for header in "${headers[@]}"; do
    if [[ "$header" == @* ]]; then
      cat "${header#@}" >>"$CURL_HEADER_DUMP_FILE"
      printf '\n' >>"$CURL_HEADER_DUMP_FILE"
    else
      printf '%s\n' "$header" >>"$CURL_HEADER_DUMP_FILE"
    fi
  done
fi

if [[ "$data_arg" == @* && -n "${CURL_PAYLOAD_FILE:-}" ]]; then
  cp "${data_arg#@}" "$CURL_PAYLOAD_FILE"
fi

if [[ -n "$out_file" ]]; then
  printf '{"ok":true}\n' >"$out_file"
fi

if [[ "$write_format" == *"%{http_code}"* ]]; then
  printf '%s' "${MOCK_HTTP_CODE:-200}"
fi
CURL
chmod +x "$TMP_DIR/bin/curl"

# Dry run should not require token and must not call curl.
env \
  PATH="$TMP_DIR/bin:$PATH" \
  TMP_TEST_ROOT="$TMP_DIR" \
  BOMINAL_ROOT_DIR="$TMP_DIR/repo" \
  "$SCRIPT" --dry-run >/dev/null

if [[ -f "$TMP_DIR/curl.called" ]]; then
  echo "FAIL: curl was called during --dry-run" >&2
  exit 1
fi

# Apply should patch endpoint with expected payload and header behavior.
env \
  PATH="$TMP_DIR/bin:$PATH" \
  TMP_TEST_ROOT="$TMP_DIR" \
  BOMINAL_ROOT_DIR="$TMP_DIR/repo" \
  SUPABASE_MANAGEMENT_API_TOKEN="secret-token" \
  CURL_ARGS_FILE="$TMP_DIR/curl.args" \
  CURL_HEADER_DUMP_FILE="$TMP_DIR/curl.headers" \
  CURL_PAYLOAD_FILE="$TMP_DIR/curl.payload.json" \
  "$SCRIPT" --apply >/dev/null

if ! rg -q '^method=PATCH$' "$TMP_DIR/curl.args"; then
  echo "FAIL: expected PATCH method in curl args" >&2
  cat "$TMP_DIR/curl.args" >&2
  exit 1
fi

if ! rg -q '^url=https://api\.supabase\.com/v1/projects/test-project/config/auth$' "$TMP_DIR/curl.args"; then
  echo "FAIL: expected Supabase auth config endpoint URL" >&2
  cat "$TMP_DIR/curl.args" >&2
  exit 1
fi

python3 - "$TMP_DIR/curl.payload.json" <<'PY'
import json
import sys
from pathlib import Path

payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
required = {
    "mailer_subjects_confirmation",
    "mailer_subjects_recovery",
    "mailer_templates_confirmation_content",
    "mailer_templates_recovery_content",
    "site_url",
    "uri_allow_list",
}
missing = required.difference(payload.keys())
if missing:
    raise SystemExit(f"Missing payload keys: {sorted(missing)}")

if payload["mailer_subjects_confirmation"] != "Verify your email for bominal":
    raise SystemExit("Unexpected confirmation subject")
if payload["mailer_subjects_recovery"] != "Reset your bominal password":
    raise SystemExit("Unexpected recovery subject")
if "{{ .ConfirmationURL }}" not in payload["mailer_templates_confirmation_content"]:
    raise SystemExit("Confirmation template missing ConfirmationURL placeholder")
if "{{ .Token }}" not in payload["mailer_templates_recovery_content"]:
    raise SystemExit("Recovery template missing Token placeholder")
if payload["site_url"] != "https://www.bominal.com":
    raise SystemExit("Unexpected Supabase site_url")
if "/auth/callback" not in payload["uri_allow_list"]:
    raise SystemExit("uri_allow_list missing auth callback path")
if "/reset-password" not in payload["uri_allow_list"]:
    raise SystemExit("uri_allow_list missing reset-password path")
PY

if ! rg -q 'Authorization: Bearer secret-token' "$TMP_DIR/curl.headers"; then
  echo "FAIL: authorization header not passed to curl" >&2
  cat "$TMP_DIR/curl.headers" >&2
  exit 1
fi

if rg -q 'secret-token' "$TMP_DIR/curl.args"; then
  echo "FAIL: raw token leaked into curl args" >&2
  cat "$TMP_DIR/curl.args" >&2
  exit 1
fi

# Apply without token should fail.
assert_fails \
  "missing management token should fail apply mode" \
  env PATH="$TMP_DIR/bin:$PATH" TMP_TEST_ROOT="$TMP_DIR" BOMINAL_ROOT_DIR="$TMP_DIR/repo" "$SCRIPT" --apply

echo "OK: supabase auth template sync script tests passed."
