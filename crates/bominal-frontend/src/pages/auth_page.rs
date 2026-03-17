//! Authentication page — 6-view passkey-first auth flow.
//!
//! Views: Passkey (default) → EmailForm / Signup → Forgot / VerifyEmail → AddPasskey
//! Matches the prototype in `bominal-train-reservation/src/components/AuthPage.tsx`.

use leptos::prelude::*;

use crate::api::auth::{ForgotPassword, Login, ResendVerification};
use crate::i18n::t;

// ── View state machine ──────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum AuthView {
    Passkey,
    EmailForm,
    Signup,
    Forgot,
    VerifyEmail,
    AddPasskey,
}

// ── Passkey login ceremony (WASM only) ──────────────────────────────

#[cfg(target_arch = "wasm32")]
async fn do_passkey_login() -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;

    // Step 1: Fetch challenge from server
    let resp = JsFuture::from(window.fetch_with_str("/api/auth/passkey/login/start")).await?;
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

    // Step 2: Call the JS WebAuthn interop to perform the browser ceremony
    let start_fn = js_sys::Reflect::get(&window, &JsValue::from_str("__startPasskeyLogin"))?;
    let start_fn: &js_sys::Function = start_fn
        .dyn_ref()
        .ok_or_else(|| JsValue::from_str("__startPasskeyLogin not found"))?;

    let assertion_promise = start_fn.call1(&JsValue::NULL, &JsValue::from_str(&options_str))?;
    let assertion_js = JsFuture::from(js_sys::Promise::from(assertion_promise)).await?;
    let assertion_str = assertion_js
        .as_string()
        .ok_or_else(|| JsValue::from_str("Assertion not a string"))?;

    // Step 3: POST the assertion to the server to complete authentication
    let credential: serde_json::Value = serde_json::from_str(&assertion_str)
        .map_err(|e| JsValue::from_str(&format!("Bad assertion JSON: {e}")))?;

    let body = serde_json::json!({
        "challenge_id": challenge_id,
        "credential": credential,
    });

    let mut opts = web_sys::RequestInit::new();
    opts.method("POST");
    opts.body(Some(&JsValue::from_str(
        &serde_json::to_string(&body).unwrap(),
    )));

    let headers = web_sys::Headers::new()?;
    headers.set("Content-Type", "application/json")?;
    opts.headers(&headers);
    opts.credentials(web_sys::RequestCredentials::Include);

    let request =
        web_sys::Request::new_with_str_and_init("/api/auth/passkey/login/finish", &opts)?;

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

// ── Passkey registration ceremony (WASM only) ───────────────────────

#[cfg(target_arch = "wasm32")]
async fn do_passkey_register() -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;

    // Step 1: Fetch registration challenge (requires session cookie)
    let mut start_opts = web_sys::RequestInit::new();
    start_opts.method("POST");
    start_opts.credentials(web_sys::RequestCredentials::Include);

    let start_req = web_sys::Request::new_with_str_and_init(
        "/api/auth/passkey/register/start",
        &start_opts,
    )?;

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

    // Step 2: Call JS WebAuthn interop for credential creation
    let reg_fn =
        js_sys::Reflect::get(&window, &JsValue::from_str("__startPasskeyRegistration"))?;
    let reg_fn: &js_sys::Function = reg_fn
        .dyn_ref()
        .ok_or_else(|| JsValue::from_str("__startPasskeyRegistration not found"))?;

    let cred_promise = reg_fn.call1(&JsValue::NULL, &JsValue::from_str(&options_str))?;
    let cred_js = JsFuture::from(js_sys::Promise::from(cred_promise)).await?;
    let cred_str = cred_js
        .as_string()
        .ok_or_else(|| JsValue::from_str("Credential not a string"))?;

    // Step 3: POST the attestation to complete registration
    let credential: serde_json::Value = serde_json::from_str(&cred_str)
        .map_err(|e| JsValue::from_str(&format!("Bad credential JSON: {e}")))?;

    let body = serde_json::json!({
        "challenge_id": challenge_id,
        "credential": credential,
    });

    let mut finish_opts = web_sys::RequestInit::new();
    finish_opts.method("POST");
    finish_opts.body(Some(&JsValue::from_str(
        &serde_json::to_string(&body).unwrap(),
    )));

    let headers = web_sys::Headers::new()?;
    headers.set("Content-Type", "application/json")?;
    finish_opts.headers(&headers);
    finish_opts.credentials(web_sys::RequestCredentials::Include);

    let finish_req = web_sys::Request::new_with_str_and_init(
        "/api/auth/passkey/register/finish",
        &finish_opts,
    )?;

    let finish_resp = JsFuture::from(window.fetch_with_request(&finish_req)).await?;
    let finish_resp: web_sys::Response = finish_resp.dyn_into()?;

    if !finish_resp.ok() {
        let status = finish_resp.status();
        return Err(JsValue::from_str(&format!(
            "Passkey registration failed (HTTP {status})"
        )));
    }

    // Redirect to home after successful registration
    let _ = window.location().set_href("/home");
    Ok(())
}

