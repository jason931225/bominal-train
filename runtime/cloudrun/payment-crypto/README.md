# bominal payment-crypto (Cloud Run, Go)

Cloud Run service that adds a KMS-backed envelope layer over Evervault-encrypted payment tokens before writing to `payment_method_secrets`.

## Endpoints

- `GET /healthz`
- `POST /v1/payment-methods/store`
- `POST /v1/payment-methods/decrypt`
- `POST /v1/payment-methods/execute-srt-payment` (`dry_run=true` only)

## Required environment variables

- `DATABASE_URL`
- `KMS_KEY_NAME`

## Optional environment variables

- `PORT` (default `8080`)
- `APP_ENV` (default `dev`)
- `PAYMENT_AAD_SCOPE` (default `gcp-kms:payment-method:ev-token-bundle:v1`)
- `PAYMENT_CRYPTO_TIMEOUT_SECONDS` (default `10`)
- `INTERNAL_SERVICE_TOKEN` (required in production; caller must send `x-internal-service-token` or `Authorization: Bearer ...`)

## Cloud Run deploy example

```bash
gcloud run deploy bominal-payment-crypto \
  --source runtime/cloudrun/payment-crypto \
  --region us-central1 \
  --allow-unauthenticated=false \
  --set-env-vars APP_ENV=prod
```

Bind runtime service account permissions:

- `roles/cloudkms.cryptoKeyEncrypterDecrypter` on `KMS_KEY_NAME`
- DB access permissions/network path for `DATABASE_URL`
