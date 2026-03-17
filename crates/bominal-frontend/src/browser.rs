//! Browser-only helpers kept behind a small Rust interop boundary.

pub const DEFAULT_LOCALE: &str = "ko";

pub fn current_theme() -> String {
    current_root_attr("data-theme")
        .unwrap_or_else(|| crate::theme::ThemeName::default().as_str().to_string())
}

pub fn current_mode() -> String {
    current_root_attr("data-mode")
        .unwrap_or_else(|| crate::theme::ThemeMode::default().as_str().to_string())
}

pub fn current_locale() -> String {
    cookie_value("bominal-locale").unwrap_or_else(|| DEFAULT_LOCALE.to_string())
}

pub fn set_theme(theme: &str) {
    set_root_attr("data-theme", theme);
    set_cookie(&format!(
        "{}={theme};path=/;max-age=31536000;samesite=lax",
        crate::theme::THEME_COOKIE
    ));
}

pub fn set_mode(mode: &str) {
    set_root_attr("data-mode", mode);
    set_cookie(&format!(
        "{}={mode};path=/;max-age=31536000;samesite=lax",
        crate::theme::MODE_COOKIE
    ));
}

pub fn set_locale(locale: &str) {
    set_cookie(&format!(
        "{}={locale};path=/;max-age=31536000;samesite=lax",
        crate::i18n::LOCALE_COOKIE
    ));
}

pub fn reload_page() {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        let _ = window.location().reload();
    }
}

pub fn redirect_to(_path: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(_path);
    }
}

pub async fn submit_card(
    label: &str,
    card_number: &str,
    card_password: &str,
    birthday: &str,
    expire_mmyy: &str,
    card_type: &str,
) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        use js_sys::{Function, Promise, Reflect};
        use wasm_bindgen::{JsCast, JsValue};
        use wasm_bindgen_futures::JsFuture;

        let window = web_sys::window().ok_or_else(|| "No browser window available".to_string())?;
        let submit = Reflect::get(&window, &JsValue::from_str("__submitCard"))
            .map_err(js_error_to_string)?;
        let submit = submit
            .dyn_into::<Function>()
            .map_err(|_| "Card encryption bridge is unavailable".to_string())?;

        let promise = submit
            .call6(
                &window,
                &JsValue::from_str(label),
                &JsValue::from_str(card_number),
                &JsValue::from_str(card_password),
                &JsValue::from_str(birthday),
                &JsValue::from_str(expire_mmyy),
                &JsValue::from_str(card_type),
            )
            .map_err(js_error_to_string)?;

        let result = JsFuture::from(Promise::from(promise))
            .await
            .map_err(js_error_to_string)?;

        let ok = Reflect::get(&result, &JsValue::from_str("ok"))
            .ok()
            .and_then(|value| value.as_bool())
            .unwrap_or(false);

        if ok {
            Ok(())
        } else {
            let error = Reflect::get(&result, &JsValue::from_str("error"))
                .ok()
                .and_then(|value| value.as_string())
                .unwrap_or_else(|| "Card submission failed".to_string());
            Err(error)
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (
            label,
            card_number,
            card_password,
            birthday,
            expire_mmyy,
            card_type,
        );
        Err("Card submission is only available in the browser".to_string())
    }
}

fn current_root_attr(name: &str) -> Option<String> {
    root_attr(name).filter(|value| !value.trim().is_empty())
}

#[cfg(target_arch = "wasm32")]
fn js_error_to_string(error: wasm_bindgen::JsValue) -> String {
    error
        .as_string()
        .unwrap_or_else(|| "Browser interop failed".to_string())
}

fn root_attr(_name: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window()?;
        let document = window.document()?;
        return document
            .document_element()
            .and_then(|element| element.get_attribute(_name));
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
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

fn cookie_value(_name: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window()?;
        let document = window.document()?;
        let cookies = document.cookie().ok()?;
        return cookies
            .split(';')
            .filter_map(|entry| {
                let mut parts = entry.trim().splitn(2, '=');
                Some((parts.next()?, parts.next()?))
            })
            .find_map(|(key, value)| (key == _name).then(|| value.to_string()));
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

fn set_cookie(_cookie: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window()
        && let Some(document) = window.document()
    {
        let _ = document.set_cookie(_cookie);
    }
}
