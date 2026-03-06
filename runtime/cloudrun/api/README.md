# bominal API/SSR (Future Cloud Run Cutover)

Prep-only assets for eventually moving the Rust `bominal-api` binary (`Leptos SSR + API`) to Cloud Run while Postgres, Redis, and the worker stay on the Compute Engine VM.
Current production deploy flow remains VM-first and does not consume these files.

## Files

- `service.yaml`: checked-in Cloud Run service template rendered by `bootstrap.sh`
- `bootstrap.sh`: renders a future service manifest from `env/prod/cloudrun-api.env`
- `.gitignore`: ignores rendered manifests under `rendered/`

## Architecture Boundary

- Active today: API + worker + Postgres + Redis on the VM.
- Future cutover: API on Cloud Run, worker + Postgres + Redis still on the VM.
- These files exist only to make that cutover an env switch plus a bootstrap/render step.

## Runtime shape

- Region: `us-central1`
- Billing: request-based (`run.googleapis.com/cpu-throttling: "true"`)
- CPU: `1`
- Memory: `512Mi`
- Startup CPU boost: enabled
- Min instances: `0`
- Max instances: `1`
- Cloud Run concurrency: `4`
- Request timeout: `35s`
- VPC egress: `private-ranges-only`

## Secret Placement

Policy:
- max `5` active GSM secrets at all times
- adding one requires removing another

Future Cloud Run GSM secrets:
- `DATABASE_URL`
- `SESSION_SECRET`
- `INTERNAL_IDENTITY_SECRET`
- `MASTER_KEY`
- `RESEND_API_KEY`

Future Cloud Run plain env:
- `REDIS_URL`
- `USER_APP_HOST`
- `ADMIN_APP_HOST`
- `SESSION_COOKIE_DOMAIN`
- `INVITE_BASE_URL`
- `EMAIL_FROM_ADDRESS`
- `WEBAUTHN_RP_ID`
- `WEBAUTHN_RP_ORIGIN`
- HTTP/DB guardrail env values

`REDIS_URL` is plain env on purpose so the future cutover stays within the 5-secret GSM policy while Redis remains an internal VM dependency with no credential-bearing URL.

## Bootstrap Env

- `env/prod/cloudrun-api.env.example` is the tracked template.
- Generate `env/prod/cloudrun-api.env` with:

```bash
./scripts/bootstrap-prod.sh --only cloudrun-api --interactive
```

Required values in that env:
- `CLOUDRUN_API_SERVICE`
- `CLOUDRUN_API_REGION`
- `CLOUDRUN_API_IMAGE`
- `CLOUDRUN_API_SERVICE_ACCOUNT`
- `CLOUDRUN_API_VPC_NETWORK`
- `CLOUDRUN_API_VPC_SUBNET`
- `CLOUDRUN_API_VPC_NETWORK_TAGS` (optional)
- `USER_APP_HOST`
- `ADMIN_APP_HOST`
- `SESSION_COOKIE_DOMAIN`
- `INVITE_BASE_URL`
- `EMAIL_FROM_ADDRESS`
- `WEBAUTHN_RP_ID`
- `WEBAUTHN_RP_ORIGIN`

## Network contract

- Cloud Run reaches Postgres and Redis over Direct VPC egress.
- `service.yaml` expects a network/subnetwork pair and optional comma-separated network tags.
- The VM firewall must allow the Cloud Run subnet CIDR or tags to reach `5432` and `6379`.

## Bootstrap

Render a service manifest:

```bash
./runtime/cloudrun/api/bootstrap.sh --env-file env/prod/cloudrun-api.env
```

That writes `runtime/cloudrun/api/rendered/<service>.yaml` and prints the next `gcloud run services replace ...` command, but does not deploy anything.
