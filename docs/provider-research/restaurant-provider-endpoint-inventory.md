# Restaurant Provider Endpoint Inventory

Endpoint inventory for the canonical restaurant lifecycle. This is the working source used to prioritize adapter implementation.

Status values:

- `CONFIRMED`: observed with request/response shape
- `PARTIAL`: endpoint known, but full contract capture still needed
- `TODO_CAPTURE`: required operation for full feature, not yet captured in stable form

## OpenTable

| Canonical operation | Endpoint(s) | Status | Notes |
|---|---|---|---|
| `auth.start` | `POST /dapi/authentication/sendotpfromsignin` | CONFIRMED | OTP start flow with `phoneNumberOrEmail`, `channelType`, `isReauthentication`. |
| `auth.complete` | `POST /dapi/authentication/signinwithotp` | CONFIRMED | OTP verify flow with `phoneNumberOrEmail`, `phoneCountryCode`, `countryCode`, `otp`. |
| `auth.refresh` | `GET /dapi/fe/human`, `POST /dapi/v1/session` | CONFIRMED | Observed keepalive-like calls; treat as provider-internal session touch. |
| `profile.get` | `POST /dapi/fe/gql?optype=query&opname=HeaderUserProfile` | CONFIRMED | Returns user profile and invitation/upcoming context. |
| `search.availability` | configurable adapter GraphQL contract (operation/hash/variables metadata) | PARTIAL | Adapter method implemented; stable OpenTable operation/hash capture still required. |
| `reservation.create` | configurable adapter GraphQL contract (operation/hash/variables metadata) | PARTIAL | Adapter method implemented; concrete create mutation contract still required. |
| `reservation.cancel` | `POST /dapi/fe/gql?optype=mutation&opname=CancelReservation` | CONFIRMED | Uses restaurant ID + confirmation number + security token. |
| session hygiene | `GET /_sec/cpr/params` | CONFIRMED | Security/challenge parameter endpoint; document only, no implementation. |
| logout | `POST /dapi/authentication/logout` | CONFIRMED | Auth teardown endpoint. |

## Resy

| Canonical operation | Endpoint(s) | Status | Notes |
|---|---|---|---|
| `auth.start` | `POST /4/auth/password` | CONFIRMED | Password login initiation endpoint with provider API key header. |
| `auth.complete` | same endpoint for password flow | CONFIRMED | Password flow completes in single step; no separate OTP verify observed in current captures. |
| `auth.refresh` | refresh endpoint not yet pinned | TODO_CAPTURE | Needed for long-running session maintenance. |
| `profile.get` | authenticated profile endpoint not yet pinned | TODO_CAPTURE | Needed for account verification and session diagnostics. |
| `search.availability` | availability endpoint(s) not yet pinned | PARTIAL | Covered conceptually by existing Resy playbook; endpoint mapping needs contract freeze. |
| `reservation.create` | lock/hold/book endpoint(s) not yet pinned | PARTIAL | Existing playbook outlines flow; concrete endpoint matrix required. |
| `reservation.cancel` | cancellation endpoint not yet pinned | TODO_CAPTURE | Required for canonical cancellation support. |
| logout | logout endpoint not yet pinned | TODO_CAPTURE | Useful for explicit session invalidation workflows. |

## CatchTable (reference source only)

CatchTable implementation guidance remains read-only reference under:

- `third_party/catchtable/reservation.py`
- `third_party/catchtable/session.py`
- `third_party/catchtable/configs.py`
- `third_party/catchtable/main.py`

No direct adapter implementation is in scope for this stage.

## Required next captures before live adapter execution

1. OpenTable search and reservation-create concrete operation/hash capture set.
2. OpenTable OTP success/error response schema freeze (field-by-field capture).
3. Resy availability + lock/hold + create + cancel endpoint set with payload deltas.
4. Resy refresh/profile/logout endpoint set for session lifecycle completeness.
