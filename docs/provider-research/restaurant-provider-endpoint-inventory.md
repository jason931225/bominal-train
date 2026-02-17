# Restaurant Provider Endpoint Inventory

Endpoint inventory for the canonical restaurant lifecycle. This is the working source used to prioritize adapter implementation.

Status values:

- `CONFIRMED`: observed with request/response shape
- `PARTIAL`: endpoint known, but full contract capture still needed
- `TODO_CAPTURE`: required operation for full feature, not yet captured in stable form

## OpenTable

| Canonical operation | Endpoint(s) | Status | Notes |
|---|---|---|---|
| `auth.start` | `POST /dapi/authentication/sendotpfromsignin` | CONFIRMED | OTP start flow with `phoneNumberOrEmail`, `channelType`, `isReauthentication`; success response contract frozen with required challenge-reference extraction. |
| `auth.complete` | `POST /dapi/authentication/signinwithotp` | CONFIRMED | OTP verify flow with `phoneNumberOrEmail`, `phoneCountryCode`, `countryCode`, `otp`; body-level failure handling (`success=false`/`errors`) frozen. |
| `auth.refresh` | `GET /dapi/fe/human`, `POST /dapi/v1/session` | CONFIRMED | Observed keepalive-like calls; treat as provider-internal session touch. |
| `profile.get` | `POST /dapi/fe/gql?optype=query&opname=HeaderUserProfile` | CONFIRMED | Returns user profile and invitation/upcoming context. |
| search autocomplete (supporting) | `POST /dapi/fe/gql?optype=query&opname=Autocomplete` | CONFIRMED | Search-term pre-step resolves candidate restaurant IDs (`id`) and correlation context. |
| `search.availability` | `POST /dapi/fe/gql?optype=query&opname=RestaurantsAvailability` | CONFIRMED | Persisted hash captured (`b2d05a06151b3cb21d9dfce4f021303eeba288fac347068b29c1cb66badc46af`); response slots resolved from `data.availability[].availabilityDays[].slots[]`. |
| `reservation.create` | `POST /dapi/fe/gql?optype=mutation&opname=BookDetailsStandardSlotLock` + `POST /dapi/booking/make-reservation` | CONFIRMED | Two-step create flow: slot lock (persisted hash `1100bf68905fd7cb1d4fd0f4504a4954aa28ec45fb22913fa977af8b06fd97fa`) then booking commit with contact payload. |
| `reservation.cancel` | `POST /dapi/fe/gql?optype=mutation&opname=CancelReservation` | CONFIRMED | Uses restaurant ID + confirmation number + security token. |
| reservation confirmation (supporting) | `POST /dapi/fe/gql?optype=query&opname=BookingConfirmationPageInFlow` | CONFIRMED | Post-create optional enrichment endpoint; adapter uses best-effort call gated by confirmation operation hash config. |
| session hygiene | `GET /_sec/cpr/params` | CONFIRMED | Security/challenge parameter endpoint; document only, no implementation. |
| logout | `POST /dapi/authentication/logout` | CONFIRMED | Auth teardown endpoint. |

## Resy

| Canonical operation | Endpoint(s) | Status | Notes |
|---|---|---|---|
| `auth.start` | `POST /4/auth/password` | CONFIRMED | Password login initiation endpoint with provider API key header; adapter enforces body-level failure handling and normalized safe challenge payload. |
| `auth.complete` | same endpoint for password flow | CONFIRMED | Password flow completes in single step; adapter uses password-flow challenge token and does not perform a second provider call. |
| `auth.refresh` | `POST /3/auth/refresh` (expected) | PARTIAL | Seen in external trace notes; not yet frozen in adapter/docs contract. |
| `profile.get` | `GET /2/user` | PARTIAL | Implemented in adapter with safe profile normalization; live capture freeze still pending for edge-case fields. |
| `search.availability` | `GET /4/find` | PARTIAL | Implemented in adapter with canonical slot mapping from `config.token`; live schema freeze still pending. |
| `reservation.create` | `POST /3/details` + `POST /3/book` | PARTIAL | Implemented in adapter with details->book chain, idempotency header pass-through, and payment-method/source-id support; payment/policy-heavy variants still need live freeze. |
| `reservation.cancel` | `POST /3/cancel` | PARTIAL | Implemented in adapter with fallback retry (`reservation_id` -> `reservation_id+resy_token`); fallback necessity matrix still needs live freeze. |
| logout | logout endpoint not yet pinned | TODO_CAPTURE | Useful for explicit session invalidation workflows. |

## CatchTable (reference source only)

CatchTable implementation guidance remains read-only reference under:

- `third_party/catchtable/reservation.py`
- `third_party/catchtable/session.py`
- `third_party/catchtable/configs.py`
- `third_party/catchtable/main.py`

No direct adapter implementation is in scope for this stage.

## Required next captures before live adapter execution

1. Resy session refresh/logout endpoint set for session lifecycle completeness.
2. Resy payment-required and policy-heavy create/cancel variants to harden edge-case mapping.
