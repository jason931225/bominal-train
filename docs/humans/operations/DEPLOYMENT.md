# Deployment

This project supports separated dev/prod compose stacks with zero-downtime deployment.

## Environments

- Dev compose: `infra/docker-compose.yml`
- Prod compose: `infra/docker-compose.prod.yml`
- Dev env files: `infra/env/dev/*` (`rust.env` is the Rust runtime baseline)
- Prod env files: `infra/env/prod/*.example` -> copy to real `.env` files

Canonical deployment artifacts:
- `infra/docker-compose.prod.yml`
- `infra/scripts/deploy.sh`

Runtime policy:
- Legacy Python/Next runtime services are retired.
- Canonical runtime services are Rust-only (`api`, `worker`, `web`).

---

## Zero-Downtime Deployment

### How It Works

The deployment uses Docker Compose health checks plus callback-route smoke checks:

1. **Preflight Phase** - Deploy lock + strict preflight checks run before image pull/deploy mutation.
2. **Deploy Phase** - Script branches by runtime state:
   - first deploy (no running stack): bootstrap-safe full `up -d --wait`
   - running stack: image-aware rolling updates with `--no-deps` per service
     - roll only changed services across: `api`, `worker`, `web`
     - unchanged services are skipped to reduce restart churn and blast radius
3. **Web Failover Phase** - when `web` changes and canary is enabled:
   - start `web-canary` (`127.0.0.1:3001`) on the target image and wait healthy
   - switch Caddy runtime upstream include to `web + web-canary` and reload Caddy
   - verify `/auth/verify?...type=recovery` through Caddy returns non-5xx
   - restart primary `web` (`127.0.0.1:3000`) while Caddy can fail over to canary
   - verify callback route again, then stop/remove `web-canary`
   - switch Caddy runtime upstream include back to `web` only and reload Caddy
4. **Verify Phase** - health checks confirm API/web + callback-route reachability.
5. **Failure Phase** - on smoke-check failure, script can auto-trigger rollback to previous deployment (and intentionally leaves canary running when needed for continuity).

### Health Check Configuration

Each service has a health check in `docker-compose.prod.yml`:

| Service  | Health Check | Start Period |
|----------|--------------|--------------|
| redis    | `redis-cli ping` | 0s |
| egress-train | `wget --spider` (port 8080/health) | 10s |
| api | Python urllib (port 8000/health/live) | 300s |
| worker | Python proc check for `app.worker.WorkerSettings` | 15s |
| web      | Node `fetch` (port 3000) | 60s |
| web-canary | Node `fetch` (port 3000) | 60s |
| caddy    | `wget` (admin API port 2019) | 30s |

API health endpoints:
- `GET /health/live` -> liveness-only (process up, no dependency probes).
- `GET /health/ready` -> readiness with DB + Redis dependency probes.
- `GET /health` remains backward-compatible and mirrors readiness.
- Caddy ingress now forwards `/health`, `/health/live`, and `/health/ready` to API.

Production profile currently disables the restaurant module (`RESTAURANT_MODULE_ENABLED=false`) and does not run `egress-restaurant`.
`infra/scripts/deploy.sh` now enforces this at runtime by trimming stale `bominal-egress-restaurant` containers before deploy/rollback mutation.

The `start_period` gives the container time to initialize before health checks begin counting failures.

> **Note**: Health checks use Python for api and worker containers (slim images lack curl/pgrep)
> and 127.0.0.1 instead of localhost (Alpine DNS issue).

### Deploy Commands

**Standard Zero-Downtime Deploy** (on VM):

```bash
# Deploy current main branch
sudo -u bominal /opt/bominal/repo/infra/scripts/deploy.sh

# Deploy specific commit
sudo -u bominal /opt/bominal/repo/infra/scripts/deploy.sh abc123f

# Check deployment status
sudo -u bominal /opt/bominal/repo/infra/scripts/deploy.sh --status
```

Optional image override envs (for controlled/manual rollouts):
- `API_IMAGE`, `WORKER_IMAGE`, `WEB_IMAGE`, `WEB_CANARY_IMAGE`
- `DEPLOY_WEB_CANARY_ENABLED=true|false` controls web-canary failover (default `true`)
- Backward-compatible split overrides are still accepted but should be removed from operator automation.

Evervault secret sourcing (production):
- Set `EVERVAULT_APP_ID` directly in `infra/env/prod/api.env` (env-managed, non-secret).
- Keep `EVERVAULT_API_KEY` empty and set GSM references in `infra/env/prod/api.env`:
  - `EVERVAULT_API_KEY_SECRET_ID`, `EVERVAULT_API_KEY_SECRET_VERSION` (pinned; no `latest`)
- `infra/scripts/deploy.sh` resolves `EVERVAULT_API_KEY` from GSM at deploy time and injects redacted runtime env vars for `api` and `worker`.
- `GCP_PROJECT_ID` must be set in `infra/env/prod/api.env` when `EVERVAULT_API_KEY_SECRET_ID` is used.
- If `PAYMENT_PROVIDER=evervault`, set browser encryption vars in `infra/env/prod/web.env`:
  - `NEXT_PUBLIC_EVERVAULT_TEAM_ID`
  - `NEXT_PUBLIC_EVERVAULT_APP_ID`

