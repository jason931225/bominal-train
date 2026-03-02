# evervault-test

Standalone mini fullstack tester for verifying:
1. Browser-side Evervault JS encryption of a mock PAN.
2. Relay-side decryption arriving at a listener endpoint.
3. Go SDK outbound relay request with mock SRT payment payload + mock headers.

This project is isolated under `evervault-test/` and has no dependency on the main bominal app runtime.

## Features

- Frontend input for mock card number.
- Frontend Evervault UI Card form for encrypted number/expiry/cvc capture.
- Frontend mock SRT payment form (PAN/expiry/cvc) encrypted with Evervault JS.
- Browser encryption using Evervault JS SDK (`https://js.evervault.com/v2`).
- Go helper (`evervault-go`) dispatching outbound relay requests.
- Backend relay management (`GET/POST/PATCH /relays`) to auto-create/update the listener route.
- Listener endpoint at `/evervault-test/relay-listener` validating shared secret + session nonce.
- UI Card listener endpoint at `/evervault-test/relay-listener-card` for multi-field decrypt tests.
- Mock SRT listener endpoint at `/evervault-test/srt-listener`.
- Output includes:
  - browser encrypted token (`ev:...`)
  - browser encrypted UI Card payload
  - browser encrypted SRT payment payload
  - decrypted PAN from listener
  - masked PAN + last4 verification

Note: Evervault UI Card returns encrypted `number` + `cvc`, but `expiry.month` / `expiry.year` are plaintext digits.
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
- `EV_TEST_CARD_LISTENER_PATH` (default `/evervault-test/relay-listener-card`)
- `EV_TEST_SRT_LISTENER_PATH` (default `/evervault-test/srt-listener`)
- `EV_TEST_GO_RELAY_SENDER_BIN` (default `evervault-test/bin/relay-sender`)

## VM Routing Note (`www.bominal.com`)

Relay destination must be publicly reachable over HTTPS on `www.bominal.com`.
If your production Caddy currently routes `/evervault-test/*` to another service, temporarily add a runtime route for this tester path to `host.docker.internal:8787` (or equivalent host reachability) and reload Caddy.

Keep this route temporary and remove it after validation.

## Test Flow

1. Click **Self Check** to verify Evervault management auth.
2. Enter a mock card number.
3. Click **Run Encrypt + Relay Test**.
4. Optionally fill Evervault UI Card and click **Run UI Card + Relay Test**.
5. Fill Mock SRT Payment Inputs and click **Run SRT Payload + Go SDK Relay Test**.
6. Inspect final result payload:
   - `status: received`
   - `proof.matched_expected_last4: true`
   - `proof.browser_encrypted_pan`
   - `proof.decrypted_pan` (mock PAN in this tester)
   - for SRT flow:
     - `outbound_request.headers` (mock headers including encrypted values)
     - `outbound_request.body` (mock SRT payload)
     - `outbound_response.body` (listener response body)

## Security Notes

- Do not use real card data.
- Listener rejects requests without `shared_secret` and matching `session_nonce`.
- Mock SRT listener rejects requests without matching `X-EV-Test-Shared-Secret`.
- Backend avoids payload logging; tester API response intentionally shows full mock payload for debugging.

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

## VM Run without host Node (container)

If Node is not installed on VM, run the tester as a Docker container:

```bash
cd /opt/bominal/repo
bash evervault-test/scripts/run-container-with-vm-prod-env.sh
```

What this script does:
- Sources `infra/env/prod/api.env`, `infra/env/prod/web.env`, `infra/env/prod/caddy.env`
- Resolves `EVERVAULT_TEAM_ID` / `EVERVAULT_APP_ID` from `NEXT_PUBLIC_*` fallbacks
- Resolves `EVERVAULT_API_KEY` in this order:
  1) direct value from `api.env`
  2) environment of running `bominal-api` container
  3) GSM lookup via `gcloud` (when secret refs are configured)
- Builds image `evervault-test:local`
- Runs container `evervault-test` on host port `8787`

Stop/remove container:

```bash
bash evervault-test/scripts/stop-container.sh
```