// ── Helpers ─────────────────────────────────────────────────────────

fn password_strength(pw: &str) -> usize {
    let mut score = 0;
    if pw.len() >= 8 {
        score += 1;
    }
    if pw.chars().any(|c| c.is_uppercase()) {
        score += 1;
    }
    if pw.chars().any(|c| c.is_ascii_digit()) {
        score += 1;
    }
    if pw.chars().any(|c| !c.is_alphanumeric()) {
        score += 1;
    }
    score
}

fn strength_label(score: usize) -> &'static str {
    match score {
        1 => "auth.pw_weak",
        2 => "auth.pw_fair",
        3 => "auth.pw_good",
        4 => "auth.pw_strong",
        _ => "",
    }
}

fn strength_color(score: usize) -> &'static str {
    match score {
        1 => "bg-red-400",
        2 => "bg-orange-400",
        3 => "bg-yellow-400",
        4 => "bg-emerald-500",
        _ => "bg-slate-200",
    }
}

fn strength_text_color(score: usize) -> &'static str {
    match score {
        1 => "text-red-500",
        2 => "text-orange-500",
        3 => "text-yellow-600",
        4 => "text-emerald-600",
        _ => "",
    }
}

// ── SVG icon helpers ────────────────────────────────────────────────

fn icon_fingerprint() -> impl IntoView {
    view! {
        <svg class="w-8 h-8 text-indigo-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.5">
            <path stroke-linecap="round" stroke-linejoin="round"
                d="M7.864 4.243A7.5 7.5 0 0119.5 10.5c0 2.92-.556 5.709-1.568 8.268M5.742 6.364A7.465 7.465 0 004.5 10.5a48.667 48.667 0 00-1.214 8.036M12.5 3a7.5 7.5 0 016.396 3.568M12 10.5a2 2 0 10-4 0 2 2 0 004 0zm0 0v3.5" />
        </svg>
    }
}

fn icon_mail() -> impl IntoView {
    view! {
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round"
                d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
        </svg>
    }
}

fn icon_lock() -> impl IntoView {
    view! {
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round"
                d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
        </svg>
    }
}

fn icon_key() -> impl IntoView {
    view! {
        <svg class="w-4.5 h-4.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round"
                d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
        </svg>
    }
}

fn icon_arrow_left() -> impl IntoView {
    view! {
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M10.5 19.5L3 12m0 0l7.5-7.5M3 12h18" />
        </svg>
    }
}

fn icon_user_plus() -> impl IntoView {
    view! {
        <svg class="w-6 h-6 text-emerald-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round"
                d="M19 7.5v3m0 0v3m0-3h3m-3 0h-3m-2.25-4.125a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zM4 19.235v-.11a6.375 6.375 0 0112.75 0v.109A12.318 12.318 0 0110.374 21c-2.331 0-4.512-.645-6.374-1.766z" />
        </svg>
    }
}

fn icon_eye() -> impl IntoView {
    view! {
        <svg class="w-4.5 h-4.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round"
                d="M2.036 12.322a1.012 1.012 0 010-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178z" />
            <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
    }
}

fn icon_eye_off() -> impl IntoView {
    view! {
        <svg class="w-4.5 h-4.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round"
                d="M3.98 8.223A10.477 10.477 0 001.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.45 10.45 0 0112 4.5c4.756 0 8.773 3.162 10.065 7.498a10.523 10.523 0 01-4.293 5.774M6.228 6.228L3 3m3.228 3.228l3.65 3.65m7.894 7.894L21 21m-3.228-3.228l-3.65-3.65m0 0a3 3 0 10-4.243-4.243m4.242 4.242L9.88 9.88" />
        </svg>
    }
}

