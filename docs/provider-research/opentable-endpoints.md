# OpenTable Endpoint Contract Notes

Redacted provider-contract notes for OpenTable integration. This document intentionally excludes raw credentials/tokens/cookies.

## Endpoint summary

| Endpoint | Method | Canonical operation | Current status |
|---|---|---|---|
| `/dapi/authentication/sendotpfromsignin` | `POST` | `auth.start` | observed |
| `/dapi/authentication/signinwithotp` | `POST` | `auth.complete` | observed |
| `/dapi/fe/human` | `GET` | `auth.refresh` (supporting) | observed |
| `/dapi/v1/session` | `POST` | `auth.refresh` (supporting) | observed |
| `/_sec/cpr/params` | `GET` | security/session support (non-canonical helper) | observed |
| `/dapi/fe/gql?optype=query&opname=HeaderUserProfile` | `POST` | `profile.get` | observed |
| `/dapi/fe/gql?optype=mutation&opname=CancelReservation` | `POST` | `reservation.cancel` | observed |
| `/dapi/authentication/logout` | `POST` | logout | observed |

Required for full feature but not yet contract-frozen:

- reservation-create mutation (`reservation.create`)
- stable availability/search query operation (`search.availability`)

## Adapter implementation status (2026-02-17)

Implemented in `api/app/modules/restaurant/providers/opentable_adapter.py`:

- `auth.refresh`: calls `/_sec/cpr/params`, `/dapi/fe/human`, and `/dapi/v1/session`
- `profile.get`: calls `HeaderUserProfile` persisted query
- `reservation.cancel`: calls `CancelReservation` persisted mutation
- `auth.start`: `POST /dapi/authentication/sendotpfromsignin`
- `auth.complete`: `POST /dapi/authentication/signinwithotp`
- `search.availability`: fixed operation contract (`SearchRestaurantAvailability`) with normalized variables:
  - `input.restaurantId`
  - `input.partySize`
  - `input.dateTime`
- `reservation.create`: fixed operation contract (`CreateReservation`) with normalized variables:
  - `input.restaurantId`
  - `input.partySize`
  - `input.dateTime`
  - `input.slotHash`
  - `input.availabilityToken`
- `search`/`create` persisted query hashes are configured by environment:
  - `RESTAURANT_OPENTABLE_SEARCH_OPERATION_SHA256`
  - `RESTAURANT_OPENTABLE_CREATE_OPERATION_SHA256`

This stage removes request-time metadata-driven search/create contracts. Remaining gap is freezing production hashes for search/create persisted queries.

## Observed endpoint details

### 1) `POST /dapi/authentication/sendotpfromsignin`

Canonical operation: `auth.start`

Observed request shape:

```json
{
  "phoneNumberOrEmail": "<email-or-phone>",
  "channelType": "EMAIL|SMS",
  "isReauthentication": false
}
```

Implementation notes:

- Current adapter defaults this as auth-start path.
- Password is not used for this OTP start flow.

### 2) `POST /dapi/authentication/signinwithotp`

Canonical operation: `auth.complete`

Observed request shape:

```json
{
  "phoneNumberOrEmail": "<email-or-phone>",
  "phoneCountryCode": "<country-phone-code-or-empty>",
  "countryCode": "<country-code-or-empty>",
  "otp": "<otp-code>",
  "isReauthentication": false,
  "suppressOtpMfaTokenValidationFailure": false
}
```

Implementation notes:

- Current adapter defaults this as auth-complete path.
- Adapter supports optional challenge-token metadata packaging to pass country-code context.

### 3) `GET /dapi/fe/human`

Purpose (inferred): human/telemetry/session touch endpoint likely tied to anti-automation controls.

Observed request characteristics:

- includes browser-like headers
- includes CSRF header
- depends on active cookie jar

Observed response pattern:

- response body may be minimal
- session-related cookies are often refreshed by adjacent calls

Implementation guidance:

- do not treat this endpoint as a primary auth API
- mark as supporting heartbeat candidate for controlled experiments only

### 4) `POST /dapi/v1/session`

Purpose (inferred): session touch/renew endpoint.

Observed request characteristics:

- `Content-Length: 0` POST in current traces
- includes CSRF header and authenticated cookie jar

Observed response example (redacted):

```json
{
  "facebook": "<hash-or-token-like-value>",
  "sojern": "<hash-or-token-like-value>"
}
```

Observed behavior:

- updates short-lived session cookies (`OT-Session-Update-Date`, `OT-SessionId`)

Implementation guidance:

- treat as keepalive signal, not user-identity source of truth
- never store raw response values in persistent logs

### 5) `GET /_sec/cpr/params`

Purpose (inferred): security/challenge parameter retrieval path.

Observed request characteristics:

- same-origin request with full browser cookie context
- appears on page-interaction/search paths

Implementation guidance:

- classify as security support endpoint
- document-only at this stage; do not automate bypass logic

### 6) `POST /dapi/fe/gql?optype=query&opname=HeaderUserProfile`

Canonical operation: `profile.get`

Observed request shape:

```json
{
  "operationName": "HeaderUserProfile",
  "variables": {
    "isAuthenticated": false,
    "isPrivilegedAccessEnabled": true,
    "tld": "com",
    "gpid": 0
  },
  "extensions": {
    "persistedQuery": {
      "version": 1,
      "sha256Hash": "<hash>"
    }
  }
}
```

Observed response shape (safe subset):

```json
{
  "data": {
    "userProfile": {
      "firstName": "<string>",
      "lastName": "<string>",
      "email": "<string>",
      "mobilePhoneNumber": {
        "number": "<string>",
        "countryId": "<string>"
      },
      "gpid": "<number>"
    },
    "userUpcomingTransactions": []
  }
}
```

Mapping guidance:

- store masked email/phone only in safe metadata
- use `gpid` as `provider_account_ref` candidate (safe to hash before storage if needed)

### 7) `POST /dapi/fe/gql?optype=mutation&opname=CancelReservation`

Canonical operation: `reservation.cancel`

Observed request shape:

```json
{
  "operationName": "CancelReservation",
  "variables": {
    "input": {
      "restaurantId": 349132,
      "confirmationNumber": 2110076913,
      "securityToken": "<token>",
      "databaseRegion": "NA",
      "reservationSource": "Online"
    }
  },
  "extensions": {
    "persistedQuery": {
      "version": 1,
      "sha256Hash": "<hash>"
    }
  }
}
```

Observed response shape:

```json
{
  "data": {
    "cancelReservation": {
      "statusCode": 200,
      "errors": null,
      "data": {
        "restaurantId": 349132,
        "reservationId": 2018060113,
        "reservationState": "CancelledWeb",
        "confirmationNumber": 2110076913
      }
    }
  }
}
```

Implementation guidance:

- keep `confirmationNumber` and `reservationId` as durable cancellation references
- treat `securityToken` as sensitive input (do not persist raw)

### 8) `POST /dapi/authentication/logout`

Purpose: explicit session termination.

Observed behavior:

- no request body required in current traces
- relies on active session cookie context

## Session/human/cpr analysis (documentation-only)

Current evidence strongly suggests:

1. `/dapi/v1/session` updates/extends session state.
2. `/dapi/fe/human` is likely a human-verification/telemetry signal tied to session hygiene.
3. `/_sec/cpr/params` provides security/challenge parameters used by browser flows.

These endpoints are now used by the OpenTable stage-1 adapter for refresh/session hygiene.

## Safe logging checklist for OpenTable adapters

- redact `x-csrf-token`, auth cookies, and any `securityToken` values
- log operation name and status code only
- persist only `provider_request_id`, `confirmationNumber`, and state-safe fields
