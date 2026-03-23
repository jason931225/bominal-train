//! Passkey (WebAuthn) WASM ceremony functions — shared between auth pages.
//!
//! Each fn fetches a server challenge, calls the JS interop bridge in `interop.ts`,
//! then posts the result back to the server to complete the ceremony.

// ── Passkey login ceremony (WASM only) ──────────────────────────────

#[cfg(target_arch = "wasm32")]
pub async fn do_passkey_login() -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;

    let mut start_opts = web_sys::RequestInit::new();
    start_opts.method("POST");
    start_opts.credentials(web_sys::RequestCredentials::Include);

    let start_req =
        web_sys::Request::new_with_str_and_init("/api/auth/passkey/login/start", &start_opts)?;

    let resp = JsFuture::from(window.fetch_with_request(&start_req)).await?;
    let resp: web_sys::Response = resp.dyn_into()?;

    if !resp.ok() {
        return Err(JsValue::from_str("Failed to start passkey login"));
    }

    let json = JsFuture::from(resp.json()?).await?;
    let challenge_id = js_sys::Reflect::get(&json, &JsValue::from_str("challenge_id"))?
        .as_string()
        .ok_or_else(|| JsValue::from_str("Missing challenge_id"))?;

    let options = js_sys::Reflect::get(&json, &JsValue::from_str("options"))?;
    let options_str = js_sys::JSON::stringify(&options)?
        .as_string()
        .ok_or_else(|| JsValue::from_str("Failed to serialize options"))?;

    let start_fn = js_sys::Reflect::get(&window, &JsValue::from_str("__startPasskeyLogin"))?;
    let start_fn: &js_sys::Function = start_fn
        .dyn_ref()
        .ok_or_else(|| JsValue::from_str("__startPasskeyLogin not found"))?;

    let assertion_promise = start_fn.call1(&JsValue::NULL, &JsValue::from_str(&options_str))?;
    let assertion_js = JsFuture::from(js_sys::Promise::from(assertion_promise)).await?;
    let assertion_str = assertion_js
        .as_string()
        .ok_or_else(|| JsValue::from_str("Assertion not a string"))?;

    let credential: serde_json::Value = serde_json::from_str(&assertion_str)
        .map_err(|e| JsValue::from_str(&format!("Bad assertion JSON: {e}")))?;

    let body = serde_json::json!({
        "challenge_id": challenge_id,
        "credential": credential,
    });

    let mut opts = web_sys::RequestInit::new();
    opts.method("POST");
    opts.body(Some(&JsValue::from_str(
        &serde_json::to_string(&body)
            .map_err(|e| JsValue::from_str(&format!("JSON error: {e}")))?,
    )));

    let headers = web_sys::Headers::new()?;
    headers.set("Content-Type", "application/json")?;
    opts.headers(&headers);
    opts.credentials(web_sys::RequestCredentials::Include);

    let request = web_sys::Request::new_with_str_and_init("/api/auth/passkey/login/finish", &opts)?;

    let finish_resp = JsFuture::from(window.fetch_with_request(&request)).await?;
    let finish_resp: web_sys::Response = finish_resp.dyn_into()?;

    if finish_resp.ok() {
        let _ = window.location().set_href("/home");
    } else {
        let status = finish_resp.status();
        return Err(JsValue::from_str(&format!(
            "Passkey login failed (HTTP {status})"
        )));
    }

    Ok(())
}

// ── Conditional passkey login ceremony (WASM only) ──────────────────
// Uses mediation: "conditional" so passkeys appear in the browser's
// autofill dropdown on the email field. Blocks until the user picks one.

