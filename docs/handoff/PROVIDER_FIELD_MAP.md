# Provider Field Map

Generated from captures, source code, flow artifact, user-supplied curl contracts, and auth-scope probes.

## Label Taxonomy

- Endpoint tier label: `critical`, `high_signal`, `mapped`, `unknown`
- Field confidence label: `confirmed`, `inferred`, `unresolved`
- Auth scope label: `public_stateless`, `public_context_required`, `auth_required`, `unknown`

Generated at: `2026-02-28T09:39:15.721524+00:00`

## Sources

### captures
- `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json`
- `docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json`
- `docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json`
- `docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json`
- `docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json`
- `docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json`
- `docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json`
- `docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json`
- `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json`
- `docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json`
- `docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json`
- `docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json`
- `docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json`
- `docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json`
- `docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json`
- `docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json`
- `docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json`
- `docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json`
- `docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json`
- `docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json`
- `docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json`
- `docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json`
- `docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json`
- `docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json`
- `docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json`
- `docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json`
- `docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json`
- `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json`
- `docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json`
- `docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json`
- `docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json`
- `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json`
- `docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json`
- `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json`
- `docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json`
- `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json`
- `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json`
- `docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json`
- `docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json`
- `docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json`
- `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json`
- `docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json`
- `docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json`
- `docs/handoff/output/auth_scope_probe/latest_auth_scope_probe.json`

### curls
- `user-curl-2026-02-28:/ara/selectListAra12009_n.do`
- `user-curl-2026-02-28:/ara/selectListAra13010_n.do`
- `user-curl-2026-02-28:/ata/selectListAta04A01_n.do`
- `user-curl-2026-02-28:/atc/selectListAtc02A01_n.do`
- `user-curl-2026-02-28:/atc/selectListAtc14017_n.do`
- `user-curl-2026-02-28:/atd/selectListAtd02039_n.do`
- `user-curl-2026-02-28:/js/common/bridge_new.js`
- `user-curl-2026-02-28:/js/common/messages.js`
- `user-curl-2026-02-28:/js/common/push.js`
- `user-curl-2026-02-28:/js/common/storage_new.js`
- `user-curl-2026-02-28:/js/common/util.js`
- `user-curl-2026-02-28:/js/holidays.js`
- `user-curl-2026-02-28:/js/stationInfo.js`
- `user-curl-2026-02-28:/push/surveyPush.do`

### flows
- `/Users/jasonlee/Downloads/flows`

### source_code
- `third_party/korail2/korail2/korail2.py`
- `third_party/srt/SRT/srt.py`
- `third_party/srtgo/srtgo/ktx.py`
- `third_party/srtgo/srtgo/srt.py`

### auth_scope_probe
- `docs/handoff/output/auth_scope_probe/latest_auth_scope_probe.json`

## Endpoint Matrix

### app.srail.or.kr /apb/selectListApb01080_n.do

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `unknown`
- `methods`: `POST`
- `request_fields`: `13`
- `response_fields`: `83`
- `source_only_fields`: `9`

#### Auth Scope Evidence
- _none_

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.auto` | `auto` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `request.data.check` | `check` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `request.data.customerYn` | `customerYn` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `request.data.deviceKey` | `deviceKey` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `request.data.hmpgPwdCphd` | `hmpgPwdCphd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `request.data.login_referer` | `login_referer` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `request.data.page` | `page` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `request.data.srchDvCd` | `srchDvCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `request.data.srchDvNm` | `srchDvNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `request.session_headers.accept` | `accept` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `request.session_headers.accept-encoding` | `accept-encoding` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `request.session_headers.connection` | `connection` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `request.session_headers.user-agent` | `user-agent` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |

#### Response Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `response.headers.cache-control` | `cache-control` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.headers.connection` | `connection` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.headers.content-language` | `content-language` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.headers.content-type` | `content-type` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.headers.date` | `date` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.headers.expires` | `expires` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.headers.pragma` | `pragma` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.headers.set-cookie` | `set-cookie` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.headers.transfer-encoding` | `transfer-encoding` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.MSG` | `MSG` | `critical` | `confirmed` | `docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.RTNCD` | `RTNCD` | `critical` | `confirmed` | `docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.commandMap` | `commandMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.commandMap.auto` | `auto` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.commandMap.check` | `check` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.commandMap.customerYn` | `customerYn` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.commandMap.deviceKey` | `deviceKey` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.commandMap.hmpgPwdCphd` | `hmpgPwdCphd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.commandMap.login_auto` | `login_auto` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.commandMap.login_check` | `login_check` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.commandMap.login_idIdx` | `login_idIdx` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.commandMap.login_idVal` | `login_idVal` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.commandMap.login_pwVal` | `login_pwVal` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.commandMap.login_referer` | `login_referer` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.commandMap.page` | `page` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.commandMap.srchDvCd` | `srchDvCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.commandMap.srchDvNm` | `srchDvNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.page` | `page` | `critical` | `confirmed` | `docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.strResult` | `strResult` | `critical` | `confirmed` | `docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap` | `userMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.ABRD_RS_STN_CD` | `ABRD_RS_STN_CD` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.BTDT` | `BTDT` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.CPHD_CHG_PRD` | `CPHD_CHG_PRD` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.CPHD_CHG_SKP_TNO` | `CPHD_CHG_SKP_TNO` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.CPHD_CHG_SKP_YN` | `CPHD_CHG_SKP_YN` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.CPHD_CHG_YMD` | `CPHD_CHG_YMD` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.CUST_CL_CD` | `CUST_CL_CD` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.CUST_CL_NM` | `CUST_CL_NM` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.CUST_CL_POP_YN` | `CUST_CL_POP_YN` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.CUST_DTL_SRT_CD` | `CUST_DTL_SRT_CD` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.CUST_DTL_SRT_NM` | `CUST_DTL_SRT_NM` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.CUST_MG_NO` | `CUST_MG_NO` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.CUST_MG_SRT_NM` | `CUST_MG_SRT_NM` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.CUST_NM` | `CUST_NM` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.CUST_SRT_CD` | `CUST_SRT_CD` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.DLY_WRK_SCDL_TXT` | `DLY_WRK_SCDL_TXT` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.DPT_NM` | `DPT_NM` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.DSCP_YN` | `DSCP_YN` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.EMGNC_ACTN_TXT` | `EMGNC_ACTN_TXT` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.FRST_ACTN_TXT` | `FRST_ACTN_TXT` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.GOFF_RS_STN_CD` | `GOFF_RS_STN_CD` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.GRCP_JONN_STN_NM` | `GRCP_JONN_STN_NM` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.GRD_NM` | `GRD_NM` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.HDCP_TP_CD` | `HDCP_TP_CD` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.HME_PHONE` | `HME_PHONE` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.IND_INFO_OFR_FLG` | `IND_INFO_OFR_FLG` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.JOIN_DT` | `JOIN_DT` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.KR_JSESSIONID` | `KR_JSESSIONID` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.MBL_PHONE` | `MBL_PHONE` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.MB_CRD_NO` | `MB_CRD_NO` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.MSG` | `MSG` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.NTCE_DV_CD` | `NTCE_DV_CD` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.POSI_NM` | `POSI_NM` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.PRCS_RCVRY_TXT` | `PRCS_RCVRY_TXT` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.PV_NO` | `PV_NO` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.RET_QNTY` | `RET_QNTY` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.RTNCD` | `RTNCD` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.SALE_QNTY` | `SALE_QNTY` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.SCED_PRNMNT_RMNDR_DAY` | `SCED_PRNMNT_RMNDR_DAY` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.SCED_PRNMNT_RMNDR_DNO` | `SCED_PRNMNT_RMNDR_DNO` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.SEX_DV_CD` | `SEX_DV_CD` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.SEX_DV_NM` | `SEX_DV_NM` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.SR_JSESSIONID` | `SR_JSESSIONID` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.URT_DAY_NOTI` | `URT_DAY_NOTI` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.USER_DV` | `USER_DV` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.USER_KEY` | `USER_KEY` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.USR_PWD_CPHD` | `USR_PWD_CPHD` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.UUID` | `UUID` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.WCTNO` | `WCTNO` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.deviceKey` | `deviceKey` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.requestJSessionTime` | `requestJSessionTime` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.strDeviceInfo` | `strDeviceInfo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.uuid` | `uuid` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |
| `response.json.userMap.wctNo` | `wctNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092317Z_1eefb481/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092400Z_93f9b149/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0002_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0003_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0012_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0025_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0006_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0018_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0021_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0005_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0010_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093753Z_483869ce/0001_srt_POST_selectListApb01080_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0002_srt_POST_selectListApb01080_n.do.json` |

#### Source-Only Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.auto` | `auto` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:712` |
| `request.data.check` | `check` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:712` |
| `request.data.customerYn` | `customerYn` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:712` |
| `request.data.deviceKey` | `deviceKey` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:712` |
| `request.data.hmpgPwdCphd` | `hmpgPwdCphd` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:712` |
| `request.data.login_referer` | `login_referer` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:712` |
| `request.data.page` | `page` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:712` |
| `request.data.srchDvCd` | `srchDvCd` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:712` |
| `request.data.srchDvNm` | `srchDvNm` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:712` |

