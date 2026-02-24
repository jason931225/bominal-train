# Engineering Quality Standard

## Core Requirements

- handlers thin; business logic in services/workers
- explicit boundaries for auth/crypto/payment/deploy code paths
- every behavior change has directly relevant tests
- critical paths require negative and boundary tests
- warning debt in touched scope must be resolved or explicitly owned

## Prohibited Quality Anti-Patterns

- vacuous assertions
- silent warning suppression
- blanket coverage theater without behavior signal
- introducing risky dependency changes without rationale and exit criteria

## Verification Expectations

- targeted tests first, broader suites for high-risk changes
- mutation/invariant checks for critical boundaries where signal is strong
