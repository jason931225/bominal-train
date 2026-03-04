# Frontend Static Assets

Static files copied to `runtime/frontend/dist` are sourced from this directory,
except generated/vendor artifacts emitted by frontend build scripts.

Structure:

- `icons/auth` - auth page and security iconography.
- `icons/brand` - brand-facing assets such as favicon.
- `icons/common` - shared utility icons.
- `icons/theme` - theme switch icons.
- `logos` - payment and train-provider logos used in runtime UI.

Build sync:

- `npm run build:assets` mirrors this tree into `dist/`.
- `npm run build:css` and `npm run watch:css` run asset sync automatically.
- `runtime/frontend/dist/` is generated output and is not committed to git.

SVGZ generation:

- `./runtime/frontend/assets/generate-svgz.sh` creates `.svgz` files next to every `.svg`.
- Original `.svg` files remain in place.
- Keep logo pairs as `*.min.svg` and `*.min.svgz` when optimized assets are available.
