//! Email/password login page (/auth/login).

use leptos::prelude::*;

use crate::i18n::t;

/// Phosphor Eye icon.
const EYE_ICON: &str = r#"M247.31,124.76c-.35-.79-8.82-19.58-27.65-38.41C194.57,61.26,162.88,48,128,48S61.43,61.26,36.34,86.35C17.51,105.18,9,124,8.69,124.76a8,8,0,0,0,0,6.5c.35.79,8.82,19.57,27.65,38.4C61.43,194.74,93.12,208,128,208s66.57-13.26,91.66-38.34c18.83-18.83,27.3-37.61,27.65-38.4A8,8,0,0,0,247.31,124.76ZM128,192c-30.78,0-57.67-11.19-79.93-33.29A133.47,133.47,0,0,1,25,128a133.33,133.33,0,0,1,23.07-30.71C70.33,75.19,97.22,64,128,64s57.67,11.19,79.93,33.29A133.46,133.46,0,0,1,231,128C226.94,135.84,195.17,192,128,192Zm0-112a48,48,0,1,0,48,48A48.05,48.05,0,0,0,128,80Zm0,80a32,32,0,1,1,32-32A32,32,0,0,1,128,160Z"#;

/// Phosphor EyeSlash icon.
const EYE_SLASH_ICON: &str = r#"M53.92,34.62A8,8,0,1,0,42.08,45.38L61.32,66.55C25,88.84,9.38,123.2,8.69,124.76a8,8,0,0,0,0,6.5c.35.79,8.82,19.57,27.65,38.4C61.43,194.74,93.12,208,128,208a127.11,127.11,0,0,0,52.07-10.83l22,24.21a8,8,0,1,0,11.84-10.76Zm47.33,75.8,41.67,45.85a32,32,0,0,1-41.67-45.85ZM128,192c-30.78,0-57.67-11.19-79.93-33.29A133.47,133.47,0,0,1,25,128c4.69-8.79,19.66-33.39,47.35-49.38l18,19.75a48,48,0,0,0,63.66,70l14.73,16.2A112,112,0,0,1,128,192Zm6-95.43a8,8,0,0,1,3-15.72,48.16,48.16,0,0,1,38.77,42.64,8,8,0,0,1-7.22,8.71,6.39,6.39,0,0,1-.75,0,8,8,0,0,1-8-7.26A32.09,32.09,0,0,0,134,96.57Zm113.28,34.69c-.42.94-10.55,23.37-33.36,43.8a8,8,0,1,1-10.67-11.92A132.77,132.77,0,0,0,231,128a133.15,133.15,0,0,0-23.07-30.71C185.67,75.19,158.78,64,128,64a118.37,118.37,0,0,0-19.36,1.57A8,8,0,1,1,106,49.79,134,134,0,0,1,128,48c34.88,0,66.57,13.26,91.66,38.35,18.83,18.83,27.3,37.62,27.65,38.41A8,8,0,0,1,247.31,131.26Z"#;

/// Login page with email/password form.
#[component]
pub fn LoginPage() -> impl IntoView {
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (show_password, set_show_password) = signal(false);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(String::new());

    let form_valid = Memo::new(move |_| !email.get().is_empty() && !password.get().is_empty());

    let handle_login = move |_| {
        set_loading.set(true);
        set_error.set(String::new());
        // Login will be wired when backend integration is connected.
        // For now the button shows loading state.
        set_loading.set(false);
    };

    view! {
        <div class="flex min-h-screen items-center justify-center px-4 pt-12">
            <div class="page-enter w-full max-w-sm">
                <div class="lg-glass-panel flex flex-col gap-5 p-6">
                    // Header
                    <div class="text-center">
                        <h1
                            class="text-2xl font-bold tracking-tight"
                            style="color: var(--lg-text-primary);"
                        >
                            {t("auth.welcome_back")}
                        </h1>
                        <p class="mt-1 text-sm" style="color: var(--lg-text-tertiary);">
                            {t("auth.enter_email_password")}
                        </p>
                    </div>

                    // Error message
                    {move || {
                        let err = error.get();
                        if err.is_empty() {
                            None
                        } else {
                            Some(view! {
                                <p class="text-center text-sm" style="color: var(--lg-error);">{err}</p>
                            })
                        }
                    }}

                    // Input fields
                    <div class="flex flex-col gap-3">
                        <input
                            type="email"
                            placeholder=t("auth.email_placeholder")
                            autocomplete="username"
                            class="lg-glass-card w-full rounded-xl px-4 py-3 text-sm outline-none"
                            style="color: var(--lg-text-primary); border-color: var(--lg-border-default);"
                            prop:value=move || email.get()
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                        />

                        <div class="relative">
                            <input
                                type=move || if show_password.get() { "text" } else { "password" }
                                placeholder=t("auth.password")
                                autocomplete="current-password"
                                class="lg-glass-card w-full rounded-xl px-4 py-3 pr-12 text-sm outline-none"
                                style="color: var(--lg-text-primary); border-color: var(--lg-border-default);"
                                prop:value=move || password.get()
                                on:input=move |ev| set_password.set(event_target_value(&ev))
                            />
                            <button
                                type="button"
                                class="absolute right-3 top-1/2 -translate-y-1/2 p-1 rounded"
                                style="color: var(--lg-text-tertiary);"
                                on:click=move |_| set_show_password.update(|v| *v = !*v)
                                aria-label=move || if show_password.get() { t("auth.hide_password") } else { t("auth.show_password") }
                            >
                                {move || if show_password.get() {
                                    view! {
                                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 256 256" fill="currentColor">
                                            <path d=EYE_SLASH_ICON />
                                        </svg>
                                    }.into_any()
                                } else {
                                    view! {
                                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 256 256" fill="currentColor">
                                            <path d=EYE_ICON />
                                        </svg>
                                    }.into_any()
                                }}
                            </button>
                        </div>
                    </div>

                    // Login button
                    <button
                        class="lg-btn-primary squish w-full rounded-2xl px-6 py-3.5 text-base"
                        disabled=move || loading.get() || !form_valid.get()
                        on:click=handle_login
                    >
                        {move || if loading.get() { t("common.loading") } else { t("auth.sign_in") }}
                    </button>

                    // Separator
                    <div class="relative">
                        <div class="absolute inset-0 flex items-center">
                            <div class="w-full" style="border-top: 1px solid var(--lg-border-default);"></div>
                        </div>
                    </div>

                    // Forgot password button
                    <a
                        href="/auth/forgot"
                        class="lg-btn-secondary squish block w-full rounded-2xl px-6 py-3 text-center text-sm"
                    >
                        {t("auth.forgot_password")}
                    </a>

                    // Sign up button
                    <a
                        href="/auth/signup"
                        class="lg-btn-secondary squish block w-full rounded-2xl px-6 py-3 text-center text-sm"
                    >
                        {t("auth.signup_link")}
                    </a>
                </div>
            </div>
        </div>
    }
}
