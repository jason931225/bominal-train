# External Rewrite Handoff (Third-Party Functional Parity)

## Goal And Scope

- Goal: enable an external implementer to rebuild provider integrations from scratch with functional parity to current third-party behavior.
- Canonical priority: `third_party/srtgo/srtgo/srt.py` for SRT runtime behavior.
- Supplemental sources: `third_party/srt/SRT/*` and `third_party/korail2/korail2/*` for missing context and field discovery.
- This handoff is implementation-facing and intentionally excludes raw secret-bearing payload copies.
- Base artifacts: `docs/handoff/PROVIDER_CONTRACT.md`, `docs/handoff/PROVIDER_FIELD_MAP.json`, `docs/handoff/PROVIDER_FIELD_MAP.md`.

## Locked Taxonomy

- Endpoint tier labels: `critical`, `high_signal`, `mapped`, `unknown`.
- Field confidence labels: `confirmed`, `inferred`, `unresolved`.
- Auth scope labels: `public_stateless`, `public_context_required`, `auth_required`, `unknown`.
- Current auth-scope counts (from field map):
  - `public_stateless`: `8` endpoints
  - `public_context_required`: `2` endpoints
  - `auth_required`: `13` endpoints
  - `unknown`: `186` endpoints

## Parity-Required Flows

### Required core flow endpoints
- Login: `POST app.srail.or.kr/apb/selectListApb01080_n.do`
- NetFunnel ticket: `GET nf*.letskorail.com/ts.wseq`
- Train search: `POST app.srail.or.kr/ara/selectListAra10007_n.do`
- Reserve: `POST app.srail.or.kr/arc/selectListArc05013_n.do`
- Reservation list: `POST app.srail.or.kr/atc/selectListAtc14016_n.do`
- Ticket detail: `POST app.srail.or.kr/ard/selectListArd02019_n.do`
- Cancel: `POST app.srail.or.kr/ard/selectListArd02045_n.do`
- Standby settings: `POST app.srail.or.kr/ata/selectListAta01135_n.do`
- Logout: `POST app.srail.or.kr/login/loginOut.do`

### Optional but important
- Card payment: `POST app.srail.or.kr/ata/selectListAta09036_n.do`

### Required flow order for parity testing
1. Login
2. Search train (requires NetFunnel context)
3. Reserve ticket (or standby reserve)
4. Reservation list
5. Ticket detail lookup
6. Cancel reservation
7. Logout
8. Optional payment path (safe test account only)

## Statelessly Useful Endpoint Shortlist

These endpoints are currently classified as `public_stateless` and are good candidates for no-login utility in bominal:
- `GET app.srail.or.kr/js/stationInfo.js`: Station metadata used by UI routing/lookups
- `GET app.srail.or.kr/js/common/bridge_new.js`: Bridge utilities/signaling for app-web runtime
- `GET app.srail.or.kr/js/common/storage_new.js`: Client storage adapter logic
- `GET app.srail.or.kr/js/common/util.js`: Generic utility helpers used by pages
- `GET app.srail.or.kr/js/common/push.js`: Push helper hooks/signals
- `GET app.srail.or.kr/js/holidays.js`: Holiday/date helper table
- `POST app.srail.or.kr/ara/selectListAra12009_n.do`: Train schedule detail (운행시간) payload
- `POST app.srail.or.kr/ara/selectListAra13010_n.do`: Train charge/fare detail (운임요금) payload

Context-gated but non-login endpoint:
- `POST app.srail.or.kr/ara/selectListAra10007_n.do` is `public_context_required` (NetFunnel key and strict request context).

## Divergence Matrix (Canonical vs Supplemental)

| source | what diverges | rewrite policy |
|---|---|---|
| srtgo only (canonical runtime) | CLI-requested reservation lifecycle endpoints with strict payload formatting and response parsing in `srt.py`. | Required for parity; rewrite should keep behavior-level compatibility. |
| srt supplemental | Additional endpoint constants and route variants in `third_party/srt/SRT/srt.py` and `constants.py`. | Use as discovery/source-only context; do not override srtgo semantics without explicit decision. |
| korail2 supplemental | Expanded KTX/Korail request/response shapes in `korail2.py` and constants. | Useful for KTX parity and field naming context when srtgo omits details. |
| web/mobile-web supplemental | Popup and JS-asset endpoints (`selectTrainScheduleList.do`, `selectTrainChargeList.do`, `stationInfo.js`, common JS). | High-signal for non-canonical stateless/context-gated data extraction. |

## Unresolved Semantics

- Endpoints with unresolved auth/meaning: `186`.
- Highest-priority unresolved groups:
  - Login/auth variants where body-gating was ambiguous (`/apb/selectListApb01080_n.do`).
  - NetFunnel host variants (`nf5/nf6/nf7/nf9.letskorail.com/ts.wseq`).
  - ETK popup variants (`/hpg/hra/01/selectTrainScheduleList.do`, `/hpg/hra/01/selectTrainChargeList.do`).
  - Numerous app web view/document routes where payload utility is unclear without deeper tracing.
- See `docs/handoff/UNRESOLVED_ENDPOINTS.md` for prioritized backlog.

## Evidence Map

- Contracts: `docs/handoff/PROVIDER_CONTRACT.md`
- Per-endpoint fields and labels: `docs/handoff/PROVIDER_FIELD_MAP.json` and `.md`
- Auth classification evidence: `docs/handoff/output/auth_scope_probe/latest_auth_scope_probe.json`
- Raw redacted event captures: `docs/handoff/output/<run_id>/`
- Supplemental user artifact: `/Users/jasonlee/Downloads/flows` (external, non-portable)

## Security Handling For External Team

- Never copy raw cookies/token headers/password/PAN/CVV into rewrite docs, issues, or logs.
- Keep redaction contract aligned with current capture policy (card PIN-like key families, identity-validation key families, auth headers, session cookies).
- Only include field names and sanitized snippets in collaboration artifacts.
