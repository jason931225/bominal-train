# ADR 0003: Provider SDK Boundary

- Status: Accepted
- Date: 2026-03-05

## Context

Provider integrations (SRT/KTX and future providers) require strict separation between provider-specific client behavior and runtime orchestration logic.

## Decision

- Keep provider contracts, canonical models, and shared error/retry/redaction logic in `runtime/crates/shared/src/providers/**`.
- Keep API/worker crates dependent on provider contracts rather than provider-specific transport internals.
- Preserve source-alignment constraints for train-provider behavior with `third_party/srtgo` reference implementations.
- Treat provider credential/payment payload boundaries as encrypted/redacted contracts only.

## Consequences

- Provider-specific changes can be isolated without broad API/worker rewrites.
- Runtime orchestration stays consistent across provider implementations.
- Security controls (redaction, envelope handling, secret boundaries) remain centralized.
