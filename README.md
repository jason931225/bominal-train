# bominal

bominal is a modular product foundation with:

- `web/`: Next.js App Router + TypeScript + Tailwind + Zod
- `api/`: FastAPI + Postgres + Alembic + session auth
- `worker`: arq train worker for async Train Tasks + queued email jobs
- `worker-restaurant`: arq restaurant worker (isolated queue domain scaffold)
- `redis`: queue + provider rate limiting
- `third_party/srtgo`: read-only provider behavior reference (submodule)

## Documentation pack

- Agent guide: `AGENTS.md`
- Changelog: `CHANGELOG.md`
- Docs index: `docs/README.md`
- Architecture: `docs/ARCHITECTURE.md`
- Contribution workflow: `docs/CONTRIBUTING.md`
- Deployment: `docs/DEPLOYMENT.md`
- Operations runbook: `docs/RUNBOOK.md`
- Security controls: `docs/SECURITY.md`

## Bootstrap

From repo root:

```bash
git submodule update --init --recursive
```

## Development (local)

Development compose uses hot-reload, bind mounts, and env files under `infra/env/dev/`.

If your machine supports Docker Compose v2 plugin:

```bash
docker compose -f infra/docker compose.yml up --build
```

If your machine uses Docker Compose v1 binary:

```bash
docker compose -f infra/docker compose.yml up --build
```

Services started by compose:

- web: `http://localhost:3000`
- api: `http://localhost:8000`
- postgres: `localhost:5432`
- redis: `localhost:6379`
- mailpit (dev inbox): `http://localhost:8025` (SMTP on `localhost:1025`)
- worker: consumes queued Train Tasks + queued email jobs
- worker-restaurant: reserved for restaurant queue-domain execution

Queue domains:

- `train:queue`: train tasks + queued email delivery
- `restaurant:queue`: restaurant worker domain

One-command local verification (starts stack, waits for health, runs backend tests + web typecheck):

```bash
./infra/scripts/local-check.sh
```

Optional cleanup after checks:

```bash
./infra/scripts/local-check.sh --down
```

If you pull new backend migrations while containers are already running, restart API/workers once:

```bash
docker compose -f infra/docker compose.yml restart api worker worker-restaurant
# or (Compose v1):
docker compose -f infra/docker compose.yml restart api worker worker-restaurant
```

## Production (manual bootstrap)

Production compose is separated in `infra/docker compose.prod.yml` (no bind mounts, no dev reload flags).

For production deployments, prefer the zero-downtime procedure in `docs/DEPLOYMENT.md`
(script: `infra/scripts/deploy.sh`). The steps below cover initial
env-file bootstrap.

Compatibility notice:
- `infra/docker-compose.deploy.yml.deprecated` is deprecated and no longer part of the canonical deploy workflow.
- Use `infra/docker-compose.prod.yml` with `infra/scripts/deploy.sh`.
- Removal gate: remove the deprecated compose artifact after no active callers remain (completed on 2026-02-14).

1) Create prod env files from templates:

```bash
cp infra/env/prod/postgres.env.example infra/env/prod/postgres.env
cp infra/env/prod/api.env.example infra/env/prod/api.env
cp infra/env/prod/web.env.example infra/env/prod/web.env
cp infra/env/prod/caddy.env.example infra/env/prod/caddy.env
```

2) Edit those files and replace every `CHANGE_ME...` value (especially `MASTER_KEY`), and set your public host in `infra/env/prod/caddy.env`.

3) Deploy (recommended):

```bash
bash infra/scripts/deploy.sh
```

If you intentionally need a manual bring-up (not recommended for routine deploys),
use `--wait` when available:

```bash
docker compose -f infra/docker compose.prod.yml up -d --wait
```

or (Compose v1):

```bash
docker compose -f infra/docker compose.prod.yml up -d
```

4) Verify health:

```bash
curl -sS http://localhost:8000/health
curl -sS http://localhost:3000
curl -sS -I http://localhost
docker compose -f infra/docker compose.prod.yml ps
```

## Layout

- `web/`
- `api/`
- `infra/docker compose.yml` (development)
- `infra/docker compose.prod.yml` (deployment)
- `third_party/srtgo` (read-only reference)

## Auth + modules

Implemented auth endpoints:

- `POST /api/auth/register`
- `POST /api/auth/login`
- `POST /api/auth/logout`
- `POST /api/auth/request-email-verification`
- `POST /api/auth/verify-email` (OTP or link-code verification)
- `POST /api/auth/request-password-reset`
- `POST /api/auth/reset-password` (OTP or link-code reset)
- `GET /api/auth/me`
- `PATCH /api/auth/account` (`current_password` required for changing `email` / `new_password`)
- `DELETE /api/auth/account` (blocked when outstanding worker tasks exist; marks user tasks for 365-day removal window)

Auth uniqueness rules:

