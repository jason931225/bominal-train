# Plans Directory Guide

## Purpose

This directory stores implementation planning artifacts.

## Structure

- `docs/plans/active/` - executable, decision-complete plans currently used for implementation.
- `docs/plans/archive/` - completed or superseded plans retained for traceability.

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

4. Do not store transcript fragments or mixed draft content in active plans.

## Related Governance

- `docs/EXECUTION_PROTOCOL.md`
- `docs/DOCUMENTATION_WORKFLOW.md`
- `docs/README.md`
