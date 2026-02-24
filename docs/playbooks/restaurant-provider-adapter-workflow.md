# Restaurant Provider Adapter Workflow

Deterministic workflow for adding/upgrading a restaurant provider adapter with canonical contract parity.

## Preconditions

1. Docs-first gate completed (`AGENTS.md`, `docs/README.md`, `docs/agents/EXECUTION_PROTOCOL.md`).
2. Branch/PR scope identified.
3. Provider research docs updated under `docs/provider-research/`.

## Procedure

1. Freeze contract docs first.
2. Write RED contract/factory tests.
3. Implement adapter surface.
4. Integrate policy and canonical error mapping.
5. Validate safe persistence (`meta_json_safe`, `data_json_safe`).
6. Run GREEN verification.
7. Docs-last updates and changelog.

## Verification Commands

```bash
docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_restaurant_provider_contracts.py
docker compose -f infra/docker-compose.yml run --rm api pytest -q tests/test_restaurant_policy.py tests/test_restaurant_worker_policy_flow.py
bash infra/tests/test_docs_pointers.sh
bash infra/tests/test_intent_routing.sh
bash infra/tests/test_docs_consistency.sh
```

## Recovery

- Contract drift: refreeze docs and rerun RED/GREEN.
- Sensitive leakage risk: fail closed and add redaction tests before retry.
