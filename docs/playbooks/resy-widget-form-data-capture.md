# Resy Widget Form-Data Capture and Replay

## Objective

Capture and replay the minimum verified HTTP contract required to progress from authenticated session state to pre-payment reservation state with deterministic checks.

## Scope

- In scope: session bootstrap, search/availability, lock/hold, guarded book attempt before payment completion.
- Out of scope: JavaScript reverse engineering, payment submission, card charge completion.

## Preconditions

- Valid user account with working login.
- Browser automation available for bootstrap (`playwright`).
- HTTP client with cookie jar support for replay.
- Redaction-enabled logging enabled before capture.

## Inputs

### Dependency-derived inputs

- Session cookies and anti-CSRF headers from authenticated browser session.
- Venue identifier and slot identifiers from availability responses.
- Hold/lock token IDs from lock responses.

### Non-dependency inputs

- Search query (restaurant name, date, party size, time window).
- User credentials/OTP and any manual challenge resolution.

## Deterministic Procedure

1. Start fresh browser session and authenticate.
2. Export auth state (cookies + relevant headers) for HTTP client reuse.
3. Execute availability request using exported auth state.
4. Parse and select candidate slot from availability payload.
5. Execute lock/hold request with exact slot payload.
6. Verify lock response contains expected date/time/venue identifiers.
7. Execute guarded book request up to pre-payment boundary only.
8. Confirm request body date/time matches intended slot before submission.
9. Stop before payment completion and persist redacted artifacts.

## Verification Checkpoints

- Auth checkpoint:
  - Expected: authenticated endpoint returns success with current session.
  - Failure: auth rejection or redirect to login.
- Availability checkpoint:
  - Expected: slot list includes requested date and party size.
  - Failure: empty list or mismatched date/party.
- Lock checkpoint:
  - Expected: response echoes same slot/date/time/venue.
  - Failure: token missing, slot mutated, or stale-session rejection.
- Guarded book checkpoint:
  - Expected: pre-payment transition accepted without charge execution.
  - Failure: validation mismatch or auth/session invalidation.

## Failure Modes and Recovery

- Session stale during replay:
  - Detection: auth/lock endpoints return unauthorized/forbidden.
  - Recovery: refresh via HTTP auth refresh flow; fallback to browser bootstrap.
- Slot drift between availability and lock:
  - Detection: lock rejects or returns different slot metadata.
  - Recovery: rerun availability and lock with fresh slot IDs.
- Incorrect date/time payload:
  - Detection: guarded check fails before book attempt.
  - Recovery: abort and rebuild request from latest availability payload.

## Security and Redaction

- Never persist plaintext credentials, full cookies, OTP codes, or card data.
- Persist only redacted request/response artifacts.
- Do not include CVV or payment token material in shared docs.

## Artifacts and Pointers

- `docs/README.md` canonical pointers.
- `docs/agents/EXECUTION_PROTOCOL.md` lock/request protocol.
- Provider-specific HAR and capture artifacts under approved internal storage paths only.

## Change History

- [0d84ae8] Initial standardized Resy capture/replay playbook created.
