# Provider Degradation Runbook

## Purpose

Handle SRT/KTX provider degradation without violating security boundaries or causing uncontrolled retry storms.

## Signals

- Elevated provider auth probe errors.
- Increased runtime job `retry_scheduled` or `dead_lettered` rates.
- User-facing train search/booking failures concentrated by provider.

## Procedure

1. Identify affected provider (`srt` or `ktx`) and failure mode.
2. Validate internal credential/payment secret paths are healthy.
3. Confirm queue pressure and retry behavior are bounded.
4. Apply least-risk mitigation:
   - pause or limit affected provider operations
   - preserve alternative provider path where available
5. Monitor for stabilization and capture decision log.

## Guardrails

- Do not log raw provider credential or payment payloads.
- Do not relax TLS/certificate validation.
- Keep behavior source-aligned with read-only `third_party/srtgo` references.

## Exit Criteria

- Error and retry rates return to baseline.
- Provider paths recover without policy exceptions.

## References

- `docs/MANUAL.md#security-and-data-handling-policy`
- `docs/MANUAL.md#operations-runbook-core`
