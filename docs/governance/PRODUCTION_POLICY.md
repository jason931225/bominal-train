# Production Policy

Production safety has priority over delivery speed.

## Reliability and Scalability

- bounded timeouts/retries for all external calls
- idempotent worker and side-effect paths
- explicit backpressure controls for queue/provider workloads

## Observability

- structured logs with correlation fields
- latency/error/saturation metrics for critical services
- no sensitive payloads in logs/telemetry

## Security

Follow `docs/governance/SECURITY_POLICY.md` and agent guardrails.

## Change and Rollback

- risky changes require rollback plans and verification evidence
- deploy via canonical script: `infra/scripts/deploy.sh`
- migrations must be online-safe

## Definition of Done

- docs updated
- relevant tests green
- no unresolved high-risk warnings in touched scope
- security boundaries preserved
