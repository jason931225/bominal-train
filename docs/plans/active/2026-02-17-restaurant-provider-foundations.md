# Restaurant Provider Foundations Plan (Canonical Contracts + Adapter Readiness)

## Goal

Establish a unified provider-adapter foundation for the restaurant module so new providers follow one disciplined workflow for auth, search, reserve, cancel, and safe persistence.

## Architecture direction

Mirror the train-provider style with:

- stable canonical operation IDs
- provider factory normalization
- protocol-first adapter interface
- safe outcome/error contract
- docs-first endpoint catalogs mapped to DB-safe fields

## Scope

- `api/app/modules/restaurant/providers/**`
- `api/tests/test_restaurant_provider_contracts.py`
- `docs/provider-research/**`
- `docs/playbooks/restaurant-provider-adapter-workflow.md`
- `docs/playbooks/provider-adapter-contract-template.md`
- `docs/README.md`
- `docs/INTENT_ROUTING.md`
- `docs/humans/engineering/ARCHITECTURE.md`

## Execution status (2026-02-17)

Completed:

1. Added canonical restaurant adapter contract and factory scaffolding in backend.
2. Added RED->GREEN test coverage for adapter contract/factory behavior.
3. Added provider endpoint documentation pack for OpenTable and Resy with redacted request/response shapes.
4. Added explicit documentation for OpenTable `session`, `human`, and `cpr` endpoints and integrated them in refresh stage-1 flow.
5. Added DB-safe schema mapping guide for provider payload normalization.
6. Added reusable playbook and template for streamlined future provider onboarding.
7. Implemented OpenTable stage-1 adapter paths:
   - live `auth.refresh`, `profile.get`, `reservation.cancel`
   - concrete OTP `auth.start` (`/dapi/authentication/sendotpfromsignin`) and `auth.complete` (`/dapi/authentication/signinwithotp`)
   - frozen OTP success/error response normalization contract (challenge-ref required on `auth.start`, body-level failure handling on `auth.complete`)
   - frozen `search.availability` contract with captured `RestaurantsAvailability` persisted hash
   - frozen `reservation.create` two-step contract with `BookDetailsStandardSlotLock` hash and `/dapi/booking/make-reservation` commit path
   - optional post-create confirmation enrichment path (`BookingConfirmationPageInFlow`) with non-blocking behavior
   - normalized `policy_safe` reservation-create output fields for safe artifact persistence
   - adapter coverage tests in `api/tests/test_restaurant_provider_opentable.py`
8. Implemented Resy auth stage-1 adapter paths:
   - `auth.start` password flow via `POST /4/auth/password`
   - `auth.complete` password-flow challenge-token completion path (no second provider call)
   - API key/origin config wiring in provider factory/settings/env template
   - adapter coverage tests in `api/tests/test_restaurant_provider_resy.py`
9. Cross-checked read-only `third_party/resy` references and synchronized Resy endpoint inventory:
   - pinned reference-derived endpoint chain (`/2/user`, `/4/find`, `/3/details`, `/3/book`, `/3/cancel`)
   - documented request/response anchors and pending live-capture freeze gaps
10. Implemented Resy adapter stage-2 execution paths:
   - `profile.get` using `GET /2/user` safe profile summary mapping
   - `search.availability` using `GET /4/find` canonical slot normalization
   - `reservation.create` two-step flow (`POST /3/details` then `POST /3/book`) with idempotency/payment-method/source-id support
   - `reservation.cancel` fallback flow (`reservation_id` first, retry with `resy_token` when needed)
   - config/factory/env wiring and RED->GREEN adapter tests
11. Implemented Resy auth/session stage-3 paths:
   - `auth.refresh` using config-driven `POST /3/auth/refresh` with safe refresh metadata normalization
   - provider-specific `logout` helper using config-driven `POST /3/auth/logout`
   - policy/default config + adapter tests for success/failure/unconfigured flows

Open follow-up captures (next implementation stage):

1. Resy live response-contract freeze for refresh/logout endpoints.
2. Resy live edge-case contract freeze for payment/policy-heavy create/cancel variants.

## Verification commands

```bash
docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_restaurant_provider_contracts.py
bash infra/tests/test_docs_pointers.sh
bash infra/tests/test_intent_routing.sh
bash infra/tests/test_docs_consistency.sh
```