### app.srail.or.kr /ara/selectListAra10007_n.do

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `public_context_required`
- `methods`: `POST`
- `request_fields`: `23`
- `response_fields`: `160`
- `source_only_fields`: `19`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `probe_response` | `netfunnel_key_required` | `probe_id:srt_search_with_netfunnel` |

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.arriveTime` | `arriveTime` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `request.data.arvRsStnCd` | `arvRsStnCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `request.data.chtnDvCd` | `chtnDvCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `request.data.dlayTnumAplFlg` | `dlayTnumAplFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `request.data.dptDt` | `dptDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `request.data.dptDt1` | `dptDt1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `request.data.dptRsStnCd` | `dptRsStnCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `request.data.dptTm` | `dptTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `request.data.dptTm1` | `dptTm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `request.data.netfunnelKey` | `netfunnelKey` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `request.data.psgNum` | `psgNum` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `request.data.seatAttCd` | `seatAttCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `request.data.stlbTrnClsfCd` | `stlbTrnClsfCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `request.data.tkDptDt` | `tkDptDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `request.data.tkDptTm` | `tkDptTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `request.data.tkTripChgFlg` | `tkTripChgFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `request.data.tkTrnNo` | `tkTrnNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `request.data.trnGpCd` | `trnGpCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `request.data.trnNo` | `trnNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `request.session_headers.accept` | `accept` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `request.session_headers.user-agent` | `user-agent` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `request.url_query.directYn` | `directYn` | `critical` | `inferred` | `/Users/jasonlee/Downloads/flows` |
| `request.url_query.jobId` | `jobId` | `critical` | `inferred` | `/Users/jasonlee/Downloads/flows` |

#### Response Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `response.headers.cache-control` | `cache-control` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.headers.content-language` | `content-language` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.headers.content-type` | `content-type` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.headers.date` | `date` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.headers.expires` | `expires` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.headers.pragma` | `pragma` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.headers.transfer-encoding` | `transfer-encoding` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.ErrorCode` | `ErrorCode` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.ErrorMsg` | `ErrorMsg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.commandMap` | `commandMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.commandMap.arriveTime` | `arriveTime` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.commandMap.arvRsStnCd` | `arvRsStnCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.commandMap.chtnDvCd` | `chtnDvCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.commandMap.dlayTnumAplFlg` | `dlayTnumAplFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.commandMap.dptDt` | `dptDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.commandMap.dptDt1` | `dptDt1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.commandMap.dptRsStnCd` | `dptRsStnCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.commandMap.dptTm` | `dptTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.commandMap.dptTm1` | `dptTm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.commandMap.netfunnelKey` | `netfunnelKey` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.commandMap.psgNum` | `psgNum` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.commandMap.seatAttCd` | `seatAttCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.commandMap.stlbTrnClsfCd` | `stlbTrnClsfCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.commandMap.tkDptDt` | `tkDptDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.commandMap.tkDptTm` | `tkDptTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.commandMap.tkTripChgFlg` | `tkTripChgFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.commandMap.tkTrnNo` | `tkTrnNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.commandMap.trnGpCd` | `trnGpCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.commandMap.trnNo` | `trnNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets` | `outDataSets` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput0` | `dsOutput0` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput0[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].cgPsId` | `cgPsId` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].fllwPgExt` | `fllwPgExt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].fllwPgExt2` | `fllwPgExt2` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].msgCd` | `msgCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].msgTxt` | `msgTxt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].qryCnqeCnt` | `qryCnqeCnt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].seandYo` | `seandYo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].strResult` | `strResult` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].uuid` | `uuid` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].wctNo` | `wctNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1` | `dsOutput1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].arvDt` | `arvDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].arvRsStnCd` | `arvRsStnCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].arvStnConsOrdr` | `arvStnConsOrdr` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].arvStnRunOrdr` | `arvStnRunOrdr` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].arvTm` | `arvTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].chtnDvCd` | `chtnDvCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].dlaySaleFlg` | `dlaySaleFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].doReserv` | `doReserv` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].dptDt` | `dptDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].dptRsStnCd` | `dptRsStnCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].dptStnConsOrdr` | `dptStnConsOrdr` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].dptStnRunOrdr` | `dptStnRunOrdr` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].dptTm` | `dptTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].etcRsvPsbCdNm` | `etcRsvPsbCdNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].expnDptDlayTnum` | `expnDptDlayTnum` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].fresOprCno` | `fresOprCno` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].fresRsvPsbCdNm` | `fresRsvPsbCdNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].gnrmRsvPsbCdNm` | `gnrmRsvPsbCdNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].gnrmRsvPsbColor` | `gnrmRsvPsbColor` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].gnrmRsvPsbImg` | `gnrmRsvPsbImg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].gnrmRsvPsbStr` | `gnrmRsvPsbStr` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].ocurDlayTnum` | `ocurDlayTnum` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].payTable` | `payTable` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].rcvdAmt` | `rcvdAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].rcvdFare` | `rcvdFare` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].rsvWaitPsbCd` | `rsvWaitPsbCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].rsvWaitPsbCdNm` | `rsvWaitPsbCdNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].runDt` | `runDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].runTm` | `runTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].seatAttCd` | `seatAttCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].seatSelect` | `seatSelect` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].sprmRsvPsbCdNm` | `sprmRsvPsbCdNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].sprmRsvPsbColor` | `sprmRsvPsbColor` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].sprmRsvPsbImg` | `sprmRsvPsbImg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].sprmRsvPsbStr` | `sprmRsvPsbStr` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].stlbDturDvCd` | `stlbDturDvCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].stlbTrnClsfCd` | `stlbTrnClsfCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].stmpRsvPsbFlgCd` | `stmpRsvPsbFlgCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].stndRsvPsbCdNm` | `stndRsvPsbCdNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].timeTable` | `timeTable` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].trainDiscGenRt` | `trainDiscGenRt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].trnCpsCd1` | `trnCpsCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].trnCpsCd2` | `trnCpsCd2` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].trnCpsCd3` | `trnCpsCd3` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].trnCpsCd4` | `trnCpsCd4` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].trnCpsCd5` | `trnCpsCd5` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].trnGpCd` | `trnGpCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].trnNo` | `trnNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].trnNstpLeadInfo` | `trnNstpLeadInfo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].trnOrdrNo` | `trnOrdrNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].ymsAplFlg` | `ymsAplFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.resultMap` | `resultMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.resultMap[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.resultMap[*].cgPsId` | `cgPsId` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.resultMap[*].fllwPgExt` | `fllwPgExt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.resultMap[*].fllwPgExt2` | `fllwPgExt2` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.resultMap[*].msgCd` | `msgCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.resultMap[*].msgTxt` | `msgTxt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.resultMap[*].qryCnqeCnt` | `qryCnqeCnt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.resultMap[*].seandYo` | `seandYo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.resultMap[*].strResult` | `strResult` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.resultMap[*].uuid` | `uuid` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.resultMap[*].wctNo` | `wctNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap` | `trainListMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].arvDt` | `arvDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].arvRsStnCd` | `arvRsStnCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].arvStnConsOrdr` | `arvStnConsOrdr` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].arvStnRunOrdr` | `arvStnRunOrdr` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].arvTm` | `arvTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].chtnDvCd` | `chtnDvCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].dlaySaleFlg` | `dlaySaleFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].doReserv` | `doReserv` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].dptDt` | `dptDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].dptRsStnCd` | `dptRsStnCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].dptStnConsOrdr` | `dptStnConsOrdr` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].dptStnRunOrdr` | `dptStnRunOrdr` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].dptTm` | `dptTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].etcRsvPsbCdNm` | `etcRsvPsbCdNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].expnDptDlayTnum` | `expnDptDlayTnum` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].fresOprCno` | `fresOprCno` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].fresRsvPsbCdNm` | `fresRsvPsbCdNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].gnrmRsvPsbCdNm` | `gnrmRsvPsbCdNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].gnrmRsvPsbColor` | `gnrmRsvPsbColor` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].gnrmRsvPsbImg` | `gnrmRsvPsbImg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].gnrmRsvPsbStr` | `gnrmRsvPsbStr` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].ocurDlayTnum` | `ocurDlayTnum` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].payTable` | `payTable` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].rcvdAmt` | `rcvdAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].rcvdFare` | `rcvdFare` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].rsvWaitPsbCd` | `rsvWaitPsbCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].rsvWaitPsbCdNm` | `rsvWaitPsbCdNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].runDt` | `runDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].runTm` | `runTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].seatAttCd` | `seatAttCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].seatSelect` | `seatSelect` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].sprmRsvPsbCdNm` | `sprmRsvPsbCdNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].sprmRsvPsbColor` | `sprmRsvPsbColor` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].sprmRsvPsbImg` | `sprmRsvPsbImg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].sprmRsvPsbStr` | `sprmRsvPsbStr` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].stlbDturDvCd` | `stlbDturDvCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].stlbTrnClsfCd` | `stlbTrnClsfCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].stmpRsvPsbFlgCd` | `stmpRsvPsbFlgCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].stndRsvPsbCdNm` | `stndRsvPsbCdNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].timeTable` | `timeTable` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].trainDiscGenRt` | `trainDiscGenRt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].trnCpsCd1` | `trnCpsCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].trnCpsCd2` | `trnCpsCd2` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].trnCpsCd3` | `trnCpsCd3` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].trnCpsCd4` | `trnCpsCd4` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].trnCpsCd5` | `trnCpsCd5` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].trnGpCd` | `trnGpCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].trnNo` | `trnNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].trnNstpLeadInfo` | `trnNstpLeadInfo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].trnOrdrNo` | `trnOrdrNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |
| `response.json.trainListMap[*].ymsAplFlg` | `ymsAplFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0007_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0015_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0016_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0024_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0028_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0029_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0030_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0031_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0032_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0033_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0034_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0035_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0036_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0037_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0038_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0039_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0013_srt_POST_selectListAra10007_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0014_srt_POST_selectListAra10007_n.do.json` |

#### Source-Only Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.arriveTime` | `arriveTime` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:822` |
| `request.data.arvRsStnCd` | `arvRsStnCd` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:822` |
| `request.data.chtnDvCd` | `chtnDvCd` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:822` |
| `request.data.dlayTnumAplFlg` | `dlayTnumAplFlg` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:822` |
| `request.data.dptDt` | `dptDt` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:822` |
| `request.data.dptDt1` | `dptDt1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:822` |
| `request.data.dptRsStnCd` | `dptRsStnCd` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:822` |
| `request.data.dptTm` | `dptTm` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:822` |
| `request.data.dptTm1` | `dptTm1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:822` |
| `request.data.netfunnelKey` | `netfunnelKey` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:822` |
| `request.data.psgNum` | `psgNum` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:822` |
| `request.data.seatAttCd` | `seatAttCd` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:822` |
| `request.data.stlbTrnClsfCd` | `stlbTrnClsfCd` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:822` |
| `request.data.tkDptDt` | `tkDptDt` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:822` |
| `request.data.tkDptTm` | `tkDptTm` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:822` |
| `request.data.tkTripChgFlg` | `tkTripChgFlg` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:822` |
| `request.data.tkTrnNo` | `tkTrnNo` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:822` |
| `request.data.trnGpCd` | `trnGpCd` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:822` |
| `request.data.trnNo` | `trnNo` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:822` |

### app.srail.or.kr /arc/selectListArc05013_n.do

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `auth_required`
- `methods`: `POST`
- `request_fields`: `47`
- `response_fields`: `103`
- `source_only_fields`: `25`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `source_code` | `session_bound_endpoint` | `third_party/srtgo/srtgo/srt.py:is_login_guard` |

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.arvRsStnCd` | `arvRsStnCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `request.data.arvRsStnCd1` | `arvRsStnCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.arvRsStnCdNm1` | `arvRsStnCdNm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.arvStnConsOrdr1` | `arvStnConsOrdr1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.arvStnRunOrdr1` | `arvStnRunOrdr1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.arvTm` | `arvTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `request.data.arvTm1` | `arvTm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.dirSeatAttCd1` | `dirSeatAttCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.dptDt` | `dptDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `request.data.dptDt1` | `dptDt1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.dptRsStnCd` | `dptRsStnCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `request.data.dptRsStnCd1` | `dptRsStnCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.dptRsStnCdNm1` | `dptRsStnCdNm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.dptStnConsOrdr1` | `dptStnConsOrdr1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.dptStnRunOrdr1` | `dptStnRunOrdr1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.dptTm` | `dptTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `request.data.dptTm1` | `dptTm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.etcSeatAttCd1` | `etcSeatAttCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.grpDv` | `grpDv` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.jobId` | `jobId` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `request.data.jrnyCnt` | `jrnyCnt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `request.data.jrnySqno1` | `jrnySqno1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.jrnyTpCd` | `jrnyTpCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `request.data.locSeatAttCd1` | `locSeatAttCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.mblPhone` | `mblPhone` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.netfunnelKey` | `netfunnelKey` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.psgGridcnt` | `psgGridcnt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.psgInfoPerPrnb1` | `psgInfoPerPrnb1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.psgNum` | `psgNum` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `request.data.psgTpCd1` | `psgTpCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.psrmClCd1` | `psrmClCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.reserveType` | `reserveType` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `request.data.rqSeatAttCd1` | `rqSeatAttCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.rtnDv` | `rtnDv` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.runDt1` | `runDt1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.seatAttCd` | `seatAttCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `request.data.smkSeatAttCd1` | `smkSeatAttCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.stlbTrnClsfCd1` | `stlbTrnClsfCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.stndFlg` | `stndFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `request.data.totPrnb` | `totPrnb` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.trnGpCd` | `trnGpCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `request.data.trnGpCd1` | `trnGpCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.data.trnNo1` | `trnNo1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `request.session_headers.accept` | `accept` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `request.session_headers.accept-encoding` | `accept-encoding` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `request.session_headers.connection` | `connection` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `request.session_headers.user-agent` | `user-agent` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |

