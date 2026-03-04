# Frontend Static Assets

All static files copied to `runtime/frontend/dist` are sourced from this directory.

Structure:

- `icons/auth` - auth page and security iconography.
- `icons/brand` - brand-facing assets such as favicon.
- `icons/common` - shared utility icons.
- `icons/theme` - theme switch icons.

Build sync:

- `npm run build:assets` mirrors this tree into `dist/`.
- `npm run build:css` and `npm run watch:css` run asset sync automatically.

SVGZ generation:

- `./runtime/frontend/assets/generate-svgz.sh` creates `.svgz` files next to every `.svg`.
- Original `.svg` files remain in place.