### Deploy Script Safety Controls

- Single-run deploy lock (default path: `/tmp/bominal-deploy.lock`) blocks concurrent invocations.
- Strict preflight gate runs before pull/deploy:
  - env-file + placeholder checks
  - compose config validation
  - resource profile threshold gate (memory/swap)
- Smoke checks are retry-based and tunable (`SMOKE_MAX_ATTEMPTS`, `SMOKE_RETRY_DELAY_SECONDS`).
- Auth callback smoke gate is enforced during deploy and verify:
  - probe path: `/auth/verify?token_hash=aaaaaaaa&type=recovery`
  - deploy fails on callback `5xx` responses
- Caddy web-upstream runtime include is deploy-managed:
  - active include file: `infra/caddy/upstreams/active.caddy`
  - templates: `infra/caddy/upstreams/web-only.caddy`, `infra/caddy/upstreams/web-with-canary.caddy`
  - deploy reloads Caddy after include changes so active health checks do not probe canary when canary is down
- Caddy access logging is enabled in JSON to container stdout for route/latency triage (`docker logs bominal-caddy`)
- Auto rollback on smoke failure is enabled by default (`AUTO_ROLLBACK_ON_SMOKE_FAILURE=true`).
- Evervault relay verification is disabled by default and reserved for troubleshooting:
  - `DEPLOY_EVERVAULT_RELAY_VERIFY_ENABLED=false` (default)
  - `DEPLOY_EVERVAULT_RELAY_PROVIDER_PROBES_ENABLED=false` (default; provider probes can consume relay credits)
  - helper script: `infra/scripts/evervault-relay-probe.sh`
- Post-verify Docker cleanup runs on the VM after successful deploy verification:
  - unused image prune: `docker image prune -a -f` (`DEPLOY_DOCKER_PRUNE_UNUSED_IMAGES`, default `true`)
  - build cache prune: `docker builder prune -a -f` (`DEPLOY_DOCKER_PRUNE_BUILD_CACHE`, default `true`)
  - optional bominal-image retention cap: `DEPLOY_DOCKER_KEEP_BOMINAL_IMAGES=<N>` (default `0`)
    - when `N > 0`, deploy cleanup retains latest `N` bominal image tags plus currently running images, removes older bominal tags, and uses dangling-only image prune (`docker image prune -f`) instead of full `-a`.
- Repo state is checked before deployment:
  - tracked dirty state is logged
  - set `DEPLOY_FAIL_ON_DIRTY_REPO=true` to block deploy when tracked files are dirty
- Optional DB-path regression gate after smoke health checks:
  - enable with `DB_SLO_GATE_ENABLED=true`
  - threshold knobs:
    - `DB_SLO_CONNECT_P95_MAX_MS` (fail when DB connect p95 exceeds threshold)
    - `DB_SLO_AUTH_TIMEOUT_MAX` (fail when timeout/error count exceeds threshold)
    - `DB_SLO_CONNECT_ITERATIONS` (default `20`)
    - `DB_SLO_LOG_WINDOW_MINUTES` (default `30`)
  - implementation uses `infra/scripts/db-slo-check.sh`
- Threshold knobs:
  - `DEPLOY_MIN_TOTAL_MEMORY_MB` (default `900`)
  - `DEPLOY_MIN_TOTAL_SWAP_MB` (default `900`)
- e2-micro tuned service limits in production compose:
  - steady-state: `api=320M`, `web=200M`, `worker=224M`, `egress-train=96M`, `redis=64M`
  - rollout-only: `web-canary=200M` (temporary during web cutover windows)
- Deprecation deploy gate is enforced during predeploy:
  - registry: `docs/deprecations/registry.json`
  - policy: `docs/governance/DEPRECATION_POLICY.md`
  - guard command: `python3 infra/scripts/deprecation_guard.py enforce-deploy ...`
  - host requirement: `python3` available on deploy VM
- Host-side Python verification protocol:
  - `uv` must be installed on the VM.
  - host-side `pytest` runs must execute through a `uv`-managed `api/.venv`.
  - bootstrap helper: `bash infra/scripts/ensure-uv-api-venv.sh`
- Emergency bypass (approval required):
  - `PREDEPLOY_ALLOW_DEPRECATION_BYPASS=true`

**CI Deploy (Recommended: Pub/Sub, no SSH)**

GitHub Actions publishes a deploy request to Pub/Sub (authenticated via WIF).
The VM runs a pull-based deploy agent (systemd) that consumes the request and
runs `infra/scripts/deploy.sh` locally.

