# Architecture

## System overview

Bominal is now a Rust runtime with three executable surfaces:

1. **Web/API SSR runtime** (`rust/crates/api`): axum service serving both API routes and Leptos SSR pages.
2. **Worker runtime** (`rust/crates/worker`): Tokio loop worker for queue polling/reconcile/watch/rotation schedules.
3. **Shared runtime crate** (`rust/crates/shared`): typed config, Supabase JWT/JWKS verification helpers, queue contract types, and telemetry/bootstrap helpers.

Shared infrastructure:

- **Postgres** for primary data storage
- **Redis** split by purpose:
  - non-CDE Redis for queueing + token-bucket rate limiting (`REDIS_URL_NON_CDE` or fallback `REDIS_URL`)
  - CDE Redis for payment/runtime sensitive flows (`REDIS_URL_CDE` or fallback `REDIS_URL`)
  - CDE Redis endpoint must be non-durable and must not be Upstash-hosted
- **Mailpit** in local dev (SMTP sink + inbox UI)
- **Provider egress gateways** (Caddy path-allowlist proxies):
  - `egress-train` allows only `/srt/*`, `/korail/*`, `/netfunnel/*`
  - `egress-restaurant` allows only `/opentable/*`, `/resy/*` (development profile only while restaurant is disabled in production)
  - unmatched routes are denied (`403`)
  - only `GET` / `POST` / `HEAD` methods are accepted (`405` otherwise)

Queue domain contracts:

- `train:queue` is consumed by `bominal-rust-worker`.
- malformed queue payloads are moved to `train:queue:dlq`.
- queue payload schema is `RuntimeQueueJob` in `rust/crates/shared/src/queue.rs`.

## Runtime topology (Docker Compose)

- `infra/docker-compose.yml` (development): hot reload + bind mounts
- `infra/docker-compose.prod.yml` (deployment): immutable containers, no dev reload flags
- API runtime:
  - `api` (Rust axum runtime, bound to host `:8000`)
- Worker runtime:
  - `worker` consumes `train:queue`
- Web runtime:
  - `web` (Rust axum + Leptos SSR runtime, bound to host `:3000`)
- Internal outbound-control runtime:
  - `egress-train` (`infra/egress/train/Caddyfile`)
  - `egress-restaurant` (`infra/egress/restaurant/Caddyfile`, development profile only)

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
- `/modules/restaurant` (feature-gated; disabled in production by default)
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
- Restaurant is controlled by `RESTAURANT_MODULE_ENABLED`; production sets it to `false` to hide the module while train remains active.

### Auth model

- `AUTH_MODE=supabase`: stateless Bearer token auth; API verifies Supabase JWT (`iss`/`aud`/`exp`) and maps `sub` to local user role.
- `AUTH_MODE=legacy`: session cookie auth (`bominal_session`) with DB-backed hashed session tokens (development/backward-compatibility mode).
- Production deploy/predeploy gates enforce `AUTH_MODE=supabase`.
- Session cookie behavior remains `HttpOnly`, `SameSite=Lax`, and `Secure` only in production.
- Local authorization source of truth remains `users.role_id` / `roles.name` (JWT role claims are not trusted for privilege).
- Direct frontend-to-Supabase auth is supported behind web feature flags:
  - web sign-in/sign-up can hit Supabase Auth directly
  - API session cookie bootstrap occurs at `POST /api/auth/supabase/session`
  - legacy password endpoints remain available during migration and emit deprecation/sunset headers
- Password reset trigger behavior is mode-specific:
  - `AUTH_MODE=supabase`: `POST /api/auth/request-password-reset` always triggers Supabase recovery (`/auth/v1/recover`) with redirect target `/auth/verify?type=recovery` (not gated by local user presence)
  - `AUTH_MODE=legacy`: local OTP reset token issuance/email delivery path remains active
- Supabase recovery-link confirmation (`POST /api/auth/supabase/confirm`, `type=recovery`) now issues an opaque, short-lived one-time recovery context cookie and redirects to `/reset-password` without exposing auth mode in URL query params.
- Supabase password reset completion (`POST /api/auth/reset-password/supabase`) accepts bearer recovery tokens for compatibility but primarily uses server-stored recovery context (with refresh-token retry fallback). Invalid/expired-token failures clear context, while password-policy failures return Supabase policy detail and keep context so users can retry with a stronger password.
- Sign-in capability discovery is API-owned via `GET /api/auth/methods` (`password`, `passkey`, `magic_link`, `otp`) so web login alternatives stay mode/config aware.
- Request-style auth endpoints keep anti-enumeration-safe generic responses:
  - `POST /api/auth/request-password-reset`
  - `POST /api/auth/request-magic-link`
  - `POST /api/auth/request-signin-otp` (only active when Supabase OTP sign-in is enabled)
- Magic-link + OTP sign-in contracts:
  - `AUTH_MODE=supabase`: magic-link requests call Supabase Auth (`/auth/v1/otp`) and callback confirmation redirects to `/auth/passkey/add?source=magiclink&next=/modules/train`
  - `AUTH_MODE=legacy`: magic-link requests issue local one-time verification tokens (`purpose=magic_login`) and confirm through `POST /api/auth/magic-link/confirm`
  - Supabase OTP sign-in endpoints are opt-in behind `SUPABASE_SIGNIN_OTP_ENABLED`
