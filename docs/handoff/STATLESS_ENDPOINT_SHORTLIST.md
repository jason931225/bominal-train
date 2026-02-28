# Statless Endpoint Shortlist

This file lists endpoints currently classified as `public_stateless` or `public_context_required` for quick integration planning.

## public_stateless

| method | host | path | endpoint_tier_label | why useful |
|---|---|---|---|---|
| POST | app.srail.or.kr | /ara/selectListAra12009_n.do | high_signal | business payload for schedule/charge detail |
| POST | app.srail.or.kr | /ara/selectListAra13010_n.do | high_signal | business payload for schedule/charge detail |
| GET | app.srail.or.kr | /js/common/bridge_new.js | high_signal | supporting asset or business payload without login session |
| GET | app.srail.or.kr | /js/common/push.js | high_signal | supporting asset or business payload without login session |
| GET | app.srail.or.kr | /js/common/storage_new.js | high_signal | supporting asset or business payload without login session |
| GET | app.srail.or.kr | /js/common/util.js | high_signal | supporting asset or business payload without login session |
| GET | app.srail.or.kr | /js/holidays.js | high_signal | supporting asset or business payload without login session |
| GET | app.srail.or.kr | /js/stationInfo.js | high_signal | supporting asset or business payload without login session |

## public_context_required

| method | host | path | endpoint_tier_label | context required |
|---|---|---|---|---|
| POST | app.srail.or.kr | /ara/selectListAra10007_n.do | critical | NetFunnel key / flow context |
| GET | nf.letskorail.com | /ts.wseq | critical | NetFunnel key / flow context |

## Notes

- `200 OK` alone is not treated as public; classification is response-content validated.
- Current evidence source: `docs/handoff/output/auth_scope_probe/latest_auth_scope_probe.json`.
