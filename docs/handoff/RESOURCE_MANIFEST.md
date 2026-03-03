# Handoff Resource Manifest

- Manifest scope date: `2026-03-03`
- Purpose: maintained list of currently valid handoff references.

## Inventory

| id | path | kind | provider | canonicality | required_for_parity | sensitivity | purpose |
|---|---|---|---|---|---|---|---|
| RES-305 | `docs/handoff/README.md` | handoff_doc | meta | canonical | required | low | Handoff docs entrypoint |
| RES-306 | `docs/handoff/RESOURCE_INDEX.md` | handoff_doc | meta | canonical | required | low | Pointer index for handoff materials |
| RES-300 | `docs/handoff/HANDOFF_EXTERNAL_REWRITE.md` | handoff_doc | srt/ktx | canonical | required | low | Rewrite handoff decisions |
| RES-001 | `docs/handoff/PROVIDER_CONTRACT.md` | contract_doc | srt | canonical | required | low | Provider contract decisions |
| RES-002 | `docs/handoff/PROVIDER_FIELD_MAP.json` | field_map_json | srt/ktx | canonical | required | low | Machine-readable endpoint and field mapping |
| RES-003 | `docs/handoff/PROVIDER_FIELD_MAP.md` | field_map_doc | srt/ktx | canonical | required | low | Human-readable endpoint and field mapping |
| RES-303 | `docs/handoff/STATLESS_ENDPOINT_SHORTLIST.md` | handoff_doc | srt | canonical | required | low | Stateless/context-gated shortlist |
| RES-304 | `docs/handoff/UNRESOLVED_ENDPOINTS.md` | handoff_doc | srt/ktx | canonical | required | low | Unresolved endpoint backlog |
| RES-309 | `docs/handoff/RUST_BACKEND_PARITY.md` | handoff_doc | runtime | supporting | optional | low | Historical parity gap analysis |
| RES-310 | `docs/handoff/RUST_IMPLEMENTATION_FILE_MANIFEST.md` | handoff_doc | runtime | canonical | required | low | Current Rust runtime file inventory |
| RES-100 | `third_party/srtgo/README.md` | reference_doc | srt | supplemental | required | low | Upstream SRT context |
| RES-101 | `third_party/srtgo/srtgo/srt.py` | source_code | srt | canonical | required | low | Canonical SRT behavior reference |
| RES-102 | `third_party/srtgo/srtgo/ktx.py` | source_code | ktx | canonical | required | low | Canonical KTX behavior reference |
| RES-020 | `docs/handoff/output/auth_scope_probe/latest_auth_scope_probe.json` | probe_output | srt | supporting | optional | medium | Captured auth-scope evidence |
| RES-021 | `docs/handoff/output/` | capture_output | srt | supporting | optional | medium | Redacted capture bundles |

## Notes

- This manifest only lists paths that currently exist in the repository.
- Historical snapshot manifests were retired to remove stale path references.
