//! SignupPage — account registration at `/auth/signup`.

use leptos::prelude::*;

use crate::i18n::t;

use super::{
    auth_shell, icon_arrow_left, icon_eye, icon_eye_off, icon_user_plus, password_strength,
    strength_color, strength_label, strength_text_color,
};

#[component]
pub fn SignupPage() -> impl IntoView {
    let signup_name = RwSignal::new(String::new());
    let signup_email = RwSignal::new(String::new());
    let signup_password = RwSignal::new(String::new());
    let signup_password_confirm = RwSignal::new(String::new());
    let show_password = RwSignal::new(false);
    let show_password2 = RwSignal::new(false);
    let error_msg = RwSignal::new(Option::<String>::None);
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
        if register_pending.get() || passwords_mismatch() {
            return;
        }
        register_pending.set(true);
        error_msg.set(None);

        #[cfg(target_arch = "wasm32")]
        {
            let name = signup_name.get();
            let em = signup_email.get();
            let pw = signup_password.get();
            wasm_bindgen_futures::spawn_local(async move {
                match crate::api::auth::register(em, pw, name).await {
                    Ok(()) => {
                        register_pending.set(false);
                        if let Some(w) = web_sys::window() {
                            let _ = w.location().set_href("/auth/verify");
                        }
                    }
                    Err(e) => {
                        register_pending.set(false);
                        error_msg.set(Some(e.to_string()));
                    }
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

            <div class="text-center">
                <div class="w-14 h-14 mx-auto mb-3 rounded-2xl flex items-center justify-center ring-1 ring-emerald-200/50"
                     style="background: linear-gradient(135deg, var(--color-bg-elevated), var(--color-bg-sunken))">
                    {icon_user_plus()}
                </div>
                <h1 class="text-2xl font-bold text-[var(--color-text-primary)] tracking-tight">{t("auth.create_account")}</h1>
                <p class="text-sm text-[var(--color-text-tertiary)] mt-1">{t("auth.get_started")}</p>
            </div>

            {move || error_msg.get().map(|msg| view! {
                <div class="px-3 py-2 rounded-xl"
                     style="background: var(--color-status-error-bg); border: 1px solid var(--color-status-error-bg)">
                    <p class="text-sm text-[var(--color-status-error)]">{msg}</p>
                </div>
            })}

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

                <div class="flex flex-col gap-1.5">
                    <div class="relative">
                        <input
                            type=move || if show_password.get() { "text" } else { "password" }
                            prop:value=move || signup_password.get()
                            on:input=move |ev| signup_password.set(event_target_value(&ev))
                            placeholder=t("auth.password")
                            class="w-full pr-10 px-4 py-3 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm font-medium text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:ring-2 focus:ring-[var(--color-brand-primary)]/50 focus:border-[var(--color-border-focus)] transition-all"
                        />
                        <button
                            type="button"
                            on:click=move |_| show_password.update(|v| *v = !*v)
                            class="absolute right-3.5 top-1/2 -translate-y-1/2 text-[var(--color-text-disabled)] hover:text-[var(--color-text-secondary)]"
                        >
                            {move || if show_password.get() { icon_eye_off().into_any() } else { icon_eye().into_any() }}
                        </button>
                    </div>

                    {move || {
                        let pw = signup_password.get();
                        if pw.is_empty() {
                            None
                        } else {
                            let score = pw_strength();
                            Some(view! {
                                <div class="flex items-center gap-2">
                                    <div class="flex gap-1 flex-1">
                                        {(1usize..=4).map(|level| {
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

                <div class="flex flex-col gap-1">
                    <div class="relative">
                        <input
                            type=move || if show_password2.get() { "text" } else { "password" }
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
                            on:click=move |_| show_password2.update(|v| *v = !*v)
                            class="absolute right-3.5 top-1/2 -translate-y-1/2 text-[var(--color-text-disabled)] hover:text-[var(--color-text-secondary)]"
                        >
                            {move || if show_password2.get() { icon_eye_off().into_any() } else { icon_eye().into_any() }}
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
                <a href="/auth" class="text-[var(--color-brand-text)] font-semibold hover:underline">
                    {t("auth.signin_link")}
                </a>
            </p>
        </div>
    }.into_any())
}
