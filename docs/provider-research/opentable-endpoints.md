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
| `/dapi/fe/gql?optype=query&opname=Autocomplete` | `POST` | search autocomplete (supporting) | observed |
| `/dapi/fe/gql?optype=query&opname=RestaurantsAvailability` | `POST` | `search.availability` | observed |
| `/dapi/fe/gql?optype=mutation&opname=BookDetailsStandardSlotLock` | `POST` | `reservation.create` (slot lock step) | observed |
| `/dapi/booking/make-reservation` | `POST` | `reservation.create` (booking commit step) | observed |
| `/dapi/fe/gql?optype=query&opname=BookingConfirmationPageInFlow` | `POST` | reservation confirmation (supporting) | observed |
| `/dapi/fe/gql?optype=mutation&opname=CancelReservation` | `POST` | `reservation.cancel` | observed |
| `/dapi/authentication/logout` | `POST` | logout | observed |

## Adapter implementation status (2026-02-17)

Implemented in `api/app/modules/restaurant/providers/opentable_adapter.py`:

- `auth.refresh`: calls `/_sec/cpr/params`, `/dapi/fe/human`, and `/dapi/v1/session`
- `profile.get`: calls `HeaderUserProfile` persisted query
- `reservation.cancel`: calls `CancelReservation` persisted mutation
- `auth.start`: `POST /dapi/authentication/sendotpfromsignin`
- `auth.complete`: `POST /dapi/authentication/signinwithotp`
- frozen OTP response normalization contract:
  - `auth.start` requires challenge reference (`otpMfaToken|challengeToken|verificationId|requestId`) on HTTP success
  - `auth.start` safe output fields: `requires_otp`, `challenge_ref_present`, `phone_country_code`, `country_code`, `suppress_otp_mfa_token_validation_failure`
  - `auth.complete` fails on body-level errors even when HTTP status is 200 (`errors` present or `success=false`)
  - `auth.complete` normalizes `provider_account_ref` to string
- supporting autocomplete query contract captured:
  - operation: `Autocomplete`
  - hash: `fe1d118abd4c227750693027c2414d43014c2493f64f49bcef5a65274ce9c3c3`
- `search.availability`: `RestaurantsAvailability` persisted query with normalized variables:
  - `restaurantIds`, `date`, `time`, `partySize`, `restaurantAvailabilityTokens`
- `reservation.create`: two-step flow
  - slot lock: `BookDetailsStandardSlotLock` persisted mutation
  - booking commit: `POST /dapi/booking/make-reservation`
- optional post-create confirmation enrichment:
  - operation: `BookingConfirmationPageInFlow`
  - hash: `6be25f0bbc8fe75483bdfe96ae78fb20075b978842e4b44964aed3591611aa99`
- frozen OpenTable contract settings:
  - `RESTAURANT_OPENTABLE_SEARCH_OPERATION_SHA256`
  - `RESTAURANT_OPENTABLE_CREATE_OPERATION_SHA256`
  - `RESTAURANT_OPENTABLE_CREATE_PATH`
  - `RESTAURANT_OPENTABLE_CONFIRMATION_OPERATION_NAME`
  - `RESTAURANT_OPENTABLE_CONFIRMATION_OPERATION_SHA256`

This stage removes request-time metadata-driven search/create contracts and freezes OTP auth response handling.

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

Observed response contract (frozen):

```json
{
  "otpMfaToken|challengeToken|verificationId|requestId": "<challenge-ref>",
  "phoneCountryCode": "<optional-country-phone-code>",
  "countryCode": "<optional-country-code>",
  "suppressOtpMfaTokenValidationFailure": false
}
```

Adapter normalization contract:

- Treat HTTP-success response without challenge reference as `auth_start_challenge_missing`.
- Persist only normalized safe challenge payload:
  - `challenge_ref` (challenge handle only)
  - `phone_country_code`
  - `country_code`
  - `suppress_otp_mfa_token_validation_failure`

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

Observed response contract (frozen):

```json
{
  "success": true,
  "userId|gpid|user_id": "<provider-account-ref>"
}
```

Failure contract on HTTP 200:

```json
{
  "success": false,
  "errorCode": "<provider-error-code>"
}
```

Adapter normalization contract:

- If `errors` exists or `success=false`, return `auth_complete_failed`.
- Include `provider_error_code` in safe outcome data when available.
- Normalize success `provider_account_ref` to string.

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

### 7) `POST /dapi/fe/gql?optype=query&opname=RestaurantsAvailability`

Canonical operation: `search.availability`

Supporting pre-step: `POST /dapi/fe/gql?optype=query&opname=Autocomplete`

Autocomplete observed request shape:

