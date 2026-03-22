//! LoginPage — email + password sign-in at `/auth/login`.

use leptos::prelude::*;

use crate::api::auth::Login;
use crate::i18n::t;

use super::{auth_shell, icon_arrow_left, icon_eye, icon_eye_off, icon_key, icon_lock, icon_mail};

#[component]
pub fn LoginPage() -> impl IntoView {
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let show_password = RwSignal::new(false);
    let error_msg = RwSignal::new(Option::<String>::None);

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
        error_msg.set(None);
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = crate::api::passkey::do_passkey_login().await {
                    let msg = e
                        .as_string()
                        .unwrap_or_else(|| "Passkey login failed".to_string());
                    error_msg.set(Some(msg));
                }
            });
        }
    };

    auth_shell(view! {
        <div class="glass-panel p-8 rounded-3xl flex flex-col gap-5">
            <a
                href="/auth"
                class="flex items-center gap-1.5 text-sm text-[var(--color-text-tertiary)] hover:text-[var(--color-text-secondary)] -mb-2 transition-colors w-fit"
            >
                {icon_arrow_left()} {t("common.back")}
            </a>

            <div>
                <h1 class="text-2xl font-bold text-[var(--color-text-primary)] tracking-tight">{t("auth.sign_in")}</h1>
                <p class="text-sm text-[var(--color-text-tertiary)] mt-1">{t("auth.enter_email_password")}</p>
            </div>

            {move || error_msg.get().map(|msg| view! {
                <div role="alert" class="px-3 py-2 rounded-xl"
                     style="background: var(--color-status-error-bg); border: 1px solid var(--color-status-error-bg)">
                    <p class="text-sm text-[var(--color-status-error)]">{msg}</p>
                </div>
            })}

            <ActionForm action=login_action>
                <div class="flex flex-col gap-3">
                    <div class="relative">
                        <span class="absolute left-3.5 top-1/2 -translate-y-1/2 text-[var(--color-text-disabled)]">{icon_mail()}</span>
                        <input
                            type="email"
                            name="email"
                            required
                            aria-label=t("auth.email_placeholder")
                            prop:value=move || email.get()
                            on:input=move |ev| email.set(event_target_value(&ev))
                            placeholder=t("auth.email_placeholder")
                            class="w-full pl-10 pr-4 py-3 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm font-medium text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:ring-2 focus:ring-[var(--color-brand-primary)]/50 focus:border-[var(--color-border-focus)] transition-all"
                        />
                    </div>

                    <div class="relative">
                        <span class="absolute left-3.5 top-1/2 -translate-y-1/2 text-[var(--color-text-disabled)]">{icon_lock()}</span>
                        <input
                            type=move || if show_password.get() { "text" } else { "password" }
                            name="password"
                            required
                            aria-label=t("auth.password")
                            prop:value=move || password.get()
                            on:input=move |ev| password.set(event_target_value(&ev))
                            placeholder=t("auth.password")
                            class="w-full pl-10 pr-10 py-3 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm font-medium text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:ring-2 focus:ring-[var(--color-brand-primary)]/50 focus:border-[var(--color-border-focus)] transition-all"
                        />
                        <button
                            type="button"
                            aria-label=move || if show_password.get() { t("auth.hide_password") } else { t("auth.show_password") }
                            on:click=move |_| show_password.update(|v| *v = !*v)
                            class="absolute right-1 top-1/2 -translate-y-1/2 min-h-11 min-w-11 flex items-center justify-center text-[var(--color-text-disabled)] hover:text-[var(--color-text-secondary)] transition-colors"
                        >
                            {move || if show_password.get() { icon_eye_off().into_any() } else { icon_eye().into_any() }}
                        </button>
                    </div>

                    <div class="flex justify-end -mt-1">
                        <a href="/auth/forgot" class="text-xs text-[var(--color-brand-text)] font-medium hover:underline">
                            {t("auth.forgot_password")}
                        </a>
                    </div>
                </div>

                <button
                    type="submit"
                    class="w-full mt-4 py-3.5 btn-primary font-semibold rounded-xl active:scale-95 transition-all disabled:opacity-50"
                    disabled=login_pending
                >
                    {move || if login_pending.get() { t("common.loading") } else { t("auth.sign_in") }}
                </button>
            </ActionForm>

            <div class="flex items-center gap-2">
                <div class="flex-1 h-px bg-[var(--color-border-subtle)]"></div>
                <span class="text-xs text-[var(--color-text-disabled)] font-medium">{t("common.or")}</span>
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
                <a href="/auth/signup" class="text-[var(--color-brand-text)] font-semibold hover:underline">
                    {t("auth.signup_link")}
                </a>
            </p>
        </div>
    }.into_any())
}
