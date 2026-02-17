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
- `docs/ARCHITECTURE.md`

## Execution status (2026-02-17)

Completed:

1. Added canonical restaurant adapter contract and factory scaffolding in backend.
2. Added RED->GREEN test coverage for adapter contract/factory behavior.
3. Added provider endpoint documentation pack for OpenTable and Resy with redacted request/response shapes.
4. Added explicit documentation for OpenTable `session`, `human`, and `cpr` endpoints as observation-only.
5. Added DB-safe schema mapping guide for provider payload normalization.
6. Added reusable playbook and template for streamlined future provider onboarding.
7. Implemented OpenTable stage-1 adapter paths:
   - live `auth.refresh`, `profile.get`, `reservation.cancel`
   - configurable `auth.start`, `auth.complete`, `search.availability`, `reservation.create`
   - adapter coverage tests in `api/tests/test_restaurant_provider_opentable.py`

Open follow-up captures (next implementation stage):

1. OpenTable OTP start/verify endpoints (`auth.start`, `auth.complete`).
2. OpenTable reservation-create mutation contract freeze.
3. Resy full availability/hold/create/cancel endpoint capture and contract freeze.
4. Session refresh/logout endpoint freeze for Resy.

## Verification commands

```bash
docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_restaurant_provider_contracts.py
bash infra/tests/test_docs_pointers.sh
bash infra/tests/test_intent_routing.sh
bash infra/tests/test_docs_consistency.sh
```
