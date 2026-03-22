//! AddPasskeyPage — post-signup passkey registration at `/auth/add-passkey`.

use leptos::prelude::*;

use crate::i18n::t;

use super::{auth_shell, icon_fingerprint, icon_key};

#[component]
pub fn AddPasskeyPage() -> impl IntoView {
    let (error, set_error) = signal(Option::<String>::None);

    let on_register = move |_| {
        set_error.set(None);
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = crate::api::passkey::do_passkey_register().await {
                    let msg = e
                        .as_string()
                        .unwrap_or_else(|| "Passkey registration failed".to_string());
                    set_error.set(Some(msg));
                }
            });
        }
    };

    auth_shell(view! {
        <div class="glass-panel p-8 rounded-3xl flex flex-col gap-6">
            <div class="text-center">
                <div class="w-20 h-20 mx-auto mb-4 rounded-2xl flex items-center justify-center ring-1 ring-[var(--color-brand-border)]"
                     style="background: linear-gradient(135deg, var(--color-bg-elevated), var(--color-bg-sunken))">
                    {icon_fingerprint()}
                </div>
                <h1 class="text-2xl font-bold text-[var(--color-text-primary)] tracking-tight">{t("auth.add_passkey")}</h1>
                <p class="text-sm text-[var(--color-text-tertiary)] mt-2 leading-relaxed">
                    {t("auth.passkey_subtitle")}
                </p>
            </div>

            {move || error.get().map(|msg| view! {
                <div class="px-3 py-2 rounded-xl bg-red-50 border border-red-200">
                    <p class="text-sm text-[var(--color-status-error)]">{msg}</p>
                </div>
            })}

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
                on:click=on_register
                class="w-full py-3.5 btn-glass font-semibold rounded-xl flex items-center justify-center gap-2 shadow-lg active:scale-95 transition-all"
            >
                {icon_key()}
                {t("auth.add_passkey_now")}
            </button>

            <a
                href="/home"
                class="w-full py-3 text-sm text-[var(--color-text-disabled)] font-medium hover:text-[var(--color-text-secondary)] transition-colors text-center block"
            >
                {t("auth.skip_for_now")}
            </a>
        </div>
    }.into_any())
}
