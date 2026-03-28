.PHONY: build release serve watch

build:
	cargo leptos build

release:
	cargo leptos build --release --precompress

serve:
	cargo leptos serve

watch:
	cargo leptos watch