#[cfg(target_arch = "wasm32")]
pub async fn do_conditional_passkey_login() -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;

    // Check if conditional mediation is supported
    let pk_cred = js_sys::Reflect::get(&window, &JsValue::from_str("PublicKeyCredential"))?;
    if pk_cred.is_undefined() {
        return Ok(());
    }
    let is_available_fn = js_sys::Reflect::get(
        &pk_cred,
        &JsValue::from_str("isConditionalMediationAvailable"),
    )?;
    if let Some(func) = is_available_fn.dyn_ref::<js_sys::Function>() {
        let result = JsFuture::from(js_sys::Promise::from(func.call0(&pk_cred)?)).await?;
        if !result.as_bool().unwrap_or(false) {
            return Ok(());
        }
    } else {
        return Ok(());
    }

    let mut start_opts = web_sys::RequestInit::new();
    start_opts.method("POST");
    start_opts.credentials(web_sys::RequestCredentials::Include);

    let start_req =
        web_sys::Request::new_with_str_and_init("/api/auth/passkey/login/start", &start_opts)?;

    let resp = JsFuture::from(window.fetch_with_request(&start_req)).await?;
    let resp: web_sys::Response = resp.dyn_into()?;

    if !resp.ok() {
        return Err(JsValue::from_str("Failed to start conditional passkey login"));
    }

    let json = JsFuture::from(resp.json()?).await?;
    let challenge_id = js_sys::Reflect::get(&json, &JsValue::from_str("challenge_id"))?
        .as_string()
        .ok_or_else(|| JsValue::from_str("Missing challenge_id"))?;

    let options = js_sys::Reflect::get(&json, &JsValue::from_str("options"))?;
    let options_str = js_sys::JSON::stringify(&options)?
        .as_string()
        .ok_or_else(|| JsValue::from_str("Failed to serialize options"))?;

    let start_fn =
        js_sys::Reflect::get(&window, &JsValue::from_str("__startConditionalPasskeyLogin"))?;
    let start_fn: &js_sys::Function = start_fn
        .dyn_ref()
        .ok_or_else(|| JsValue::from_str("__startConditionalPasskeyLogin not found"))?;

    let assertion_promise = start_fn.call1(&JsValue::NULL, &JsValue::from_str(&options_str))?;
    let assertion_js = JsFuture::from(js_sys::Promise::from(assertion_promise)).await?;
    let assertion_str = assertion_js
        .as_string()
        .ok_or_else(|| JsValue::from_str("Assertion not a string"))?;

    let credential: serde_json::Value = serde_json::from_str(&assertion_str)
        .map_err(|e| JsValue::from_str(&format!("Bad assertion JSON: {e}")))?;

    let body = serde_json::json!({
        "challenge_id": challenge_id,
        "credential": credential,
    });

    let mut opts = web_sys::RequestInit::new();
    opts.method("POST");
    opts.body(Some(&JsValue::from_str(
        &serde_json::to_string(&body)
            .map_err(|e| JsValue::from_str(&format!("JSON error: {e}")))?,
    )));

    let headers = web_sys::Headers::new()?;
    headers.set("Content-Type", "application/json")?;
    opts.headers(&headers);
    opts.credentials(web_sys::RequestCredentials::Include);

    let request = web_sys::Request::new_with_str_and_init("/api/auth/passkey/login/finish", &opts)?;

    let finish_resp = JsFuture::from(window.fetch_with_request(&request)).await?;
    let finish_resp: web_sys::Response = finish_resp.dyn_into()?;

    if finish_resp.ok() {
        let _ = window.location().set_href("/home");
    } else {
        let status = finish_resp.status();
        return Err(JsValue::from_str(&format!(
            "Conditional passkey login failed (HTTP {status})"
        )));
    }

    Ok(())
}

// ── Passkey registration ceremony (WASM only) ───────────────────────

#[cfg(target_arch = "wasm32")]
pub async fn do_passkey_register() -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;

    let mut start_opts = web_sys::RequestInit::new();
    start_opts.method("POST");
    start_opts.credentials(web_sys::RequestCredentials::Include);

    let start_req =
        web_sys::Request::new_with_str_and_init("/api/auth/passkey/register/start", &start_opts)?;

    let resp = JsFuture::from(window.fetch_with_request(&start_req)).await?;
    let resp: web_sys::Response = resp.dyn_into()?;

    if !resp.ok() {
        return Err(JsValue::from_str("Failed to start passkey registration"));
    }

    let json = JsFuture::from(resp.json()?).await?;
    let challenge_id = js_sys::Reflect::get(&json, &JsValue::from_str("challenge_id"))?
        .as_string()
        .ok_or_else(|| JsValue::from_str("Missing challenge_id"))?;

    let options = js_sys::Reflect::get(&json, &JsValue::from_str("options"))?;
    let options_str = js_sys::JSON::stringify(&options)?
        .as_string()
        .ok_or_else(|| JsValue::from_str("Failed to serialize options"))?;

    let reg_fn = js_sys::Reflect::get(&window, &JsValue::from_str("__startPasskeyRegistration"))?;
    let reg_fn: &js_sys::Function = reg_fn
        .dyn_ref()
        .ok_or_else(|| JsValue::from_str("__startPasskeyRegistration not found"))?;

    let cred_promise = reg_fn.call1(&JsValue::NULL, &JsValue::from_str(&options_str))?;
    let cred_js = JsFuture::from(js_sys::Promise::from(cred_promise)).await?;
    let cred_str = cred_js
        .as_string()
        .ok_or_else(|| JsValue::from_str("Credential not a string"))?;

    let credential: serde_json::Value = serde_json::from_str(&cred_str)
        .map_err(|e| JsValue::from_str(&format!("Bad credential JSON: {e}")))?;

    let body = serde_json::json!({
        "challenge_id": challenge_id,
        "credential": credential,
    });

    let mut finish_opts = web_sys::RequestInit::new();
    finish_opts.method("POST");
    finish_opts.body(Some(&JsValue::from_str(
        &serde_json::to_string(&body)
            .map_err(|e| JsValue::from_str(&format!("JSON error: {e}")))?,
    )));

    let headers = web_sys::Headers::new()?;
    headers.set("Content-Type", "application/json")?;
    finish_opts.headers(&headers);
    finish_opts.credentials(web_sys::RequestCredentials::Include);

    let finish_req =
        web_sys::Request::new_with_str_and_init("/api/auth/passkey/register/finish", &finish_opts)?;

    let finish_resp = JsFuture::from(window.fetch_with_request(&finish_req)).await?;
    let finish_resp: web_sys::Response = finish_resp.dyn_into()?;

    if !finish_resp.ok() {
        let status = finish_resp.status();
        return Err(JsValue::from_str(&format!(
            "Passkey registration failed (HTTP {status})"
        )));
    }

    let _ = window.location().set_href("/home");
    Ok(())
}