fn icon_mail_large() -> impl IntoView {
    view! {
        <svg class="w-7 h-7 text-amber-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.5">
            <path stroke-linecap="round" stroke-linejoin="round"
                d="M21.75 6.75v10.5a2.25 2.25 0 01-2.25 2.25h-15a2.25 2.25 0 01-2.25-2.25V6.75m19.5 0A2.25 2.25 0 0019.5 4.5h-15a2.25 2.25 0 00-2.25 2.25m19.5 0v.243a2.25 2.25 0 01-1.07 1.916l-7.5 4.615a2.25 2.25 0 01-2.36 0L3.32 8.91a2.25 2.25 0 01-1.07-1.916V6.75" />
        </svg>
    }
}

// ── Sub-components ───────────────────────────────────────────────────
//
// Each #[component] creates an opaque `impl IntoView` boundary, preventing
// the compiler from computing a combined monomorphic tuple type for all 6 views.

#[component]
fn PasskeyView(
    auth_view: RwSignal<AuthView>,
    error_msg: RwSignal<Option<String>>,
) -> impl IntoView {
    let on_passkey_login = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async {
                if let Err(e) = do_passkey_login().await {
                    web_sys::console::error_1(&e.into());
                }
            });
        }
    };

    view! {
        <div class="glass-panel p-8 rounded-3xl flex flex-col gap-5">
            <div class="text-center mb-2">
                <div class="w-16 h-16 mx-auto mb-4 rounded-2xl flex items-center justify-center ring-1 ring-indigo-200/50"
                     style="background: linear-gradient(135deg, var(--color-bg-elevated), var(--color-bg-sunken))">
                    {icon_fingerprint()}
                </div>
                <h1 class="text-2xl font-bold text-[var(--color-text-primary)] tracking-tight">{t("auth.welcome_back")}</h1>
                <p class="text-sm text-[var(--color-text-tertiary)] mt-1.5">{t("auth.passkey_subtitle")}</p>
            </div>

            <button
                on:click=on_passkey_login
                class="w-full py-3.5 btn-glass font-semibold rounded-xl flex items-center justify-center gap-2.5 shadow-lg active:scale-95 transition-all"
            >
                {icon_key()}
                {t("auth.passkey_signin")}
            </button>

            <div class="flex items-center gap-2">
                <div class="flex-1 h-px bg-[var(--color-border-subtle)]"></div>
                <span class="text-xs text-[var(--color-text-disabled)] font-medium">"or"</span>
                <div class="flex-1 h-px bg-[var(--color-border-subtle)]"></div>
            </div>

            <button
                on:click=move |_| { error_msg.set(None); auth_view.set(AuthView::EmailForm); }
                class="w-full py-3 bg-[var(--color-bg-elevated)] border border-[var(--color-border-default)] text-[var(--color-text-primary)] font-semibold rounded-xl flex items-center justify-center gap-2 shadow-sm hover:bg-[var(--color-interactive-hover)] active:scale-95 transition-all"
            >
                <span class="text-[var(--color-text-disabled)]">{icon_mail()}</span>
                {t("auth.continue_email")}
            </button>

            <p class="text-center text-xs text-[var(--color-text-disabled)]">
                {t("auth.no_account")} " "
                <button
                    on:click=move |_| { error_msg.set(None); auth_view.set(AuthView::Signup); }
                    class="text-[var(--color-brand-primary)] font-semibold hover:underline"
                >
                    {t("auth.signup_link")}
                </button>
            </p>
        </div>
    }
}