- `email` is unique (case-insensitive)
- `display_name` is unique (case-insensitive)

API access tiers:

- **Public (no login required):** `/api/auth/register`, `/api/auth/login`, `/api/auth/logout`, `/api/auth/request-email-verification`, `/api/auth/verify-email`, `/api/auth/request-password-reset`, `/api/auth/reset-password`
- **Authenticated session required:** `/api/auth/me`, `/api/auth/account`, `/api/modules`, `/api/train/*`, `/api/wallet/*`, `/api/notifications/*`
- **Internal-only:** `/api/internal/*` with `X-Internal-Api-Key` matching `INTERNAL_API_KEY`
- **Admin role required:** `/api/admin`

Admin-only OpenAPI documentation:

- `GET /api/docs` (Swagger UI)
- `GET /api/openapi.json` (schema)

Implemented modules endpoint:

- `GET /api/modules`
  - Train = active
  - Restaurant = coming soon
  - Calendar = coming soon
  - Each module includes `enabled` and `capabilities` for controlled client exposure.

## Train module API

All endpoints require authenticated session cookie.

- `GET /api/train/stations`
- `GET /api/train/credentials/status`
- `GET /api/train/credentials/ktx`
- `POST /api/train/credentials/ktx`
- `POST /api/train/credentials/ktx/signout`
- `GET /api/train/credentials/srt`
- `POST /api/train/credentials/srt`
- `POST /api/train/credentials/srt/signout`
- `POST /api/train/search`
- `POST /api/train/tasks`
- `GET /api/train/tasks?status=active|completed|all`
- `GET /api/train/tasks/{id}`
- `POST /api/train/tasks/{id}/pause`
- `POST /api/train/tasks/{id}/resume`
- `POST /api/train/tasks/{id}/cancel`
- `POST /api/train/tickets/{artifact_id}/cancel`
- `GET /api/train/providers/{provider}/reservations`
- `GET /api/train/providers/{provider}/reservations/{reservation_id}/tickets`
- `POST /api/train/providers/{provider}/reservations/{reservation_id}/cancel`

Shared wallet API:

- `GET /api/wallet/payment-card`
- `POST /api/wallet/payment-card`
- `DELETE /api/wallet/payment-card`

Wallet data is shared across bominal services (not Train-specific).

Email + notification API:

- `GET /api/notifications/email/status`
- `POST /api/notifications/email/test`

`/api/notifications/email/test` enqueues delivery through the background worker, so modules can reuse the same queue-based email pipeline.

Template-based email pipeline:

- Producers can enqueue:
  - rendered payloads (`EmailJobPayload`)
  - template payloads (`EmailTemplateJobPayload`) with `theme`, `blocks`, and `context` pointers
- Worker (`deliver_email_job`) renders template payloads into final HTML/text bodies before provider send.
- `context` pointers in block data support:
  - inline placeholders: `{{ user.display_name }}`
  - explicit refs: `{"$ref":"verify.code"}` with optional `default`

Internal API:

- `GET /api/internal/health` (requires `X-Internal-Api-Key`)

## Worker behavior (current milestone)

- Task states: `QUEUED`, `RUNNING`, `POLLING`, `RESERVING`, `PAYING`, `COMPLETED`, `EXPIRED`, `PAUSED`, `CANCELLED`, `FAILED`
- Search is flexible, reserve/pay is strict by ranked schedule list
- Deadline: earliest selected departure (`deadline_at`)
- Expiration: worker marks `EXPIRED` when `now >= deadline_at`
- Idempotent active-task creation by `(user_id, module, idempotency_key)`
- Payment idempotency: worker checks existing payment artifact before pay retry
- Redis token bucket rate limiter applied to provider outbound calls
- ARQ queue domains are explicit: `train:queue` (worker) and `restaurant:queue` (worker-restaurant)

## Manual verification (Train)

1. Open `http://localhost:3000/register`, create account, then login.
2. Open `http://localhost:3000/modules/train`.
3. Credential status checks run automatically for both providers (KTX first, then SRT).
4. If stored credentials are valid, cards stay minimized and show usernames.
5. If credentials are missing/invalid, the corresponding login form is shown. You can also choose "Continue without KTX/SRT", which disables that provider in search.
6. Station dropdown defaults are `수서` -> `마산`. Backend converts station names to provider codes where needed.
7. All Train times in the UI are rendered in KST (`Asia/Seoul`), then search schedules.
8. In real mode, schedules come from live SRT/KTX only (no fallback).
9. Select schedules and rank with Up/Down.
10. Create Task.
11. Confirm Active list shows state transitions (`QUEUED` -> `POLLING` -> `RESERVING` -> `PAYING` -> `COMPLETED` for mock reserve/pay flow).
12. Open task detail page and verify attempts timeline + ticket artifact.

Check DB task rows:

