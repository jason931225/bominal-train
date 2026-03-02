# Rust Backend Parity (Legacy Python -> Rust)

Snapshot date: 2026-03-02

## Legacy API Route Inventory

Counting rule:
- `route count` is decorator-level (`@public_router.*`, `@user_router.*`, `@router.*`).
- `representative paths` is a path-level summary, not a full decorator dump.

| legacy file | mounted prefix | route count | representative paths |
|---|---|---:|---|
| `api/app/http/routes/auth.py` | `/api/auth` | 28 | `/register`, `/login`, `/methods`, `/logout`, `/me`, `/account`, `/account/verify-password`, `/account/email-change/confirm`, `/passkeys`, `/passkeys/register/options`, `/passkeys/register/verify`, `/passkeys/step-up/options`, `/passkeys/step-up/verify`, `/passkeys/auth/options`, `/passkeys/auth/verify`, `/request-email-verification`, `/verify-email`, `/request-magic-link`, `/magic-link/confirm`, `/request-password-reset`, `/request-signin-otp`, `/verify-signin-otp`, `/supabase/confirm`, `/reset-password/supabase`, `/reset-password` |
| `api/app/http/routes/admin.py` | `/api/admin` | 17 | `/`, `/payment-settings`, `/payment-settings/enabled`, `/payment-settings/card`, `/stats`, `/ops/status`, `/ops/train/stale-tasks`, `/ops/train/recent-failures`, `/ops/train/recover`, `/ops/train/tasks/{task_id}/requeue`, `/users`, `/users/{user_id}`, `/users/{user_id}/role`, `/users/{user_id}/access`, `/users/{user_id}/revoke-sessions` |
| `api/app/http/routes/internal.py` | `/api/internal` | 1 | `/health` |
| `api/app/http/routes/modules.py` | `/api` | 1 | `/modules` |
| `api/app/http/routes/notifications.py` | `/api/notifications` | 2 | `/email/status`, `/email/test` |
| `api/app/http/routes/wallet.py` | `/api/wallet` | 4 | `/payment-card`, `/payment-card/configured` |

## Legacy Worker Inventory

| legacy worker file | primary entrypoint(s) | concrete capability inventory |
|---|---|---|
| `api/app/worker.py` | `WorkerSettings.functions = [run_train_task, deliver_email_job]` | ARQ consumer for train/email jobs, startup recovery (`enqueue_recoverable_tasks`), in-flight task requeue on shutdown, Redis heartbeat, periodic attempt hygiene (`compact_and_prune_task_attempts`) |
| `api/app/worker_train.py` | `WorkerTrainSettings` alias of `app.worker.WorkerSettings` | Dedicated train-worker entrypoint wrapper; queue domain = train queue |
| `api/app/worker_restaurant.py` | `WorkerRestaurantSettings.functions = [run_restaurant_task]` | Dedicated restaurant queue worker, separate queue domain and capacity (`max_jobs=10`) |
| `api/app/modules/train/worker.py` | `run_train_task`, `enqueue_recoverable_tasks`, `compact_and_prune_task_attempts` | Train lifecycle state machine (search/reserve/pay/cancel/sync), provider credential usage, retry scheduling, terminal notifications, task-attempt persistence and pruning |
| `api/app/modules/restaurant/worker.py` | `run_restaurant_task` | Restaurant auth fallback steps, payment lease coordination, task attempt append, non-committing vs committing phase branching |

## Rust Implementation Mapping

Status contract:
- `port_status`: `ported` or `in_progress`
- `cutover_priority`: `required` or `deferred`