#[component]
fn EmailFormView(
    auth_view: RwSignal<AuthView>,
    error_msg: RwSignal<Option<String>>,
) -> impl IntoView {
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let show_password = RwSignal::new(false);

    let login_action = ServerAction::<Login>::new();
    let login_pending = login_action.pending();

    Effect::new(move |_| {
        if let Some(result) = login_action.value().get() {
            match result {
                Ok(()) => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        if let Some(w) = web_sys::window() {
                            let _ = w.location().set_href("/home");
                        }
                    }
                }
                Err(e) => {
                    error_msg.set(Some(e.to_string()));
                }
            }
        }
    });

    let on_passkey_login = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async {
                if let Err(e) = do_passkey_login().await {
                    web_sys::console::error_1(&e.into());
                }
            });
        }
    };

    view! {
        <div class="glass-panel p-8 rounded-3xl flex flex-col gap-5">
            <button
                on:click=move |_| { error_msg.set(None); auth_view.set(AuthView::Passkey); }
                class="flex items-center gap-1.5 text-sm text-[var(--color-text-tertiary)] hover:text-[var(--color-text-secondary)] -mb-2 transition-colors w-fit"
            >
                {icon_arrow_left()} {t("common.back")}
            </button>

            <div>
                <h1 class="text-2xl font-bold text-[var(--color-text-primary)] tracking-tight">{t("auth.sign_in")}</h1>
                <p class="text-sm text-[var(--color-text-tertiary)] mt-1">{t("auth.enter_email_password")}</p>
            </div>

            <ActionForm action=login_action>
                <div class="flex flex-col gap-3">
                    <div class="relative">
                        <span class="absolute left-3.5 top-1/2 -translate-y-1/2 text-[var(--color-text-disabled)]">{icon_mail()}</span>
                        <input
                            type="email"
                            name="email"
                            required
                            prop:value=move || email.get()
                            on:input=move |ev| email.set(event_target_value(&ev))
                            placeholder="Email address"
                            class="w-full pl-10 pr-4 py-3 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm font-medium text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:ring-2 focus:ring-[var(--color-brand-primary)]/50 focus:border-[var(--color-border-focus)] transition-all"
                        />
                    </div>

                    <div class="relative">
                        <span class="absolute left-3.5 top-1/2 -translate-y-1/2 text-[var(--color-text-disabled)]">{icon_lock()}</span>
                        <input
                            type=move || if show_password.get() { "text" } else { "password" }
                            name="password"
                            required
                            prop:value=move || password.get()
                            on:input=move |ev| password.set(event_target_value(&ev))
                            placeholder="Password"
                            class="w-full pl-10 pr-10 py-3 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm font-medium text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:ring-2 focus:ring-[var(--color-brand-primary)]/50 focus:border-[var(--color-border-focus)] transition-all"
                        />
                        <button
                            type="button"
                            on:click=move |_| show_password.update(|v| *v = !*v)
                            class="absolute right-3.5 top-1/2 -translate-y-1/2 text-[var(--color-text-disabled)] hover:text-[var(--color-text-secondary)] transition-colors"
                        >
                            {move || if show_password.get() { icon_eye_off().into_any() } else { icon_eye().into_any() }}
                        </button>
                    </div>

                    <div class="flex justify-end -mt-1">
                        <button
                            type="button"
                            on:click=move |_| { error_msg.set(None); auth_view.set(AuthView::Forgot); }
                            class="text-xs text-[var(--color-brand-primary)] font-medium hover:underline"
                        >
                            {t("auth.forgot_password")}
                        </button>
                    </div>
                </div>

                <button
                    type="submit"
                    class="w-full mt-4 py-3.5 btn-glass font-semibold rounded-xl shadow-lg active:scale-95 transition-all disabled:opacity-50"
                    disabled=login_pending
                >
                    {move || if login_pending.get() { t("common.loading") } else { t("auth.sign_in") }}
                </button>
            </ActionForm>

            <div class="flex items-center gap-2">
                <div class="flex-1 h-px bg-[var(--color-border-subtle)]"></div>
                <span class="text-xs text-[var(--color-text-disabled)] font-medium">"or"</span>
                <div class="flex-1 h-px bg-[var(--color-border-subtle)]"></div>
            </div>

            <button
                on:click=on_passkey_login
                class="w-full py-3 bg-[var(--color-bg-elevated)] border border-[var(--color-border-default)] text-[var(--color-text-primary)] font-semibold rounded-xl flex items-center justify-center gap-2 shadow-sm hover:bg-[var(--color-interactive-hover)] active:scale-95 transition-all"
            >
                {icon_key()}
                {t("auth.use_passkey")}
            </button>

            <p class="text-center text-xs text-[var(--color-text-disabled)]">
                {t("auth.no_account")} " "
                <button
                    on:click=move |_| { error_msg.set(None); auth_view.set(AuthView::Signup); }
                    class="text-[var(--color-brand-primary)] font-semibold hover:underline"
                >
                    {t("auth.signup_link")}
                </button>
            </p>
        </div>
    }
}