```bash
docker compose -f infra/docker compose.yml exec postgres \
  psql -U bominal -d bominal \
  -c "select id, state, deadline_at, created_at from tasks order by created_at desc limit 20;"
```

Check attempts:

```bash
docker compose -f infra/docker compose.yml exec postgres \
  psql -U bominal -d bominal \
  -c "select task_id, action, provider, ok, retryable, started_at from task_attempts order by started_at desc limit 30;"
```

Test email pipeline locally:

1. Login to bominal once in browser (to create session cookie).
2. Trigger test email:

```bash
curl -X POST http://localhost:8000/api/notifications/email/test \
  -H "Content-Type: application/json" \
  -b "bominal_session=YOUR_SESSION_COOKIE" \
  -d '{}'
```

3. Open Mailpit inbox at `http://localhost:8025`.

## Tests

Backend tests include:

- auth flow
- train task creation idempotency
- deadline expiration
- redis rate limiter concurrency behavior
- payment idempotency under retry
- provider parser tests (SRT/KTX response shapes)

Run tests:

```bash
docker compose -f infra/docker compose.yml exec api pytest -q
```

Frontend type-check:

```bash
docker compose -f infra/docker compose.yml exec web npx tsc --noEmit
```

## Pre-deploy final check

Run this checklist before first deployment:

1. **Env sanity**
   - `infra/env/prod/postgres.env`, `infra/env/prod/api.env`, `infra/env/prod/web.env`, `infra/env/prod/caddy.env` exist.
   - No `CHANGE_ME` placeholders remain.
   - `APP_ENV=production` in `infra/env/prod/api.env`.
   - `MASTER_KEY` is set to a real 32-byte base64 key (generate with `openssl rand -base64 32`).

2. **Compose validity**

```bash
docker compose -f infra/docker compose.prod.yml config >/tmp/bominal-prod-compose.txt
```

3. **App checks in dev (recommended before prod push)**

```bash
docker compose -f infra/docker compose.yml exec api pytest -q
docker compose -f infra/docker compose.yml exec web npx tsc --noEmit
```

4. **Smoke test after prod up**

```bash
curl -sS http://localhost:8000/health
curl -sS -I http://localhost:3000
curl -sS -I http://localhost
docker compose -f infra/docker compose.prod.yml logs --tail=100 caddy api worker worker-restaurant web
```

Duplicate display name pre-migration check (optional manual run):

```bash
docker compose -f infra/docker compose.prod.yml run --rm api python scripts/check_duplicate_display_names.py
```

5. **Email configuration**
   - If email is not in use yet, keep `EMAIL_PROVIDER=disabled` (default in prod template).
   - Optional: for transactional email later, set `EMAIL_PROVIDER=resend` and configure `RESEND_API_KEY`.
   - Optional: tune Resend HTTP timeout with `RESEND_TIMEOUT_SECONDS` (default `12`).
   - Optional SMTP relay: set `EMAIL_PROVIDER=smtp` and configure `SMTP_HOST`, `SMTP_PORT`, `SMTP_USERNAME`, `SMTP_PASSWORD`.
   - When enabled, set `EMAIL_FROM_ADDRESS` / `EMAIL_FROM_NAME` to your sender identity.
   - Set `APP_PUBLIC_BASE_URL` so verification/reset links in emails point to the correct environment URL.

Or run the bundled checker:

```bash
./infra/scripts/predeploy-check.sh
```

## Notes on provider implementation

- Source-of-truth reference repo is available at `third_party/srtgo`.
- `TRAIN_PROVIDER_MODE` options:
  - `mock`: mock schedules + mock reserve/pay
  - `hybrid`: live schedules first, fallback to mock; mock reserve/pay
  - `real`: live provider clients for all calls
- `TRAIN_PROVIDER_TRANSPORT` options:
  - `curl_cffi` (recommended for live providers)
  - `httpx`
  - `auto` (try curl_cffi, fallback to httpx)
- SRT/KTX parser/client scaffolding is in:
  - `api/app/modules/train/providers/srt_client.py`
  - `api/app/modules/train/providers/ktx_client.py`

## Security notes

- Password hashing: Argon2id
- Session IDs are never returned in JSON responses
- Cookie: `httpOnly`, `SameSite=Lax`, `Secure` in production only
- Secrets use envelope encryption (`AES-256-GCM` DEK per record + KEK wrap via `MASTER_KEY`)
- Payment CVV is not stored in Postgres; it is cached encrypted in Redis with TTL (`PAYMENT_CVV_TTL_SECONDS`, default 3600)
- Never run production with default `MASTER_KEY`; app now rejects that in production mode
- Email is queued via Redis/arq and sent from worker; dev mode uses Mailpit inbox (`http://localhost:8025`)
- Production recommendation: use a managed transactional provider (e.g. Resend) instead of self-hosting a full mail server, for deliverability and security hardening
