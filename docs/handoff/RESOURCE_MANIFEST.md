# Handoff Resource Manifest

- Source revision: `1328f32935874cf4cab89fc61a03afc4ff77c04b`
- Generated at (UTC): `2026-02-28T21:48:26.646973+00:00`
- Total items: `53`

## Inventory

| id | path | kind | provider | canonicality | required_for_parity | sensitivity | purpose |
|---|---|---|---|---|---|---|---|
| RES-200 | `/Users/jasonlee/Downloads/flows` | external_artifact | srt_web | supplemental | optional | high | User-captured flow artifact outside repository |
| RES-307 | `docs/handoff/EXTERNAL_REWRITE_RUNBOOK.md` | handoff_doc | meta | canonical | required | low | Execution sequence for rewrite team |
| RES-308 | `docs/handoff/NON_REPO_ARTIFACTS.md` | handoff_doc | meta | canonical | required | low | External artifact locations and portability notes |
| RES-305 | `docs/handoff/README.md` | handoff_doc | meta | canonical | required | low | Docs-side handoff entrypoint |
| RES-306 | `docs/handoff/RESOURCE_INDEX.md` | handoff_doc | meta | canonical | required | low | Pointer index for external team |
| RES-108 | `third_party/korail2/README.md` | reference_doc | ktx | supplemental | required | low | Korail2 behavior and usage |
| RES-110 | `third_party/korail2/korail2/constants.py` | source_code | ktx | supplemental | optional | low | Korail constants and field names |
| RES-109 | `third_party/korail2/korail2/korail2.py` | source_code | ktx | supplemental | required | low | Supplemental KTX/Korail endpoint contracts |
| RES-104 | `third_party/srt/README.md` | reference_doc | srt | supplemental | required | low | Legacy/expanded SRT endpoint context |
| RES-107 | `third_party/srt/SRT/constants.py` | source_code | srt | supplemental | optional | low | Supplemental constants/endpoint names |
| RES-106 | `third_party/srt/SRT/netfunnel.py` | source_code | srt | supplemental | required | low | NetFunnel token flow details |
| RES-105 | `third_party/srt/SRT/srt.py` | source_code | srt | supplemental | required | low | Supplemental SRT endpoint contracts |
| RES-100 | `third_party/srtgo/README.md` | reference_doc | srt | supplemental | required | low | Upstream srtgo usage/context |
| RES-102 | `third_party/srtgo/srtgo/ktx.py` | source_code | ktx | canonical | required | low | Canonical KTX adapter behavior in bominal context |
| RES-101 | `third_party/srtgo/srtgo/srt.py` | source_code | srt | canonical | required | low | Canonical SRT provider behavior |
| RES-103 | `third_party/srtgo/srtgo/srtgo.py` | source_code | srt | supplemental | optional | low | CLI flow and request sequencing reference |
| RES-300 | `docs/handoff/HANDOFF_EXTERNAL_REWRITE.md` | handoff_doc | srt/ktx | canonical | required | low | Decision-complete external rewrite handoff |
| RES-001 | `docs/handoff/PROVIDER_CONTRACT.md` | contract_doc | srt | canonical | required | low | Canonical contract decisions and divergence notes |
| RES-002 | `docs/handoff/PROVIDER_FIELD_MAP.json` | field_map_json | srt/ktx | canonical | required | low | Machine-readable endpoint/field/auth map |
| RES-003 | `docs/handoff/PROVIDER_FIELD_MAP.md` | field_map_doc | srt/ktx | canonical | required | low | Human-readable endpoint/field/auth map |
| RES-014 | `docs/handoff/generate_provider_field_map.py` | tooling | srt/ktx | supporting | required | low | Field map generator from captures/code/curl evidence |
| RES-301 | `docs/handoff/RESOURCE_MANIFEST.json` | handoff_manifest | srt/ktx | canonical | required | low | Machine-readable handoff inventory |
| RES-302 | `docs/handoff/RESOURCE_MANIFEST.md` | handoff_doc | srt/ktx | canonical | required | low | Human-readable handoff inventory |
| RES-303 | `docs/handoff/STATLESS_ENDPOINT_SHORTLIST.md` | handoff_doc | srt | canonical | required | low | Stateless/context-gated endpoint shortlist |
| RES-304 | `docs/handoff/UNRESOLVED_ENDPOINTS.md` | handoff_doc | srt/ktx | canonical | required | low | Unresolved endpoint semantics and priorities |
| RES-021 | `docs/handoff/output` | capture_output | srt | supporting | required | medium | Redacted run captures (request/response events) |
| RES-031 | `docs/handoff/output/20260228T090104Z_86924e9a` | capture_run | srt | supplemental | optional | medium | Captured run bundle 20260228T090104Z_86924e9a |
| RES-032 | `docs/handoff/output/20260228T090334Z_c6ce741d` | capture_run | srt | supplemental | optional | medium | Captured run bundle 20260228T090334Z_c6ce741d |
| RES-033 | `docs/handoff/output/20260228T091827Z_f7255b20` | capture_run | srt | supplemental | optional | medium | Captured run bundle 20260228T091827Z_f7255b20 |
| RES-034 | `docs/handoff/output/20260228T091921Z_1476b8e2` | capture_run | srt | supplemental | optional | medium | Captured run bundle 20260228T091921Z_1476b8e2 |
| RES-035 | `docs/handoff/output/20260228T092005Z_8ecabd9a` | capture_run | srt | supplemental | optional | medium | Captured run bundle 20260228T092005Z_8ecabd9a |
| RES-036 | `docs/handoff/output/20260228T092020Z_bca8e9be` | capture_run | srt | supplemental | optional | medium | Captured run bundle 20260228T092020Z_bca8e9be |
| RES-037 | `docs/handoff/output/20260228T092141Z_679a7a7b` | capture_run | srt | supplemental | optional | medium | Captured run bundle 20260228T092141Z_679a7a7b |
| RES-038 | `docs/handoff/output/20260228T092147Z_2f792011` | capture_run | srt | supplemental | optional | medium | Captured run bundle 20260228T092147Z_2f792011 |
| RES-039 | `docs/handoff/output/20260228T092205Z_70825b27` | capture_run | srt | supplemental | optional | medium | Captured run bundle 20260228T092205Z_70825b27 |
| RES-040 | `docs/handoff/output/20260228T092310Z_1feac094` | capture_run | srt | supplemental | optional | medium | Captured run bundle 20260228T092310Z_1feac094 |
| RES-041 | `docs/handoff/output/20260228T092317Z_1eefb481` | capture_run | srt | supplemental | optional | medium | Captured run bundle 20260228T092317Z_1eefb481 |
| RES-042 | `docs/handoff/output/20260228T092400Z_93f9b149` | capture_run | srt | supplemental | optional | medium | Captured run bundle 20260228T092400Z_93f9b149 |
| RES-043 | `docs/handoff/output/20260228T092529Z_7e8b27ab` | capture_run | srt | supplemental | optional | medium | Captured run bundle 20260228T092529Z_7e8b27ab |
| RES-044 | `docs/handoff/output/20260228T092536Z_fa8fb7f2` | capture_run | srt | supplemental | optional | medium | Captured run bundle 20260228T092536Z_fa8fb7f2 |
| RES-045 | `docs/handoff/output/20260228T092614Z_9155f005` | capture_run | srt | supplemental | optional | medium | Captured run bundle 20260228T092614Z_9155f005 |
| RES-046 | `docs/handoff/output/20260228T092916Z_17168665` | capture_run | srt | supplemental | optional | medium | Captured run bundle 20260228T092916Z_17168665 |
| RES-047 | `docs/handoff/output/20260228T093249Z_98e24e48` | capture_run | srt | supplemental | optional | medium | Captured run bundle 20260228T093249Z_98e24e48 |
| RES-048 | `docs/handoff/output/20260228T093746Z_dde2877e` | capture_run | srt | supplemental | optional | medium | Captured run bundle 20260228T093746Z_dde2877e |
| RES-049 | `docs/handoff/output/20260228T093753Z_483869ce` | capture_run | srt | supplemental | optional | medium | Captured run bundle 20260228T093753Z_483869ce |
| RES-050 | `docs/handoff/output/20260228T093914Z_1ef8280a` | capture_run | srt | supplemental | optional | medium | Captured run bundle 20260228T093914Z_1ef8280a |
| RES-020 | `docs/handoff/output/auth_scope_probe/latest_auth_scope_probe.json` | probe_output | srt | canonical | required | medium | Auth-scope classification evidence |
| RES-013 | `docs/handoff/probe_auth_scope.py` | tooling | srt | supporting | required | low | Auth-scope response classifier/prober |
| RES-016 | `docs/handoff/pyproject.toml` | env | srt | supporting | optional | low | Capture workspace dependencies |
| RES-010 | `docs/handoff/run_cli.py` | tooling | srt | supporting | required | low | Instrumented CLI launcher |
| RES-015 | `docs/handoff/tests` | tests | srt | supporting | optional | low | Capture/redaction smoke/unit tests |
| RES-011 | `docs/handoff/vendor/srtgo_capture/capture_runtime.py` | tooling | srt | supporting | required | medium | HTTP capture hooks + redaction for CLI flows |
| RES-012 | `docs/handoff/web_capture_runtime.py` | tooling | srt_web | supporting | optional | low | Playwright web capture runtime |

## Notes

- `sensitivity=high` means the artifact location may contain raw or user-captured data and should not be copied into docs.
- `required_for_parity=required` marks minimum resources for functional parity rewrite.