#[component]
fn SignupView(
    auth_view: RwSignal<AuthView>,
    error_msg: RwSignal<Option<String>>,
    signup_email: RwSignal<String>,
) -> impl IntoView {
    let signup_name = RwSignal::new(String::new());
    let signup_password = RwSignal::new(String::new());
    let signup_password_confirm = RwSignal::new(String::new());
    let show_signup_password = RwSignal::new(false);
    let show_signup_password2 = RwSignal::new(false);
    let register_pending = RwSignal::new(false);

    let pw_strength = move || password_strength(&signup_password.get());
    let passwords_match = move || {
        let pw = signup_password.get();
        let confirm = signup_password_confirm.get();
        !pw.is_empty() && !confirm.is_empty() && pw == confirm
    };
    let passwords_mismatch = move || {
        let confirm = signup_password_confirm.get();
        !confirm.is_empty() && signup_password.get() != confirm
    };

    let on_register_submit = move |_| {
        if register_pending.get() {
            return;
        }
        if passwords_mismatch() {
            return;
        }
        register_pending.set(true);
        error_msg.set(None);

        #[cfg(target_arch = "wasm32")]
        {
            let name = signup_name.get();
            let em = signup_email.get();
            let pw = signup_password.get();
            use crate::api::auth::register;
            wasm_bindgen_futures::spawn_local(async move {
                match register(em, pw, name).await {
                    Ok(()) => {
                        register_pending.set(false);
                        auth_view.set(AuthView::VerifyEmail);
                    }
                    Err(e) => {
                        register_pending.set(false);
                        error_msg.set(Some(e.to_string()));
                    }
                }
            });
        }
    };

    view! {
        <div class="glass-panel p-8 rounded-3xl flex flex-col gap-5">
            <button
                on:click=move |_| { error_msg.set(None); auth_view.set(AuthView::Passkey); }
                class="flex items-center gap-1.5 text-sm text-[var(--color-text-tertiary)] hover:text-[var(--color-text-secondary)] -mb-2 transition-colors w-fit"
            >
                {icon_arrow_left()} {t("common.back")}
            </button>

            <div class="text-center">
                <div class="w-14 h-14 mx-auto mb-3 rounded-2xl flex items-center justify-center ring-1 ring-emerald-200/50"
                     style="background: linear-gradient(135deg, var(--color-bg-elevated), var(--color-bg-sunken))">
                    {icon_user_plus()}
                </div>
                <h1 class="text-2xl font-bold text-[var(--color-text-primary)] tracking-tight">{t("auth.create_account")}</h1>
                <p class="text-sm text-[var(--color-text-tertiary)] mt-1">{t("auth.get_started")}</p>
            </div>

            <div class="flex flex-col gap-3">
                <input
                    type="text"
                    prop:value=move || signup_name.get()
                    on:input=move |ev| signup_name.set(event_target_value(&ev))
                    placeholder=t("auth.display_name")
                    class="w-full px-4 py-3 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm font-medium text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:ring-2 focus:ring-[var(--color-brand-primary)]/50 focus:border-[var(--color-border-focus)] transition-all"
                />
                <input
                    type="email"
                    prop:value=move || signup_email.get()
                    on:input=move |ev| signup_email.set(event_target_value(&ev))
                    placeholder="Email address"
                    class="w-full px-4 py-3 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm font-medium text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:ring-2 focus:ring-[var(--color-brand-primary)]/50 focus:border-[var(--color-border-focus)] transition-all"
                />

                // Password + strength meter
                <div class="flex flex-col gap-1.5">
                    <div class="relative">
                        <input
                            type=move || if show_signup_password.get() { "text" } else { "password" }
                            prop:value=move || signup_password.get()
                            on:input=move |ev| signup_password.set(event_target_value(&ev))
                            placeholder=t("auth.password")
                            class="w-full pr-10 px-4 py-3 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm font-medium text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:ring-2 focus:ring-[var(--color-brand-primary)]/50 focus:border-[var(--color-border-focus)] transition-all"
                        />
                        <button
                            type="button"
                            on:click=move |_| show_signup_password.update(|v| *v = !*v)
                            class="absolute right-3.5 top-1/2 -translate-y-1/2 text-[var(--color-text-disabled)] hover:text-[var(--color-text-secondary)]"
                        >
                            {move || if show_signup_password.get() { icon_eye_off().into_any() } else { icon_eye().into_any() }}
                        </button>
                    </div>

                    // Strength indicator
                    {move || {
                        let pw = signup_password.get();
                        if pw.is_empty() {
                            None
                        } else {
                            let score = pw_strength();
                            Some(view! {
                                <div class="flex items-center gap-2">
                                    <div class="flex gap-1 flex-1">
                                        {(1..=4).map(|level| {
                                            let bar_class = if level <= score {
                                                format!("h-1 flex-1 rounded-full transition-all duration-300 {}", strength_color(score))
                                            } else {
                                                "h-1 flex-1 rounded-full transition-all duration-300 bg-[var(--color-border-subtle)]".to_string()
                                            };
                                            view! { <div class=bar_class></div> }
                                        }).collect_view()}
                                    </div>
                                    <span class={format!("text-xs font-semibold {}", strength_text_color(score))}>
                                        {t(strength_label(score))}
                                    </span>
                                </div>
                            })
                        }
                    }}
                </div>

                // Confirm password
                <div class="flex flex-col gap-1">
                    <div class="relative">
                        <input
                            type=move || if show_signup_password2.get() { "text" } else { "password" }
                            prop:value=move || signup_password_confirm.get()
                            on:input=move |ev| signup_password_confirm.set(event_target_value(&ev))
                            placeholder=t("auth.confirm_password")
                            class=move || {
                                let base = "w-full pr-10 px-4 py-3 bg-[var(--color-bg-sunken)] border rounded-xl text-sm font-medium text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:ring-2 transition-all";
                                if passwords_mismatch() {
                                    format!("{base} border-red-300 focus:ring-red-400/50 focus:border-red-400")
                                } else if passwords_match() {
                                    format!("{base} border-emerald-300 focus:ring-emerald-400/50 focus:border-emerald-400")
                                } else {
                                    format!("{base} border-[var(--color-border-default)] focus:ring-[var(--color-brand-primary)]/50 focus:border-[var(--color-border-focus)]")
                                }
                            }
                        />
                        <button
                            type="button"
                            on:click=move |_| show_signup_password2.update(|v| *v = !*v)
                            class="absolute right-3.5 top-1/2 -translate-y-1/2 text-[var(--color-text-disabled)] hover:text-[var(--color-text-secondary)]"
                        >
                            {move || if show_signup_password2.get() { icon_eye_off().into_any() } else { icon_eye().into_any() }}
                        </button>
                    </div>
                    {move || if passwords_mismatch() {
                        Some(view! { <p class="text-xs text-red-500 font-medium pl-1">{t("auth.passwords_mismatch")}</p> })
                    } else if passwords_match() {
                        Some(view! { <p class="text-xs text-emerald-600 font-medium pl-1">{t("auth.passwords_match")}</p> })
                    } else {
                        None
                    }}
                </div>
            </div>

            <button
                on:click=on_register_submit
                class="w-full py-3.5 btn-glass font-semibold rounded-xl shadow-lg active:scale-95 transition-all disabled:opacity-50"
                disabled=move || register_pending.get() || passwords_mismatch()
            >
                {move || if register_pending.get() { t("common.loading") } else { t("auth.create_account") }}
            </button>

            <p class="text-center text-xs text-[var(--color-text-disabled)]">
                {t("auth.has_account")} " "
                <button
                    on:click=move |_| { error_msg.set(None); auth_view.set(AuthView::Passkey); }
                    class="text-[var(--color-brand-primary)] font-semibold hover:underline"
                >
                    {t("auth.signin_link")}
                </button>
            </p>
        </div>
    }
}

