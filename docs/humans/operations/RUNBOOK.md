# Runbook

Operational procedures for local/dev/prod maintenance.

## Service commands

### VM production (Debian 12 + Docker deploy)

**Shell Aliases** (available after SSH login):

| Alias | Command |
|-------|---------|
| `bominal-deploy` | Zero-downtime deployment |
| `bominal-admin` | Admin CLI for user/task/db management |

Deploy with zero-downtime (recommended):

```bash
bominal-deploy
# Or: sudo -u bominal /opt/bominal/repo/infra/scripts/deploy.sh
```

Rollback to previous deployment:

```bash
bominal-deploy --rollback
# Or: sudo -u bominal /opt/bominal/repo/infra/scripts/deploy.sh --rollback
```

Note: `deploy.sh` pulls pre-built GHCR images (there is no `--skip-build` flag).
If GHCR packages are private, provide `GHCR_USERNAME` + `GHCR_TOKEN` in the deploy agent env.

Deploy script guardrails:
- Enforces single-run deploy lock (`DEPLOY_LOCK_FILE`, default `/tmp/bominal-deploy.lock`).
- Runs strict preflight checks before pull/deploy (env + compose + memory/swap threshold).
- Enforces deprecation registry gate for production-scoped removals/deadlines.
- Chooses bootstrap-safe first deploy path when no stack is running.
- Uses image-aware rolling updates on running stacks (skip unchanged per-service image groups: api/worker/web).
- Auto rollback on smoke failure is enabled by default (`AUTO_ROLLBACK_ON_SMOKE_FAILURE=true`).
- Logs tracked working-tree dirty state before deploy; set `DEPLOY_FAIL_ON_DIRTY_REPO=true` to hard-fail on dirty tracked files.
- GitHub deploy workflow guardrails:
  - auto deploy request is published only for GitHub Release `published` events (tagged releases), with optional manual dispatch;
  - deploy request is published only after same-commit `CI - Infra Quality Gates` and `CI - Build and Publish Images` both succeed;
  - image publish is blocked when Trivy reports `HIGH` or `CRITICAL` vulnerabilities on pushed images;
  - deploy payload pins changed services to commit-tagged GHCR images and leaves unchanged services on latest baseline;
  - post-deploy CI verification checks production `/health` (`db=true`, `redis=true`) and production web endpoint HTTP `200`/`3xx`.
- VM deploy agent guardrail:
  - `DEPLOY_SCRIPT` must point to canonical `infra/scripts/deploy.sh`; deprecated script paths are rejected fail-closed.

Compatibility notice:
- `infra/docker-compose.deploy.yml.deprecated` is deprecated and removed from active operator workflow.
- Use `infra/docker-compose.prod.yml` and `infra/scripts/deploy.sh` exclusively.
- Deprecation lifecycle policy is tracked in `docs/governance/DEPRECATION_POLICY.md` and `docs/deprecations/registry.json`.

Quick restart after VM reset (no rebuild, existing images):

```bash
sudo -u bominal /opt/bominal/repo/infra/scripts/quick-restart.sh
```

Check VM stack:

```bash
docker compose -f infra/docker-compose.prod.yml ps
docker compose -f infra/docker-compose.prod.yml logs -f caddy api worker web
```

### Docker (local simulation)

Start stack (dev):

```bash
docker compose -f infra/docker-compose.yml up --build
```

Run frontend E2E with dedicated Chromium profile:

```bash
docker compose -f infra/docker-compose.yml --profile e2e run --rm --build web-e2e
```

`infra/scripts/local-run.sh` and `infra/scripts/local-check.sh` now invoke compose with `--remove-orphans` to prevent stale-service warning drift when switching stacks.

If you see orphan warnings after switching compose files (for example, legacy `bominal-caddy`), run one-time cleanup:

```bash
docker compose -p bominal -f infra/docker-compose.yml down --remove-orphans || true
docker compose -p bominal -f infra/docker-compose.prod.yml down --remove-orphans || true
```

Start stack (prod profile file):

```bash
docker compose -f infra/docker-compose.prod.yml up -d --build
```

Stop stack:

```bash
docker compose -f infra/docker-compose.yml down
```

Hard reset (destroys local DB volume):

```bash
docker compose -f infra/docker-compose.yml down -v
```

## Observability quick checks

API/web health:

```bash
curl -sS http://localhost:8000/health
curl -sS -I http://localhost:3000
```

Production health (via Caddy on 80/443):

```bash
curl -sS -I http://localhost
curl -sS https://localhost/health -k
```