| legacy source | legacy contract (concrete) | rust mapping (concrete) | port_status | cutover_priority | notes |
|---|---|---|---|---|---|
| `api/app/http/routes/auth.py` | Session/account/passkey/magic-link/password endpoints under `/api/auth/*` | No equivalent route set in `rust/crates/api/src/main.rs` | `in_progress` | `required` | Rust currently exposes infra/auth primitives only, not user session/account API parity. |
| `api/app/http/routes/auth.py` | Supabase token verification responsibilities in legacy auth/deps stack | `GET /api/auth/supabase/verify` -> `verify_supabase_token` | `ported` | `required` | Dedicated Rust JWT verification endpoint exists. |
| `api/app/http/routes/auth.py` | Auth sync to local DB on Supabase-side events | `POST /api/auth/supabase/webhook` -> `supabase_auth_webhook` + `persist_auth_sync` | `ported` | `required` | Persists into `supabase_auth_user_sync`. |
| `api/app/http/routes/modules.py` | `GET /api/modules` module capability listing | `GET /api/modules` -> `list_modules` | `in_progress` | `required` | Rust response is reduced (`train`,`auth`) vs legacy (`train`,`calendar`, optional `restaurant`, richer capabilities). |
| `api/app/http/routes/internal.py` | `GET /api/internal/health` (`require_internal_access` protected) | `GET /health/live`, `GET /health/ready` (currently public) | `in_progress` | `required` | Security contract differs: legacy health is internal-only; Rust health probes are public unless fronted by network policy. |
| `api/app/http/routes/admin.py` | `/api/admin/*` operational/admin APIs | No equivalent route set in Rust API | `in_progress` | `deferred` | Admin dashboard/ops APIs are not yet ported. |
| `api/app/http/routes/notifications.py` | `/api/notifications/email/status`, `/api/notifications/email/test` | No equivalent route set in Rust API | `in_progress` | `deferred` | Email diagnostics/testing APIs remain legacy-only. |
| `api/app/http/routes/wallet.py` | `/api/wallet/payment-card*` card status/config endpoints | No equivalent route set in Rust API | `in_progress` | `deferred` | Wallet card CRUD endpoints are not yet ported. |
| `api/app/worker.py` | Queue consumer + startup/shutdown recovery + heartbeat + attempt hygiene | `poll_loop`, `poll_queue_once`, `reconcile_loop`, `watch_loop`, `rotation_loop` in `rust/crates/worker/src/main.rs` | `in_progress` | `required` | Rust has loop scaffolding and queue read/DLQ behavior; legacy recovery/hygiene semantics are not yet equivalent. |
| `api/app/worker_train.py` + `api/app/worker_restaurant.py` | Separate worker entrypoints and queue domains | Single Rust worker binary currently handles one runtime queue contract | `in_progress` | `required` | Rust runtime needs explicit queue-domain topology decision (single multiplexed worker vs split worker binaries). |
| `api/app/modules/train/worker.py` | Train runtime job execution (`run_train_task`) and provider workflow | `poll_queue_once` decodes `RuntimeQueueJob` and logs job metadata | `in_progress` | `required` | Core train execution state machine is not implemented yet in Rust worker. |
| `api/app/modules/train/worker.py` | Recoverable-task enqueue and attempt pruning | `reconcile_loop` currently runs DB probe only (`select 1`) | `in_progress` | `required` | No Rust parity yet for recovery/pruning logic. |
| `api/app/modules/restaurant/worker.py` | Restaurant auth fallback + payment lease worker path | No restaurant-specific runtime in Rust worker | `in_progress` | `deferred` | Restaurant worker behavior is currently unported. |

## Required for Cutover

- Port missing `/api/auth/*` user-facing contract from `api/app/http/routes/auth.py` (session, account, passkeys, recovery/magic-link flows).
- Reach contract parity for `/api/modules` and align health contract expectations (`/api/internal/health` vs Rust health endpoints) or update callers.
- Implement DB-backed runtime job lifecycle in Rust worker (queue -> running -> terminal, retries, recovery, reconcile), replacing train-worker semantics now in Python.
- Decide and implement worker topology parity for train/restaurant queue domains currently split across `worker_train.py` and `worker_restaurant.py`.
- Port startup recovery and attempt-hygiene semantics currently supplied by `enqueue_recoverable_tasks` and `compact_and_prune_task_attempts`.

## Deferred

- `api/app/http/routes/admin.py` admin/ops route family.
- `api/app/http/routes/notifications.py` email status/test endpoints.
- `api/app/http/routes/wallet.py` payment-card route family.
- `api/app/modules/restaurant/worker.py` restaurant-specific worker policy/runtime.
