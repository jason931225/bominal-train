//! Auth pages — main sign-in page with passkey + email navigation.

mod add_passkey;
mod forgot;
mod login;
mod signup;
mod verify;

pub use add_passkey::AddPasskeyPage;
pub use forgot::ForgotPage;
pub use login::LoginPage;
pub use signup::SignupPage;
pub use verify::AuthVerifyPage;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use server_fn::error::ServerFnError;

use crate::i18n::t;

// =============================================================================
// Phosphor icon SVG paths (inline to avoid external dependency)
// =============================================================================

/// Phosphor Train icon (bold).
const TRAIN_ICON: &str = r#"M184,24H72A32,32,0,0,0,40,56V184a32,32,0,0,0,32,32h8L64,232a8,8,0,0,0,16,0l16-16h64l16,16a8,8,0,0,0,16,0l-16-16h8a32,32,0,0,0,32-32V56A32,32,0,0,0,184,24ZM72,40H184a16,16,0,0,1,16,16v64H56V56A16,16,0,0,1,72,40ZM184,200H72a16,16,0,0,1-16-16V136H200v48A16,16,0,0,1,184,200ZM96,172a12,12,0,1,1-12-12A12,12,0,0,1,96,172Zm88,0a12,12,0,1,1-12-12A12,12,0,0,1,184,172Z"#;

/// Phosphor Fingerprint icon.
const FINGERPRINT_ICON: &str = r#"M72,128a134.63,134.63,0,0,1-14.16,60.47,8,8,0,1,1-14.32-7.12A118.8,118.8,0,0,0,56,128,72.08,72.08,0,0,1,128,56a71.56,71.56,0,0,1,44.6,15.49,8,8,0,0,1-10,12.49A55.56,55.56,0,0,0,128,72,56.06,56.06,0,0,0,72,128Zm56-104A104.11,104.11,0,0,0,24,128a87.53,87.53,0,0,1-5.33,30.15,8,8,0,1,0,14.86,5.94A103.62,103.62,0,0,0,40,128a88,88,0,0,1,176,0,281.6,281.6,0,0,1-7.11,56.37,8,8,0,0,0,5.67,9.79,8.11,8.11,0,0,0,2.06.27,8,8,0,0,0,7.73-5.93A297.88,297.88,0,0,0,232,128,104.12,104.12,0,0,0,128,24Zm0,32a72.08,72.08,0,0,0-72,72,8,8,0,0,0,16,0,56.06,56.06,0,0,1,112,0,245.22,245.22,0,0,1-6.21,49.19,8,8,0,0,0,5.65,9.81,8.13,8.13,0,0,0,2.08.27,8,8,0,0,0,7.73-5.93A261.42,261.42,0,0,0,200,128,72.08,72.08,0,0,0,128,56Zm0,40a32,32,0,0,0-32,32,167.43,167.43,0,0,1-8.51,53.06,8,8,0,0,0,14.86,5.94A183.33,183.33,0,0,0,112,128a16,16,0,0,1,32,0,214.67,214.67,0,0,1-20.51,92.34,8,8,0,1,0,14.28,7.22A230.69,230.69,0,0,0,160,128,32,32,0,0,0,128,96Zm0-64A96.11,96.11,0,0,0,32,128a55.8,55.8,0,0,1-4.28,21.57,8,8,0,0,0,14.86,5.94A71.87,71.87,0,0,0,48,128a80,80,0,0,1,160,0,317.35,317.35,0,0,1-7.78,62.57,8,8,0,1,0,15.56,3.88A332.91,332.91,0,0,0,224,128,96.11,96.11,0,0,0,128,32Z"#;

pub(super) fn auth_shell(content: AnyView) -> impl IntoView {
    view! {
        <div class="flex min-h-screen items-center justify-center px-4">
            <div class="page-enter w-full max-w-sm">{content}</div>
        </div>
    }
}