Container status/logs:

```bash
docker compose -f infra/docker-compose.yml ps
docker compose -f infra/docker-compose.yml logs -f api worker web
```

Production status/logs:

```bash
docker compose -f infra/docker-compose.prod.yml ps
docker compose -f infra/docker-compose.prod.yml logs -f caddy api worker web
```

Performance pressure quick triage (production):

```bash
# Host pressure snapshot
uptime
free -h

# Top containers by CPU/memory
docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}"

# Worker restart + health
docker inspect -f '{{.Name}} restart={{.RestartCount}} status={{.State.Status}} health={{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}' bominal-worker

# Zombie count on host
ps -eo stat | awk '$1 ~ /^Z/ {c++} END {print c+0}'

# Recent worker timeout signals
docker logs --since=30m bominal-worker 2>&1 | grep -c "Timeout connecting to server" || true
docker logs --since=30m bominal-worker 2>&1 | grep -c "redis connection error" || true
```

Security checks for payment/CDE runtime:

```bash
# Confirm Redis split config (non-CDE vs CDE) in API runtime.
docker compose -f infra/docker-compose.prod.yml exec api env | rg 'REDIS_URL(_NON_CDE|_CDE)?'

# Confirm effective CDE Redis endpoint is not Upstash-hosted.
docker compose -f infra/docker-compose.prod.yml exec api python -c "from app.core.config import get_settings,is_upstash_redis_url; s=get_settings(); print('resolved_redis_url_cde=', s.resolved_redis_url_cde); assert not is_upstash_redis_url(s.resolved_redis_url_cde)"

# Confirm Redis persistence is disabled for CDE runtime.
docker compose -f infra/docker-compose.prod.yml exec redis redis-cli CONFIG GET save
docker compose -f infra/docker-compose.prod.yml exec redis redis-cli CONFIG GET appendonly

# Validate provider egress allowlist and timeout envs are set.
docker compose -f infra/docker-compose.prod.yml exec api env | rg 'PAYMENT_PROVIDER_ALLOWED_HOSTS|TRAIN_PROVIDER_TIMEOUT_|PAYMENT_TRANSPORT_TRUST_ENV|PROVIDER_EGRESS_PROXY_URL'

# Confirm egress gateways are healthy and deny unknown routes by default.
docker compose -f infra/docker-compose.prod.yml exec egress-train wget --spider -q http://127.0.0.1:8080/health
docker compose -f infra/docker-compose.prod.yml exec egress-train wget -qO- --server-response http://127.0.0.1:8080/not-allowed 2>&1 | rg '403'

# Confirm payment logs do not include request/response payload bodies.
docker compose -f infra/docker-compose.prod.yml logs --since=30m api worker | rg -i 'card_number|cvv|authorization|set-cookie|wrapped_dek|ciphertext'

# One-time cleanup for legacy CVV cache keys from older releases (safe to re-run).
docker compose -f infra/docker-compose.prod.yml exec api python -m app.admin_cli secret purge-payment-cvv --yes
```

Hosted service budget checks (non-CDE Upstash + Supabase free-tier planning):

- Upstash (non-CDE Redis only):
  - Commands/month: `500000`
  - Bandwidth/month: `50 GB`
  - Storage: `256 MB`
  - Requests/second: `10000`
- Supabase:
  - DB size: `0.5 GB`
  - Egress: `5 GB`
  - Cached egress: `5 GB`
  - MAU: `50000` (first-party), `50000` (third-party)
  - Storage: `1 GB`
  - Realtime peak connections: `200`
  - Realtime messages: `2000000`
  - Edge invocations/month: `500000`

Operational policy:

- Keep CDE Redis on a separate non-Upstash endpoint (`REDIS_URL_CDE`).
- Keep Upstash traffic for queue/rate-limit/non-sensitive cache only (`REDIS_URL_NON_CDE`).
- Set warning thresholds at 70% and hard alerts at 85% of each limit.
- If any hard alert is reached, disable non-critical background jobs and increase polling intervals before quota exhaustion.

Task-list latency benchmark (backend API):

```bash
# Host execution against local API
infra/scripts/benchmark-train-task-list.sh \
  --base-url http://localhost:8000 \
  --iterations 30 \
  --active-limit 60 \
  --completed-limit 80
```

If running from a containerized shell where API is on the compose network:

```bash
infra/scripts/benchmark-train-task-list.sh --base-url http://api:8000
```

Hybrid benchmark gate check (relative improvement + absolute SLO ceilings):