#[component]
fn ForgotView(
    auth_view: RwSignal<AuthView>,
    error_msg: RwSignal<Option<String>>,
) -> impl IntoView {
    let forgot_email = RwSignal::new(String::new());
    let forgot_sent = RwSignal::new(false);
    let forgot_action = ServerAction::<ForgotPassword>::new();
    let forgot_pending = forgot_action.pending();

    Effect::new(move |_| {
        if let Some(result) = forgot_action.value().get() {
            if result.is_ok() {
                forgot_sent.set(true);
            }
        }
    });

    view! {
        <div class="glass-panel p-8 rounded-3xl flex flex-col gap-5">
            <button
                on:click=move |_| { error_msg.set(None); auth_view.set(AuthView::EmailForm); }
                class="flex items-center gap-1.5 text-sm text-[var(--color-text-tertiary)] hover:text-[var(--color-text-secondary)] -mb-2 transition-colors w-fit"
            >
                {icon_arrow_left()} {t("auth.back_to_signin")}
            </button>

            <div>
                <h1 class="text-2xl font-bold text-[var(--color-text-primary)] tracking-tight">{t("auth.reset_password")}</h1>
                <p class="text-sm text-[var(--color-text-tertiary)] mt-1">{t("auth.reset_subtitle")}</p>
            </div>

            {move || forgot_sent.get().then(|| view! {
                <div class="px-3 py-2 rounded-xl bg-emerald-500/10 border border-emerald-500/20">
                    <p class="text-sm text-emerald-600 font-medium">{t("auth.reset_link_sent")}</p>
                </div>
            })}

            <ActionForm action=forgot_action>
                <div class="relative">
                    <span class="absolute left-3.5 top-1/2 -translate-y-1/2 text-[var(--color-text-disabled)]">{icon_mail()}</span>
                    <input
                        type="email"
                        name="email"
                        required
                        prop:value=move || forgot_email.get()
                        on:input=move |ev| forgot_email.set(event_target_value(&ev))
                        placeholder="Email address"
                        class="w-full pl-10 pr-4 py-3 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm font-medium text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:ring-2 focus:ring-[var(--color-brand-primary)]/50 focus:border-[var(--color-border-focus)] transition-all"
                    />
                </div>

                <button
                    type="submit"
                    class="w-full mt-4 py-3.5 btn-glass font-semibold rounded-xl shadow-lg active:scale-95 transition-all disabled:opacity-50"
                    disabled=forgot_pending
                >
                    {move || if forgot_pending.get() { t("common.loading") } else { t("auth.send_reset_link") }}
                </button>
            </ActionForm>
        </div>
    }
}

