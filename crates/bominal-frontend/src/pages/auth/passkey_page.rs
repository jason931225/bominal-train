//! PasskeyPage — default auth entry point at `/auth`.
//! Shows fingerprint hero, passkey sign-in button, and email fallback link.

use leptos::prelude::*;

use crate::i18n::t;

use super::{auth_shell, icon_fingerprint, icon_key, icon_mail};

#[component]
pub fn PasskeyPage() -> impl IntoView {
    let error_msg = RwSignal::new(Option::<String>::None);

    let on_passkey_login = move |_| {
        error_msg.set(None);
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                match crate::api::passkey::do_passkey_login().await {
                    Ok(()) => {}
                    Err(e) => {
                        let msg = e
                            .as_string()
                            .unwrap_or_else(|| crate::i18n::t("error.passkey_failed").into());
                        error_msg.set(Some(msg));
                    }
                }
            });
        }
    };

    auth_shell(view! {
        <div class="glass-panel p-8 rounded-3xl flex flex-col gap-5">
            <div class="text-center mb-2">
                <div class="w-20 h-20 mx-auto mb-4 rounded-2xl flex items-center justify-center ring-1 ring-[var(--color-brand-border)]"
                     style="background: linear-gradient(135deg, var(--color-bg-elevated), var(--color-bg-sunken))">
                    {icon_fingerprint()}
                </div>
                <h1 class="text-2xl font-bold text-[var(--color-text-primary)] tracking-tight">{t("auth.welcome_back")}</h1>
                <p class="text-sm text-[var(--color-text-tertiary)] mt-1.5">{t("auth.passkey_subtitle")}</p>
            </div>

            {move || error_msg.get().map(|msg| view! {
                <div class="px-3 py-2 rounded-xl"
                     style="background: var(--color-status-error-bg); border: 1px solid var(--color-status-error-bg)">
                    <p class="text-sm text-[var(--color-status-error)]">{msg}</p>
                </div>
            })}

            <button
                on:click=on_passkey_login
                class="w-full py-3.5 btn-primary font-semibold rounded-xl flex items-center justify-center gap-2.5 active:scale-95 transition-all"
            >
                {icon_key()}
                {t("auth.passkey_signin")}
            </button>

            <div class="flex items-center gap-2">
                <div class="flex-1 h-px bg-[var(--color-border-subtle)]"></div>
                <span class="text-xs text-[var(--color-text-disabled)] font-medium">{t("common.or")}</span>
                <div class="flex-1 h-px bg-[var(--color-border-subtle)]"></div>
            </div>

            <a
                href="/auth/login"
                class="w-full py-3 bg-[var(--color-bg-elevated)] border border-[var(--color-border-default)] text-[var(--color-text-primary)] font-semibold rounded-xl flex items-center justify-center gap-2 shadow-sm hover:bg-[var(--color-interactive-hover)] active:scale-95 transition-all"
            >
                <span class="text-[var(--color-text-disabled)]">{icon_mail()}</span>
                {t("auth.continue_email")}
            </a>

            <p class="text-center text-xs text-[var(--color-text-disabled)]">
                {t("auth.no_account")} " "
                <a href="/auth/signup" class="text-[var(--color-brand-text)] font-semibold hover:underline">
                    {t("auth.signup_link")}
                </a>
            </p>
        </div>
    }.into_any())
}