```bash
infra/scripts/benchmark-train-task-list-compare.sh \
  --baseline-json infra/benchmarks/train-task-list-baseline.json \
  --run-live \
  --relative-p95-min-improvement 15 \
  --relative-mean-min-improvement 10 \
  --absolute-p95-max 12 \
  --absolute-mean-max 10 \
  -- \
  --base-url http://localhost:8000 \
  --iterations 30 \
  --active-limit 60 \
  --completed-limit 80
```

Live system monitor (production):

```bash
/opt/bominal/repo/infra/scripts/bominal-monitor --watch
```

Admin CLI (production):

```bash
# List users
bominal-admin user list

# User info
bominal-admin user info <user_id>

# Promote to admin
bominal-admin user set-role <email> admin

# List tasks
bominal-admin task list

# DB stats
bominal-admin db stats

# Check encryption status
bominal-admin secret check

# Prepare a new primary KEK version (prints env payload)
bominal-admin secret prepare-kek-rotation --new-version <N>

# KEK rotation dry-run / execute
bominal-admin secret rotate-kek --dry-run
bominal-admin secret rotate-kek --yes

# Continuous batch rewrap in background
bominal-admin secret rotate-kek-background --yes --batch-size 200 --sleep-seconds 0.5

# Retire an old KEK only after rewrap completion + retention window
bominal-admin secret retire-kek --version <OLD> --rotation-completed-at <UTC_ISO8601> --yes
```

Rapid successive KEK rotations:
- Each deploy uses current `KEK_VERSION` as the write key for new wraps.
- Keep all intermediate/old versions in `MASTER_KEYS_BY_VERSION` until the latest rewrap completes.
- If a new rotation is introduced before prior rewrap completion, rerun `rotate-kek-background` after the newest deploy so all rows converge to the newest `KEK_VERSION`.
- Do not retire any old version until `retire-kek` confirms zero references + retention window elapsed.

DB checks:

```bash
docker compose -f infra/docker-compose.yml exec postgres \
  psql -U bominal -d bominal \
  -c "select id, state, created_at, completed_at from tasks order by created_at desc limit 20;"
```

```bash
docker compose -f infra/docker-compose.yml exec postgres \
  psql -U bominal -d bominal \
  -c "select task_id, action, provider, ok, retryable, started_at from task_attempts order by started_at desc limit 30;"
```

## Common incidents

## 0) Local DB reset for performance testing

High-risk reset workflows (local/dev only):

```bash
# Preserve sign-in credentials when available (default):
# users.email, users.display_name, users.password_hash
infra/scripts/reset-local-db.sh --yes

# Fresh schema rebuild + migration replay + credential restore
infra/scripts/reset-local-db.sh --fresh-schema --yes
```

Notes:
- `--yes` is required.
- `--no-preserve-signin` drops all user sign-in credentials.
- Script blocks prod compose files unless `--allow-non-dev` is explicitly provided.

## 0) VM docker deploy unhealthy after deploy

Checklist:

1. `docker compose -f infra/docker-compose.prod.yml ps`
2. `docker compose -f infra/docker-compose.prod.yml logs --tail=200 caddy api worker web`
3. Verify env files exist:
   - `infra/env/prod/api.env`
   - `infra/env/prod/web.env`
4. Re-run deploy script:

```bash
sudo -u bominal /opt/bominal/repo/infra/scripts/deploy.sh
```

Or rollback if previous deployment was stable:

```bash
sudo -u bominal /opt/bominal/repo/infra/scripts/deploy.sh --rollback
```

## 0.1) VM restart / GCE reset recovery

After an abrupt VM restart (GCE preemption, maintenance, etc.), containers stop but images remain cached.

Quick recovery without rebuilding:

```bash
sudo -u bominal /opt/bominal/repo/infra/scripts/quick-restart.sh
```

This starts containers in the correct order using existing images. For a specific service only:

```bash
sudo -u bominal /opt/bominal/repo/infra/scripts/quick-restart.sh api
```

Verify after restart:

```bash
curl -sS https://www.bominal.com/health
/opt/bominal/repo/infra/scripts/bominal-monitor
```

## 0.2) Deploy exits early (lock contention or preflight gate)

Symptoms:

- `Another deployment is already running`
- `Preflight checks failed. Deployment aborted before pull/deploy.`
- `Insufficient total memory` or `Insufficient total swap`
- `Deprecation deploy gate failed`

Checks:

1. Confirm no duplicate deploy command is active (`ps -ef | grep deploy.sh`).
2. Validate swap/memory on VM:

```bash
free -m
swapon --show
```