- Password reset confirmation endpoints now bootstrap an authenticated session cookie on success in both modes (`POST /api/auth/reset-password`, `POST /api/auth/reset-password/supabase`).
- Server-side web auth resolution (`web/lib/server-auth.ts`) calls `GET /api/auth/me` with timeout-bounded retry (`8s` timeout, single short backoff retry) to reduce transient false sign-out redirects on back/forward navigation under temporary API latency.
- Passkeys (WebAuthn) are optional and supported in session-auth flows:
  - authenticated enrollment (`/api/auth/passkeys/register/options`, `/api/auth/passkeys/register/verify`)
  - passkey login bootstrap (`/api/auth/passkeys/auth/options`, `/api/auth/passkeys/auth/verify`)
  - account-level passkey listing/removal (`/api/auth/passkeys`, `/api/auth/passkeys/{id}`)
  - post-auth passkey-offer interstitial route (`/auth/passkey/add`) is shared by sign-up, reset, and magic-link flows; users can enroll passkey or skip.
- Account email changes are two-step:
  - settings update requests verification to the new email address
  - email value changes only after `/api/auth/account/email-change/confirm` succeeds

### API access tiers

- Public routes: register/login/logout/password-reset + magic-link + auth-method discovery + optional OTP request/verify, Supabase session bootstrap (`/api/auth/supabase/session`), and email-verification request endpoints.
- Authenticated routes: account profile routes, modules, train, wallet, notifications.
- Internal-only routes: `/api/internal/*` guarded by either:
  - `X-Internal-Api-Key` against `INTERNAL_API_KEY`, or
  - `X-Internal-Service-Token` (short-lived HMAC token, audience `internal-api`).
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
- `/api/train/tasks` supports payload shape control via `view=full|compact`:
  - `full` preserves full task summary payload.
  - `compact` trims list-path `spec_json` to UI-critical keys and reduces nested ranked-row metadata.
- Latest attempt/ticket summary rows are selected using per-task latest-row ranking queries (window-function strategy) instead of loading full per-task histories in list view.
- PostgreSQL task-summary paths use `DISTINCT ON` plus descending `(task_id, timestamp, id)` indexes for latest attempt/artifact retrieval; non-Postgres test backends fall back to ranking-query compatibility path.
- Train list reads use partial indexes for active and terminal states (`user_id + created_at desc`) to reduce tail latency for bounded list fetches.
- Train dashboard fetches active/completed tasks on initial load, then applies realtime task/ticket deltas client-side for known tasks.
- Visibility restore and unknown-task events use a Supabase Data API delta reconciliation pass against `task_realtime_events`; full list API reload is reserved for unknown/missed deltas.
- Train list bootstrap can read from Supabase Data API view `public.v_train_task_list_compact` when `NEXT_PUBLIC_TRAIN_READS_VIA_DATA_API=true`, with automatic fallback to `/api/train/tasks` on read-path errors.
- Supabase read-model view `public.v_train_task_list_compact` projects latest-attempt metadata and ticket/list-bucket summary for compact dashboard cards.
- Owner-scoped RLS read policies are enabled for `tasks`, `task_attempts`, and `artifacts` to support direct Supabase Data API/GraphQL user reads without exposing cross-user data.
- Frontend task list state updates are key-compared before commit to avoid unnecessary rerender churn when payloads are unchanged.
- List endpoints use short private revalidation caching (`Cache-Control: private, max-age=5, must-revalidate`) plus weak ETag for conditional GET (`If-None-Match`) to reduce repeated transfer bytes.
- Performance regression safeguards include hybrid benchmark gate scripts (relative improvement + absolute ceilings) and frontend live-update behavior unit tests in CI.
- Dashboard/task-detail/top-nav-alert live updates:
  - API publishes per-user train task state events to Redis channel `train:task-events:user:{user_id}` on user-visible state transitions.
  - High-frequency internal retry states (`RUNNING`, `RESERVING`, `PAYING`, `POLLING`) are intentionally suppressed from publish emits to avoid refetch churn in web clients.
  - UI live updates are managed by `web/lib/train/task-events.ts` transport manager.
  - Primary transport is Supabase Realtime (`public.task_realtime_events` on publication `bominal_realtime`) when all realtime eligibility gates pass:
    - `NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_ENABLED=true`
    - browser Supabase config present (`NEXT_PUBLIC_SUPABASE_URL`, `NEXT_PUBLIC_SUPABASE_ANON_KEY`)
    - user has a valid Supabase browser access token
  - On realtime subscribe failure/timeout/channel error/close, or when token/config is unavailable, transport falls back automatically to SSE endpoint `/api/train/tasks/events`.
  - While on SSE fallback, the manager retries realtime cutover every `NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_RETRY_SECONDS` and switches back to realtime once subscribe succeeds.
  - SSE transport is deprecated (registry `DEP-2026-03-01-001`) as of `2026-03-01` with scheduled removal after `2026-03-31`; it remains functional during the compatibility window.
  - Dashboard, task-detail, and top-nav attention do not use fixed-interval task polling.
  - Event-driven reconciliation is dedupe-throttled client-side to avoid burst list refetch loops on repeated terminal/ticket transitions.
  - `public.task_realtime_events` is maintained by DB triggers and owner-scoped RLS, includes ticket/list-bucket projection fields, suppresses no-op updates, and is shared across Realtime/Data API/GraphQL consumers.