pub(crate) fn format_server_error(error: &ServerFnError) -> String {
    let message = error.to_string();
    message
        .strip_prefix("error running server function: ")
        .unwrap_or(&message)
        .trim()
        .to_string()
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum PasskeyActionKind {
    Login,
    Register,
}

#[island]
pub(super) fn PasskeyActionButton(
    kind: PasskeyActionKind,
    class_name: String,
    label: String,
    loading_label: String,
    error_fallback: String,
    show_icon: bool,
) -> impl IntoView {
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(String::new());

    let on_click = move |_| {
        set_loading.set(true);
        set_error.set(String::new());
        let fallback = error_fallback.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let result = match kind {
                PasskeyActionKind::Login => crate::api::passkey::do_passkey_login().await,
                PasskeyActionKind::Register => crate::api::passkey::do_passkey_register().await,
            };

            if let Err(error) = result {
                let message = error.as_string().unwrap_or_else(|| fallback.clone());
                set_error.set(message);
            }

            set_loading.set(false);
        });
    };

    view! {
        <div class="flex flex-col gap-3">
            <button
                type="button"
                class=class_name
                disabled=move || loading.get()
                on:click=on_click
            >
                {show_icon.then_some(view! {
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="24"
                        height="24"
                        viewBox="0 0 256 256"
                        fill="currentColor"
                    >
                        <path d=FINGERPRINT_ICON />
                    </svg>
                })}
                {move || if loading.get() { loading_label.clone() } else { label.clone() }}
            </button>

            {move || {
                let message = error.get();
                if message.is_empty() {
                    None
                } else {
                    Some(view! {
                        <p class="text-sm text-center" style="color: var(--lg-error);">{message}</p>
                    })
                }
            }}
        </div>
    }
}

#[island]
pub(super) fn ConditionalPasskeyLogin() -> impl IntoView {
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(error) = crate::api::passkey::do_conditional_passkey_login().await {
                leptos::logging::log!(
                    "Conditional passkey login failed: {}",
                    error.as_string().unwrap_or_default()
                );
            }
        });
    });

    view! { <></> }
}

/// Main auth page (/auth) — passkey button + email/signup navigation.
#[component]
pub fn SignInPage() -> impl IntoView {
    auth_shell(view! {
        <div class="lg-glass-panel flex flex-col items-center gap-6 p-6 text-center">
            <div
                class="flex h-16 w-16 items-center justify-center rounded-2xl"
                style="background-color: var(--lg-accent-bg); color: var(--lg-text-primary);"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="36"
                    height="36"
                    viewBox="0 0 256 256"
                    fill="currentColor"
                    aria-hidden="true"
                >
                    <path d=TRAIN_ICON />
                </svg>
            </div>

            <h1
                class="app-brand-wordmark bg-clip-text text-4xl font-bold tracking-tight text-transparent"
            >
                "Bominal"
            </h1>
            <p class="text-sm" style="color: var(--lg-text-tertiary);">
                {t("auth.get_started")}
            </p>

            <PasskeyActionButton
                kind=PasskeyActionKind::Login
                class_name="lg-btn-primary squish flex w-full items-center justify-center gap-3 rounded-2xl px-6 py-3.5 text-base".to_string()
                label=t("auth.passkey_signin").to_string()
                loading_label=t("common.loading").to_string()
                error_fallback="Passkey login failed".to_string()
                show_icon=true
            />

            <div class="relative w-full">
                <div class="absolute inset-0 flex items-center">
                    <div class="w-full" style="border-top: 1px solid var(--lg-border-default);"></div>
                </div>
            </div>

            <a
                href="/auth/login"
                class="lg-btn-secondary squish block w-full rounded-2xl px-6 py-3 text-center text-sm"
            >
                {t("auth.continue_email")}
            </a>

            <a
                href="/auth/signup"
                class="lg-btn-secondary squish block w-full rounded-2xl px-6 py-3 text-center text-sm"
            >
                {t("auth.signup_link")}
            </a>
        </div>
    }
    .into_any())
}