3. Run preflight manually for explicit output:

```bash
bash infra/scripts/predeploy-check.sh \
  --min-total-memory-mb 900 \
  --min-total-swap-mb 900 \
  --skip-smoke-tests
```

Recovery:

1. If lock contention is legitimate, wait for active deploy completion and retry.
2. If swap is missing, run `infra/scripts/vm-docker-bootstrap.sh` or recreate swap, then retry deploy.
3. If deprecation gate fails, fix/remove overdue references and update `docs/deprecations/registry.json`.
4. Emergency-only bypass (approval required):

```bash
PREDEPLOY_ALLOW_DEPRECATION_BYPASS=true \
  bash infra/scripts/predeploy-check.sh --skip-smoke-tests
```

## 0.3) Emergency cost-stop actions (GCP)

Use this only when you need to stop unexpected spend quickly and can accept temporary service disruption.

Rapid audit checklist:

1. Open Billing Reports and group by SKU to identify active charge sources.
2. Confirm Compute Engine free-tier profile:
   - one `e2-micro` VM only
   - zone in `us-central1`, `us-east1`, or `us-west1`
   - standard persistent disk within free allocation
3. Check for non-free resources:
   - extra VMs/disks/snapshots
   - unused reserved external IPs
   - unexpected Pub/Sub/IAP/egress traffic spikes
   - managed services not covered by free tier

Hard-stop operations used in incident response:

```bash
# Secret Manager hard stop
gcloud secrets list --project "$GCP_PROJECT_ID"
gcloud secrets delete <secret-name> --project "$GCP_PROJECT_ID" --quiet
gcloud services disable secretmanager.googleapis.com --project "$GCP_PROJECT_ID" --quiet
```

Post-action validation:

```bash
gcloud services list --enabled --project "$GCP_PROJECT_ID" \
  --filter='name:secretmanager.googleapis.com'
```

Impact notes:

1. Disabling Secret Manager breaks any runtime/bootstrap path that reads secrets from Secret Manager.
2. Container image publish/pull is GHCR-backed; review GHCR package retention and pull frequency for registry-side cost control.
3. If immediate cost stop is required and source is still unclear, disable project billing as a final emergency step.

## 0.4) High load with web zombie buildup or worker timeout noise

Symptoms:

- load average remains elevated while traffic is normal
- zombie count trends upward
- worker logs contain repeated `Timeout connecting to server` entries

Checks:

1. Confirm top resource consumers:
   - `docker stats --no-stream ...`
2. Confirm zombie parent process:
   - `ps -eo stat,ppid,pid,user,comm,args | awk '$1 ~ /^Z/ {print}' | head`
3. Confirm worker restart stability:
   - `docker inspect -f '{{.RestartCount}}' bominal-worker`

Recovery:

1. Recreate only web service to reset stale zombie process trees:

```bash
sudo -u bominal docker compose -f /opt/bominal/repo/infra/docker-compose.prod.yml up -d --no-deps --force-recreate web
```

2. Verify web and worker health after recreate:

```bash
sudo -u bominal docker compose -f /opt/bominal/repo/infra/docker-compose.prod.yml ps --format "table {{.Name}}\t{{.Status}}" web worker
ps -eo stat | awk '$1 ~ /^Z/ {c++} END {print c+0}'
docker inspect -f '{{.Name}} restart={{.RestartCount}} status={{.State.Status}} health={{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}' bominal-worker
```

## 1) Web route fails to load (`Cannot find module './901.js'`)

Cause: stale/corrupt Next build cache.

Fix:

```bash
docker compose -f infra/docker-compose.yml exec web sh -lc "cd /app && rm -rf .next && npm run dev -- -H 0.0.0.0 -p 3000"
```

If still broken, restart web container.

## 2) API healthy but Train task states not moving

Checklist:

1. Verify worker is running (`docker compose ... ps`).
2. Check worker logs for provider/auth/rate-limit failures.
3. Ensure Redis is healthy.
4. Verify task is not paused/cancelled/expired.

Recovery:

```bash
docker compose -f infra/docker-compose.yml restart worker
```

Worker startup automatically re-enqueues recoverable tasks from DB.

## 3) Provider credential verification appears stuck

Credential checks time out at `TRAIN_CREDENTIAL_VERIFY_TIMEOUT_SECONDS` (default 8s).

Actions:

1. Inspect API logs for provider timeout/auth errors.
2. Confirm outbound networking from API container.
3. Re-save provider credentials from UI.
4. Use `TRAIN_PROVIDER_MODE=mock` to isolate provider/network issues.

