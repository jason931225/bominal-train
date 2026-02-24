# Plans Directory Guide

## Purpose

This directory stores implementation planning artifacts.

## Structure

- `docs/plans/active/` - executable, decision-complete plans currently used for implementation.
- `docs/plans/archive/` - completed or superseded plans retained for traceability.

When no execution plan is open, `docs/plans/active/` should contain only
`README.md` that points to the latest closure/report artifact.

## Rules

1. Active plans must be executable:
- clear goal and architecture
- exact file paths
- explicit RED -> GREEN verification steps
- concrete verification commands

2. Archived plans must not be edited for new implementation scope.

3. If a plan is superseded:
- move it to `archive/`
- add a short archive note with replacement pointer
- register replacement plan in `docs/README.md`

4. If a stage/program is complete:
- move its executable plan from `active/` to `archive/`
- keep closure evidence in an archive report
- keep `active/README.md` accurate for current open work

5. Do not store transcript fragments or mixed draft content in active plans.

## Related Governance

- `docs/agents/EXECUTION_PROTOCOL.md`
- `docs/governance/DOCUMENTATION_POLICY.md`
- `docs/README.md`