CI-triggered deploys use a **latest baseline with per-service commit-tag overrides**:
- changed services are pinned to `ghcr.io/...:<commit_sha>`;
- unchanged services stay on `:latest`.
Canonical workflow files:
- `.github/workflows/ci-infra-quality-gates.yml`
- `.github/workflows/ci-build-publish-images.yml`
- `.github/workflows/cd-deploy-production.yml`
Deploy gating in CI:
- Deploy workflow is auto-triggered only when a GitHub Release is published (tagged release), plus optional manual dispatch.
- Release/manual deploys are gated against prerequisite workflow status for the selected commit.
- Before publish, CI blocks deploy unless **both** `CI - Infra Quality Gates` and `CI - Build and Publish Images` for the same commit completed with `success`.
- `CI - Build and Publish Images` manual dispatch supports `build_api` / `build_web` boolean inputs so operators can avoid unneeded image publishes.
- Deprecation policy checks run in `CI - Infra Quality Gates` and deploy preflight; CD publish no longer duplicates that check set.
- Image publish job is fail-closed on Trivy scan findings (`HIGH` / `CRITICAL`) and emits SBOM/provenance attestations.
- API mutation smoke gate in infra quality checks installs `uv` explicitly and uses deterministic fallback paths (`uv` -> host `python3` -> `docker compose run`) without requiring a pre-running `api` container.
- After publish, CI runs a post-deploy verification gate:
  - production API health must report `db=true` and `redis=true`;
  - production web endpoint must return `200` or `3xx`.

One-time GCP setup:

```bash
# Topic (publisher: GitHub Actions)
gcloud pubsub topics create bominal-deploy-requests --project bominal

# Subscription (consumer: VM)
gcloud pubsub subscriptions create bominal-deploy-requests-vm \
  --project bominal \
  --topic bominal-deploy-requests \
  --ack-deadline 600

# Allow GitHub Actions SA to publish
gcloud pubsub topics add-iam-policy-binding bominal-deploy-requests \
  --project bominal \
  --member="serviceAccount:github-actions@bominal.iam.gserviceaccount.com" \
  --role="roles/pubsub.publisher"

# Allow the VM service account to pull/ack (replace with the instance SA email)
gcloud compute instances describe bominal-deploy --zone us-central1-a \
  --format='value(serviceAccounts.email)'

gcloud pubsub subscriptions add-iam-policy-binding bominal-deploy-requests-vm \
  --project bominal \
  --member="serviceAccount:<VM_SERVICE_ACCOUNT_EMAIL>" \
  --role="roles/pubsub.subscriber"
```

VM install (one-time):

```bash
# Copy systemd unit
sudo cp /opt/bominal/repo/infra/systemd/bominal-deploy-agent.service /etc/systemd/system/

# Configure agent env
sudo mkdir -p /etc/bominal
sudo tee /etc/bominal/deploy-agent.env >/dev/null <<'EOF'
GCP_PROJECT_ID=bominal
GCP_REGION=us-central1
DEPLOY_SUBSCRIPTION=bominal-deploy-requests-vm
REPO_DIR=/opt/bominal/repo
DEPLOY_SCRIPT=/opt/bominal/repo/infra/scripts/deploy.sh
GHCR_NAMESPACE=ghcr.io/jason931225/bominal
# Optional for private GHCR packages:
# GHCR_USERNAME=CHANGE_ME_GHCR_USERNAME
# GHCR_TOKEN=CHANGE_ME_GHCR_TOKEN
EOF

sudo systemctl daemon-reload
sudo systemctl enable --now bominal-deploy-agent
sudo journalctl -u bominal-deploy-agent -f
```

Safety note:
- Deploy agent is fail-closed for non-canonical script paths.
- `DEPLOY_SCRIPT` must be `/opt/bominal/repo/infra/scripts/deploy.sh`.
- Deployment messages are ACKed only after successful deploy execution.

**Remote Deploy** (from local machine):

```bash
gcloud compute ssh bominal-deploy --zone=us-central1-a --tunnel-through-iap \
  --command="cd /opt/bominal/repo && sudo -u bominal infra/scripts/deploy.sh"
```

IAP tunnel policy:
- Every `gcloud compute ssh` command for this project must include `--tunnel-through-iap`.
- Direct/non-IAP SSH to production infrastructure is not permitted.

---

## Rollback Procedures

### Automatic Rollback

The deployment script tracks versions in `/opt/bominal/deployments/`:

```bash
# Rollback to previous deployment
sudo -u bominal /opt/bominal/repo/infra/scripts/deploy.sh --rollback
```

### Manual Rollback

If the script fails, manually rollback:

```bash
cd /opt/bominal/repo

# Find the previous good commit
cat /opt/bominal/deployments/previous
# Or: git log --oneline -10

# Checkout and redeploy
sudo -u bominal git checkout <commit>
sudo docker compose -f infra/docker-compose.prod.yml build api web
sudo docker compose -f infra/docker-compose.prod.yml up -d --wait
```

### Database Migration Rollback

If a migration was applied and needs reverting:

```bash
# 1. Check current migration state
sudo docker compose -f infra/docker-compose.prod.yml exec api alembic current

# 2. Downgrade to specific revision
sudo docker compose -f infra/docker-compose.prod.yml exec api alembic downgrade <revision>

# 3. Checkout old code and redeploy
sudo -u bominal git checkout <commit>
sudo docker compose -f infra/docker-compose.prod.yml build api
sudo docker compose -f infra/docker-compose.prod.yml up -d --wait api worker
```

