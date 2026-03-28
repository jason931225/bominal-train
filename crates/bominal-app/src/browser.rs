//! Browser-only DOM helpers kept behind a small Rust boundary.

pub fn sync_theme_attrs(theme: &str, mode: &str) {
    set_root_attr("data-theme", theme);
    set_root_attr("data-mode", mode);
}

pub fn redirect_to(_path: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(_path);
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub async fn conditional_passkey_mediation_available() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        use js_sys::{Function, Promise, Reflect};
        use wasm_bindgen::JsCast;
        use wasm_bindgen::JsValue;
        use wasm_bindgen_futures::JsFuture;

        let Some(window) = web_sys::window() else {
            return false;
        };

        let public_key_credential =
            match Reflect::get(&window, &JsValue::from_str("PublicKeyCredential")) {
                Ok(value) if !value.is_null() && !value.is_undefined() => value,
                _ => return false,
            };

        let availability_fn = match Reflect::get(
            &public_key_credential,
            &JsValue::from_str("isConditionalMediationAvailable"),
        ) {
            Ok(value) => value,
            Err(_) => return false,
        };

        let Some(availability_fn) = availability_fn.dyn_ref::<Function>() else {
            return false;
        };

        let promise = match availability_fn.call0(&public_key_credential) {
            Ok(value) => Promise::from(value),
            Err(_) => return false,
        };

        match JsFuture::from(promise).await {
            Ok(result) => result.as_bool().unwrap_or(false),
            Err(_) => false,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub async fn start_passkey_login(options_json: &str) -> Result<String, String> {
    call_window_string_function("__startPasskeyLogin", options_json).await
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub async fn start_passkey_registration(options_json: &str) -> Result<String, String> {
    call_window_string_function("__startPasskeyRegistration", options_json).await
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub async fn start_conditional_passkey_login(options_json: &str) -> Result<String, String> {
    call_window_string_function("__startConditionalPasskeyLogin", options_json).await
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
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
        use wasm_bindgen::JsValue;

        let (_, submit) = window_function("__submitCard")?;
        let promise = submit
            .call6(
                &web_sys::window()
                    .ok_or_else(|| "No browser window available".to_string())?
                    .into(),
                &JsValue::from_str(label),
                &JsValue::from_str(card_number),
                &JsValue::from_str(card_password),
                &JsValue::from_str(birthday),
                &JsValue::from_str(expire_mmyy),
                &JsValue::from_str(card_type),
            )
            .map_err(js_error_to_string)?;

        let result = await_promise(promise).await?;

        let ok = js_sys::Reflect::get(&result, &JsValue::from_str("ok"))
            .ok()
            .and_then(|value| value.as_bool())
            .unwrap_or(false);

        if ok {
            Ok(())
        } else {
            let error = js_sys::Reflect::get(&result, &JsValue::from_str("error"))
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

#[cfg(target_arch = "wasm32")]
fn js_error_to_string(error: wasm_bindgen::JsValue) -> String {
    error
        .as_string()
        .unwrap_or_else(|| "Browser interop failed".to_string())
}

#[cfg(target_arch = "wasm32")]
fn window_function(name: &str) -> Result<(web_sys::Window, js_sys::Function), String> {
    use js_sys::Reflect;
    use wasm_bindgen::{JsCast, JsValue};

    let window = web_sys::window().ok_or_else(|| "No browser window available".to_string())?;
    let function = Reflect::get(&window, &JsValue::from_str(name)).map_err(js_error_to_string)?;
    let function = function
        .dyn_into::<js_sys::Function>()
        .map_err(|_| format!("Browser interop function {name} is unavailable"))?;
    Ok((window, function))
}

#[cfg(target_arch = "wasm32")]
async fn call_window_string_function(name: &str, input: &str) -> Result<String, String> {
    use wasm_bindgen::JsValue;

    let (window, function) = window_function(name)?;
    let result = function
        .call1(&window.into(), &JsValue::from_str(input))
        .map_err(js_error_to_string)?;
    let result = await_promise(result).await?;
    result
        .as_string()
        .ok_or_else(|| format!("Browser interop function {name} returned a non-string result"))
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
async fn call_window_string_function(_name: &str, _input: &str) -> Result<String, String> {
    Err("Browser interop is only available in the browser".to_string())
}

#[cfg(target_arch = "wasm32")]
async fn await_promise(value: wasm_bindgen::JsValue) -> Result<wasm_bindgen::JsValue, String> {
    use js_sys::Promise;
    use wasm_bindgen_futures::JsFuture;

    JsFuture::from(Promise::from(value))
        .await
        .map_err(js_error_to_string)
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
