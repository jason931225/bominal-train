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
# Or: sudo -u bominal /opt/bominal/repo/infra/scripts/deploy-zero-downtime.sh
```

Rollback to previous deployment:

```bash
bominal-deploy --rollback
# Or: sudo -u bominal /opt/bominal/repo/infra/scripts/deploy-zero-downtime.sh --rollback
```

Note: `deploy-zero-downtime.sh` pulls pre-built images (there is no `--skip-build` flag).

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
sudo -u bominal /opt/bominal/repo/infra/scripts/deploy-zero-downtime.sh
```

Or rollback if previous deployment was stable:

```bash
sudo -u bominal /opt/bominal/repo/infra/scripts/deploy-zero-downtime.sh --rollback
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
