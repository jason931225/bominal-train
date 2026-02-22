#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/infra/scripts/deploy.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p "$TMP_DIR/bin" "$TMP_DIR/history"

cat >"$TMP_DIR/bin/docker" <<'DOCKER'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == "compose" && "${2:-}" == "version" ]]; then
  exit 0
fi
if [[ "${1:-}" == "compose" ]]; then
  if [[ "$*" == *"ps --services --filter status=running"* ]]; then
    printf 'api-gateway\nweb\n'
    exit 0
  fi
  exit 0
fi
if [[ "${1:-}" == "ps" ]]; then
  echo "NAMES"
  exit 0
fi
exit 0
DOCKER
chmod +x "$TMP_DIR/bin/docker"

# Seed required pointers for --status.
echo "currentcommit" > "$TMP_DIR/history/current"
echo "oldcommit" > "$TMP_DIR/history/previous"

# Malicious deployment record that should never be executed.
cat >"$TMP_DIR/history/20260222_120000" <<EOF_RECORD
commit='deadbeef'
timestamp='20260222_120000'
api_digest='api@sha256:aaa'
web_digest='web@sha256:bbb'
deployed_by='tester'
previous='oldcommit'
injected=\$(touch "$TMP_DIR/pwned")
EOF_RECORD

PATH="$TMP_DIR/bin:$PATH" \
  REPO_DIR="$ROOT_DIR" \
  DEPLOY_HISTORY_DIR="$TMP_DIR/history" \
  DEPLOY_LOCK_FILE="$TMP_DIR/deploy.lock" \
  bash "$SCRIPT" --status >/dev/null

if [[ -f "$TMP_DIR/pwned" ]]; then
  echo "FAIL: deployment record shell content was executed" >&2
  exit 1
fi

echo "OK: deploy history records are treated as data, not executable shell."