#### Response Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `response.headers.cache-control` | `cache-control` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `response.headers.connection` | `connection` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `response.headers.content-language` | `content-language` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `response.headers.content-type` | `content-type` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `response.headers.date` | `date` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `response.headers.expires` | `expires` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `response.headers.pragma` | `pragma` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `response.headers.set-cookie` | `set-cookie` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `response.headers.transfer-encoding` | `transfer-encoding` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `response.json.ERROR_CODE` | `ERROR_CODE` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `response.json.ERROR_MSG` | `ERROR_MSG` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap` | `commandMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.arvRsStnCd1` | `arvRsStnCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.arvRsStnCdNm1` | `arvRsStnCdNm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.arvStnConsOrdr1` | `arvStnConsOrdr1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.arvStnRunOrdr1` | `arvStnRunOrdr1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.arvTm1` | `arvTm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.dirSeatAttCd1` | `dirSeatAttCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.dptDt1` | `dptDt1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.dptRsStnCd1` | `dptRsStnCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.dptRsStnCdNm1` | `dptRsStnCdNm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.dptStnConsOrdr1` | `dptStnConsOrdr1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.dptStnRunOrdr1` | `dptStnRunOrdr1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.dptTm1` | `dptTm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.etcSeatAttCd1` | `etcSeatAttCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.grpDv` | `grpDv` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.jobId` | `jobId` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.jrnyCnt` | `jrnyCnt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.jrnySqno1` | `jrnySqno1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.jrnyTpCd` | `jrnyTpCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.locSeatAttCd1` | `locSeatAttCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.mblPhone` | `mblPhone` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.netfunnelKey` | `netfunnelKey` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.psgGridcnt` | `psgGridcnt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.psgInfoPerPrnb1` | `psgInfoPerPrnb1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.psgTpCd1` | `psgTpCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.psrmClCd1` | `psrmClCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.reserveType` | `reserveType` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.rqSeatAttCd1` | `rqSeatAttCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.rtnDv` | `rtnDv` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.runDt1` | `runDt1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.smkSeatAttCd1` | `smkSeatAttCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.stlbTrnClsfCd1` | `stlbTrnClsfCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.stndFlg` | `stndFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.totPrnb` | `totPrnb` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.trnGpCd` | `trnGpCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.trnGpCd1` | `trnGpCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.commandMap.trnNo1` | `trnNo1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.outDataSets` | `outDataSets` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `response.json.outDataSets.dsOutput0` | `dsOutput0` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `response.json.outDataSets.dsOutput0[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].msgCd` | `msgCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].msgTxt` | `msgTxt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].strResult` | `strResult` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `response.json.payListMap` | `payListMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.payListMap[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.reservListMap` | `reservListMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.reservListMap[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.reservListMap[*].JRNYLIST_KEY` | `JRNYLIST_KEY` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.reservListMap[*].arvDt` | `arvDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.reservListMap[*].arvRsStnCd` | `arvRsStnCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.reservListMap[*].arvTm` | `arvTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.reservListMap[*].dlayAcptFlg` | `dlayAcptFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.reservListMap[*].dptDt` | `dptDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.reservListMap[*].dptRsStnCd` | `dptRsStnCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.reservListMap[*].dptTm` | `dptTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.reservListMap[*].jrnyTpCd` | `jrnyTpCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.reservListMap[*].lumpStlTgtNo` | `lumpStlTgtNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.reservListMap[*].pnrNo` | `pnrNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.reservListMap[*].proyStlTgtFlg` | `proyStlTgtFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.reservListMap[*].stlbTrnClsfCd` | `stlbTrnClsfCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.reservListMap[*].totSeatNum` | `totSeatNum` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.reservListMap[*].trnGpCd` | `trnGpCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.reservListMap[*].trnNo` | `trnNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.resultMap` | `resultMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `response.json.resultMap[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `response.json.resultMap[*].alertMsg` | `alertMsg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.resultMap[*].cgPsId` | `cgPsId` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.resultMap[*].jrnyCnt` | `jrnyCnt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.resultMap[*].msgCd` | `msgCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `response.json.resultMap[*].msgTxt` | `msgTxt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `response.json.resultMap[*].sprmFare` | `sprmFare` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.resultMap[*].strResult` | `strResult` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T093914Z_1ef8280a/0003_srt_POST_selectListArc05013_n.do.json` |
| `response.json.resultMap[*].tmpJobSqno1` | `tmpJobSqno1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.resultMap[*].tmpJobSqno2` | `tmpJobSqno2` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.resultMap[*].totRcvdAmt` | `totRcvdAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.resultMap[*].uuid` | `uuid` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.resultMap[*].wctNo` | `wctNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.trainListMap` | `trainListMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.trainListMap[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.trainListMap[*].JRNYLIST_KEY` | `JRNYLIST_KEY` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.trainListMap[*].SEATLIST_KEY` | `SEATLIST_KEY` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.trainListMap[*].dcntAmt` | `dcntAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.trainListMap[*].dcntKndCd` | `dcntKndCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.trainListMap[*].psgTpCd` | `psgTpCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.trainListMap[*].psgTpDvCd` | `psgTpDvCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.trainListMap[*].psrmClCd` | `psrmClCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.trainListMap[*].rcvdAmt` | `rcvdAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.trainListMap[*].rqSeatAttCd` | `rqSeatAttCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.trainListMap[*].scarNo` | `scarNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.trainListMap[*].seatFare` | `seatFare` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.trainListMap[*].seatNo` | `seatNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |
| `response.json.trainListMap[*].seatPrc` | `seatPrc` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0017_srt_POST_selectListArc05013_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0015_srt_POST_selectListArc05013_n.do.json` |

#### Source-Only Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.arvRsStnCd1` | `arvRsStnCd1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.arvRsStnCdNm1` | `arvRsStnCdNm1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.arvStnConsOrdr1` | `arvStnConsOrdr1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.arvStnRunOrdr1` | `arvStnRunOrdr1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.arvTm1` | `arvTm1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.dptDt1` | `dptDt1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.dptRsStnCd1` | `dptRsStnCd1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.dptRsStnCdNm1` | `dptRsStnCdNm1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.dptStnConsOrdr1` | `dptStnConsOrdr1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.dptStnRunOrdr1` | `dptStnRunOrdr1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.dptTm1` | `dptTm1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.grpDv` | `grpDv` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.jobId` | `jobId` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.jrnyCnt` | `jrnyCnt` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.jrnySqno1` | `jrnySqno1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.jrnyTpCd` | `jrnyTpCd` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.mblPhone` | `mblPhone` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.netfunnelKey` | `netfunnelKey` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.rtnDv` | `rtnDv` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.runDt1` | `runDt1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.stlbTrnClsfCd1` | `stlbTrnClsfCd1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.stndFlg` | `stndFlg` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.trnGpCd` | `trnGpCd` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.trnGpCd1` | `trnGpCd1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |
| `request.data.trnNo1` | `trnNo1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:999` |

### app.srail.or.kr /ard/selectListArd02019_n.do

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `auth_required`
- `methods`: `POST`
- `request_fields`: `4`
- `response_fields`: `39`
- `source_only_fields`: `2`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `probe_response` | `login_marker_in_body` | `probe_id:srt_ticket_detail` |

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.jrnySqno` | `jrnySqno` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `request.data.pnrNo` | `pnrNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `request.session_headers.accept` | `accept` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `request.session_headers.user-agent` | `user-agent` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |

#### Response Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `response.headers.cache-control` | `cache-control` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.headers.content-language` | `content-language` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.headers.content-type` | `content-type` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.headers.date` | `date` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.headers.expires` | `expires` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.headers.pragma` | `pragma` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.headers.transfer-encoding` | `transfer-encoding` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.commandMap` | `commandMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.commandMap.jrnySqno` | `jrnySqno` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.commandMap.pnrNo` | `pnrNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.resultMap` | `resultMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.resultMap[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.resultMap[*].cgPsId` | `cgPsId` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.resultMap[*].msgCd` | `msgCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.resultMap[*].msgTxt` | `msgTxt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.resultMap[*].strResult` | `strResult` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.resultMap[*].uuid` | `uuid` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.resultMap[*].wctNo` | `wctNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.trainListMap` | `trainListMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.trainListMap[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.trainListMap[*].dcntKndCd` | `dcntKndCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.trainListMap[*].dcntPrc` | `dcntPrc` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.trainListMap[*].dirSeatAttCd` | `dirSeatAttCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.trainListMap[*].etcSeatAttCd` | `etcSeatAttCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.trainListMap[*].locSeatAttCd` | `locSeatAttCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.trainListMap[*].psgTpCd` | `psgTpCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.trainListMap[*].psrmClCd` | `psrmClCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.trainListMap[*].rcvdAmt` | `rcvdAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.trainListMap[*].rqSeatAttCd` | `rqSeatAttCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.trainListMap[*].runDt` | `runDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.trainListMap[*].scarNo` | `scarNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.trainListMap[*].seatNo` | `seatNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.trainListMap[*].seatNum` | `seatNum` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.trainListMap[*].sgrNm` | `sgrNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.trainListMap[*].smkSeatAttCd` | `smkSeatAttCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.trainListMap[*].stdrFare` | `stdrFare` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.trainListMap[*].stdrPrc` | `stdrPrc` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.trainListMap[*].trnNo` | `trnNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |
| `response.json.tranferListMap` | `tranferListMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0019_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0004_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0008_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0017_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0020_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0023_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0003_srt_POST_selectListArd02019_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0007_srt_POST_selectListArd02019_n.do.json` |

#### Source-Only Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.jrnySqno` | `jrnySqno` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1102` |
| `request.data.pnrNo` | `pnrNo` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1102` |

### app.srail.or.kr /ard/selectListArd02045_n.do

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `auth_required`
- `methods`: `POST`
- `request_fields`: `5`
- `response_fields`: `20`
- `source_only_fields`: `3`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `probe_response` | `login_marker_in_body` | `probe_id:srt_cancel` |

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.jrnyCnt` | `jrnyCnt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `request.data.pnrNo` | `pnrNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `request.data.rsvChgTno` | `rsvChgTno` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `request.session_headers.accept` | `accept` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `request.session_headers.user-agent` | `user-agent` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |

#### Response Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `response.headers.cache-control` | `cache-control` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `response.headers.content-language` | `content-language` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `response.headers.content-type` | `content-type` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `response.headers.date` | `date` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `response.headers.expires` | `expires` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `response.headers.pragma` | `pragma` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `response.headers.transfer-encoding` | `transfer-encoding` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `response.json.commandMap` | `commandMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `response.json.commandMap.jrnyCnt` | `jrnyCnt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `response.json.commandMap.pnrNo` | `pnrNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `response.json.commandMap.rsvChgTno` | `rsvChgTno` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `response.json.resultMap` | `resultMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `response.json.resultMap[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `response.json.resultMap[*].cgPsId` | `cgPsId` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `response.json.resultMap[*].msgCd` | `msgCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `response.json.resultMap[*].msgTxt` | `msgTxt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `response.json.resultMap[*].strResult` | `strResult` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `response.json.resultMap[*].uuid` | `uuid` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `response.json.resultMap[*].wctNo` | `wctNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |
| `response.json.trainListMap` | `trainListMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0009_srt_POST_selectListArd02045_n.do.json` |

#### Source-Only Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.jrnyCnt` | `jrnyCnt` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1140` |
| `request.data.pnrNo` | `pnrNo` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1140` |
| `request.data.rsvChgTno` | `rsvChgTno` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1140` |

### app.srail.or.kr /ata/selectListAta01135_n.do

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `auth_required`
- `methods`: `POST`
- `request_fields`: `6`
- `response_fields`: `15`
- `source_only_fields`: `4`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `probe_response` | `login_marker_in_body` | `probe_id:srt_standby_option` |

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.pnrNo` | `pnrNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json` |
| `request.data.psrmClChgFlg` | `psrmClChgFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json` |
| `request.data.smsSndFlg` | `smsSndFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json` |
| `request.data.telNo` | `telNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json` |
| `request.session_headers.accept` | `accept` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json` |
| `request.session_headers.user-agent` | `user-agent` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json` |

#### Response Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `response.headers.cache-control` | `cache-control` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json` |
| `response.headers.content-language` | `content-language` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json` |
| `response.headers.content-type` | `content-type` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json` |
| `response.headers.date` | `date` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json` |
| `response.headers.expires` | `expires` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json` |
| `response.headers.pragma` | `pragma` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json` |
| `response.headers.transfer-encoding` | `transfer-encoding` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json` |
| `response.json.ERROR_CODE` | `ERROR_CODE` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json` |
| `response.json.ERROR_MSG` | `ERROR_MSG` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json` |
| `response.json.dsResultMap` | `dsResultMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json` |
| `response.json.dsResultMap.dsOutput` | `dsOutput` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json` |
| `response.json.dsResultMap.dsOutput.intgMsgCd` | `intgMsgCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json` |
| `response.json.dsResultMap.dsOutput.msgCd` | `msgCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json` |
| `response.json.dsResultMap.dsOutput.msgTxt` | `msgTxt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json` |
| `response.json.dsResultMap.dsOutput.strResult` | `strResult` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json` |

