//! Minimal WebAuthn client for auth landing and add-passkey flows.

#[cfg(target_arch = "wasm32")]
pub async fn do_passkey_login() -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::prelude::*;

    let (challenge_id, options_json) = start_passkey(
        "/api/auth/passkey/login/start",
        "Failed to start passkey login",
    )
    .await?;
    let assertion = crate::browser::start_passkey_login(&options_json)
        .await
        .map_err(|error| JsValue::from_str(&error))?;
    let credential: serde_json::Value = serde_json::from_str(&assertion)
        .map_err(|error| JsValue::from_str(&format!("Bad assertion JSON: {error}")))?;

    finish_passkey(
        "/api/auth/passkey/login/finish",
        serde_json::json!({
            "challenge_id": challenge_id,
            "credential": credential,
        }),
    )
    .await?;

    crate::browser::redirect_to("/home");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub async fn do_passkey_register() -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::prelude::*;
    let (challenge_id, options_json) = start_passkey(
        "/api/auth/passkey/register/start",
        "Failed to start passkey registration",
    )
    .await?;
    let credential = crate::browser::start_passkey_registration(&options_json)
        .await
        .map_err(|error| JsValue::from_str(&error))?;
    let credential: serde_json::Value = serde_json::from_str(&credential)
        .map_err(|error| JsValue::from_str(&format!("Bad credential JSON: {error}")))?;

    finish_passkey(
        "/api/auth/passkey/register/finish",
        serde_json::json!({
            "challenge_id": challenge_id,
            "credential": credential,
        }),
    )
    .await?;

    crate::browser::redirect_to("/home");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub async fn do_conditional_passkey_login() -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::prelude::*;

    if !crate::browser::conditional_passkey_mediation_available().await {
        return Ok(());
    }

    let (challenge_id, options_json) = start_passkey(
        "/api/auth/passkey/login/start",
        "Failed to start conditional passkey login",
    )
    .await?;
    let assertion = crate::browser::start_conditional_passkey_login(&options_json)
        .await
        .map_err(|error| JsValue::from_str(&error))?;
    let credential: serde_json::Value = serde_json::from_str(&assertion)
        .map_err(|error| JsValue::from_str(&format!("Bad assertion JSON: {error}")))?;

    finish_passkey(
        "/api/auth/passkey/login/finish",
        serde_json::json!({
            "challenge_id": challenge_id,
            "credential": credential,
        }),
    )
    .await?;

    crate::browser::redirect_to("/home");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
async fn start_passkey(
    path: &str,
    start_error: &str,
) -> Result<(String, String), wasm_bindgen::JsValue> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    let start_opts = web_sys::RequestInit::new();
    start_opts.set_method("POST");
    start_opts.set_credentials(web_sys::RequestCredentials::Include);

    let start_req = web_sys::Request::new_with_str_and_init(path, &start_opts)?;
    let response = JsFuture::from(window.fetch_with_request(&start_req)).await?;
    let response: web_sys::Response = response.dyn_into()?;

    if !response.ok() {
        return Err(JsValue::from_str(start_error));
    }

    let json = JsFuture::from(response.json()?).await?;
    let challenge_id = js_sys::Reflect::get(&json, &JsValue::from_str("challenge_id"))?
        .as_string()
        .ok_or_else(|| JsValue::from_str("Missing challenge_id"))?;
    let options = js_sys::Reflect::get(&json, &JsValue::from_str("options"))?;
    let options_json = js_sys::JSON::stringify(&options)?
        .as_string()
        .ok_or_else(|| JsValue::from_str("Failed to serialize options"))?;

    Ok((challenge_id, options_json))
}

#[cfg(target_arch = "wasm32")]
async fn finish_passkey(
    path: &str,
    payload: serde_json::Value,
) -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;

    let finish_opts = web_sys::RequestInit::new();
    finish_opts.set_method("POST");
    finish_opts.set_body(&JsValue::from_str(
        &serde_json::to_string(&payload)
            .map_err(|error| JsValue::from_str(&format!("JSON error: {error}")))?,
    ));
    finish_opts.set_credentials(web_sys::RequestCredentials::Include);

    let headers = web_sys::Headers::new()?;
    headers.set("Content-Type", "application/json")?;
    finish_opts.set_headers(&headers);

    let request = web_sys::Request::new_with_str_and_init(path, &finish_opts)?;
    let response = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response: web_sys::Response = response.dyn_into()?;

    if response.ok() {
        Ok(())
    } else {
        Err(JsValue::from_str(&format!(
            "Passkey request failed (HTTP {})",
            response.status()
        )))
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn do_passkey_login() -> Result<(), wasm_bindgen::JsValue> {
    Err(wasm_bindgen::JsValue::from_str(
        "Passkey login requires a browser runtime",
    ))
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn do_conditional_passkey_login() -> Result<(), wasm_bindgen::JsValue> {
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn do_passkey_register() -> Result<(), wasm_bindgen::JsValue> {
    Err(wasm_bindgen::JsValue::from_str(
        "Passkey registration requires a browser runtime",
    ))
}
