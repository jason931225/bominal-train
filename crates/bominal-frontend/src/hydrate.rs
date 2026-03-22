//! WASM hydration entry point.
//!
//! Called by the browser after SSR HTML loads. Attaches Leptos reactivity
//! to the server-rendered DOM so all `on:click`, `on:input` etc. handlers work.

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub fn hydrate() {
    use crate::app::App;
    leptos::mount::hydrate_body(App);
}
