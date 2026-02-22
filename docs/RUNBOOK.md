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

Note: `deploy.sh` pulls pre-built images (there is no `--skip-build` flag).

Deploy script guardrails:
- Enforces single-run deploy lock (`DEPLOY_LOCK_FILE`, default `/tmp/bominal-deploy.lock`).
- Runs strict preflight checks before pull/deploy (env + compose + memory/swap threshold).
- Enforces deprecation registry gate for production-scoped removals/deadlines.
- Chooses bootstrap-safe first deploy path when no stack is running.
- Auto rollback on smoke failure is enabled by default (`AUTO_ROLLBACK_ON_SMOKE_FAILURE=true`).

Compatibility notice:
- `infra/docker-compose.deploy.yml.deprecated` is deprecated and removed from active operator workflow.
- Use `infra/docker-compose.prod.yml` and `infra/scripts/deploy.sh` exclusively.
- Deprecation lifecycle policy is tracked in `docs/DEPRECATION_WORKFLOW.md` and `docs/deprecations/registry.json`.

Quick restart after VM reset (no rebuild, existing images):

```bash
sudo -u bominal /opt/bominal/repo/infra/scripts/quick-restart.sh
```

Check VM stack:

```bash
docker compose -f infra/docker compose.prod.yml ps
docker compose -f infra/docker compose.prod.yml logs -f caddy api worker worker-restaurant web
```

### Docker (local simulation)

Start stack (dev):

```bash
docker compose -f infra/docker compose.yml up --build
```

Start stack (prod profile file):

```bash
docker compose -f infra/docker compose.prod.yml up -d --build
```

Stop stack:

```bash
docker compose -f infra/docker compose.yml down
```

Hard reset (destroys local DB volume):

```bash
docker compose -f infra/docker compose.yml down -v
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
docker compose -f infra/docker compose.yml ps
docker compose -f infra/docker compose.yml logs -f api worker worker-restaurant web
```

Production status/logs:

```bash
docker compose -f infra/docker compose.prod.yml ps
docker compose -f infra/docker compose.prod.yml logs -f caddy api worker worker-restaurant web
```

Security checks for payment/CDE runtime:

```bash
# Confirm Redis split config (non-CDE vs CDE) in API runtime.
docker compose -f infra/docker compose.prod.yml exec api env | rg 'REDIS_URL(_NON_CDE|_CDE)?'

# Confirm effective CDE Redis endpoint is not Upstash-hosted.
docker compose -f infra/docker compose.prod.yml exec api python -c "from app.core.config import get_settings,is_upstash_redis_url; s=get_settings(); print('resolved_redis_url_cde=', s.resolved_redis_url_cde); assert not is_upstash_redis_url(s.resolved_redis_url_cde)"

# Confirm Redis persistence is disabled for CDE runtime.
docker compose -f infra/docker compose.prod.yml exec redis redis-cli CONFIG GET save
docker compose -f infra/docker compose.prod.yml exec redis redis-cli CONFIG GET appendonly

# Validate provider egress allowlist and timeout envs are set.
docker compose -f infra/docker compose.prod.yml exec api env | rg 'PAYMENT_PROVIDER_ALLOWED_HOSTS|TRAIN_PROVIDER_TIMEOUT_|PAYMENT_TRANSPORT_TRUST_ENV'

# Confirm payment logs do not include request/response payload bodies.
docker compose -f infra/docker compose.prod.yml logs --since=30m api worker | rg -i 'card_number|cvv|authorization|set-cookie|wrapped_dek|ciphertext'
```

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
```

DB checks:

```bash
docker compose -f infra/docker compose.yml exec postgres \
  psql -U bominal -d bominal \
  -c "select id, state, created_at, completed_at from tasks order by created_at desc limit 20;"
```

```bash
docker compose -f infra/docker compose.yml exec postgres \
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

1. `docker compose -f infra/docker compose.prod.yml ps`
2. `docker compose -f infra/docker compose.prod.yml logs --tail=200 caddy api worker worker-restaurant web`
3. Verify env files exist:
   - `infra/env/prod/postgres.env`
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

## 1) Web route fails to load (`Cannot find module './901.js'`)

Cause: stale/corrupt Next build cache.

Fix:

```bash
docker compose -f infra/docker compose.yml exec web sh -lc "cd /app && rm -rf .next && npm run dev -- -H 0.0.0.0 -p 3000"
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
docker compose -f infra/docker compose.yml restart worker
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

## 5) Migrations drift or fail

Check current revision:

```bash
docker compose -f infra/docker compose.yml exec api alembic current
```

Bring to head:

```bash
docker compose -f infra/docker compose.yml exec api alembic upgrade head
```

If stack already running after pulling migrations:

```bash
docker compose -f infra/docker compose.yml restart api worker worker-restaurant
```

## 6) API crash loop on startup (ImportError)

Symptom: site is up but `/health` fails or the API is constantly restarting.

Checklist:

1. Inspect API logs:

```bash
docker compose -f infra/docker compose.prod.yml logs --tail=200 api
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
docker compose -f infra/docker compose.yml restart worker worker-restaurant
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
