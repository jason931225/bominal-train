# Reliability and Observability Standard

## Reliability Contract

- every outbound call uses bounded connect/read/total timeouts
- retries are bounded and jittered
- side-effecting operations are idempotent
- saturation degrades safely

## Observability Minimums

- structured logs with request/task correlation IDs when available
- golden signal metrics (latency, traffic, errors, saturation)
- provider-level error/retry/latency metrics

## Performance Regression Governance

Performance-sensitive changes must include measured verification and must not reintroduce periodic polling regressions in event-driven paths.