#### Source-Only Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.pnrNo` | `pnrNo` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1049` |
| `request.data.psrmClChgFlg` | `psrmClChgFlg` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1049` |
| `request.data.smsSndFlg` | `smsSndFlg` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1049` |
| `request.data.telNo` | `telNo` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1049` |

### app.srail.or.kr /ata/selectListAta09036_n.do

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `auth_required`
- `methods`: `POST`
- `request_fields`: `33`
- `response_fields`: `179`
- `source_only_fields`: `31`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `source_code` | `session_bound_endpoint` | `third_party/srtgo/srtgo/srt.py:is_login_guard` |

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.arvStnConsOrdr2` | `arvStnConsOrdr2` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.arvTm` | `arvTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.athnDvCd1` | `athnDvCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.athnVal1` | `athnVal1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.cgPsId` | `cgPsId` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.chgMcs` | `chgMcs` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.crdInpWayCd1` | `crdInpWayCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.crdVlidTrm1` | `crdVlidTrm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.ctlDvCd` | `ctlDvCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.dptStnConsOrdr2` | `dptStnConsOrdr2` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.dptTm` | `dptTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.inrecmnsGridcnt` | `inrecmnsGridcnt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.ismtMnthNum1` | `ismtMnthNum1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.jrnyCnt` | `jrnyCnt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.mbCrdNo` | `mbCrdNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.mnsStlAmt1` | `mnsStlAmt1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.pageNo` | `pageNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.pageUrl` | `pageUrl` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.pnrNo` | `pnrNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.rowCnt` | `rowCnt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.rsvChgTno` | `rsvChgTno` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.stlCrCrdNo1` | `stlCrCrdNo1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.stlDmnDt` | `stlDmnDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.stlMnsCd1` | `stlMnsCd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.stlMnsSqno1` | `stlMnsSqno1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.strJobId` | `strJobId` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.ststlGridcnt` | `ststlGridcnt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.totNewStlAmt` | `totNewStlAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.totPrnb` | `totPrnb` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.trnGpCd` | `trnGpCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.data.vanPwd1` | `vanPwd1` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.session_headers.accept` | `accept` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `request.session_headers.user-agent` | `user-agent` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |

#### Response Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `response.headers.cache-control` | `cache-control` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.headers.content-language` | `content-language` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.headers.content-type` | `content-type` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.headers.date` | `date` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.headers.expires` | `expires` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.headers.pragma` | `pragma` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.headers.transfer-encoding` | `transfer-encoding` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.ERROR_CODE` | `ERROR_CODE` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.ERROR_MSG` | `ERROR_MSG` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap` | `dsResultMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput0` | `dsOutput0` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput0[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput0[*].acmMlgNum` | `acmMlgNum` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput0[*].apvNo` | `apvNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput0[*].buyPsNm` | `buyPsNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput0[*].cgPsId` | `cgPsId` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput0[*].msgCd` | `msgCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput0[*].msgTxt` | `msgTxt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput0[*].pnrNo` | `pnrNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput0[*].rmkCont` | `rmkCont` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput0[*].stlAmt` | `stlAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput0[*].stlMnsCd` | `stlMnsCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput0[*].strResult` | `strResult` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput0[*].tkNum` | `tkNum` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput0[*].totDcntAmt` | `totDcntAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput0[*].totNewStlAmt` | `totNewStlAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput0[*].totPrc` | `totPrc` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput0[*].totRcvdAmt` | `totRcvdAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput0[*].uuid` | `uuid` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput0[*].wctNo` | `wctNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput1` | `dsOutput1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput1[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput1[*].apvDt` | `apvDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput1[*].apvNo` | `apvNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput1[*].mnsStlAmt` | `mnsStlAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput1[*].pontDvCd` | `pontDvCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput1[*].stlApvTm` | `stlApvTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput1[*].stlCrCrdNo` | `stlCrCrdNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput1[*].stlMnsCd` | `stlMnsCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput1[*].stlSqno` | `stlSqno` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2` | `dsOutput2` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].TKLIST_KEY` | `TKLIST_KEY` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].arvDt` | `arvDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].arvDt1` | `arvDt1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].arvRsStnNm1` | `arvRsStnNm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].arvRsStnNm2` | `arvRsStnNm2` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].arvRsStnNmEn` | `arvRsStnNmEn` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].arvRsStnNmKo` | `arvRsStnNmKo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].arvTm` | `arvTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].arvTm1` | `arvTm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].crwPwd` | `crwPwd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].dcntAmt` | `dcntAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].dcntGroupGridcnt` | `dcntGroupGridcnt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].dlayDscpFlg` | `dlayDscpFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].dptDt` | `dptDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].dptDt1` | `dptDt1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].dptRsStnNm1` | `dptRsStnNm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].dptRsStnNm2` | `dptRsStnNm2` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].dptRsStnNmEn` | `dptRsStnNmEn` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].dptRsStnNmKo` | `dptRsStnNmKo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].dptTm` | `dptTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].dptTm1` | `dptTm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].entNm` | `entNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].jrnyTpCd` | `jrnyTpCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].mogoNm` | `mogoNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].psgTpCdNm` | `psgTpCdNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].psrmClCdNm` | `psrmClCdNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].psrmClCdNm1` | `psrmClCdNm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].rcvdAmt` | `rcvdAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].rmkCont` | `rmkCont` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].saleDt` | `saleDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].saleSqno` | `saleSqno` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].scarNo` | `scarNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].scarNo1` | `scarNo1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].seatNo` | `seatNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].seatNo1` | `seatNo1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].stdrPrc` | `stdrPrc` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].tkKndCdNm` | `tkKndCdNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].tkPrc` | `tkPrc` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].tkRetNo` | `tkRetNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].tkSqno` | `tkSqno` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].trnClsfNm` | `trnClsfNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].trnClsfNm1` | `trnClsfNm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].trnGpCd` | `trnGpCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].trnGpNm` | `trnGpNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].trnGpNm1` | `trnGpNm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].trnNo` | `trnNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput2[*].trnNo1` | `trnNo1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput3` | `dsOutput3` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput3[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput3[*].DCNTLIST_KEY` | `DCNTLIST_KEY` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput3[*].TKLIST_KEY` | `TKLIST_KEY` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.dsResultMap.dsOutput3[*].dcntKndNm` | `dcntKndNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets` | `outDataSets` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput0` | `dsOutput0` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput0[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].acmMlgNum` | `acmMlgNum` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].apvNo` | `apvNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].buyPsNm` | `buyPsNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].cgPsId` | `cgPsId` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].msgCd` | `msgCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].msgTxt` | `msgTxt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].pnrNo` | `pnrNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].rmkCont` | `rmkCont` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].stlAmt` | `stlAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].stlMnsCd` | `stlMnsCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].strResult` | `strResult` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].tkNum` | `tkNum` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].totDcntAmt` | `totDcntAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].totNewStlAmt` | `totNewStlAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].totPrc` | `totPrc` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].totRcvdAmt` | `totRcvdAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].uuid` | `uuid` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput0[*].wctNo` | `wctNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput1` | `dsOutput1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput1[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].apvDt` | `apvDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].apvNo` | `apvNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].mnsStlAmt` | `mnsStlAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].pontDvCd` | `pontDvCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].stlApvTm` | `stlApvTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].stlCrCrdNo` | `stlCrCrdNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].stlMnsCd` | `stlMnsCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput1[*].stlSqno` | `stlSqno` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2` | `dsOutput2` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].TKLIST_KEY` | `TKLIST_KEY` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].arvDt` | `arvDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].arvDt1` | `arvDt1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].arvRsStnNm1` | `arvRsStnNm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].arvRsStnNm2` | `arvRsStnNm2` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].arvRsStnNmEn` | `arvRsStnNmEn` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].arvRsStnNmKo` | `arvRsStnNmKo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].arvTm` | `arvTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].arvTm1` | `arvTm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].crwPwd` | `crwPwd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].dcntAmt` | `dcntAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].dcntGroupGridcnt` | `dcntGroupGridcnt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].dlayDscpFlg` | `dlayDscpFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].dptDt` | `dptDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].dptDt1` | `dptDt1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].dptRsStnNm1` | `dptRsStnNm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].dptRsStnNm2` | `dptRsStnNm2` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].dptRsStnNmEn` | `dptRsStnNmEn` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].dptRsStnNmKo` | `dptRsStnNmKo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].dptTm` | `dptTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].dptTm1` | `dptTm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].entNm` | `entNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].jrnyTpCd` | `jrnyTpCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].mogoNm` | `mogoNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].psgTpCdNm` | `psgTpCdNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].psrmClCdNm` | `psrmClCdNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].psrmClCdNm1` | `psrmClCdNm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].rcvdAmt` | `rcvdAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].rmkCont` | `rmkCont` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].saleDt` | `saleDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].saleSqno` | `saleSqno` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].scarNo` | `scarNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].scarNo1` | `scarNo1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].seatNo` | `seatNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].seatNo1` | `seatNo1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].stdrPrc` | `stdrPrc` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].tkKndCdNm` | `tkKndCdNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].tkPrc` | `tkPrc` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].tkRetNo` | `tkRetNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].tkSqno` | `tkSqno` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].trnClsfNm` | `trnClsfNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].trnClsfNm1` | `trnClsfNm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].trnGpCd` | `trnGpCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].trnGpNm` | `trnGpNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].trnGpNm1` | `trnGpNm1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].trnNo` | `trnNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput2[*].trnNo1` | `trnNo1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput3` | `dsOutput3` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput3[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput3[*].DCNTLIST_KEY` | `DCNTLIST_KEY` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput3[*].TKLIST_KEY` | `TKLIST_KEY` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |
| `response.json.outDataSets.dsOutput3[*].dcntKndNm` | `dcntKndNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0004_srt_POST_selectListAta09036_n.do.json` |

#### Source-Only Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.arvStnConsOrdr2` | `arvStnConsOrdr2` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.arvTm` | `arvTm` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.athnDvCd1` | `athnDvCd1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.athnVal1` | `athnVal1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.cgPsId` | `cgPsId` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.chgMcs` | `chgMcs` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.crdInpWayCd1` | `crdInpWayCd1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.crdVlidTrm1` | `crdVlidTrm1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.ctlDvCd` | `ctlDvCd` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.dptStnConsOrdr2` | `dptStnConsOrdr2` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.dptTm` | `dptTm` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.inrecmnsGridcnt` | `inrecmnsGridcnt` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.ismtMnthNum1` | `ismtMnthNum1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.jrnyCnt` | `jrnyCnt` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.mbCrdNo` | `mbCrdNo` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.mnsStlAmt1` | `mnsStlAmt1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.pageNo` | `pageNo` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.pageUrl` | `pageUrl` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.pnrNo` | `pnrNo` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.rowCnt` | `rowCnt` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.rsvChgTno` | `rsvChgTno` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.stlCrCrdNo1` | `stlCrCrdNo1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.stlDmnDt` | `stlDmnDt` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.stlMnsCd1` | `stlMnsCd1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.stlMnsSqno1` | `stlMnsSqno1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.strJobId` | `strJobId` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.ststlGridcnt` | `ststlGridcnt` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.totNewStlAmt` | `totNewStlAmt` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.totPrnb` | `totPrnb` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.trnGpCd` | `trnGpCd` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |
| `request.data.vanPwd1` | `vanPwd1` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1218` |

### app.srail.or.kr /atc/getListAtc14087.do

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `auth_required`
- `methods`: `POST`
- `request_fields`: `3`
- `response_fields`: `58`
- `source_only_fields`: `0`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `source_code` | `session_bound_endpoint` | `third_party/srtgo/srtgo/srt.py:is_login_guard` |

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.session_headers.accept` | `accept` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `request.session_headers.referer` | `referer` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `request.session_headers.user-agent` | `user-agent` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |

#### Response Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `response.headers.cache-control` | `cache-control` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.headers.content-language` | `content-language` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.headers.content-type` | `content-type` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.headers.date` | `date` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.headers.expires` | `expires` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.headers.pragma` | `pragma` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.headers.transfer-encoding` | `transfer-encoding` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.ErrorCode` | `ErrorCode` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.ErrorMsg` | `ErrorMsg` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets` | `outDataSets` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput0` | `dsOutput0` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput0[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput0[*].cgPsId` | `cgPsId` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput0[*].msgCd` | `msgCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput0[*].msgTxt` | `msgTxt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput0[*].strResult` | `strResult` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput0[*].uuid` | `uuid` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput0[*].wctNo` | `wctNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1` | `dsOutput1` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].TKINFOLIST_KEY` | `TKINFOLIST_KEY` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].arvDt` | `arvDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].arvRsStnCd` | `arvRsStnCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].arvTm` | `arvTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].buyPsNm` | `buyPsNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].cashRcetIssPsbFlg` | `cashRcetIssPsbFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].cashRcetIssPsbFlgPrev` | `cashRcetIssPsbFlgPrev` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].dptDt` | `dptDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].dptRsStnCd` | `dptRsStnCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].dptTm` | `dptTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].lumpStlTgtNo` | `lumpStlTgtNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].mogoNm` | `mogoNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].ogtkRetPwd` | `ogtkRetPwd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].ogtkSaleDt` | `ogtkSaleDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].ogtkSaleSqno` | `ogtkSaleSqno` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].ogtkSaleWctNo` | `ogtkSaleWctNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].pbpAcepTgtFlg` | `pbpAcepTgtFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].pnrNo` | `pnrNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].psgTpDvCd` | `psgTpDvCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].psrmClCd` | `psrmClCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].rcvdAmt` | `rcvdAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].reSalePsbFlg` | `reSalePsbFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].retPsbFlg` | `retPsbFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].rsvChgTno` | `rsvChgTno` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].runDt` | `runDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].scarNo` | `scarNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].seatNo` | `seatNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].seatNo2` | `seatNo2` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].seatNum` | `seatNum` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].sgrNm` | `sgrNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].stlTpFlg` | `stlTpFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].stlbTrnClsfCd` | `stlbTrnClsfCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].tkKndCd` | `tkKndCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].tkSqno` | `tkSqno` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].tkSttCd` | `tkSttCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].tkSttNm` | `tkSttNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].trnNo` | `trnNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |
| `response.json.outDataSets.dsOutput1[*].wnFlg` | `wnFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json` |

#### Source-Only Fields
- _none_

### app.srail.or.kr /atc/selectListAtc02063_n.do

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `auth_required`
- `methods`: `POST`
- `request_fields`: `10`
- `response_fields`: `15`
- `source_only_fields`: `7`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `probe_response` | `login_marker_in_body` | `probe_id:srt_refund` |

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.cnc_dmn_cont` | `cnc_dmn_cont` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `request.data.pnr_no` | `pnr_no` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `request.data.psgNm` | `psgNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `request.data.saleDt` | `saleDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `request.data.saleSqno` | `saleSqno` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `request.data.saleWctNo` | `saleWctNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `request.data.tkRetPwd` | `tkRetPwd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `request.session_headers.accept` | `accept` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `request.session_headers.referer` | `referer` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `request.session_headers.user-agent` | `user-agent` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |

#### Response Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `response.headers.cache-control` | `cache-control` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `response.headers.content-language` | `content-language` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `response.headers.content-type` | `content-type` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `response.headers.date` | `date` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `response.headers.expires` | `expires` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `response.headers.pragma` | `pragma` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `response.headers.transfer-encoding` | `transfer-encoding` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `response.json.resultMap` | `resultMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `response.json.resultMap[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `response.json.resultMap[*].cgPsId` | `cgPsId` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `response.json.resultMap[*].msgCd` | `msgCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `response.json.resultMap[*].msgTxt` | `msgTxt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `response.json.resultMap[*].strResult` | `strResult` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `response.json.resultMap[*].uuid` | `uuid` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |
| `response.json.resultMap[*].wctNo` | `wctNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json` |

#### Source-Only Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.cnc_dmn_cont` | `cnc_dmn_cont` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1250` |
| `request.data.pnr_no` | `pnr_no` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1250` |
| `request.data.psgNm` | `psgNm` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1250` |
| `request.data.saleDt` | `saleDt` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1250` |
| `request.data.saleSqno` | `saleSqno` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1250` |
| `request.data.saleWctNo` | `saleWctNo` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1250` |
| `request.data.tkRetPwd` | `tkRetPwd` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1250` |

### app.srail.or.kr /atc/selectListAtc14016_n.do

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `auth_required`
- `methods`: `POST`
- `request_fields`: `3`
- `response_fields`: `111`
- `source_only_fields`: `1`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `probe_response` | `login_marker_in_body` | `probe_id:srt_tickets` |

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.pageNo` | `pageNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `request.session_headers.accept` | `accept` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `request.session_headers.user-agent` | `user-agent` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |

#### Response Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `response.headers.cache-control` | `cache-control` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.headers.content-language` | `content-language` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.headers.content-type` | `content-type` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.headers.date` | `date` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.headers.expires` | `expires` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.headers.pragma` | `pragma` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.headers.transfer-encoding` | `transfer-encoding` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.commandMap` | `commandMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.commandMap.pageNo` | `pageNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap` | `payListMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].JRNYLIST_KEY` | `JRNYLIST_KEY` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].PNRLIST_KEY` | `PNRLIST_KEY` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].arvRsStnCd` | `arvRsStnCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].arvTm` | `arvTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].dptDt` | `dptDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].dptRsStnCd` | `dptRsStnCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].dptTm` | `dptTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].iseLmtDt` | `iseLmtDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].iseLmtTm` | `iseLmtTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].isePsbDt` | `isePsbDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].jrnyPrgSttCd` | `jrnyPrgSttCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].jrnySqno` | `jrnySqno` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].jrnyTpCd` | `jrnyTpCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].mviePsrmFlg` | `mviePsrmFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].plchNum` | `plchNum` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].proyStlTgtFlg` | `proyStlTgtFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].rsvTpCd` | `rsvTpCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].runDt` | `runDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].seatNum` | `seatNum` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].stlFlg` | `stlFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].stlbTrnClsfCd` | `stlbTrnClsfCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].stndNum` | `stndNum` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].tkCnlRetFlg` | `tkCnlRetFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].tkIseFlg` | `tkIseFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].tkStlFlg` | `tkStlFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].trnGpCd` | `trnGpCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.payListMap[*].trnNo` | `trnNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.resultMap` | `resultMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.resultMap[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.resultMap[*].cgPsId` | `cgPsId` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.resultMap[*].msgCd` | `msgCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.resultMap[*].msgTxt` | `msgTxt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.resultMap[*].rowCnt` | `rowCnt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.resultMap[*].strResult` | `strResult` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.resultMap[*].totPageCnt` | `totPageCnt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.resultMap[*].uuid` | `uuid` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.resultMap[*].wctNo` | `wctNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap` | `rsListMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].TKINFOLIST_KEY` | `TKINFOLIST_KEY` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].arvDt` | `arvDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].arvRsStnCd` | `arvRsStnCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].arvTm` | `arvTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].buyPsNm` | `buyPsNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].cashRcetIssPsbFlg` | `cashRcetIssPsbFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].cashRcetIssPsbFlgPrev` | `cashRcetIssPsbFlgPrev` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].dptDt` | `dptDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].dptRsStnCd` | `dptRsStnCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].dptTm` | `dptTm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].lumpStlTgtNo` | `lumpStlTgtNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].mogoNm` | `mogoNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].ogtkRetPwd` | `ogtkRetPwd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].ogtkSaleDt` | `ogtkSaleDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].ogtkSaleSqno` | `ogtkSaleSqno` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].ogtkSaleWctNo` | `ogtkSaleWctNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].pbpAcepTgtFlg` | `pbpAcepTgtFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].pnrNo` | `pnrNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].psgTpDvCd` | `psgTpDvCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].psrmClCd` | `psrmClCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].rcvdAmt` | `rcvdAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].reSalePsbFlg` | `reSalePsbFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].retPsbFlg` | `retPsbFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].rsvChgTno` | `rsvChgTno` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].runDt` | `runDt` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].scarNo` | `scarNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].seatNo` | `seatNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].seatNo2` | `seatNo2` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].seatNum` | `seatNum` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].sgrNm` | `sgrNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].stlTpFlg` | `stlTpFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].stlbTrnClsfCd` | `stlbTrnClsfCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].tkKndCd` | `tkKndCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].tkSqno` | `tkSqno` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].tkSttCd` | `tkSttCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].tkSttNm` | `tkSttNm` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].trnNo` | `trnNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsListMap[*].wnFlg` | `wnFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsMap` | `rsMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsMap[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsMap[*].cgPsId` | `cgPsId` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsMap[*].msgCd` | `msgCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsMap[*].msgTxt` | `msgTxt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsMap[*].strResult` | `strResult` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsMap[*].uuid` | `uuid` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.rsMap[*].wctNo` | `wctNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.trainListMap` | `trainListMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.trainListMap[*]` | `*` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0011_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.trainListMap[*].PNRLIST_KEY` | `PNRLIST_KEY` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.trainListMap[*].cpnyTkFlg` | `cpnyTkFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.trainListMap[*].dptDttm` | `dptDttm` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.trainListMap[*].jrnyCnt` | `jrnyCnt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.trainListMap[*].pbpAcepTgtFlg` | `pbpAcepTgtFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.trainListMap[*].pnrNo` | `pnrNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.trainListMap[*].rcvdAmt` | `rcvdAmt` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.trainListMap[*].reSaleFlg` | `reSaleFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.trainListMap[*].retFlg` | `retFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.trainListMap[*].rsvChgTno` | `rsvChgTno` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.trainListMap[*].stlFlg` | `stlFlg` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.trainListMap[*].tkKndCd` | `tkKndCd` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |
| `response.json.trainListMap[*].tkSpecNum` | `tkSpecNum` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092614Z_9155f005/0018_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0007_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0016_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0019_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T092916Z_17168665/0022_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0002_srt_POST_selectListAtc14016_n.do.json; docs/handoff/output/20260228T093249Z_98e24e48/0006_srt_POST_selectListAtc14016_n.do.json` |