### Version Tracking

The deployment system maintains:

| File | Purpose |
|------|---------|
| `/opt/bominal/deployments/current` | Currently deployed commit hash |
| `/opt/bominal/deployments/previous` | Previous deployment (for rollback) |
| `/opt/bominal/deployments/<timestamp>` | Historical deployment records |

Each `<timestamp>` record includes the deployed commit and image references. Newer
records also include `api_digest` / `web_digest` (immutable `name@sha256:...`)
which the rollback path prefers for deterministic rollbacks (even if `:latest`
has moved).

Only the last 10 deployment records are kept.

---

## Production Bootstrap (GCE e2-micro)

### 1) Create env files

```bash
bash infra/scripts/bootstrap-prod-env.sh
```

Optional:
- `infra/env/prod/deploy.env` can be created from `infra/env/prod/deploy.env.example` for helper workflows.
- Canonical `infra/scripts/deploy.sh` does not require `deploy.env`.

### 2) Configure secrets

`infra/scripts/bootstrap-prod-env.sh` is the canonical bootstrap path. It:
- prompts interactively for required sensitive values,
- writes `infra/env/prod/api.env`, `infra/env/prod/web.env`, and `infra/env/prod/caddy.env`,
- optionally writes `infra/env/prod/deploy.env`,
- validates critical contracts (Supabase URLs, `MASTER_KEY` format, unresolved placeholders).

If you choose manual editing instead, required values are:
- `infra/env/prod/api.env`: `DATABASE_URL`, `SYNC_DATABASE_URL`, `AUTH_MODE=supabase`, `SUPABASE_URL`, `SUPABASE_JWT_ISSUER`, `SUPABASE_AUTH_ENABLED=true`, `SUPABASE_AUTH_API_KEY` (or `SUPABASE_SERVICE_ROLE_KEY` fallback), `SUPABASE_STORAGE_ENABLED=true`, `SUPABASE_SERVICE_ROLE_KEY`, sender-domain placeholder in `EMAIL_FROM_ADDRESS`, passkey origin settings (`PASSKEY_RP_ID`, `PASSKEY_ORIGIN`), optional provider session-cache knobs (`TRAIN_PROVIDER_CLIENT_CACHE_SECONDS`, `TRAIN_PROVIDER_CLIENT_CACHE_MAX_ENTRIES`), optional edge-notify knobs (`EDGE_TASK_NOTIFY_ENABLED`, `SUPABASE_EDGE_FUNCTIONS_BASE_URL`, `SUPABASE_EDGE_TASK_NOTIFY_FUNCTION_NAME`, `SUPABASE_EDGE_TIMEOUT_SECONDS`), Evervault app identifier (`EVERVAULT_APP_ID`), Evervault API key source (`EVERVAULT_API_KEY` or `EVERVAULT_API_KEY_SECRET_ID` + pinned version), optional pinned relay mapping (`EVERVAULT_{KTX,SRT}_PAYMENT_RELAY_ID`, `EVERVAULT_{KTX,SRT}_PAYMENT_RELAY_DOMAIN`), and valid secret sources for `MASTER_KEY`, `INTERNAL_API_KEY`, and `RESEND_API_KEY` (prefer GSM references)
- `infra/env/prod/api.env`: payment cutover contract when `PAYMENT_ENABLED=true`:
  - `PAYMENT_PROVIDER=evervault`
  - `PAYMENT_EVERVAULT_ENFORCE=true`
  - `AUTOPAY_REQUIRE_USER_WALLET=true`
  - `AUTOPAY_ALLOW_SERVER_FALLBACK=false`
- `infra/env/prod/api.env`: optional Supabase Auth redirect overrides:
  - `SUPABASE_AUTH_SITE_URL` (defaults to `NEXT_PUBLIC_API_BASE_URL`, then `https://$CADDY_SITE_ADDRESS`)
  - `SUPABASE_AUTH_REDIRECT_URLS` (comma-separated allow-list; defaults to `<site_url>/auth/verify,<site_url>/auth/confirm,<site_url>/reset-password,<site_url>/login`)
- `infra/env/prod/web.env`: `NEXT_PUBLIC_API_BASE_URL`, `API_SERVER_URL` (`http://api:8000` for monolithic API runtime), and Supabase browser auth/realtime/read-path keys (`NEXT_PUBLIC_SUPABASE_DIRECT_AUTH_ENABLED`, `NEXT_PUBLIC_SUPABASE_REALTIME_ENABLED`, `NEXT_PUBLIC_SUPABASE_REALTIME_DELTA_READ_ENABLED`, `NEXT_PUBLIC_TRAIN_READS_VIA_DATA_API`, `NEXT_PUBLIC_TRAIN_DETAIL_VIA_GRAPHQL`, `NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_ENABLED`, `NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_RETRY_SECONDS`, `NEXT_PUBLIC_SUPABASE_URL`, `NEXT_PUBLIC_SUPABASE_ANON_KEY`)
- `infra/env/prod/caddy.env`: `CADDY_SITE_ADDRESS`, `CADDY_ACME_EMAIL`
- `infra/env/prod/deploy.env` (optional helper): set `GHCR_USERNAME` + `GHCR_TOKEN` when GHCR packages are private

