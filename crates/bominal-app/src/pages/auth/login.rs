//! Email/password login page (/auth/login).

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::{api, i18n::t};

use super::{ConditionalPasskeyLogin, PasskeyActionButton, PasskeyActionKind, auth_shell};

/// Login page with email/password form.
#[component]
pub fn LoginPage() -> impl IntoView {
    let query = use_query_map();
    let login_action = ServerAction::<api::LoginSubmit>::new();
    let error = move || query.get().get("error");

    auth_shell(view! {
        <ConditionalPasskeyLogin />
        <div class="lg-glass-panel flex flex-col gap-5 p-6">
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

            {move || error().map(|message| {
                view! {
                    <p class="text-center text-sm" style="color: var(--lg-error);">{message}</p>
                }
            })}

            <ActionForm
                action=login_action
                attr:class="flex flex-col gap-5"
            >
                <div class="flex flex-col gap-3">
                    <input
                        type="email"
                        name="email"
                        placeholder=t("auth.email_placeholder")
                        autocomplete="username webauthn"
                        class="lg-glass-card w-full rounded-xl px-4 py-3 text-sm outline-none"
                        style="color: var(--lg-text-primary); border-color: var(--lg-border-default);"
                        required
                    />

                    <input
                        type="password"
                        name="password"
                        placeholder=t("auth.password")
                        autocomplete="current-password"
                        class="lg-glass-card w-full rounded-xl px-4 py-3 text-sm outline-none"
                        style="color: var(--lg-text-primary); border-color: var(--lg-border-default);"
                        required
                    />
                </div>

                <button
                    type="submit"
                    class="lg-btn-primary squish w-full rounded-2xl px-6 py-3.5 text-base"
                >
                    {t("auth.sign_in")}
                </button>
            </ActionForm>

            <div class="relative">
                <div class="absolute inset-0 flex items-center">
                    <div class="w-full" style="border-top: 1px solid var(--lg-border-default);"></div>
                </div>
            </div>

            <a
                href="/auth/forgot"
                class="lg-btn-secondary squish block w-full rounded-2xl px-6 py-3 text-center text-sm"
            >
                {t("auth.forgot_password")}
            </a>

            <a
                href="/auth/signup"
                class="lg-btn-secondary squish block w-full rounded-2xl px-6 py-3 text-center text-sm"
            >
                {t("auth.signup_link")}
            </a>

            <PasskeyActionButton
                kind=PasskeyActionKind::Login
                class_name="lg-btn-secondary squish block w-full rounded-2xl px-6 py-3 text-center text-sm".to_string()
                label=t("auth.use_passkey").to_string()
                loading_label=t("common.loading").to_string()
                error_fallback="Passkey login failed".to_string()
                show_icon=false
            />
        </div>
    }
    .into_any())
}
