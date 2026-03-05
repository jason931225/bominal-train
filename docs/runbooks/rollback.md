# Rollback Runbook

## Purpose

Restore service stability quickly when a production deploy causes sustained health or integrity regressions.

## Trigger Conditions

- Sustained `/health` or `/ready` failures after deploy.
- Data integrity risk from migration/runtime behavior.
- Security regression in auth/session/secret boundaries.

## Procedure

1. Declare rollback decision and incident owner.
2. Execute rollback through approved deploy controls.
3. Confirm API and worker recovery:
   - health endpoints pass
   - queue workers reconnect and process safely
4. Record rollback evidence:
   - trigger condition
   - rollback command/workflow reference
   - post-rollback verification
5. Open preventive action follow-up issue.

## Success Criteria

- Service returns to stable known-good behavior.
- Incident timeline and rollback rationale are documented.

## References

- `docs/MANUAL.md#deployment-and-rollback-standard`
- `docs/playbooks/RUST_PRODUCTION_CUTOVER.md`
