//! Registration page (/auth/signup) — stub with full form layout.

use leptos::prelude::*;

use crate::i18n::t;

/// Compute password strength score and label key.
fn password_strength(pw: &str) -> (u32, &'static str, &'static str) {
    if pw.is_empty() {
        return (0, "", "");
    }
    let mut score = 0u32;
    if pw.len() >= 8 {
        score += 1;
    }
    if pw.len() >= 12 {
        score += 1;
    }
    let has_lower = pw.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = pw.chars().any(|c| c.is_ascii_uppercase());
    if has_lower && has_upper {
        score += 1;
    }
    if pw.chars().any(|c| c.is_ascii_digit()) {
        score += 1;
    }
    if pw.chars().any(|c| !c.is_alphanumeric()) {
        score += 1;
    }
    match score {
        0..=1 => (20, "auth.pw_weak", "var(--lg-error)"),
        2 => (40, "auth.pw_fair", "var(--lg-warning)"),
        3 => (65, "auth.pw_good", "var(--lg-warning)"),
        _ => (100, "auth.pw_strong", "var(--lg-success)"),
    }
}

/// Sign-up page with display name, email, and password fields.
#[component]
pub fn SignupPage() -> impl IntoView {
    let (display_name, set_display_name) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (show_password, set_show_password) = signal(false);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(String::new());

    let pw_strength = Memo::new(move |_| password_strength(&password.get()));

    let is_email_valid = Memo::new(move |_| {
        let e = email.get();
        e.contains('@') && e.contains('.')
    });

    let is_form_valid = Memo::new(move |_| {
        is_email_valid.get()
            && password.get().len() >= 8
            && !display_name.get().trim().is_empty()
    });

    let handle_signup = move |_| {
        set_loading.set(true);
        set_error.set(String::new());
        // Registration will be wired when backend integration is connected.
        set_loading.set(false);
    };

    view! {
        <div class="flex min-h-screen items-center justify-center px-4">
            <div class="page-enter w-full max-w-sm">
                <div class="lg-glass-panel flex flex-col gap-5 p-6">
                    // Header
                    <div class="text-center">
                        <h1
                            class="text-2xl font-bold tracking-tight"
                            style="color: var(--lg-text-primary);"
                        >
                            {t("auth.create_account")}
                        </h1>
                        <p class="mt-1 text-sm" style="color: var(--lg-text-tertiary);">
                            {t("auth.get_started")}
                        </p>
                    </div>

                    // Error
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

                    // Form fields
                    <div class="flex flex-col gap-3">
                        <input
                            type="text"
                            placeholder=t("auth.display_name")
                            autocomplete="name"
                            class="lg-glass-card w-full rounded-xl px-4 py-3 text-sm outline-none"
                            style="color: var(--lg-text-primary); border-color: var(--lg-border-default);"
                            prop:value=move || display_name.get()
                            on:input=move |ev| set_display_name.set(event_target_value(&ev))
                        />

                        <input
                            type="email"
                            placeholder=t("auth.email_placeholder")
                            autocomplete="email"
                            class="lg-glass-card w-full rounded-xl px-4 py-3 text-sm outline-none"
                            style="color: var(--lg-text-primary); border-color: var(--lg-border-default);"
                            prop:value=move || email.get()
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                        />

                        <div class="relative">
                            <input
                                type=move || if show_password.get() { "text" } else { "password" }
                                placeholder=t("auth.password")
                                autocomplete="new-password"
                                class="lg-glass-card w-full rounded-xl px-4 py-3 pr-12 text-sm outline-none"
                                style="color: var(--lg-text-primary); border-color: var(--lg-border-default);"
                                prop:value=move || password.get()
                                on:input=move |ev| set_password.set(event_target_value(&ev))
                            />
                            <button
                                type="button"
                                class="absolute right-3 top-1/2 -translate-y-1/2 text-xs"
                                style="color: var(--lg-text-tertiary);"
                                on:click=move |_| set_show_password.update(|v| *v = !*v)
                                aria-label=move || if show_password.get() { t("auth.hide_password") } else { t("auth.show_password") }
                            >
                                {move || if show_password.get() { t("auth.hide_password") } else { t("auth.show_password") }}
                            </button>
                        </div>

                        // Password strength meter
                        {move || {
                            let pw = password.get();
                            if pw.is_empty() {
                                None
                            } else {
                                let (score, label_key, color) = pw_strength.get();
                                Some(view! {
                                    <div class="flex flex-col gap-1.5">
                                        <div
                                            class="h-1.5 w-full overflow-hidden rounded-full"
                                            style="background: var(--lg-bg-sunken);"
                                        >
                                            <div
                                                class="h-full rounded-full transition-all duration-300"
                                                style=format!("width: {}%; background: {};", score, color)
                                            ></div>
                                        </div>
                                        <span
                                            class="text-xs font-medium"
                                            style=format!("color: {};", color)
                                        >
                                            {t(label_key)}
                                        </span>
                                    </div>
                                })
                            }
                        }}
                    </div>

                    // Submit button
                    <button
                        class="lg-btn-primary squish w-full rounded-2xl px-6 py-3.5 text-base"
                        disabled=move || loading.get() || !is_form_valid.get()
                        on:click=handle_signup
                    >
                        {move || if loading.get() { t("common.loading") } else { t("auth.create_account") }}
                    </button>

                    // Sign in link
                    <div class="text-center text-sm" style="color: var(--lg-text-tertiary);">
                        {t("auth.has_account")}
                        " "
                        <a
                            href="/auth/login"
                            class="font-medium underline"
                            style="color: var(--lg-accent-text);"
                        >
                            {t("auth.signin_link")}
                        </a>
                    </div>
                </div>
            </div>
        </div>
    }
}