#### Source-Only Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.pageNo` | `pageNo` | `critical` | `confirmed` | `third_party/srtgo/srtgo/srt.py:1069` |

### app.srail.or.kr /login/loginOut.do

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `auth_required`
- `methods`: `POST`
- `request_fields`: `4`
- `response_fields`: `17`
- `source_only_fields`: `0`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `source_code` | `session_bound_endpoint` | `third_party/srtgo/srtgo/srt.py:is_login_guard` |

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.session_headers.accept` | `accept` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json` |
| `request.session_headers.accept-encoding` | `accept-encoding` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json` |
| `request.session_headers.connection` | `connection` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json` |
| `request.session_headers.user-agent` | `user-agent` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json` |

#### Response Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `response.headers.cache-control` | `cache-control` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json` |
| `response.headers.connection` | `connection` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json` |
| `response.headers.content-language` | `content-language` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json` |
| `response.headers.content-type` | `content-type` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json` |
| `response.headers.date` | `date` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json` |
| `response.headers.expires` | `expires` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json` |
| `response.headers.pragma` | `pragma` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json` |
| `response.headers.transfer-encoding` | `transfer-encoding` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json` |
| `response.json.commandMap` | `commandMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json` |
| `response.json.commandMap.AOS_VERSION` | `AOS_VERSION` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json` |
| `response.json.commandMap.iOS_VERSION` | `iOS_VERSION` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json` |
| `response.json.commandMap.isEnableUpdate` | `isEnableUpdate` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json` |
| `response.json.commandMap.isForceUpdate` | `isForceUpdate` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json` |
| `response.json.userMap` | `userMap` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json` |
| `response.json.userMap.USER_DV` | `USER_DV` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json` |
| `response.json.userMap.uuid` | `uuid` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json` |
| `response.json.userMap.wctNo` | `wctNo` | `critical` | `confirmed` | `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json` |

#### Source-Only Fields
- _none_