```json
{
  "operationName": "Autocomplete",
  "variables": {
    "term": "<restaurant-search-term>",
    "latitude": 40.8058863,
    "longitude": -73.9658847,
    "useNewVersion": true
  },
  "extensions": {
    "persistedQuery": {
      "version": 1,
      "sha256Hash": "fe1d118abd4c227750693027c2414d43014c2493f64f49bcef5a65274ce9c3c3"
    }
  }
}
```

Autocomplete response fields used for follow-up availability:

- `data.autocomplete.autocompleteResults[].id` (candidate `restaurantId`)
- `data.autocomplete.correlationId` (optional request correlation context)

Observed request shape:

```json
{
  "operationName": "RestaurantsAvailability",
  "variables": {
    "restaurantIds": [349132],
    "date": "2026-02-26",
    "time": "19:00",
    "partySize": 2,
    "restaurantAvailabilityTokens": ["<availability-token>"],
    "databaseRegion": "EU",
    "requireTypes": ["Standard", "Experience"],
    "privilegedAccess": ["UberOneDiningProgram", "VisaDiningProgram", "VisaEventsProgram", "ChaseDiningProgram"]
  },
  "extensions": {
    "persistedQuery": {
      "version": 1,
      "sha256Hash": "b2d05a06151b3cb21d9dfce4f021303eeba288fac347068b29c1cb66badc46af"
    }
  }
}
```

Observed response shape (slot subset):

```json
{
  "data": {
    "availability": [
      {
        "restaurantId": 349132,
        "availabilityDays": [
          {
            "date": "2026-02-26",
            "slots": [
              {
                "slotHash": "750944791",
                "timeOffsetMinutes": -30,
                "slotAvailabilityToken": "<slot-token>",
                "type": "Standard",
                "attributes": ["default"]
              }
            ]
          }
        ]
      }
    ]
  }
}
```

Implementation guidance:

- resolve slots from `data.availability[].availabilityDays[].slots[]`
- treat `slotHash` as `provider_slot_ref` and `slotAvailabilityToken` as sensitive transient data

### 8) `POST /dapi/fe/gql?optype=mutation&opname=BookDetailsStandardSlotLock`

Canonical operation: `reservation.create` (pre-commit lock step)

Observed request shape:

```json
{
  "operationName": "BookDetailsStandardSlotLock",
  "variables": {
    "input": {
      "restaurantId": 349132,
      "seatingOption": "DEFAULT",
      "reservationDateTime": "2026-02-26T19:00",
      "partySize": 2,
      "databaseRegion": "NA",
      "slotHash": "750944791",
      "reservationType": "STANDARD",
      "diningAreaId": 1
    }
  },
  "extensions": {
    "persistedQuery": {
      "version": 1,
      "sha256Hash": "1100bf68905fd7cb1d4fd0f4504a4954aa28ec45fb22913fa977af8b06fd97fa"
    }
  }
}
```

Observed response shape:

```json
{
  "data": {
    "lockSlot": {
      "success": true,
      "slotLock": {
        "slotLockId": 1587118118
      },
      "slotLockErrors": null
    }
  }
}
```

### 9) `POST /dapi/booking/make-reservation`

Canonical operation: `reservation.create` (commit step)

Observed request shape (redacted):

```json
{
  "restaurantId": 349132,
  "partySize": 2,
  "reservationDateTime": "2026-02-26T19:00",
  "slotHash": "750944791",
  "slotAvailabilityToken": "<slot-token>",
  "slotLockId": 1587118118,
  "email": "<email>",
  "firstName": "<first-name>",
  "lastName": "<last-name>",
  "phoneNumber": "<phone-number>",
  "phoneNumberCountryId": "US",
  "country": "US"
}
```

Observed response shape:

```json
{
  "success": true,
  "reservationId": 2018438767,
  "confirmationNumber": 2110076985,
  "securityToken": "<security-token>",
  "reservationType": "Standard",
  "reservationSource": "Online"
}
```

Implementation guidance:

- this is the durable reservation-create commit endpoint
- treat `securityToken` as sensitive operational input for later cancellation

### 10) `POST /dapi/fe/gql?optype=query&opname=BookingConfirmationPageInFlow`

Purpose: supporting post-create confirmation enrichment endpoint.

Observed request includes:

- `rid`, `confirmationNumber`, `databaseRegion`, `securityToken`, `isLoggedIn`

Observed response includes:

- reservation summary, restaurant context, and user profile linkage

Implementation guidance:

- keep as optional enrichment path; do not block core create success on this call
- adapter executes this query only when confirmation operation hash is configured
- adapter stores safe enrichment subset only (restaurant/confirmation/reservation summary fields)

### 11) `POST /dapi/fe/gql?optype=mutation&opname=CancelReservation`

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

### 12) `POST /dapi/authentication/logout`

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