#[component]
fn VerifyEmailView(
    auth_view: RwSignal<AuthView>,
    error_msg: RwSignal<Option<String>>,
    signup_email: RwSignal<String>,
) -> impl IntoView {
    let resend_action = ServerAction::<ResendVerification>::new();
    let resend_pending = resend_action.pending();

    view! {
        <div class="glass-panel p-8 rounded-3xl flex flex-col gap-6">
            <div class="text-center">
                <div class="w-16 h-16 mx-auto mb-4 rounded-2xl flex items-center justify-center ring-1 ring-amber-200/50"
                     style="background: linear-gradient(135deg, var(--color-bg-elevated), var(--color-bg-sunken))">
                    {icon_mail_large()}
                </div>
                <h1 class="text-2xl font-bold text-[var(--color-text-primary)] tracking-tight">{t("auth.check_email")}</h1>
                <p class="text-sm text-[var(--color-text-tertiary)] mt-2 leading-relaxed">
                    {t("auth.verify_sent_to")} " "
                    <span class="font-semibold text-[var(--color-text-primary)]">
                        {move || {
                            let em = signup_email.get();
                            if em.is_empty() { "your email".to_string() } else { em }
                        }}
                    </span>
                    ". " {t("auth.verify_click_link")}
                </p>
            </div>

            <div class="bg-amber-500/10 border border-amber-500/20 rounded-2xl p-4 text-sm text-amber-600 font-medium text-center">
                {t("auth.resend_prompt")} " "
                <ActionForm action=resend_action attr:class="inline">
                    <button
                        type="submit"
                        class="underline font-semibold hover:text-amber-700"
                        disabled=resend_pending
                    >
                        {move || if resend_pending.get() { t("common.loading") } else { t("auth.resend_link") }}
                    </button>
                </ActionForm>
            </div>

            <button
                on:click=move |_| auth_view.set(AuthView::AddPasskey)
                class="w-full py-3.5 btn-glass font-semibold rounded-xl shadow-lg active:scale-95 transition-all"
            >
                {t("auth.verified_continue")} " \u{2192}"
            </button>

            <button
                on:click=move |_| { error_msg.set(None); auth_view.set(AuthView::Signup); }
                class="w-full py-2.5 text-sm text-[var(--color-text-disabled)] font-medium hover:text-[var(--color-text-secondary)] transition-colors"
            >
                "\u{2190} " {t("auth.back_to_signup")}
            </button>
        </div>
    }
}

