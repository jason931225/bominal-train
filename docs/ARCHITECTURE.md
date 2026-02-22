# Architecture

## System overview

Bominal is a modular monorepo with three runtime tiers:

1. **Web** (`web/`): Next.js App Router UI, server-rendered pages + client components
2. **API** (`api/`): FastAPI REST backend with configurable auth mode (`legacy` | `supabase` | `dual`)
3. **Workers**:
   - `api/app/worker_train.py` -> train-task + queued email execution
   - `api/app/worker_restaurant.py` -> restaurant queue-domain execution scaffold

Shared infrastructure:

- **Postgres** for primary data storage
- **Redis** for queueing + token-bucket rate limiting
- **Mailpit** in local dev (SMTP sink + inbox UI)

Queue domain contracts:

- `train:queue` is consumed by `worker` (`app.worker_train.WorkerTrainSettings`) for train tasks and queued email delivery jobs.
- `restaurant:queue` is consumed by `worker-restaurant` (`app.worker_restaurant.WorkerRestaurantSettings`) for restaurant-domain jobs.

## Runtime topology (Docker Compose)

- `infra/docker-compose.yml` (development): hot reload + bind mounts
- `infra/docker-compose.prod.yml` (deployment): immutable containers, no dev reload flags

Main service ports:

- Web: `3000`
- API: `8000`
- Postgres: `5432`
- Redis: `6379`
- Mailpit UI (dev): `8025`

## Web layer

Core routes:

- `/login`, `/register`, `/dashboard`
- `/modules/train`
- `/modules/restaurant` (coming soon)
- `/modules/calendar` (coming soon)
- `/admin`
- `/settings/account`
- `/settings/payment`

Design system:

- Tailwind + shared tokens in `web/lib/ui.ts`
- Seasonal theme engine in `web/components/theme-provider.tsx` + `web/lib/theme.ts`
- Theme persisted in `localStorage` via `bominal_theme_mode`

## API layer

### Core domains

- **Auth**: register/login/logout/me/account patch + account delete
- **Modules**: module availability list (train active, others coming soon)
- **Train**: search, credentials, tasks, reservation/ticket actions
- **Wallet**: shared payment card storage (module-agnostic)
- **Notifications**: queued test email/status

Module contract:

- `/api/modules` returns `slug`, `name`, `coming_soon`, `enabled`, and `capabilities`.
- Capability flags are backend-owned strings intended for progressive feature exposure.
- Restaurant remains controlled exposure (`coming_soon=true`, `enabled=false`) until policy/runtime stages are completed.

### Auth model

- `AUTH_MODE=legacy`: session cookie auth (`bominal_session`) with DB-backed hashed session tokens.
- `AUTH_MODE=supabase`: stateless Bearer token auth; API verifies Supabase JWT (`iss`/`aud`/`exp`) and maps `sub` to local user role.
- `AUTH_MODE=dual`: Bearer token takes precedence when present; otherwise falls back to session cookie.
- Session cookie behavior remains `HttpOnly`, `SameSite=Lax`, and `Secure` only in production.
- Local authorization source of truth remains `users.role_id` / `roles.name` (JWT role claims are not trusted for privilege).

### API access tiers

- Public routes: register/login/logout/password-reset and email-verification request endpoints.
- Authenticated routes: account profile routes, modules, train, wallet, notifications.
- Internal-only routes: `/api/internal/*` guarded by `X-Internal-Api-Key` against `INTERNAL_API_KEY`.
- Admin routes: `/api/admin/*` guarded by role check (`admin`).

### Train module architecture

Train request path:

1. User submits search/task request in UI.
2. API validates + persists task intent (`tasks` row).
3. API enqueues ARQ job to Redis (`run_train_task`).
4. Worker consumes task and executes strict ranked selection.
5. Worker persists:
   - `task_attempts` timeline entries
   - `artifacts` (ticket/receipt safe metadata)
   - task state transitions

Task list performance controls:

- `/api/train/tasks` supports bounded list reads via `limit` query (`1..500`, default `200`).
- Latest attempt/ticket summary rows are selected using per-task latest-row ranking queries (window-function strategy) instead of loading full per-task histories in list view.

Provider integration:

- Interface in `api/app/modules/train/providers/base.py`
- Implementations:
  - `srt_client.py`
  - `ktx_client.py`
  - `mock.py`
  - `hybrid.py`
- Factory switching by env:
  - `TRAIN_PROVIDER_MODE`: `mock` | `hybrid` | `real`
  - `TRAIN_PROVIDER_TRANSPORT`: `auto` | `curl_cffi` | `httpx`

### Restaurant policy architecture (stage scaffold)

- Restaurant worker policy enforces auth fallback sequence: refresh retries -> bootstrap -> fail.
- Payment-step concurrency is guarded by Redis lease key:
  - `provider + account_ref + restaurant_id`
- Non-committing restaurant phases (for example search/availability) do not acquire payment lease.
- Policy writes only safe attempt metadata (`meta_json_safe`) and avoids credential/token persistence.

## Data model highlights

Main tables:

- `users`, `roles`, `sessions`
- `tasks`, `task_attempts`, `artifacts`
- `secrets` (encrypted provider credentials and wallet payloads)
- `verification_tokens`, `password_reset_tokens` (future email flows)

Key train states:

- Active: `QUEUED`, `RUNNING`, `POLLING`, `RESERVING`, `PAYING`, `PAUSED`
- Terminal: `COMPLETED`, `EXPIRED`, `CANCELLED`, `FAILED`

## Crypto and sensitive data

Envelope encryption (`api/app/core/crypto/envelope.py`):

- Per-record random 256-bit DEK
- AES-256-GCM payload encryption
- DEK wrapped with `MASTER_KEY` (KEK) via AES-256-GCM
- Versioned key metadata (`kek_version`)

Data controls:

- Redaction helper for logs/safe metadata in `redaction.py`
- Payment CVV not persisted to Postgres; encrypted short-term cache in Redis

Cardholder Data Environment (CDE) boundary:

- In scope: wallet secret decryption paths, payment execution worker paths, CVV Redis keys, in-memory decrypted PAN/CVV.
- Out of scope: web layer, queue payloads, `meta_json_safe`/`data_json_safe`, non-sensitive artifacts, logs.
- Raw cardholder data must never cross from CDE code paths into non-CDE persistence or transport layers.

## Operational notes

- Worker startup re-enqueues recoverable active tasks from DB.
- Provider outbound calls pass through Redis token-bucket limiter.
- Completed task visibility is soft-delete style (`hidden_at`) for UX behavior.
- Account deletion requires no outstanding worker tasks; it scrubs account/profile fields and marks tasks for 365-day removal.
- Queue producers use explicit ARQ queue names to avoid cross-domain consumption regressions.