Production auth/storage mode (hard gate):
- `AUTH_MODE` must be `supabase`
- `SUPABASE_AUTH_ENABLED` must be `true` and requires `SUPABASE_AUTH_API_KEY` or `SUPABASE_SERVICE_ROLE_KEY`, plus positive `SUPABASE_AUTH_TIMEOUT_SECONDS`
- `SUPABASE_STORAGE_ENABLED` must be `true` and requires `SUPABASE_SERVICE_ROLE_KEY`
- When `EDGE_TASK_NOTIFY_ENABLED=true`, `SUPABASE_SERVICE_ROLE_KEY` is required; `SUPABASE_EDGE_FUNCTIONS_BASE_URL` must be `https://` when set; and `SUPABASE_EDGE_TIMEOUT_SECONDS` must be positive when set.

Production master-key source contract:
- If `GSM_MASTER_KEY_ENABLED=true`:
  - provide `GSM_MASTER_KEY_PROJECT_ID` (or `GCP_PROJECT_ID`), `GSM_MASTER_KEY_SECRET_ID`, and pinned `GSM_MASTER_KEY_VERSION`
  - `GSM_MASTER_KEY_VERSION=latest` is rejected
  - `GSM_MASTER_KEY_ALLOW_ENV_FALLBACK` must be `false`
  - `deploy.sh` fetches the secret and injects it as runtime-only `MASTER_KEY_OVERRIDE` for `api` and `worker`
- If `GSM_MASTER_KEY_ENABLED=false`:
  - `MASTER_KEY` must be set to a base64-encoded 32-byte key

Production internal API key source contract:
- Configure exactly one source:
  - `INTERNAL_API_KEY`, or
  - `INTERNAL_API_KEY_SECRET_ID` + pinned `INTERNAL_API_KEY_SECRET_VERSION`
- `INTERNAL_API_KEY_SECRET_ID` requires `GCP_PROJECT_ID`.
- `deploy.sh` resolves GSM references and injects runtime `INTERNAL_API_KEY` into `api`/`worker`.

Production Resend key source contract (`EMAIL_PROVIDER=resend`):
- Configure exactly one source:
  - `RESEND_API_KEY`, or
  - `RESEND_API_KEY_SECRET_ID` + pinned `RESEND_API_KEY_SECRET_VERSION`, or
  - `RESEND_API_KEY_VAULT_NAME` (edge-only contract)
- `RESEND_API_KEY_VAULT_NAME` is allowed only when `EDGE_TASK_NOTIFY_ENABLED=true` and `SUPABASE_VAULT_ENABLED=true`.
- `RESEND_API_KEY_SECRET_ID` requires `GCP_PROJECT_ID`.
- `deploy.sh` resolves GSM references and injects runtime `RESEND_API_KEY` into `api`/`worker`.

Production Supabase Management API token contract (`sync-supabase-auth-templates.sh --apply`):
- Configure `SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID` + pinned `SUPABASE_MANAGEMENT_API_TOKEN_SECRET_VERSION`.
- Configure `SUPABASE_MANAGEMENT_API_TOKEN_PROJECT_ID` or rely on `GCP_PROJECT_ID`.
- `latest` is rejected for `SUPABASE_MANAGEMENT_API_TOKEN_SECRET_VERSION`.
- Plaintext `SUPABASE_MANAGEMENT_API_TOKEN` and `SUPABASE_ACCESS_TOKEN` must not be stored in `infra/env/prod/api.env`.

Production payment safety contract (Evervault-only, fail-closed):
- Server-side card fallback aliases (`CARDNUMBER`, `EXPIRYMM`, `EXPIRYYY`, `DOB`, `NN`) are forbidden in `infra/env/prod/api.env`.
- `infra/env/prod/pay.env` is retired and must not be referenced by deployment compose/runtime.
- Admin server-wide payment-card override routes are retired (`PUT/DELETE /api/admin/payment-settings/card` return `410 Gone`).
- Admin kill switch (`PATCH /api/admin/payment-settings/enabled`) is authoritative for runtime dispatch: when disabled, API/worker payment calls fail closed before Evervault/provider payment submission.

Optional helper to automate GSM setup from existing `MASTER_KEY` in `api.env`:

```bash
bash infra/scripts/setup-gsm-master-key.sh --project-id <gcp_project_id>
# or
python3 infra/scripts/setup-gsm-master-key.py --project-id <gcp_project_id>
```

By default, helper IAM bindings target `bominal-runtime@<project>.iam.gserviceaccount.com`.
Override with `--runtime-service-account-email <sa_email>` when needed.

