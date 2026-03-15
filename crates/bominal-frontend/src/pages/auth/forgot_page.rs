//! ForgotPage — request password reset at `/auth/forgot`.

use leptos::prelude::*;

use crate::api::auth::ForgotPassword;
use crate::i18n::t;

use super::{auth_shell, icon_arrow_left, icon_mail};

#[component]
pub fn ForgotPage() -> impl IntoView {
    let forgot_email = RwSignal::new(String::new());
    let forgot_sent = RwSignal::new(false);
    let forgot_action = ServerAction::<ForgotPassword>::new();
    let forgot_pending = forgot_action.pending();

    Effect::new(move |_| {
        if let Some(result) = forgot_action.value().get()
            && result.is_ok()
        {
            forgot_sent.set(true);
        }
    });

    auth_shell(view! {
        <div class="glass-panel p-8 rounded-3xl flex flex-col gap-5">
            <a
                href="/auth/login"
                class="flex items-center gap-1.5 text-sm text-[var(--color-text-tertiary)] hover:text-[var(--color-text-secondary)] -mb-2 transition-colors w-fit"
            >
                {icon_arrow_left()} {t("auth.back_to_signin")}
            </a>

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
    }.into_any())
}
