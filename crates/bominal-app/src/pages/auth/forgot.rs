//! Forgot password page (/auth/forgot) — stub with email input and reset link.

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::{api, i18n::t};

use super::auth_shell;

/// Phosphor Check icon.
const CHECK_ICON: &str = r#"M229.66,77.66l-128,128a8,8,0,0,1-11.32,0l-56-56a8,8,0,0,1,11.32-11.32L96,188.69,218.34,66.34a8,8,0,0,1,11.32,11.32Z"#;

/// Forgot password page with email input and success state.
#[component]
pub fn ForgotPage() -> impl IntoView {
    let query = use_query_map();
    let forgot_action = ServerAction::<api::ForgotPasswordSubmit>::new();
    let error = move || query.get().get("error");
    let sent = move || query.get().get("sent").is_some_and(|value| value == "1");

    auth_shell(view! {
        <div class="lg-glass-panel flex flex-col gap-5 p-6">
            <div class="text-center">
                <h1
                    class="text-2xl font-bold tracking-tight"
                    style="color: var(--lg-text-primary);"
                >
                    {t("auth.reset_password")}
                </h1>
                <p class="mt-1 text-sm" style="color: var(--lg-text-tertiary);">
                    {t("auth.reset_subtitle")}
                </p>
            </div>

            {move || error().map(|message| {
                view! {
                    <p class="text-center text-sm" style="color: var(--lg-error);">{message}</p>
                }
            })}

            {move || if sent() {
                view! {
                    <div class="flex flex-col items-center gap-3 py-4 text-center">
                        <div
                            class="flex h-12 w-12 items-center justify-center rounded-full"
                            style="background: var(--lg-success-bg);"
                        >
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                width="24"
                                height="24"
                                viewBox="0 0 256 256"
                                fill="currentColor"
                                style="color: var(--lg-success);"
                            >
                                <path d=CHECK_ICON />
                            </svg>
                        </div>
                        <p
                            class="text-sm font-medium"
                            style="color: var(--lg-text-primary);"
                        >
                            {t("auth.reset_link_sent")}
                        </p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <ActionForm
                        action=forgot_action
                        attr:class="flex flex-col gap-5"
                    >
                        <input
                            type="email"
                            name="email"
                            placeholder=t("auth.email_placeholder")
                            autocomplete="email"
                            class="lg-glass-card w-full rounded-xl px-4 py-3 text-sm outline-none"
                            style="color: var(--lg-text-primary); border-color: var(--lg-border-default);"
                            required
                        />

                        <button
                            type="submit"
                            class="lg-btn-primary squish w-full rounded-2xl px-6 py-3.5 text-base"
                        >
                            {t("auth.send_reset_link")}
                        </button>
                    </ActionForm>
                }.into_any()
            }}

            <div class="text-center">
                <a
                    href="/auth"
                    class="text-sm font-medium"
                    style="color: var(--lg-accent-text);"
                >
                    {t("auth.back_to_signin")}
                </a>
            </div>
        </div>
    }
    .into_any())
}