### nf.letskorail.com /ts.wseq

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `public_context_required`
- `methods`: `GET`
- `request_fields`: `28`
- `response_fields`: `10`
- `source_only_fields`: `0`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `source_code` | `netfunnel_key_required` | `third_party/srtgo/srtgo/srt.py:NetFunnelHelper` |

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.params.1772270827497` | `1772270827497` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json` |
| `request.params.1772270871380` | `1772270871380` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json` |
| `request.params.1772270900963` | `1772270900963` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json` |
| `request.params.1772270917664` | `1772270917664` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json` |
| `request.params.1772271077959` | `1772271077959` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.params.aid` | `aid` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.params.js` | `js` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.params.nfid` | `nfid` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.params.opcode` | `opcode` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.params.prefix` | `prefix` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.params.sid` | `sid` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.accept` | `accept` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.accept-encoding` | `accept-encoding` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.accept-language` | `accept-language` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.cache-control` | `cache-control` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.connection` | `connection` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.host` | `host` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.pragma` | `pragma` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.referer` | `referer` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-ch-ua` | `sec-ch-ua` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-ch-ua-mobile` | `sec-ch-ua-mobile` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-ch-ua-platform` | `sec-ch-ua-platform` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-fetch-dest` | `sec-fetch-dest` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-fetch-mode` | `sec-fetch-mode` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-fetch-site` | `sec-fetch-site` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-fetch-storage-access` | `sec-fetch-storage-access` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.user-agent` | `user-agent` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.x-requested-with` | `x-requested-with` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |

#### Response Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `response.headers.access-control-allow-origin` | `access-control-allow-origin` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `response.headers.connection` | `connection` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `response.headers.content-length` | `content-length` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `response.headers.content-security-policy` | `content-security-policy` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `response.headers.content-type` | `content-type` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `response.headers.date` | `date` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `response.headers.keep-alive` | `keep-alive` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `response.headers.x-content-type-options` | `x-content-type-options` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `response.headers.x-xss-protection` | `x-xss-protection` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |
| `response.text.netfunnel.result` | `result` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0004_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0013_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0022_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092614Z_9155f005/0026_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json` |

#### Source-Only Fields
- _none_

### nf5.letskorail.com /ts.wseq

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `24`
- `response_fields`: `10`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.params.1772270828998` | `1772270828998` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json` |
| `request.params.1772271079065` | `1772271079065` | `critical` | `confirmed` | `docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `request.params.js` | `js` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `request.params.key` | `key` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `request.params.nfid` | `nfid` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `request.params.opcode` | `opcode` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `request.params.prefix` | `prefix` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.accept` | `accept` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.accept-encoding` | `accept-encoding` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.accept-language` | `accept-language` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.cache-control` | `cache-control` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.connection` | `connection` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.host` | `host` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.pragma` | `pragma` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.referer` | `referer` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-ch-ua` | `sec-ch-ua` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-ch-ua-mobile` | `sec-ch-ua-mobile` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-ch-ua-platform` | `sec-ch-ua-platform` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-fetch-dest` | `sec-fetch-dest` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-fetch-mode` | `sec-fetch-mode` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-fetch-site` | `sec-fetch-site` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-fetch-storage-access` | `sec-fetch-storage-access` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.user-agent` | `user-agent` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.x-requested-with` | `x-requested-with` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |

#### Response Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `response.headers.access-control-allow-origin` | `access-control-allow-origin` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `response.headers.connection` | `connection` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `response.headers.content-length` | `content-length` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `response.headers.content-security-policy` | `content-security-policy` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `response.headers.content-type` | `content-type` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `response.headers.date` | `date` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `response.headers.keep-alive` | `keep-alive` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `response.headers.x-content-type-options` | `x-content-type-options` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `response.headers.x-xss-protection` | `x-xss-protection` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |
| `response.text.netfunnel.result` | `result` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0005_netfunnel_GET_ts.wseq.json; docs/handoff/output/20260228T092916Z_17168665/0012_netfunnel_GET_ts.wseq.json` |

#### Source-Only Fields
- _none_

### nf6.letskorail.com /ts.wseq

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `23`
- `response_fields`: `10`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.params.1772270919111` | `1772270919111` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `request.params.js` | `js` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `request.params.key` | `key` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `request.params.nfid` | `nfid` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `request.params.opcode` | `opcode` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `request.params.prefix` | `prefix` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.accept` | `accept` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.accept-encoding` | `accept-encoding` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.accept-language` | `accept-language` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.cache-control` | `cache-control` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.connection` | `connection` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.host` | `host` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.pragma` | `pragma` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.referer` | `referer` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-ch-ua` | `sec-ch-ua` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-ch-ua-mobile` | `sec-ch-ua-mobile` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-ch-ua-platform` | `sec-ch-ua-platform` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-fetch-dest` | `sec-fetch-dest` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-fetch-mode` | `sec-fetch-mode` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-fetch-site` | `sec-fetch-site` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-fetch-storage-access` | `sec-fetch-storage-access` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.user-agent` | `user-agent` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.x-requested-with` | `x-requested-with` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |

#### Response Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `response.headers.access-control-allow-origin` | `access-control-allow-origin` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `response.headers.connection` | `connection` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `response.headers.content-length` | `content-length` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `response.headers.content-security-policy` | `content-security-policy` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `response.headers.content-type` | `content-type` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `response.headers.date` | `date` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `response.headers.keep-alive` | `keep-alive` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `response.headers.x-content-type-options` | `x-content-type-options` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `response.headers.x-xss-protection` | `x-xss-protection` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |
| `response.text.netfunnel.result` | `result` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0027_netfunnel_GET_ts.wseq.json` |

#### Source-Only Fields
- _none_

### nf7.letskorail.com /ts.wseq

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `23`
- `response_fields`: `10`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.params.1772270872494` | `1772270872494` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `request.params.js` | `js` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `request.params.key` | `key` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `request.params.nfid` | `nfid` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `request.params.opcode` | `opcode` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `request.params.prefix` | `prefix` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.accept` | `accept` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.accept-encoding` | `accept-encoding` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.accept-language` | `accept-language` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.cache-control` | `cache-control` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.connection` | `connection` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.host` | `host` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.pragma` | `pragma` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.referer` | `referer` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-ch-ua` | `sec-ch-ua` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-ch-ua-mobile` | `sec-ch-ua-mobile` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-ch-ua-platform` | `sec-ch-ua-platform` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-fetch-dest` | `sec-fetch-dest` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-fetch-mode` | `sec-fetch-mode` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-fetch-site` | `sec-fetch-site` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-fetch-storage-access` | `sec-fetch-storage-access` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.user-agent` | `user-agent` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.x-requested-with` | `x-requested-with` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |

#### Response Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `response.headers.access-control-allow-origin` | `access-control-allow-origin` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `response.headers.connection` | `connection` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `response.headers.content-length` | `content-length` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `response.headers.content-security-policy` | `content-security-policy` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `response.headers.content-type` | `content-type` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `response.headers.date` | `date` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `response.headers.keep-alive` | `keep-alive` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `response.headers.x-content-type-options` | `x-content-type-options` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `response.headers.x-xss-protection` | `x-xss-protection` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |
| `response.text.netfunnel.result` | `result` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0014_netfunnel_GET_ts.wseq.json` |

#### Source-Only Fields
- _none_

### nf9.letskorail.com /ts.wseq

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `23`
- `response_fields`: `10`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.params.1772270902215` | `1772270902215` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `request.params.js` | `js` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `request.params.key` | `key` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `request.params.nfid` | `nfid` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `request.params.opcode` | `opcode` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `request.params.prefix` | `prefix` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.accept` | `accept` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.accept-encoding` | `accept-encoding` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.accept-language` | `accept-language` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.cache-control` | `cache-control` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.connection` | `connection` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.host` | `host` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.pragma` | `pragma` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.referer` | `referer` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-ch-ua` | `sec-ch-ua` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-ch-ua-mobile` | `sec-ch-ua-mobile` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-ch-ua-platform` | `sec-ch-ua-platform` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-fetch-dest` | `sec-fetch-dest` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-fetch-mode` | `sec-fetch-mode` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-fetch-site` | `sec-fetch-site` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.sec-fetch-storage-access` | `sec-fetch-storage-access` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.user-agent` | `user-agent` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `request.session_headers.x-requested-with` | `x-requested-with` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |

#### Response Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `response.headers.access-control-allow-origin` | `access-control-allow-origin` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `response.headers.connection` | `connection` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `response.headers.content-length` | `content-length` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `response.headers.content-security-policy` | `content-security-policy` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `response.headers.content-type` | `content-type` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `response.headers.date` | `date` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `response.headers.keep-alive` | `keep-alive` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `response.headers.x-content-type-options` | `x-content-type-options` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `response.headers.x-xss-protection` | `x-xss-protection` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |
| `response.text.netfunnel.result` | `result` | `critical` | `confirmed` | `docs/handoff/output/20260228T092614Z_9155f005/0023_netfunnel_GET_ts.wseq.json` |

#### Source-Only Fields
- _none_

### smart.letskorail.com .certification.ReservationList

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com .certification.TicketReservation

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com .common.code.do

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com .common.logout

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com .login.Login

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com .myTicket.MyTicketList

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com .payment.ReservationPayment

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com .refunds.RefundsRequest

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com .refunds.SelTicketInfo

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com .reservation.ReservationView

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com .reservationCancel.ReservationCancelChk

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com .seatMovie.ScheduleView

- `endpoint_tier_label`: `critical`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /apb/selectListApb01017_n.do

- `endpoint_tier_label`: `high_signal`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `1`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.mbCrdNo` | `mbCrdNo` | `high_signal` | `inferred` | `/Users/jasonlee/Downloads/flows#offset:49361` |

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /ara/ara0101v.do

- `endpoint_tier_label`: `high_signal`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /ara/selectListAra12009_n.do

- `endpoint_tier_label`: `high_signal`
- `auth_scope_label`: `public_stateless`
- `methods`: `POST`
- `request_fields`: `4`
- `response_fields`: `3`
- `source_only_fields`: `0`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `probe_response` | `business_payload_without_cookie` | `probe_id:srt_schedule_detail` |

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.runDt` | `runDt` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra12009_n.do` |
| `request.data.stnCourseNm` | `stnCourseNm` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra12009_n.do` |
| `request.data.trnNo` | `trnNo` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra12009_n.do` |
| `request.data.trnSort` | `trnSort` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra12009_n.do` |

#### Response Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `response.schedule.rows[*].arvTm` | `arvTm` | `high_signal` | `inferred` | `user-curl-2026-02-28:/ara/selectListAra12009_n.do` |
| `response.schedule.rows[*].dptTm` | `dptTm` | `high_signal` | `inferred` | `user-curl-2026-02-28:/ara/selectListAra12009_n.do` |
| `response.schedule.rows[*].stnNm` | `stnNm` | `high_signal` | `inferred` | `user-curl-2026-02-28:/ara/selectListAra12009_n.do` |

#### Source-Only Fields
- _none_

### app.srail.or.kr /ara/selectListAra13010_n.do

- `endpoint_tier_label`: `high_signal`
- `auth_scope_label`: `public_stateless`
- `methods`: `POST`
- `request_fields`: `25`
- `response_fields`: `3`
- `source_only_fields`: `0`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `probe_response` | `business_payload_without_cookie` | `probe_id:srt_fare_detail` |

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.arvRsStnCd1` | `arvRsStnCd1` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.arvRsStnCd2` | `arvRsStnCd2` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.chtnDvCd` | `chtnDvCd` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.dptRsStnCd1` | `dptRsStnCd1` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.dptRsStnCd2` | `dptRsStnCd2` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.psgInfoPerPrnb1` | `psgInfoPerPrnb1` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.psgInfoPerPrnb2` | `psgInfoPerPrnb2` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.psgInfoPerPrnb3` | `psgInfoPerPrnb3` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.psgInfoPerPrnb4` | `psgInfoPerPrnb4` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.psgInfoPerPrnb5` | `psgInfoPerPrnb5` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.psgInfoPerPrnb6` | `psgInfoPerPrnb6` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.psgTpCd1` | `psgTpCd1` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.psgTpCd2` | `psgTpCd2` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.psgTpCd3` | `psgTpCd3` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.psgTpCd4` | `psgTpCd4` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.psgTpCd5` | `psgTpCd5` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.psgTpCd6` | `psgTpCd6` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.runDt` | `runDt` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.runDt1` | `runDt1` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.runDt2` | `runDt2` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.stnCourseNm` | `stnCourseNm` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.trnNo` | `trnNo` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.trnNo1` | `trnNo1` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.trnNo2` | `trnNo2` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `request.data.trnSort` | `trnSort` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |

#### Response Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `response.fare.rows[*].psrmClCd` | `psrmClCd` | `high_signal` | `inferred` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `response.fare.rows[*].rcvdAmt` | `rcvdAmt` | `high_signal` | `inferred` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |
| `response.fare.rows[*].rcvdFare` | `rcvdFare` | `high_signal` | `inferred` | `user-curl-2026-02-28:/ara/selectListAra13010_n.do` |

#### Source-Only Fields
- _none_

### app.srail.or.kr /ata/selectListAta04A01_n.do

- `endpoint_tier_label`: `high_signal`
- `auth_scope_label`: `unknown`
- `methods`: `POST`
- `request_fields`: `11`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.PushContentArv` | `PushContentArv` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ata/selectListAta04A01_n.do` |
| `request.data.PushContentDpt` | `PushContentDpt` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ata/selectListAta04A01_n.do` |
| `request.data.arv_snd_dttm` | `arv_snd_dttm` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ata/selectListAta04A01_n.do` |
| `request.data.arv_stn_cd` | `arv_stn_cd` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ata/selectListAta04A01_n.do` |
| `request.data.dpt_dt` | `dpt_dt` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ata/selectListAta04A01_n.do` |
| `request.data.dpt_snd_dttm` | `dpt_snd_dttm` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ata/selectListAta04A01_n.do` |
| `request.data.dpt_stn_cd` | `dpt_stn_cd` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ata/selectListAta04A01_n.do` |
| `request.data.mb_crd_no` | `mb_crd_no` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ata/selectListAta04A01_n.do` |
| `request.data.pnr_no` | `pnr_no` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ata/selectListAta04A01_n.do` |
| `request.data.run_dt` | `run_dt` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ata/selectListAta04A01_n.do` |
| `request.data.trn_no` | `trn_no` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/ata/selectListAta04A01_n.do` |

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /atc/selectListAtc02085_n.do

- `endpoint_tier_label`: `high_signal`
- `auth_scope_label`: `auth_required`
- `methods`: `POST`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `probe_response` | `login_marker_in_body` | `probe_id:srt_refund_precheck` |

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /atc/selectListAtc02A01_n.do

- `endpoint_tier_label`: `high_signal`
- `auth_scope_label`: `unknown`
- `methods`: `POST`
- `request_fields`: `2`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.cnc_dmn_cont` | `cnc_dmn_cont` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/atc/selectListAtc02A01_n.do` |
| `request.data.pnrNo` | `pnrNo` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/atc/selectListAtc02A01_n.do` |

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /atc/selectListAtc14017_n.do

