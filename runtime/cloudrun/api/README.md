# bominal API/SSR (Cloud Run, Rust)

Cloud Run service definition for the Rust `bominal-api` binary (`Leptos SSR + API`) when Postgres, Redis, and the worker stay on the Compute Engine VM.

## Files

- `service.yaml`: checked-in Cloud Run service template rendered by `.github/workflows/cd.yml`

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

## Required GitHub production variables

- `GCP_REGION`
- `ARTIFACT_REGISTRY_REPOSITORY`
- `CLOUDRUN_API_SERVICE`
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

## Required Secret Manager secrets

These names are fixed in `service.yaml` and must exist in the deploy project:

- `DATABASE_URL`
- `REDIS_URL`
- `SESSION_SECRET`
- `INTERNAL_IDENTITY_SECRET`
- `MASTER_KEY`
- `RESEND_API_KEY`

## Network contract

- Cloud Run reaches Postgres and Redis over Direct VPC egress.
- `service.yaml` expects a network/subnetwork pair and optional network tags.
- The VM firewall must allow the Cloud Run subnet CIDR or tags to reach `5432` and `6379`.

## Deploy notes

- CD copies the API image from GHCR to Artifact Registry using the commit SHA tag before the Cloud Run deploy.
- The deploy workflow renders `service.yaml`, replaces the Cloud Run service, grants public invoker access, then smoke-tests `/health` and `/ready` on the Cloud Run service URL before deploying the worker on the VM.
