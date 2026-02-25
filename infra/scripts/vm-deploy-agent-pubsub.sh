#!/usr/bin/env bash
set -euo pipefail

# Pull-based deploy agent:
# - GitHub Actions publishes deploy requests to a Pub/Sub topic (no SSH).
# - This agent runs on the VM, pulls messages from a subscription, runs the existing
#   zero-downtime deploy script, and ACKs messages only after success.

log() {
  printf '[%s] %s\n' "$(date -u +'%Y-%m-%dT%H:%M:%SZ')" "$*"
}

cleanup_pid() {
  local pid="${1:-}"
  if [[ -z "$pid" ]]; then
    return 0
  fi
  kill "$pid" >/dev/null 2>&1 || true
  wait "$pid" >/dev/null 2>&1 || true
}

die() {
  log "ERROR: $*"
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "Missing required command: $1"
}

require_env() {
  local name="$1"
  if [[ -z "${!name:-}" ]]; then
    die "Missing required env var: $name"
  fi
}

is_truthy() {
  case "${1:-}" in
    true|1|yes)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

main() {
  require_cmd gcloud
  require_cmd git
  require_cmd python3
  require_cmd flock

  require_env GCP_PROJECT_ID
  require_env DEPLOY_SUBSCRIPTION

  local repo_dir="${REPO_DIR:-/opt/bominal/repo}"
  local canonical_deploy_script="$repo_dir/infra/scripts/deploy.sh"
  local deploy_script="${DEPLOY_SCRIPT:-$canonical_deploy_script}"
  local allow_noncanonical_deploy_script="${ALLOW_NONCANONICAL_DEPLOY_SCRIPT:-false}"
  local gcp_region="${GCP_REGION:-us-central1}"
  local sleep_seconds="${SLEEP_SECONDS:-5}"
  local lock_file="${LOCK_FILE:-/tmp/bominal-deploy.lock}"
  local ack_deadline_seconds="${ACK_DEADLINE_SECONDS:-600}"
  local ack_extend_interval_seconds="${ACK_EXTEND_INTERVAL_SECONDS:-60}"
  local once="${DEPLOY_AGENT_ONCE:-0}"

  local script_dir
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  local parser_script="${PARSER_SCRIPT:-$script_dir/pubsub_parse.py}"

  if [[ ! -d "$repo_dir" ]]; then
    die "Repo directory not found: $repo_dir"
  fi

  if [[ "$deploy_script" != "$canonical_deploy_script" ]] && ! is_truthy "$allow_noncanonical_deploy_script"; then
    die "Non-canonical DEPLOY_SCRIPT is blocked: $deploy_script (required: $canonical_deploy_script; set ALLOW_NONCANONICAL_DEPLOY_SCRIPT=true for test-only overrides)"
  fi

  if [[ ! -f "$deploy_script" ]]; then
    die "Deploy script not found: $deploy_script"
  fi

  if [[ ! -f "$parser_script" ]]; then
    die "Parser script not found: $parser_script"
  fi

  log "bominal deploy agent starting"
  log "  project:      $GCP_PROJECT_ID"
  log "  region:       $gcp_region"
  log "  subscription: $DEPLOY_SUBSCRIPTION"
  log "  repo_dir:     $repo_dir"
  log "  deploy_script:$deploy_script"
  log "  lock_file:    $lock_file"
  log "  ack_deadline: $ack_deadline_seconds"
  log "  ack_interval: $ack_extend_interval_seconds"
  log "  once:         $once"

  # Lock file descriptor 9 is reserved for the deploy lock.
  exec 9>"$lock_file"

  # Ensure only one agent instance runs at a time.
  flock 9
  log "Deploy lock acquired (agent singleton)"

  while true; do
    # Pull one message (do NOT auto-ack).
    local pull_json
    pull_json="$(gcloud pubsub subscriptions pull "$DEPLOY_SUBSCRIPTION" \
      --project="$GCP_PROJECT_ID" \
      --limit=1 \
      --format=json 2>/dev/null || true)"

    if [[ -z "$pull_json" || "$pull_json" == "[]" ]]; then
      sleep "$sleep_seconds"
      continue
    fi

    # Extract ackId + attributes (mode/commit) via a shared, tested parser.
    local ack_id deploy_mode deploy_commit
    local deploy_api_image deploy_worker_image deploy_web_image
    local deploy_api_gateway_image deploy_api_train_image deploy_api_restaurant_image
    local deploy_worker_train_image deploy_worker_restaurant_image
    local parsed
    if ! parsed="$(python3 "$parser_script" <<<"$pull_json")"; then
      log "WARN: pulled message but parser failed; sleeping"
      sleep "$sleep_seconds"
      continue
    fi
    eval "$parsed"

    if [[ -z "${ACK_ID:-}" ]]; then
      log "WARN: pulled message but could not parse ackId; sleeping"
      sleep "$sleep_seconds"
      continue
    fi

    ack_id="$ACK_ID"
    deploy_mode="${DEPLOY_MODE:-latest}"
    deploy_commit="${DEPLOY_COMMIT:-}"
    deploy_api_image="${DEPLOY_API_IMAGE:-}"
    deploy_worker_image="${DEPLOY_WORKER_IMAGE:-}"
    # Backward compatibility with split-runtime payload vars.
    deploy_api_gateway_image="${DEPLOY_API_GATEWAY_IMAGE:-}"
    deploy_api_train_image="${DEPLOY_API_TRAIN_IMAGE:-}"
    deploy_api_restaurant_image="${DEPLOY_API_RESTAURANT_IMAGE:-}"
    deploy_worker_train_image="${DEPLOY_WORKER_TRAIN_IMAGE:-}"
    deploy_worker_restaurant_image="${DEPLOY_WORKER_RESTAURANT_IMAGE:-}"
    deploy_web_image="${DEPLOY_WEB_IMAGE:-}"

    if [[ -z "$deploy_api_image" ]]; then
      deploy_api_image="${deploy_api_gateway_image:-${deploy_api_train_image:-${deploy_api_restaurant_image:-}}}"
    fi
    if [[ -z "$deploy_worker_image" ]]; then
      deploy_worker_image="${deploy_worker_train_image:-${deploy_worker_restaurant_image:-${deploy_api_image:-}}}"
    fi

    log "Received deploy request (mode=$deploy_mode, commit=${deploy_commit:-none})"

    # Keep extending the ack deadline while deploying; Pub/Sub max is 600s but
    # can be extended repeatedly. We ACK only after a successful deploy.
    local keepalive_pid=""
    gcloud pubsub subscriptions modify-ack-deadline "$DEPLOY_SUBSCRIPTION" \
      --project="$GCP_PROJECT_ID" \
      --ack-ids="$ack_id" \
      --ack-deadline="$ack_deadline_seconds" >/dev/null 2>&1 || true
    (
      while true; do
        # Avoid spawning an external `sleep` process so cleanup can reliably
        # terminate this loop without leaving orphan children holding stdout/stderr.
        read -r -t "$ack_extend_interval_seconds" _ || true
        gcloud pubsub subscriptions modify-ack-deadline "$DEPLOY_SUBSCRIPTION" \
          --project="$GCP_PROJECT_ID" \
          --ack-ids="$ack_id" \
          --ack-deadline="$ack_deadline_seconds" >/dev/null 2>&1 || true
      done
    ) &
    keepalive_pid="$!"

    {
      # Update repo and align infra config to the requested commit (or origin/main).
      if git -C "$repo_dir" fetch origin --prune; then
        :
      else
        log "WARN: git fetch origin failed; continuing with existing repo state"
      fi

      # Latest-only deploys: always prefer the infra/scripts on origin/main.
      # If local tracked/untracked edits block checkout, auto-stash and retry so
      # deploys do not silently stay pinned to stale infrastructure code.
      if git -C "$repo_dir" checkout --detach origin/main >/dev/null 2>&1; then
        log "Repo aligned to origin/main ($(git -C "$repo_dir" rev-parse --short HEAD 2>/dev/null || echo unknown))"
      else
        log "WARN: checkout --detach origin/main failed; attempting auto-stash and retry"
        git -C "$repo_dir" stash push -u -m "deploy-agent-auto-stash-$(date -u +%Y%m%dT%H%M%SZ)" >/dev/null 2>&1 || true
        if git -C "$repo_dir" checkout --detach origin/main >/dev/null 2>&1; then
          log "Repo aligned to origin/main after auto-stash ($(git -C "$repo_dir" rev-parse --short HEAD 2>/dev/null || echo unknown))"
        else
          log "WARN: repo checkout to origin/main still failing; continuing with existing repo state"
        fi
      fi

      # Run deploy. Use "latest" by default to avoid missing image tags when only one image was rebuilt.
      export GCP_PROJECT_ID
      export GCP_REGION="$gcp_region"

      if [[ -n "$deploy_api_image" ]]; then export API_IMAGE="$deploy_api_image"; fi
      if [[ -n "$deploy_worker_image" ]]; then export WORKER_IMAGE="$deploy_worker_image"; fi
      if [[ -n "$deploy_web_image" ]]; then export WEB_IMAGE="$deploy_web_image"; fi

      log "Running deploy script (latest baseline with optional api/worker/web image overrides)"
      bash "$deploy_script"

      # ACK only after successful deploy.
      gcloud pubsub subscriptions ack "$DEPLOY_SUBSCRIPTION" \
        --project="$GCP_PROJECT_ID" \
        --ack-ids="$ack_id" >/dev/null

      log "Deploy completed; message ACKed"
    } || {
      # Do not ACK on failure. Pub/Sub will redeliver.
      log "ERROR: deploy failed; message NOT ACKed (will be retried)"
      sleep 10
    }

    cleanup_pid "$keepalive_pid"

    if [[ "$once" == "1" ]]; then
      log "Once mode enabled; exiting"
      exit 0
    fi
  done
}

main "$@"