- `endpoint_tier_label`: `high_signal`
- `auth_scope_label`: `auth_required`
- `methods`: `GET`
- `request_fields`: `1`
- `response_fields`: `1`
- `source_only_fields`: `0`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `probe_response` | `login_marker_in_body` | `probe_id:srt_ticket_page` |

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.url_query.pageNo` | `pageNo` | `high_signal` | `confirmed` | `/Users/jasonlee/Downloads/flows; user-curl-2026-02-28:/atc/selectListAtc14017_n.do` |

#### Response Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `response.ticket_list.rows[*].pnrNo` | `pnrNo` | `high_signal` | `inferred` | `user-curl-2026-02-28:/atc/selectListAtc14017_n.do` |

#### Source-Only Fields
- _none_

### app.srail.or.kr /atd/selectListAtd02039_n.do

- `endpoint_tier_label`: `high_signal`
- `auth_scope_label`: `auth_required`
- `methods`: `POST`
- `request_fields`: `3`
- `response_fields`: `3`
- `source_only_fields`: `0`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `source_code` | `session_bound_endpoint` | `third_party/srtgo/srtgo/srt.py:is_login_guard` |

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.dptDtFrom` | `dptDtFrom` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/atd/selectListAtd02039_n.do` |
| `request.data.dptDtTo` | `dptDtTo` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/atd/selectListAtd02039_n.do` |
| `request.data.qryNumNext` | `qryNumNext` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/atd/selectListAtd02039_n.do` |

#### Response Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `response.history.rows[*].dptDt` | `dptDt` | `high_signal` | `inferred` | `user-curl-2026-02-28:/atd/selectListAtd02039_n.do` |
| `response.history.rows[*].pnrNo` | `pnrNo` | `high_signal` | `inferred` | `user-curl-2026-02-28:/atd/selectListAtd02039_n.do` |
| `response.history.rows[*].trnNo` | `trnNo` | `high_signal` | `inferred` | `user-curl-2026-02-28:/atd/selectListAtd02039_n.do` |

#### Source-Only Fields
- _none_

### app.srail.or.kr /js/common/bridge_new.js

- `endpoint_tier_label`: `high_signal`
- `auth_scope_label`: `public_stateless`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `probe_response` | `business_payload_without_cookie` | `probe_id:srt_bridge_js` |

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /js/common/messages.js

- `endpoint_tier_label`: `high_signal`
- `auth_scope_label`: `auth_required`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `probe_response` | `login_marker_in_body` | `probe_id:srt_messages_js` |

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /js/common/push.js

- `endpoint_tier_label`: `high_signal`
- `auth_scope_label`: `public_stateless`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `probe_response` | `business_payload_without_cookie` | `probe_id:srt_push_js` |

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /js/common/storage_new.js

- `endpoint_tier_label`: `high_signal`
- `auth_scope_label`: `public_stateless`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `probe_response` | `business_payload_without_cookie` | `probe_id:srt_storage_js` |

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /js/common/util.js

- `endpoint_tier_label`: `high_signal`
- `auth_scope_label`: `public_stateless`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `probe_response` | `business_payload_without_cookie` | `probe_id:srt_util_js` |

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /js/holidays.js

- `endpoint_tier_label`: `high_signal`
- `auth_scope_label`: `public_stateless`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `probe_response` | `business_payload_without_cookie` | `probe_id:srt_holidays_js` |

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /js/stationInfo.js

- `endpoint_tier_label`: `high_signal`
- `auth_scope_label`: `public_stateless`
- `methods`: `GET`
- `request_fields`: `1`
- `response_fields`: `2`
- `source_only_fields`: `0`

#### Auth Scope Evidence
| source | signal | reference |
|---|---|---|
| `probe_response` | `business_payload_without_cookie` | `probe_id:srt_station_info_js` |

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data._` | `_` | `high_signal` | `confirmed` | `user-curl-2026-02-28:/js/stationInfo.js` |

#### Response Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `response.stations[*].stnCd` | `stnCd` | `high_signal` | `inferred` | `user-curl-2026-02-28:/js/stationInfo.js` |
| `response.stations[*].stnNm` | `stnNm` | `high_signal` | `inferred` | `user-curl-2026-02-28:/js/stationInfo.js` |

#### Source-Only Fields
- _none_

### etk.srail.kr /hpg/hra/01/selectTrainChargeList.do

- `endpoint_tier_label`: `high_signal`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### etk.srail.kr /hpg/hra/01/selectTrainScheduleList.do

- `endpoint_tier_label`: `high_signal`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /ara/selectListAra2700V.do

- `endpoint_tier_label`: `mapped`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `1`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.url_query.ticketType` | `ticketType` | `mapped` | `inferred` | `/Users/jasonlee/Downloads/flows` |

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /ara/selectListAra2701V.do

- `endpoint_tier_label`: `mapped`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /ara/selectListAra2702V.do

- `endpoint_tier_label`: `mapped`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /atc/selectListAtc14021_n.do

- `endpoint_tier_label`: `mapped`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /atc/selectListAtc14022_n.do

- `endpoint_tier_label`: `mapped`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /atc/selectListAtc14040_n.do

- `endpoint_tier_label`: `mapped`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /atc/selectListAtc14087_n.do

- `endpoint_tier_label`: `mapped`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `2`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.url_query.tConfirm` | `tConfirm` | `mapped` | `inferred` | `/Users/jasonlee/Downloads/flows` |
| `request.url_query.ticketType` | `ticketType` | `mapped` | `inferred` | `/Users/jasonlee/Downloads/flows` |

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/ATA/ATA0204C/view.do

