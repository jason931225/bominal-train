# Runbook

Operational procedures for local/dev/prod maintenance.

## Service commands

Start stack (dev):

```bash
docker-compose -f infra/docker-compose.yml up --build
```

Start stack (prod profile file):

```bash
docker-compose -f infra/docker-compose.prod.yml up -d --build
```

Stop stack:

```bash
docker-compose -f infra/docker-compose.yml down
```

Hard reset (destroys local DB volume):

```bash
docker-compose -f infra/docker-compose.yml down -v
```

## Observability quick checks

API/web health:

```bash
curl -sS http://localhost:8000/health
curl -sS -I http://localhost:3000
```

Container status/logs:

```bash
docker-compose -f infra/docker-compose.yml ps
docker-compose -f infra/docker-compose.yml logs -f api worker web
```

DB checks:

```bash
docker-compose -f infra/docker-compose.yml exec postgres \
  psql -U bominal -d bominal \
  -c "select id, state, created_at, completed_at from tasks order by created_at desc limit 20;"
```

```bash
docker-compose -f infra/docker-compose.yml exec postgres \
  psql -U bominal -d bominal \
  -c "select task_id, action, provider, ok, retryable, started_at from task_attempts order by started_at desc limit 30;"
```

## Common incidents

## 1) Web route fails to load (`Cannot find module './901.js'`)

Cause: stale/corrupt Next build cache.

Fix:

```bash
docker-compose -f infra/docker-compose.yml exec web sh -lc "cd /app && rm -rf .next && npm run dev -- -H 0.0.0.0 -p 3000"
```

If still broken, restart web container.

## 2) API healthy but Train task states not moving

Checklist:

1. Verify worker is running (`docker-compose ... ps`).
2. Check worker logs for provider/auth/rate-limit failures.
3. Ensure Redis is healthy.
4. Verify task is not paused/cancelled/expired.

Recovery:

```bash
docker-compose -f infra/docker-compose.yml restart worker
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
docker-compose -f infra/docker-compose.yml exec api alembic current
```

Bring to head:

```bash
docker-compose -f infra/docker-compose.yml exec api alembic upgrade head
```

If stack already running after pulling migrations:

```bash
docker-compose -f infra/docker-compose.yml restart api worker
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

