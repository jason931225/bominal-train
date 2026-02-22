# Restaurant Provider Research

Canonical research set for restaurant-provider endpoint contracts and adapter implementation readiness.

## Scope

- In scope: OpenTable and Resy endpoint catalogs, canonical operation mapping, payload-shape documentation, and safe DB-mapping guidance.
- Out of scope: plaintext secrets, raw cookie dumps, full HAR exports, and direct anti-bot bypass implementation.

## Canonical docs in this folder

- `docs/provider-research/restaurant-provider-canonical-contract.md`
- `docs/provider-research/restaurant-provider-endpoint-inventory.md`
- `docs/provider-research/restaurant-db-schema-mapping.md`
- `docs/provider-research/opentable-endpoints.md`
- `docs/provider-research/resy-endpoints.md`

## Source and redaction policy

- Capture sources:
  - browser/devtools request traces
  - provider-facing reverse-engineering notes already approved in-session
  - read-only reference repos under `third_party/**` (reference only)
- Required redaction:
  - never store plaintext passwords, OTP values, session cookies, CSRF values, refresh/access tokens, or payment secrets
  - examples must use placeholders (`<redacted>`, `<csrf-token>`, `<cookie-jar>`)

## Implementation relationship

- Runtime adapter scaffolding is defined under `api/app/modules/restaurant/providers/`.
- These research docs define the contract the adapters must satisfy before moving from scaffold to live execution.