#[component]
fn AddPasskeyView() -> impl IntoView {
    let on_passkey_register = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async {
                if let Err(e) = do_passkey_register().await {
                    web_sys::console::error_1(&e.into());
                }
            });
        }
    };

    let on_skip_passkey = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(w) = web_sys::window() {
                let _ = w.location().set_href("/home");
            }
        }
    };

    view! {
        <div class="glass-panel p-8 rounded-3xl flex flex-col gap-6">
            <div class="text-center">
                <div class="w-16 h-16 mx-auto mb-4 rounded-2xl flex items-center justify-center ring-1 ring-violet-200/50"
                     style="background: linear-gradient(135deg, var(--color-bg-elevated), var(--color-bg-sunken))">
                    {icon_fingerprint()}
                </div>
                <h1 class="text-2xl font-bold text-[var(--color-text-primary)] tracking-tight">{t("auth.add_passkey")}</h1>
                <p class="text-sm text-[var(--color-text-tertiary)] mt-2 leading-relaxed">
                    {t("auth.passkey_subtitle")}
                </p>
            </div>

            <div class="bg-[var(--color-bg-sunken)] rounded-2xl p-4 flex flex-col gap-2.5">
                <div class="flex items-center gap-3 text-sm text-[var(--color-text-secondary)]">
                    <span class="text-base shrink-0">"&#x1F680;"</span>
                    <span>{t("auth.passkey_benefit_1")}</span>
                </div>
                <div class="flex items-center gap-3 text-sm text-[var(--color-text-secondary)]">
                    <span class="text-base shrink-0">"&#x1F512;"</span>
                    <span>{t("auth.passkey_benefit_2")}</span>
                </div>
                <div class="flex items-center gap-3 text-sm text-[var(--color-text-secondary)]">
                    <span class="text-base shrink-0">"&#x1F310;"</span>
                    <span>{t("auth.passkey_benefit_3")}</span>
                </div>
            </div>

            <button
                on:click=on_passkey_register
                class="w-full py-3.5 btn-glass font-semibold rounded-xl flex items-center justify-center gap-2 shadow-lg active:scale-95 transition-all"
            >
                {icon_key()}
                {t("auth.add_passkey_now")}
            </button>

            <button
                on:click=on_skip_passkey
                class="w-full py-3 text-sm text-[var(--color-text-disabled)] font-medium hover:text-[var(--color-text-secondary)] transition-colors"
            >
                {t("auth.skip_for_now")}
            </button>
        </div>
    }
}

// ── Main component ──────────────────────────────────────────────────

/// Auth page with 6-view passkey-first flow matching the prototype.
#[component]
pub fn AuthPage() -> impl IntoView {
    let auth_view = RwSignal::new(AuthView::Passkey);
    let error_msg = RwSignal::new(Option::<String>::None);
    // Shared between SignupView (write) and VerifyEmailView (read)
    let signup_email = RwSignal::new(String::new());

    view! {
        <div class="min-h-screen w-full flex items-center justify-center px-4 relative overflow-hidden">
            // Background blobs
            <div class="absolute top-1/4 -left-32 w-96 h-96 rounded-full blur-[120px] pointer-events-none opacity-20"
                 style="background: var(--color-brand-primary)"></div>
            <div class="absolute bottom-1/4 -right-32 w-96 h-96 rounded-full blur-[120px] pointer-events-none opacity-20"
                 style="background: var(--color-mesh-2)"></div>

            <div class="w-full max-w-sm relative z-10">
                // Logo
                <div class="flex justify-center mb-8">
                    <div class="flex items-center gap-3">
                        <div class="app-brand-badge flex h-10 w-10 items-center justify-center rounded-xl rotate-3">
                            <div class="app-brand-badge-inner h-5 w-5 rounded-md border-[2.5px]"></div>
                        </div>
                        <span class="app-brand-wordmark text-2xl font-bold bg-clip-text text-transparent tracking-tight">"Bominal"</span>
                    </div>
                </div>

                // Error message (shared across views)
                {move || error_msg.get().map(|msg| view! {
                    <div class="mb-4 px-3 py-2 rounded-xl"
                         style="background: var(--color-status-error-bg); border: 1px solid var(--color-status-error-bg)">
                        <p class="text-sm text-[var(--color-status-error)]">{msg}</p>
                    </div>
                })}

                <Show when=move || auth_view.get() == AuthView::Passkey>
                    <PasskeyView auth_view=auth_view error_msg=error_msg />
                </Show>
                <Show when=move || auth_view.get() == AuthView::EmailForm>
                    <EmailFormView auth_view=auth_view error_msg=error_msg />
                </Show>
                <Show when=move || auth_view.get() == AuthView::Signup>
                    <SignupView auth_view=auth_view error_msg=error_msg signup_email=signup_email />
                </Show>
                <Show when=move || auth_view.get() == AuthView::Forgot>
                    <ForgotView auth_view=auth_view error_msg=error_msg />
                </Show>
                <Show when=move || auth_view.get() == AuthView::VerifyEmail>
                    <VerifyEmailView auth_view=auth_view error_msg=error_msg signup_email=signup_email />
                </Show>
                <Show when=move || auth_view.get() == AuthView::AddPasskey>
                    <AddPasskeyView />
                </Show>
            </div>
        </div>
    }
}
