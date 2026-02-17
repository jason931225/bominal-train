# Restaurant Provider Adapter Workflow

Deterministic workflow for adding or upgrading a restaurant provider adapter with canonical contract parity.

## Objective

Streamline provider implementation with a shared lifecycle (`auth -> search -> reserve -> cancel`) and enforce safe payload handling from day one.

## Preconditions

1. Docs-first gate completed (`AGENTS.md`, `docs/README.md`, `docs/EXECUTION_PROTOCOL.md`, required core docs).
2. Active lock acquired for code + docs scope.
3. Provider endpoint research docs created/updated under `docs/provider-research/`.

## Deterministic Procedure

1. Freeze contract docs first
- update `docs/provider-research/restaurant-provider-endpoint-inventory.md`
- update provider-specific endpoint doc (`opentable-endpoints.md` or `resy-endpoints.md`)
- confirm canonical operation mapping in `restaurant-provider-canonical-contract.md`

2. Write RED tests
- add/extend adapter contract tests under `api/tests/`
- include factory mapping tests and operation outcome tests
- include retry/error-code expectations for auth/search/reserve/cancel

3. Implement adapter surface
- implement/extend `api/app/modules/restaurant/providers/<provider>_adapter.py`
- keep `RestaurantProviderClient` signature stable
- preserve normalized provider key discipline (`RESY`, `OPENTABLE`, `CATCHTABLE`)

4. Integrate with worker policy
- map provider failures to canonical error codes
- keep auth fallback routing in policy layer (`refresh -> bootstrap -> fail`)
- preserve payment lease semantics for committing steps only

5. Validate safe persistence
- ensure only safe subsets are persisted to `meta_json_safe`/`data_json_safe`
- verify no raw credential/session data in logs or artifacts

6. GREEN verification
- run provider-specific test file(s)
- run broader restaurant policy/worker suites
- run docs validators and pointer checks

7. Docs-last and closure
- update `docs/README.md` pointers if new canonical docs were introduced
- update `CHANGELOG.md` with commit-based entries
- release lock in `docs/LOCK.md` with unlock-only commit

## Verification commands

```bash
docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_restaurant_provider_contracts.py
docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_restaurant_policy.py tests/test_restaurant_worker_policy_flow.py
bash infra/tests/test_docs_pointers.sh
bash infra/tests/test_intent_routing.sh
bash infra/tests/test_docs_consistency.sh
```

## Failure modes and recovery

- Contract drift between docs and adapters:
  - Recovery: freeze docs first, then re-run RED/GREEN on adapter tests.
- Sensitive field leakage in logs:
  - Recovery: fail closed, redact, and backfill tests for redaction guarantees.
- Provider endpoint instability:
  - Recovery: keep adapter behavior behind explicit retryable errors and safe fallback states.