Production URL scheme enforcement (predeploy gate):
- `SUPABASE_URL` and `SUPABASE_JWT_ISSUER` must be `https://`.
- `CORS_ORIGINS` entries must be `https://`.
- `RESEND_API_BASE_URL` must be `https://` when set.
- `NEXT_PUBLIC_API_BASE_URL` may be empty (recommended same-origin) or must be `https://` if set.
- When any of `NEXT_PUBLIC_SUPABASE_DIRECT_AUTH_ENABLED=true`, `NEXT_PUBLIC_SUPABASE_REALTIME_ENABLED=true`, `NEXT_PUBLIC_SUPABASE_REALTIME_DELTA_READ_ENABLED=true`, `NEXT_PUBLIC_TRAIN_READS_VIA_DATA_API=true`, or `NEXT_PUBLIC_TRAIN_DETAIL_VIA_GRAPHQL=true`, `NEXT_PUBLIC_SUPABASE_URL` and `NEXT_PUBLIC_SUPABASE_ANON_KEY` are required, and `NEXT_PUBLIC_SUPABASE_URL` must be `https://`.
- `API_SERVER_URL` must be an absolute `http(s)://` URL.
- Supabase auth redirect URLs must resolve to `https://`, must not use localhost/loopback hosts, and must include `/auth/verify`.
- `EMAIL_PROVIDER=disabled`: Resend credentials may remain unset
- `EMAIL_PROVIDER=smtp`: `SMTP_HOST`, `SMTP_PORT`, and SMTP credentials/TLS settings as required
- `EMAIL_PROVIDER=resend`: configure exactly one source (`RESEND_API_KEY`, `RESEND_API_KEY_SECRET_ID` + pinned version, or edge-only `RESEND_API_KEY_VAULT_NAME` with `SUPABASE_VAULT_ENABLED=true` and `EDGE_TASK_NOTIFY_ENABLED=true`)
- `TRAIN_PROVIDER_EGRESS_PROXY_URL` / `RESTAURANT_PROVIDER_EGRESS_PROXY_URL`: set to internal egress gateways when outbound provider traffic must be centralized through path-allowlist proxies
- `NEXT_PUBLIC_FONT_BASE_URL`: optional remote font base URL (must be `https://` when set). Expected files at that base path: `NotoSansKR-Regular.woff2`, `NotoSerifKR-Regular.woff2`, `NotoSerifKR-SemiBold.woff2`, `NotoSerifKR-Bold.woff2`, `DynaPuff-SemiBold.woff2`
- Train live-event cutover flags:
  - `NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_ENABLED`: enables realtime-primary transport manager (SSE remains automatic fallback during deprecation window).
  - `NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_RETRY_SECONDS`: retry cadence while temporarily operating on SSE fallback.
  - Immediate rollback: set `NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_ENABLED=false` and redeploy web.

Production note: `DATABASE_URL` / `SYNC_DATABASE_URL` must target Supabase Postgres (`*.supabase.co`) with TLS required (`sslmode=require` or equivalent). Local development must use Docker-local Postgres/Redis (no VM/remote Postgres URLs).

Generate secure `MASTER_KEY` (required only when GSM is disabled):

```bash
openssl rand -base64 32
```

If using GSM, store the generated value as the Secret Manager payload value and pin a concrete version in `GSM_MASTER_KEY_VERSION`.

### 3) Run predeploy checks

```bash
bash infra/scripts/predeploy-check.sh \
  --min-total-memory-mb 900 \
  --min-total-swap-mb 900
```

`deploy.sh` runs this gate automatically before pull/deploy. Running it manually is still recommended for operator visibility.

### 3.1) Sync Supabase auth templates and redirect targets

After production env files are finalized, sync Supabase auth email templates and auth redirect/site URL config:

- confirmation template + subject
- password recovery template + subject
- one-time sign-in template + subject (magic-link email)

```bash
bash infra/scripts/sync-supabase-auth-templates.sh --dry-run
bash infra/scripts/sync-supabase-auth-templates.sh --apply
```

`--apply` in production resolves token from GSM via:
- `SUPABASE_MANAGEMENT_API_TOKEN_SECRET_ID`
- `SUPABASE_MANAGEMENT_API_TOKEN_SECRET_VERSION` (pinned; no `latest`)
- `SUPABASE_MANAGEMENT_API_TOKEN_PROJECT_ID` (optional; falls back to `GCP_PROJECT_ID`)

Local/non-production fallback remains available with exported `SUPABASE_MANAGEMENT_API_TOKEN` or `SUPABASE_ACCESS_TOKEN`.

### 3.2) Sync Supabase Edge notify secrets from GSM

When `EDGE_TASK_NOTIFY_ENABLED=true`, sync delivery secrets for `task-notify` from authoritative GSM:

```bash
bash infra/scripts/sync-edge-secrets-from-gsm.sh --dry-run
bash infra/scripts/sync-edge-secrets-from-gsm.sh --apply
```

Expected source/target:
- source: `RESEND_API_KEY_SECRET_ID` + `RESEND_API_KEY_SECRET_VERSION` in GSM
- target edge secrets: `RESEND_API_KEY`, `EMAIL_FROM_ADDRESS`, optional `EMAIL_FROM_NAME`