- `endpoint_tier_label`: `mapped`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `2`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.url_query.directYn` | `directYn` | `mapped` | `inferred` | `/Users/jasonlee/Downloads/flows` |
| `request.url_query.psgNum2` | `psgNum2` | `mapped` | `inferred` | `/Users/jasonlee/Downloads/flows` |

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /login/loginOutFido.do

- `endpoint_tier_label`: `mapped`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /neo/apb/selectListApb01080_n.do

- `endpoint_tier_label`: `mapped`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /MB_CRD_NO

- `endpoint_tier_label`: `mapped`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /app.login.cphd

- `endpoint_tier_label`: `mapped`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /dsOutput0

- `endpoint_tier_label`: `mapped`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /msgTxt

- `endpoint_tier_label`: `mapped`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /outDataSets

- `endpoint_tier_label`: `mapped`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /strMbCrdNo

- `endpoint_tier_label`: `mapped`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /strResult

- `endpoint_tier_label`: `mapped`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /userMap

- `endpoint_tier_label`: `mapped`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /apa/selectListApa03020_n.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /apc/selectAPC0109C1.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /apc/selectAPC0109C2.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /apc/selectApc10A01_n.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `1`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.url_query.mainFlag` | `mainFlag` | `unknown` | `inferred` | `/Users/jasonlee/Downloads/flows` |

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /apc/selectListApc04034_n.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /ara/ara0101v2.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /cms/archive.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `2`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.url_query.htmlNo` | `htmlNo` | `unknown` | `inferred` | `/Users/jasonlee/Downloads/flows` |
| `request.url_query.pageId` | `pageId` | `unknown` | `inferred` | `/Users/jasonlee/Downloads/flows` |

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/AAA/AAA0101L/view.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `1`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.url_query.pageId` | `pageId` | `unknown` | `inferred` | `/Users/jasonlee/Downloads/flows` |

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/AAA/AAA0102P/view.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/APA/APA0101P/view.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/APA/APA0102L/view.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/APA/APA0103L/view.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/APA/APA0103P/view.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/APA/APA0104P/view.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/APA/APA0105L/view.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/APA/APA0106L/view.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/APA/APA0201L/view.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/APA/APA0202L/view.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/APA/APA0303P/view.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/APC/APC0106C/view.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/APD/APD0101L/view.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/ARA/ARA0301V/view.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/ARA/ARA2700V/view.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/ARA/ARA2701V/view.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/ATC/ATC0201L/view.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/ATD/ATD0101V/view.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/ATD/ATD0301V/view.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/main/picture/view.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /common/pop/trailLocation/view.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `1`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.url_query.maxTime` | `maxTime` | `unknown` | `inferred` | `/Users/jasonlee/Downloads/flows` |

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /js/jquery.js

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /js/lib/html2canvas/html2canvas.js

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /js/loginCheck_new.js

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /json2.js

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /jsonp.js

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /login/login.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `2`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.url_query.customerYn` | `customerYn` | `unknown` | `inferred` | `/Users/jasonlee/Downloads/flows` |
| `request.url_query.page` | `page` | `unknown` | `inferred` | `/Users/jasonlee/Downloads/flows` |

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /main.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /main/main.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /push/surveyPush.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `POST`
- `request_fields`: `1`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.SIUFlg` | `SIUFlg` | `unknown` | `confirmed` | `user-curl-2026-02-28:/push/surveyPush.do` |

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /sns/snsInfoSelect.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### app.srail.or.kr /unit/core.js

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### etk.srail.co.kr /cms/archive.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `1`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.url_query.pageId` | `pageId` | `unknown` | `inferred` | `/Users/jasonlee/Downloads/flows` |

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### etk.srail.kr /cms/archive.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `2`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.url_query.htmlNo` | `htmlNo` | `unknown` | `inferred` | `/Users/jasonlee/Downloads/flows` |
| `request.url_query.pageId` | `pageId` | `unknown` | `inferred` | `/Users/jasonlee/Downloads/flows` |

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### etk.srail.kr /hpg/haa/02/srt/etc/2021003.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `1`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.url_query.pageId` | `pageId` | `unknown` | `inferred` | `/Users/jasonlee/Downloads/flows` |

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### json.org /json2.js

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /ErrorCode

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /ErrorMsg

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /arvRsStnCd

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /arvTm

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /buyPsNm

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /classes/com.korail.mobile.certification.ReservationList

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `4`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.params.Device` | `Device` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:853` |
| `request.params.Key` | `Key` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:853` |
| `request.params.Version` | `Version` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:853` |
| `request.params.hidPnrNo` | `hidPnrNo` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:853` |

### smart.letskorail.com /classes/com.korail.mobile.certification.TicketReservation

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `39`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.params.Device` | `Device` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.Key` | `Key` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.Version` | `Version` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.hidFreeFlg` | `hidFreeFlg` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtArvRsStnCd1` | `txtArvRsStnCd1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtArvRsStnCd2` | `txtArvRsStnCd2` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtChgFlg1` | `txtChgFlg1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtChgFlg2` | `txtChgFlg2` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtDptDt1` | `txtDptDt1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtDptDt2` | `txtDptDt2` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtDptRsStnCd1` | `txtDptRsStnCd1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtDptRsStnCd2` | `txtDptRsStnCd2` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtDptTm1` | `txtDptTm1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtDptTm2` | `txtDptTm2` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtGdNo` | `txtGdNo` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtJobId` | `txtJobId` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtJrnyCnt` | `txtJrnyCnt` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtJrnySqno1` | `txtJrnySqno1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtJrnySqno2` | `txtJrnySqno2` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtJrnyTpCd1` | `txtJrnyTpCd1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtJrnyTpCd2` | `txtJrnyTpCd2` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtMenuId` | `txtMenuId` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtPsrmClCd1` | `txtPsrmClCd1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtPsrmClCd2` | `txtPsrmClCd2` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtRunDt1` | `txtRunDt1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtRunDt2` | `txtRunDt2` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtSeatAttCd1` | `txtSeatAttCd1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtSeatAttCd2` | `txtSeatAttCd2` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtSeatAttCd3` | `txtSeatAttCd3` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtSeatAttCd4` | `txtSeatAttCd4` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtSeatAttCd5` | `txtSeatAttCd5` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtSrcarCnt` | `txtSrcarCnt` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtStndFlg` | `txtStndFlg` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtTotPsgCnt` | `txtTotPsgCnt` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtTrnClsfCd1` | `txtTrnClsfCd1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtTrnClsfCd2` | `txtTrnClsfCd2` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtTrnGpCd1` | `txtTrnGpCd1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtTrnNo1` | `txtTrnNo1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |
| `request.params.txtTrnNo2` | `txtTrnNo2` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:760` |

### smart.letskorail.com /classes/com.korail.mobile.common.logout

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /classes/com.korail.mobile.login.Login

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `POST`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `7`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.Device` | `Device` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:578` |
| `request.data.Key` | `Key` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:578` |
| `request.data.Version` | `Version` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:578` |
| `request.data.idx` | `idx` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:578` |
| `request.data.txtInputFlg` | `txtInputFlg` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:578` |
| `request.data.txtMemberNo` | `txtMemberNo` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:578` |
| `request.data.txtPwd` | `txtPwd` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:578` |

### smart.letskorail.com /classes/com.korail.mobile.myTicket.MyTicketList

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `9`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.params.Device` | `Device` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:783` |
| `request.params.Key` | `Key` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:783` |
| `request.params.Version` | `Version` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:783` |
| `request.params.h_abrd_dt_from` | `h_abrd_dt_from` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:783` |
| `request.params.h_abrd_dt_to` | `h_abrd_dt_to` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:783` |
| `request.params.h_page_no` | `h_page_no` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:783` |
| `request.params.hiduserYn` | `hiduserYn` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:783` |
| `request.params.txtDeviceId` | `txtDeviceId` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:783` |
| `request.params.txtIndex` | `txtIndex` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:783` |

### smart.letskorail.com /classes/com.korail.mobile.payment.ReservationPayment

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `POST`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `20`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.Device` | `Device` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:904` |
| `request.data.Key` | `Key` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:904` |
| `request.data.Version` | `Version` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:904` |
| `request.data.hidAthnDvCd1` | `hidAthnDvCd1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:904` |
| `request.data.hidAthnVal1` | `hidAthnVal1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:904` |
| `request.data.hidCrdInpWayCd1` | `hidCrdInpWayCd1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:904` |
| `request.data.hidCrdVlidTrm1` | `hidCrdVlidTrm1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:904` |
| `request.data.hidInrecmnsGridcnt` | `hidInrecmnsGridcnt` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:904` |
| `request.data.hidIsmtMnthNum1` | `hidIsmtMnthNum1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:904` |
| `request.data.hidMnsStlAmt1` | `hidMnsStlAmt1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:904` |
| `request.data.hidPnrNo` | `hidPnrNo` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:904` |
| `request.data.hidRsvChgNo` | `hidRsvChgNo` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:904` |
| `request.data.hidStlCrCrdNo1` | `hidStlCrCrdNo1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:904` |
| `request.data.hidStlMnsCd1` | `hidStlMnsCd1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:904` |
| `request.data.hidStlMnsSqno1` | `hidStlMnsSqno1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:904` |
| `request.data.hidTmpJobSqno1` | `hidTmpJobSqno1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:904` |
| `request.data.hidTmpJobSqno2` | `hidTmpJobSqno2` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:904` |
| `request.data.hidVanPwd1` | `hidVanPwd1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:904` |
| `request.data.hidWctNo` | `hidWctNo` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:904` |
| `request.data.hiduserYn` | `hiduserYn` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:904` |

### smart.letskorail.com /classes/com.korail.mobile.refunds.RefundsRequest

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `POST`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `14`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.Device` | `Device` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:945` |
| `request.data.Key` | `Key` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:945` |
| `request.data.Version` | `Version` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:945` |
| `request.data.h_mlg_stl` | `h_mlg_stl` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:945` |
| `request.data.h_orgtk_ret_pwd` | `h_orgtk_ret_pwd` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:945` |
| `request.data.h_orgtk_sale_dt` | `h_orgtk_sale_dt` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:945` |
| `request.data.h_orgtk_sale_sqno` | `h_orgtk_sale_sqno` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:945` |
| `request.data.h_orgtk_sale_wct_no` | `h_orgtk_sale_wct_no` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:945` |
| `request.data.latitude` | `latitude` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:945` |
| `request.data.longitude` | `longitude` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:945` |
| `request.data.pbpAcepTgtFlg` | `pbpAcepTgtFlg` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:945` |
| `request.data.tk_ret_tms_dv_cd` | `tk_ret_tms_dv_cd` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:945` |
| `request.data.trnNo` | `trnNo` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:945` |
| `request.data.txtPrnNo` | `txtPrnNo` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:945` |

### smart.letskorail.com /classes/com.korail.mobile.refunds.SelTicketInfo

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `7`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.params.Device` | `Device` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:800` |
| `request.params.Key` | `Key` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:800` |
| `request.params.Version` | `Version` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:800` |
| `request.params.h_orgtk_ret_pwd` | `h_orgtk_ret_pwd` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:800` |
| `request.params.h_orgtk_ret_sale_dt` | `h_orgtk_ret_sale_dt` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:800` |
| `request.params.h_orgtk_sale_sqno` | `h_orgtk_sale_sqno` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:800` |
| `request.params.h_orgtk_wct_no` | `h_orgtk_wct_no` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:800` |

### smart.letskorail.com /classes/com.korail.mobile.reservation.ReservationView

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `3`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.params.Device` | `Device` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:821` |
| `request.params.Key` | `Key` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:821` |
| `request.params.Version` | `Version` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:821` |

### smart.letskorail.com /classes/com.korail.mobile.reservationCancel.ReservationCancelChk

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `POST`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `7`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.data.Device` | `Device` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:923` |
| `request.data.Key` | `Key` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:923` |
| `request.data.Version` | `Version` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:923` |
| `request.data.hidRsvChgNo` | `hidRsvChgNo` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:923` |
| `request.data.txtJrnyCnt` | `txtJrnyCnt` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:923` |
| `request.data.txtJrnySqno` | `txtJrnySqno` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:923` |
| `request.data.txtPnrNo` | `txtPnrNo` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:923` |

### smart.letskorail.com /classes/com.korail.mobile.seatMovie.ScheduleView

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `24`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.params.Device` | `Device` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.Sid` | `Sid` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.Version` | `Version` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.adjStnScdlOfrFlg` | `adjStnScdlOfrFlg` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.ebizCrossCheck` | `ebizCrossCheck` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.mbCrdNo` | `mbCrdNo` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.radJobId` | `radJobId` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.rtYn` | `rtYn` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.selGoTrain` | `selGoTrain` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.srtCheckYn` | `srtCheckYn` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.txtGoAbrdDt` | `txtGoAbrdDt` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.txtGoEnd` | `txtGoEnd` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.txtGoHour` | `txtGoHour` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.txtGoStart` | `txtGoStart` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.txtMenuId` | `txtMenuId` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.txtPsgFlg_1` | `txtPsgFlg_1` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.txtPsgFlg_2` | `txtPsgFlg_2` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.txtPsgFlg_3` | `txtPsgFlg_3` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.txtPsgFlg_4` | `txtPsgFlg_4` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.txtPsgFlg_5` | `txtPsgFlg_5` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.txtSeatAttCd_2` | `txtSeatAttCd_2` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.txtSeatAttCd_3` | `txtSeatAttCd_3` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.txtSeatAttCd_4` | `txtSeatAttCd_4` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |
| `request.params.txtTrnGpCd` | `txtTrnGpCd` | `unknown` | `confirmed` | `third_party/srtgo/srtgo/ktx.py:672` |

### smart.letskorail.com /dcntKndCd

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /dcntPrc

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /dptDt

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /dptRsStnCd

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /dptTm

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /dsOutput1

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_arv_dt

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_arv_rs_stn_cd

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_arv_rs_stn_nm

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_arv_tm

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_buy_ps_nm

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_dcnt_amt

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_dpt_dt

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_dpt_rs_stn_cd

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_dpt_rs_stn_nm

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_dpt_tm

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_expct_dlay_hr

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_gen_rsv_cd

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_msg_cd

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_msg_txt

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_ntisu_lmt_dt

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_ntisu_lmt_tm

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_orgtk_ret_pwd

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_orgtk_ret_sale_dt

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_orgtk_sale_dt

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_orgtk_sale_sqno

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_orgtk_wct_no

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_pnr_no

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_psg_tp_dv_nm

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_psrm_cl_nm

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_rcvd_amt

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_rsv_amt

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_rsv_psb_flg

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_rsv_psb_nm

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_run_dt

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_seat_cnt

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_seat_no

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_seat_no_end

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_seat_prc

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_spe_rsv_cd

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_srcar_no

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_tot_seat_cnt

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_trn_clsf_cd

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_trn_clsf_nm

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_trn_gp_cd

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_trn_no

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_wait_rsv_flg

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /h_wct_no

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /hidRsvChgNo

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /iseLmtDt

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /iseLmtTm

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /jrny_info

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /jrny_infos

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /key

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /nwait

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /ogtkRetPwd

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /ogtkSaleDt

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /ogtkSaleSqno

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /ogtkSaleWctNo

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /pnrNo

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /psrmClCd

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /rcvdAmt

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /reservation_list

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /scarNo

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /seatNo

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /seatNum

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /seat_info

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /seat_infos

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /status

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /stdrPrc

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /stlFlg

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /stlbTrnClsfCd

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /ticket_info

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /ticket_infos

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /tkSpecNum

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /tk_seat_info

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /train_info

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /train_infos

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /trnNo

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /trn_info

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /trn_infos

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /txtJrnyCnt

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### smart.letskorail.com /txtJrnySqno

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `GET`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### www.srail.co.kr /cms/archive.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `1`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
| path | name | endpoint_tier_label | field_confidence_label | evidence |
|---|---|---|---|---|
| `request.url_query.pageId` | `pageId` | `unknown` | `inferred` | `/Users/jasonlee/Downloads/flows` |

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

### www.srail.co.kr /main.do

- `endpoint_tier_label`: `unknown`
- `auth_scope_label`: `unknown`
- `methods`: `_unknown_`
- `request_fields`: `0`
- `response_fields`: `0`
- `source_only_fields`: `0`

#### Auth Scope Evidence
- _none_

#### Request Fields
- _none_

#### Response Fields
- _none_

#### Source-Only Fields
- _none_

