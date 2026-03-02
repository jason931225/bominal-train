# evervault-test

Standalone mini fullstack tester for verifying:
1. Browser-side Evervault JS encryption of a mock PAN.
2. Relay-side decryption arriving at a listener endpoint.

This project is isolated under `evervault-test/` and has no dependency on the main bominal app runtime.

## Features

- Frontend input for mock card number.
- Browser encryption using Evervault JS SDK (`https://js.evervault.com/v2`).
- Backend relay management (`GET/POST/PATCH /relays`) to auto-create/update the listener route.
- Listener endpoint at `/evervault-test/relay-listener` validating shared secret + session nonce.
- One-time full PAN proof mode (explicit unsafe toggle); default output remains masked.
- In-memory session storage with TTL (no DB or disk persistence).

## Local Run

```bash
cd evervault-test
cp .env.example .env
npm install --cache /tmp/evervault-test-npm-cache
npm start
```

Open `http://localhost:8787`.

## Required Environment

- `EVERVAULT_TEAM_ID`
- `EVERVAULT_APP_ID`
- `EVERVAULT_API_KEY`
- `EV_TEST_SHARED_SECRET`

Optional:
- `EVERVAULT_API_BASE_URL` (default `https://api.evervault.com`)
- `EV_TEST_DESTINATION_DOMAIN` (default `www.bominal.com`)
- `EV_TEST_LISTENER_PATH` (default `/evervault-test/relay-listener`)

## VM Routing Note (`www.bominal.com`)

Relay destination must be publicly reachable over HTTPS on `www.bominal.com`.
If your production Caddy currently routes `/evervault-test/*` to another service, temporarily add a runtime route for this tester path to `host.docker.internal:8787` (or equivalent host reachability) and reload Caddy.

Keep this route temporary and remove it after validation.

## Test Flow

1. Click **Self Check** to verify Evervault management auth.
2. Enter a mock card number.
3. Optionally enable **Allow one-time full PAN proof in result**.
4. Click **Run Encrypt + Relay Test**.
5. Inspect final result payload:
   - `status: received`
   - `proof.matched_expected_last4: true`
   - `proof.full_pan_once` appears only once when unsafe mode is enabled

## Security Notes

- Do not use real card data.
- Listener rejects requests without `shared_secret` and matching `session_nonce`.
- Backend avoids payload logging and only returns masked PAN by default.

## VM Run with prod env files

From repo root on VM:

```bash
npm --prefix evervault-test install --cache /tmp/evervault-test-npm-cache
bash evervault-test/scripts/start-with-vm-prod-env.sh
```

This script sources:
- `infra/env/prod/api.env`
- `infra/env/prod/web.env`
- `infra/env/prod/caddy.env`

And maps:
- `NEXT_PUBLIC_EVERVAULT_TEAM_ID` -> `EVERVAULT_TEAM_ID` (fallback)
- `NEXT_PUBLIC_EVERVAULT_APP_ID` -> `EVERVAULT_APP_ID` (fallback)
- `CADDY_SITE_ADDRESS` -> `EV_TEST_DESTINATION_DOMAIN` (when unset)

If `EVERVAULT_API_KEY` is empty but GSM refs are present, it resolves key via `gcloud secrets versions access`.