Rollback:
- disable edge notify (`EDGE_TASK_NOTIFY_ENABLED=false`) and redeploy to force queue-only delivery
- or resync previous GSM version by pinning older `RESEND_API_KEY_SECRET_VERSION` then rerunning `--apply`

Deprecation gate behavior:
- `predeploy-check.sh` validates the registry and enforces production deprecation deadlines.
- Deploy fails if removed artifacts are still referenced or deadline-past production deprecations are unresolved.
- Bypass is allowed only with explicit approval:
  - `PREDEPLOY_ALLOW_DEPRECATION_BYPASS=true bash infra/scripts/predeploy-check.sh --skip-smoke-tests`

Optional manual pre-migration duplicate check:

```bash
docker compose -f infra/docker-compose.prod.yml run --rm api python scripts/check_duplicate_display_names.py
```

### 4) Initial Deploy

```bash
bash infra/scripts/deploy.sh
```

The API startup runs duplicate `display_name` check before `alembic upgrade head`.

## GCE e2-micro (Free Tier) Setup

The e2-micro has 1GB RAM and 0.25 vCPU. Key optimizations:

- **Swap**: 1GB swap file created by `vm-docker-bootstrap.sh`
- **Memory limits**: All containers have memory caps to prevent OOM
- **Pre-built images**: deployment pulls GHCR images; no image build happens on the VM during normal deploy
- **Container hardening**: production images run as non-root users; web image uses Next.js standalone runtime to reduce size/startup overhead
- **Build efficiency**: production Dockerfiles use BuildKit cache mounts for `pip`/`npm` dependency install layers, and web uses `npm ci --prefer-offline --no-audit --no-fund` on the cached npm store

### 1) One-time VM bootstrap

Run the bootstrap script as root:

```bash
sudo bash infra/scripts/vm-docker-bootstrap.sh
```

This installs Docker, creates a `bominal` user, and adds 1GB swap.
It also installs `uv` at `/usr/local/bin/uv` for host-side Python venv execution.

Verify after bootstrap:

```bash
uv --version
```

### 2) Clone repo and configure

```bash
sudo -u bominal -i
cd /opt/bominal
git clone https://github.com/<owner>/<repo>.git repo
cd repo
git submodule update --init --recursive
```

Create prod env files:

```bash
bash infra/scripts/bootstrap-prod-env.sh
```

The script prompts for required values and writes production env files safely.
Review the generated files before deploy and ensure they are not committed.

Generate secrets:

```bash
# MASTER_KEY (for encryption)
openssl rand -base64 32

# INTERNAL_API_KEY
openssl rand -hex 32
```

Prepare API host venv for any host-side Python checks:

```bash
bash infra/scripts/ensure-uv-api-venv.sh
```

Run host-side API tests through uv-managed venv (if needed):

```bash
cd api
uv run --python .venv/bin/python -m pytest -q
```

### 3) Deploy

```bash
bash infra/scripts/deploy.sh
```

This will:
1. Acquire deploy lock and run preflight gate
2. Pull pre-built API/Web images from GHCR
3. Choose first-deploy bootstrap path or rolling-update path automatically
4. Run smoke checks, then optionally auto-rollback if smoke verification fails

### 4) Verify

```bash
curl -sS https://www.bominal.com/health
docker compose -f infra/docker-compose.prod.yml logs --tail=50
```

---

## Monitoring Deployment Health

### Container Health

```bash
# Check all container health statuses
docker ps --format "table {{.Names}}\t{{.Status}}"

# Watch health in real-time
watch -n2 'docker ps --format "table {{.Names}}\t{{.Status}}" | grep bominal'
```

### Service Health Endpoints

```bash
# API health
curl -s http://127.0.0.1:8000/health

# Web via Caddy
curl -sI http://127.0.0.1/

# Full external check
curl -sI https://www.bominal.com/
```

### Logs During Deploy

```bash
# Follow logs during deployment
sudo docker compose -f infra/docker-compose.prod.yml logs -f --tail=50 api worker web
```

### Runtime Pressure Checks

```bash
# Host pressure
uptime
free -h

# Container resource usage
sudo docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}"

# Worker restart/health guard
sudo docker inspect -f '{{.Name}} restart={{.RestartCount}} status={{.State.Status}} health={{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}' bominal-worker

# Zombie count trend
ps -eo stat | awk '$1 ~ /^Z/ {c++} END {print c+0}'
```

---

## Troubleshooting

### Deploy Hangs on Health Check

If deployment hangs waiting for a container to become healthy:

1. Check container logs:
   ```bash
   docker logs bominal-api --tail=100
   ```

2. Check what's failing:
   ```bash
   docker inspect bominal-api --format='{{json .State.Health}}'
   ```

3. Force restart if needed:
   ```bash
   docker compose -f infra/docker-compose.prod.yml restart api
   ```

### Container Starts but Unhealthy

Common causes:
- **API**: Database migration failed, missing env vars
- **Web**: Build failed, missing env vars
- **Worker**: Redis connection failed

Check logs:
```bash
docker logs bominal-api 2>&1 | tail -50
docker logs bominal-web 2>&1 | tail -50
```

