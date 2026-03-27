//! Browser-only DOM helpers kept behind a small Rust boundary.

pub fn sync_theme_attrs(theme: &str, mode: &str) {
    set_root_attr("data-theme", theme);
    set_root_attr("data-mode", mode);
}

fn set_root_attr(_name: &str, _value: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window()
        && let Some(document) = window.document()
        && let Some(element) = document.document_element()
    {
        let _ = element.set_attribute(_name, _value);
    }
}