- Task-detail read path can use Supabase GraphQL (`/graphql/v1`) when `NEXT_PUBLIC_TRAIN_DETAIL_VIA_GRAPHQL=true`, with automatic fallback to `/api/train/tasks/{id}`.

Worker provider-search retry delay model:

- Worker retry timing for provider polling paths keeps the existing **stretched-exponential base curve** and adds two runtime overlays:
  - lane-aware provider limiter (`priority` lane bypass, polling lane capped per provider)
  - stress-mode delay floor anchors (`24h=1.25s`, `48h=2.5s`, `72h=4.0s`) applied only when provider polling contention is observed.
- Polling jitter now supports a mean-reverting OU-style stateful multiplier per task key (with bounded clamp) while preserving the existing gamma fallback for non-keyed calls.
- Priority/manual operations (`reserve`, `pay`, `cancel`, explicit refresh/search`) can run on `priority` lane when `TRAIN_PRIORITY_EXEMPT_FROM_CAPS=true`; polling traffic remains capped by `TRAIN_POLL_GLOBAL_REFILL_PER_SECOND` + `TRAIN_POLL_GLOBAL_BUCKET_CAPACITY`.
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

Worker bootstrap compatibility:

- Worker containers run via `python -m app.worker_entrypoint <settings-class>`.
- The entrypoint creates an explicit event loop before invoking ARQ worker settings, which is required for Python 3.14+ runtime compatibility.
- Terminal train-task email notifications can be delivered directly through Supabase Edge Function `task-notify` when `EDGE_TASK_NOTIFY_ENABLED=true`; worker falls back to queue-based template email delivery when edge invoke is disabled or fails.

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
- Provider client instances are cached per `(mode, provider, user)` for session continuity across API/worker calls:
  - `TRAIN_PROVIDER_CLIENT_CACHE_SECONDS` controls TTL for live/hybrid clients (default `600`).
  - `TRAIN_PROVIDER_CLIENT_CACHE_MAX_ENTRIES` bounds memory by evicting oldest-expiring entries first (default `256`).
  - cache entries are invalidated when provider credentials are updated/cleared.
- Optional egress routing by domain set:
  - `TRAIN_PROVIDER_EGRESS_PROXY_URL` routes train-provider calls through `egress-train`
  - `RESTAURANT_PROVIDER_EGRESS_PROXY_URL` routes restaurant-provider calls through `egress-restaurant` when the restaurant module is enabled
  - provider host allowlist validation still executes before outbound dispatch
- SRT contract alignment:
  - Unpaid reservation expiry is determined by `stlFlg != "Y"` and KST comparison `now > iseLmtDt+iseLmtTm`.
  - Sold-out standby fallback is allowed only when `rsvWaitPsbCd` contains `"9"`.
  - `reserve_info`/`ticket_info` no-data signals (for example `조회자료가 없습니다.`) are mapped to `reservation_not_found`.
  - Passenger payload fields are emitted per passenger index (`psgTpCd{n}`, `psgInfoPerPrnb{n}`, seat-attribute keys), including adults (`1`), children (`5`), seniors (`4`), disability 1-3 (`2`), and disability 4-6 (`3`).

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
- Payment CVV is never accepted or stored by wallet APIs
- Wallet auto-pay source is user-scoped (`/api/wallet/payment-card/configured` reflects the current user wallet, not serverwide defaults)
- Payment contract mode is Evervault-only in production (`PAYMENT_PROVIDER=evervault` when `PAYMENT_ENABLED=true`).
- KTX/SRT payment submissions use Evervault Relay with API-managed per-provider relay routes and exact form-field decrypt selectors (no wildcard selectors).
- Serverwide payment fallback/override execution is retired (`AUTOPAY_ALLOW_SERVER_FALLBACK=false` fail-closed contract).

Cardholder Data Environment (CDE) boundary:

- In scope: wallet secret decryption paths, payment execution worker paths, and in-memory decrypted PAN.
- Out of scope: web layer, queue payloads, `meta_json_safe`/`data_json_safe`, non-sensitive artifacts, logs.
- Raw cardholder data must never cross from CDE code paths into non-CDE persistence or transport layers.

## Operational notes

- Worker startup re-enqueues recoverable active tasks from DB.
- Provider outbound calls pass through Redis token-bucket limiter.
- Completed task visibility is soft-delete style (`hidden_at`) for UX behavior.
- Account deletion requires no outstanding worker tasks; it scrubs account/profile fields and marks tasks for 365-day removal.
- Queue producers use explicit ARQ queue names to avoid cross-domain consumption regressions.
