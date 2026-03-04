# Runtime UI SVGZ Icons

Optimized icon assets used by the runtime web shell (`runtime/crates/api/src/web.rs`).

## Contents

- `.svgz` variants only, flattened into this directory for direct URL usage.
- `source-manifest.json` from the original extraction pass (line mappings + hashes).

## Usage

Reference these assets via `/assets/icons/runtime-ui/<name>.svgz`.
The pages that consume dark/light variants should set both:

- `data-svgz-light="/assets/icons/runtime-ui/<name>-light.svgz"`
- `data-svgz-dark="/assets/icons/runtime-ui/<name>-dark.svgz"`

and update `src` from `data-theme-mode`.

## Notes

- `*-themed.svgz` and `*-currentColor.svgz` are included for parity but are typically not used with plain `<img>`.
- Provider status assets are icon-only; keep `.provider-status-chip` wrappers for card/chip backgrounds.
