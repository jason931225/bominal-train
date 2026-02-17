# OpenTable Endpoint Contract Notes

Redacted provider-contract notes for OpenTable integration. This document intentionally excludes raw credentials/tokens/cookies.

## Endpoint summary

| Endpoint | Method | Canonical operation | Current status |
|---|---|---|---|
| `/dapi/fe/human` | `GET` | `auth.refresh` (supporting) | observed |
| `/dapi/v1/session` | `POST` | `auth.refresh` (supporting) | observed |
| `/_sec/cpr/params` | `GET` | security/session support (non-canonical helper) | observed |
| `/dapi/fe/gql?optype=query&opname=HeaderUserProfile` | `POST` | `profile.get` | observed |
| `/dapi/fe/gql?optype=mutation&opname=CancelReservation` | `POST` | `reservation.cancel` | observed |
| `/dapi/authentication/logout` | `POST` | logout | observed |

Required for full feature but not yet contract-frozen:

- OTP start endpoint (`auth.start`)
- OTP verify endpoint (`auth.complete`)
- reservation-create mutation (`reservation.create`)
- stable availability/search query operation (`search.availability`)

## Observed endpoint details

### 1) `GET /dapi/fe/human`

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

### 2) `POST /dapi/v1/session`

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

### 3) `GET /_sec/cpr/params`

Purpose (inferred): security/challenge parameter retrieval path.

Observed request characteristics:

- same-origin request with full browser cookie context
- appears on page-interaction/search paths

Implementation guidance:

- classify as security support endpoint
- document-only at this stage; do not automate bypass logic

### 4) `POST /dapi/fe/gql?optype=query&opname=HeaderUserProfile`

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

### 5) `POST /dapi/fe/gql?optype=mutation&opname=CancelReservation`

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

### 6) `POST /dapi/authentication/logout`

Purpose: explicit session termination.

Observed behavior:

- no request body required in current traces
- relies on active session cookie context

## Session/human/cpr analysis (documentation-only)

Current evidence strongly suggests:

1. `/dapi/v1/session` updates/extends session state.
2. `/dapi/fe/human` is likely a human-verification/telemetry signal tied to session hygiene.
3. `/_sec/cpr/params` provides security/challenge parameters used by browser flows.

These endpoints can improve robustness of session maintenance if integrated carefully, but they are intentionally not implemented in this stage.

## Safe logging checklist for OpenTable adapters

- redact `x-csrf-token`, auth cookies, and any `securityToken` values
- log operation name and status code only
- persist only `provider_request_id`, `confirmationNumber`, and state-safe fields
