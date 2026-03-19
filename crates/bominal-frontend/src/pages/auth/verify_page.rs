//! AuthVerifyPage — post-signup email verification at `/auth/verify`.
//!
//! Reads the user's email from the session via `get_current_user()` Resource
//! — no URL params or cross-page signals needed.

use leptos::prelude::*;

use crate::api::auth::{ResendVerification, get_current_user};
use crate::i18n::t;

use super::{auth_shell, icon_mail_large};

#[component]
pub fn AuthVerifyPage() -> impl IntoView {
    let user_resource = Resource::new(|| (), |_| get_current_user());

    let email = move || {
        user_resource
            .get()
            .and_then(|r| r.ok())
            .flatten()
            .map(|u| u.email)
            .unwrap_or_default()
    };

    let resend_action = ServerAction::<ResendVerification>::new();
    let resend_pending = resend_action.pending();

    auth_shell(view! {
        <div class="glass-panel p-8 rounded-3xl flex flex-col gap-6">
            <div class="text-center">
                <div class="w-16 h-16 mx-auto mb-4 rounded-2xl flex items-center justify-center ring-1 ring-[var(--color-border-default)]"
                     style="background: linear-gradient(135deg, var(--color-bg-elevated), var(--color-bg-sunken))">
                    {icon_mail_large()}
                </div>
                <h1 class="text-2xl font-bold text-[var(--color-text-primary)] tracking-tight">{t("auth.check_email")}</h1>
                <p class="text-sm text-[var(--color-text-tertiary)] mt-2 leading-relaxed">
                    {t("auth.verify_sent_to")} " "
                    <span class="font-semibold text-[var(--color-text-primary)]">
                        {move || { let em = email(); if em.is_empty() { "your email".to_string() } else { em } }}
                    </span>
                    ". " {t("auth.verify_click_link")}
                </p>
            </div>

            <div class="border border-[var(--color-status-warning)]/20 rounded-2xl p-4 text-sm text-[var(--color-status-warning)] font-medium text-center"
                 style="background: var(--color-status-warning-bg)">
                {t("auth.resend_prompt")} " "
                <ActionForm action=resend_action attr:class="inline">
                    <button
                        type="submit"
                        class="underline font-semibold hover:text-[var(--color-status-warning)]"
                        disabled=resend_pending
                    >
                        {move || if resend_pending.get() { t("common.loading") } else { t("auth.resend_link") }}
                    </button>
                </ActionForm>
            </div>

            <a
                href="/auth/add-passkey"
                class="w-full py-3.5 btn-glass font-semibold rounded-xl shadow-lg active:scale-95 transition-all text-center block"
            >
                {t("auth.verified_continue")} " \u{2192}"
            </a>

            <a
                href="/auth/signup"
                class="w-full py-2.5 text-sm text-[var(--color-text-disabled)] font-medium hover:text-[var(--color-text-secondary)] transition-colors text-center block"
            >
                "\u{2190} " {t("auth.back_to_signup")}
            </a>
        </div>
    }.into_any())
}
