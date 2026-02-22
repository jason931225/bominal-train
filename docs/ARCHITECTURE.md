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
- **Redis** split by purpose:
  - non-CDE Redis for queueing + token-bucket rate limiting (`REDIS_URL_NON_CDE` or fallback `REDIS_URL`)
  - CDE Redis for encrypted CVV TTL cache (`REDIS_URL_CDE` or fallback `REDIS_URL`)
  - CDE Redis endpoint must be non-durable and must not be Upstash-hosted
- **Mailpit** in local dev (SMTP sink + inbox UI)
- **Provider egress gateways** (Caddy path-allowlist proxies):
  - `egress-train` allows only `/srt/*`, `/korail/*`, `/netfunnel/*`
  - `egress-restaurant` allows only `/opentable/*`, `/resy/*`
  - unmatched routes are denied (`403`)
  - only `GET` / `POST` / `HEAD` methods are accepted (`405` otherwise)

Queue domain contracts:

- `train:queue` is consumed by `worker-train` (`app.worker_train.WorkerTrainSettings`) for train tasks and queued email delivery jobs.
- `restaurant:queue` is consumed by `worker-restaurant` (`app.worker_restaurant.WorkerRestaurantSettings`) for restaurant-domain jobs.

## Runtime topology (Docker Compose)

- `infra/docker-compose.yml` (development): hot reload + bind mounts
- `infra/docker-compose.prod.yml` (deployment): immutable containers, no dev reload flags
- API runtime split:
  - `api-gateway` (public edge API, bound to host `:8000`)
  - `api-train` (private train-domain API container)
  - `api-restaurant` (private restaurant-domain API container)
- Worker runtime split:
  - `worker-train` consumes `train:queue`
  - `worker-restaurant` consumes `restaurant:queue`
- Internal outbound-control runtime:
  - `egress-train` (`infra/egress/train/Caddyfile`)
  - `egress-restaurant` (`infra/egress/restaurant/Caddyfile`)

Main service ports:

- Web: `3000`
- API gateway: `8000`
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
- PostgreSQL task-summary paths use `DISTINCT ON` plus descending `(task_id, timestamp, id)` indexes for latest attempt/artifact retrieval; non-Postgres test backends fall back to ranking-query compatibility path.
- Train list reads use partial indexes for active and terminal states (`user_id + created_at desc`) to reduce tail latency for bounded list fetches.
- Train dashboard polling fetches active tasks every poll cycle while completed tasks refresh on periodic/forced triggers (initial load, visibility restore, action mutations) to reduce steady-state list load.
- Frontend task list state updates are key-compared before commit to avoid unnecessary rerender churn when payloads are unchanged.
- Performance regression safeguards include hybrid benchmark gate scripts (relative improvement + absolute ceilings) and frontend polling-behavior unit tests in CI.

Worker provider-search retry delay model:

- Worker retry timing for provider search/reserve paths uses a **single stretched-exponential mean curve** with multiplicative **mean-preserving gamma jitter** (implemented in `api/app/modules/train/worker.py`).
- Definitions:
  - `t`: seconds until departure/expiry.
  - `M`: `TRAIN_POLL_MAX_SECONDS`.
  - `B`: baseline mean at 24h (`1.25s`).
  - `t0 = 86400` (24h).
- Mean curve:
  - `x = max(t - t0, 0)`
  - `mu(t) = M - (M - B) * exp(-(x / tau)^p)`
- Parameters `(p, tau)` are solved from fixed anchors (closed form, no regression):
  - `mu(48h) = 1.5`
  - `mu(72h) = 2.0`
  - `p = ln(y2 / y1) / ln(x2 / x1)`, `tau = x1 / y1^(1/p)`,
    where `y_i = -ln((M - mu_i)/(M - B))`, `x_i = t_i - t0`.
- Jitter:
  - `G ~ Gamma(k=4, theta=1/k)` so `E[G] = 1`.
  - `raw = mu(t) * G`, `delay = clamp(raw, 0.1, M)`.
  - Unclamped expectation remains `E[raw] = mu(t)`.

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
- Optional egress routing by domain set:
  - `TRAIN_PROVIDER_EGRESS_PROXY_URL` routes train-provider calls through `egress-train`
  - `RESTAURANT_PROVIDER_EGRESS_PROXY_URL` routes restaurant-provider calls through `egress-restaurant`
  - provider host allowlist validation still executes before outbound dispatch
- SRT contract alignment:
  - Unpaid reservation expiry is determined by `stlFlg != "Y"` and KST comparison `now > iseLmtDt+iseLmtTm`.
  - Sold-out standby fallback is allowed only when `rsvWaitPsbCd` contains `"9"`.
  - `reserve_info`/`ticket_info` no-data signals (for example `조회자료가 없습니다.`) are mapped to `reservation_not_found`.
  - Passenger payload fields are emitted per passenger index (`psgTpCd{n}`, `psgInfoPerPrnb{n}`, seat-attribute keys).

### Restaurant policy architecture (stage scaffold)

- Restaurant worker policy enforces auth fallback sequence: refresh retries -> bootstrap -> fail.
- Payment-step concurrency is guarded by Redis lease key:
  - `provider + account_ref + restaurant_id`
- Non-committing restaurant phases (for example search/availability) do not acquire payment lease.
- Policy writes only safe attempt metadata (`meta_json_safe`) and avoids credential/token persistence.
- Provider adapter scaffolding is defined in `api/app/modules/restaurant/providers/` with canonical operations:
  - `auth.start`, `auth.complete`, `auth.refresh`, `profile.get`, `search.availability`, `reservation.create`, `reservation.cancel`
- OpenTable stage-1 adapter implementation currently provides:
  - live refresh/profile/cancel paths
  - concrete OTP auth paths (`/dapi/authentication/sendotpfromsignin`, `/dapi/authentication/signinwithotp`) with env override support
  - frozen OTP response normalization (challenge-reference enforcement on `auth.start`; body-level failure enforcement on `auth.complete`)
  - frozen `search.availability` contract via `RestaurantsAvailability` persisted query (hash-configurable)
  - frozen `reservation.create` contract via two-step flow:
    - `BookDetailsStandardSlotLock` persisted mutation (hash-configurable)
    - `POST /dapi/booking/make-reservation` commit path (configurable path)
  - optional post-create `BookingConfirmationPageInFlow` enrichment (hash-configurable, non-blocking on failure)
  - normalized safe reservation-create output fields (`confirmation_enrichment`, `policy_safe`) for artifact persistence
- Resy stage-2 adapter implementation currently provides:
  - password auth via `POST /4/auth/password` for `auth.start`
  - challenge-token based `auth.complete` for single-step password flow
  - config-driven API key/origin headers with body-level failure normalization on HTTP 200 responses
  - `auth.refresh` via `POST /3/auth/refresh` with safe refresh metadata normalization
  - `profile.get` via `GET /2/user` with safe identity/count normalization
  - `search.availability` via `GET /4/find` with canonical slot token extraction
  - `reservation.create` via two-step `POST /3/details` -> `POST /3/book`
  - `reservation.cancel` via `POST /3/cancel` with token fallback retry path
  - supporting provider-specific `logout` helper via `POST /3/auth/logout` (non-canonical operation)
- Provider contract research and payload schemas are maintained under `docs/provider-research/`.
- CatchTable endpoint implementation references are sourced from read-only `third_party/catchtable` files (`reservation.py`, `session.py`, `configs.py`, `main.py`).

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