## 4) Search returns no schedules unexpectedly

Actions:

1. Confirm provider credentials are verified.
2. Check selected providers and station names.
3. Inspect API logs for provider-specific error codes.
4. Validate station mapping behavior through `/api/train/stations`.

## 4.1) SRT reservation expired or no longer found

Primary expiry indicator (SRT reservation list payload):

1. `stlFlg == "N"` (unpaid)
2. `now(KST) > iseLmtDt + iseLmtTm` (payment cutoff passed)

Secondary confirmation indicators:

1. `selectListAtc14016_n.do` no longer returns the `pnrNo` (or `rowCnt: 0`)
2. `getListAtc14087.do` returns `조회자료가 없습니다.`

Bominal behavior:

1. Ticket sync sets status to `expired` for unpaid cutoff-passed reservations.
2. Manual pay rejects status `expired` with payment-window-expired response.
3. Non-auto-pay worker reserve failures with not-found/expiry markers are classified retryable and return to `POLLING`.

## 5) Migrations drift or fail

Check current revision:

```bash
docker compose -f infra/docker-compose.yml exec api alembic current
```

Bring to head:

```bash
docker compose -f infra/docker-compose.yml exec api alembic upgrade head
```

If stack already running after pulling migrations:

```bash
docker compose -f infra/docker-compose.yml restart api worker
```

## 6) API crash loop on startup (ImportError)

Symptom: site is up but `/health` fails or the API is constantly restarting.

Checklist:

1. Inspect API logs:

```bash
docker compose -f infra/docker-compose.prod.yml logs --tail=200 api
# Or: sudo -u bominal docker logs --tail=200 bominal-api
```

2. If you see an import error like:
   - `ImportError: cannot import name 'SPEC_KEY_NEXT_RUN_AT' from 'app.modules.train.constants'`

Recovery:

1. Roll back to the previous deployment:

```bash
bominal-deploy --rollback
```

2. Or redeploy after a hotfix image is published:

```bash
bominal-deploy
```

## 6) Notification email not arriving in dev

1. Confirm `mailpit` service is up.
2. Open Mailpit UI: `http://localhost:8025`.
3. Check API `EMAIL_PROVIDER` setting (`smtp` for local Mailpit).
4. Trigger `/api/notifications/email/test` while logged in.

## 6.1) Train notifications not arriving with Resend (production)

1. Check API env values in `infra/env/prod/api.env`:
   - `EMAIL_PROVIDER=resend`
   - `RESEND_API_KEY` set
   - `EMAIL_FROM_ADDRESS` uses a verified Resend sender/domain
2. Verify worker is healthy and consuming `train:queue`.
3. Trigger `/api/notifications/email/test` from an authenticated session.
4. Inspect API/worker logs for redacted delivery errors (no payload bodies).

## 7) Restaurant task policy behavior (stage scaffold)

Symptoms:

- Restaurant task repeatedly returns to `POLLING`
- Restaurant task fails quickly after auth retries
- Payment step remains blocked (`payment_lease_busy`)

Checks:

1. Inspect `task_attempts` for safe error codes:
   - `auth_refresh_retry`
   - `auth_bootstrap_required`
   - `auth_failed`
   - `payment_lease_busy`
2. Verify policy settings in API env:
   - `RESTAURANT_AUTH_REFRESH_RETRIES`
   - `RESTAURANT_PAYMENT_LEASE_TTL_SECONDS`
   - `RESTAURANT_BOOTSTRAP_TIMEOUT_SECONDS`
3. Confirm Redis is healthy and reachable from API/worker containers.

Recovery:

1. If retries are too aggressive, raise `RESTAURANT_AUTH_REFRESH_RETRIES`.
2. If lease collisions are frequent, tune `RESTAURANT_PAYMENT_LEASE_TTL_SECONDS`.
3. Restart worker domains after config changes:

```bash
docker compose -f infra/docker-compose.yml restart worker
```

## Task/ticket lifecycle support playbook

- For "awaiting payment" tasks:
  - user can trigger manual pay action
  - re-check reservation viability before pay
- For cancellation:
  - record cancel attempt in `task_attempts`
  - refresh reservation/ticket status when opening details
- For completed tasks:
  - dashboard hides delete for paid tickets
  - detail page can expose delete based on status safety rules

## Log retention and sensitive output

- Do not print decrypted secrets or raw provider credentials.
- Use redacted metadata fields for persisted attempt context.
- For debugging provider payloads, keep only safe subsets in `meta_json_safe`.
