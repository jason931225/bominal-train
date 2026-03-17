.PHONY: css css-watch ts

TAILWIND := $(shell command -v tailwindcss 2>/dev/null || echo "npx --yes @tailwindcss/cli")

css:
	$(TAILWIND) -i crates/bominal-frontend/style/main.css \
	            -o crates/bominal-frontend/style/output.css \
	            --content 'crates/bominal-frontend/src/**/*.rs'

css-watch:
	$(TAILWIND) -i crates/bominal-frontend/style/main.css \
	            -o crates/bominal-frontend/style/output.css \
	            --content 'crates/bominal-frontend/src/**/*.rs' --watch

ts:
	npx --yes esbuild crates/bominal-frontend/ts/interop.ts \
	  --bundle --outfile=crates/bominal-frontend/ts/interop.js \
	  --format=iife --global-name=BominalInterop
