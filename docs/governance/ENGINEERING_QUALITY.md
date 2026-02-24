# Engineering Quality Standard

## Core Requirements

- handlers thin; business logic in services/workers
- explicit boundaries for auth/crypto/payment/deploy code paths
- every behavior change has directly relevant tests
- critical paths require negative and boundary tests
- maintain baseline coverage floors as a safety net:
  - API: line coverage >= 75%
  - Web: lines/functions/branches/statements >= 70%
- warning debt in touched scope must be resolved or explicitly owned

## Prohibited Quality Anti-Patterns

- vacuous assertions
- silent warning suppression
- blanket coverage theater without behavior signal
- introducing risky dependency changes without rationale and exit criteria

## Verification Expectations

- targeted tests first, broader suites for high-risk changes
- baseline tests and coverage gates run for all in-scope changes
- mutation/invariant checks (plus assertiveness checks) are required for critical boundaries where signal is strong
- treat coverage floors as minimum guardrails, not as a substitute for behavior-level verification