### Web Process Is Healthy but Zombie Count Keeps Growing

Symptoms:
- `bominal-web` stays healthy, but host zombie count keeps increasing.
- Zombies map to the web parent PID.

Actions:
1. Recreate only web service (do not restart entire stack):

```bash
sudo -u bominal docker compose -f /opt/bominal/repo/infra/docker-compose.prod.yml up -d --no-deps --force-recreate web
```

2. Verify web/worker health and restart stability:

```bash
sudo -u bominal docker compose -f /opt/bominal/repo/infra/docker-compose.prod.yml ps --format "table {{.Name}}\t{{.Status}}" web worker
sudo docker inspect -f '{{.Name}} restart={{.RestartCount}} status={{.State.Status}} health={{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}' bominal-worker
ps -eo stat | awk '$1 ~ /^Z/ {c++} END {print c+0}'
```

### Rollback Not Working

If deployment history is corrupted:

```bash
# Find recent commits
cd /opt/bominal/repo
git log --oneline -20

# Manually set version pointers
echo "abc123f" > /opt/bominal/deployments/current
echo "def456a" > /opt/bominal/deployments/previous

# Then retry rollback
sudo -u bominal infra/scripts/deploy.sh --rollback
```

Malformed historical records (e.g. bad files under `/opt/bominal/deployments/<timestamp>`)
should not break `--status` or deploy/rollback runs; the script will warn and skip
records it can’t read.

If you want to remove legacy/malformed historical records without touching the
`current` / `previous` pointers, run:

```bash
# Preview what would be deleted
sudo -u bominal /opt/bominal/repo/infra/scripts/deploy.sh --purge-legacy-records --dry-run

# Create a tarball backup and then delete legacy/malformed records
sudo -u bominal /opt/bominal/repo/infra/scripts/deploy.sh --purge-legacy-records --backup
```

---

## Network Guidance

- One Debian VM running Docker/Compose
- Caddy terminates TLS via ACME (Let's Encrypt)
- Containers private on bridge network; expose only 80/443 publicly

Firewall rules:

- VM tags: `bominal-web`, `bominal-ops`
- Public ingress: TCP `443` and `80` (for ACME challenge + HTTP->HTTPS redirect)
- SSH via IAP only (`35.235.240.0/20`)
- Do not open Postgres/Redis/API/Web internal ports publicly

## Domain Layout

Current Caddy setup is single-host path routing:

- `www.bominal.com` -> web (`3000`) for UI routes
- `www.bominal.com/api/*` -> api (`8000`)
- `www.bominal.com/health` and `/healthz` -> api health endpoints
- `bominal.com` -> redirects to `www.bominal.com`

Configure domain + ACME contact in `infra/env/prod/caddy.env`:

- `CADDY_SITE_ADDRESS=www.bominal.com`
- `CADDY_ACME_EMAIL=ops@bominal.com`

---

## For AI Agents

> **IMPORTANT**: Read this section before making deployment changes.

### Deployment Rules

1. **Always use the zero-downtime script** for production deploys:
   ```bash
   sudo -u bominal /opt/bominal/repo/infra/scripts/deploy.sh
   ```

2. **Never modify these files** without explicit user approval:
   - `infra/scripts/deploy.sh`
   - `infra/docker-compose.prod.yml` (health checks section)
   - `/opt/bominal/deployments/*` (version tracking)

3. **Preserve version history** - the deployment system tracks:
   - `/opt/bominal/deployments/current` - current commit
   - `/opt/bominal/deployments/previous` - for rollback

4. **Test changes locally first** when possible:
   ```bash
   docker compose -f infra/docker-compose.yml up --build
   ```

5. **Always verify after deploy**:
   ```bash
   curl -s https://www.bominal.com/health
   docker ps --format "table {{.Names}}\t{{.Status}}" | grep bominal
   ```

### Rollback Procedure for Agents

If a deployment causes issues:

```bash
# Automatic rollback (preferred)
sudo -u bominal /opt/bominal/repo/infra/scripts/deploy.sh --rollback

# Manual rollback if automatic fails
cd /opt/bominal/repo
cat /opt/bominal/deployments/previous  # Get previous commit
sudo -u bominal git checkout <commit>
sudo docker compose -f infra/docker-compose.prod.yml build api web
sudo docker compose -f infra/docker-compose.prod.yml up -d --wait
```

### What Triggers Downtime

These actions **will cause downtime**:
- `docker compose down` (stops all containers)
- `docker compose restart` without health checks
- Breaking migrations without rollback plan

These actions are **zero-downtime**:
- `deploy.sh` (uses health checks)
- `docker compose up -d --wait` (waits for healthy)
- Code changes without schema changes

---

## Scaling Path

Phase 1 (current): single VM with zero-downtime deploys.

Phase 2:
- Move Postgres to managed DB (Cloud SQL)
- Move Redis to managed Redis (Memorystore)
- Keep API/worker on Compute Engine

Phase 3:
- Add load balancer with multiple app nodes
- Blue-green or canary deployments
- CI/CD with migration gating + secret manager integration
