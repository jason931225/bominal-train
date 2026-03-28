//! Token-driven password reset page.

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::{api, i18n::t};

#[component]
pub fn ResetPasswordPage() -> impl IntoView {
    let query = use_query_map();
    let token = move || query.get().get("token").unwrap_or_default().to_string();
    let reset_action = ServerAction::<api::ResetPasswordSubmit>::new();
    let error = move || query.get().get("error");
    let reset_succeeded = move || query.get().get("done").is_some_and(|value| value == "1");

    view! {
        <div class="flex min-h-screen items-center justify-center px-4">
            <div class="page-enter w-full max-w-sm">
                <div class="lg-glass-panel flex flex-col gap-4 p-6">
                    <h1
                        class="text-center text-2xl font-bold tracking-tight"
                        style="color: var(--lg-text-primary);"
                    >
                        {t("auth.reset_password")}
                    </h1>

                    {move || error().map(|message| {
                        view! {
                            <p class="text-sm" style="color: var(--lg-error);">
                                {message}
                            </p>
                        }
                    })}

                    {move || {
                        if reset_succeeded() {
                            Some(view! {
                                <div class="flex flex-col gap-3 text-center">
                                    <p class="text-sm font-medium" style="color: var(--lg-success);">
                                        {t("auth.password_reset_success")}
                                    </p>
                                    <a href="/auth/login" class="lg-btn-primary squish rounded-2xl px-5 py-3 text-sm">
                                        {t("auth.go_to_login")}
                                    </a>
                                </div>
                            })
                        } else {
                            None
                        }
                    }}

                    {move || {
                        let current_token = token();
                        if current_token.is_empty() {
                            Some(view! {
                                <p class="text-center text-sm" style="color: var(--lg-error);">
                                    {t("auth.missing_token")}
                                </p>
                            }.into_any())
                        } else if reset_succeeded() {
                            None
                        } else {
                            Some(view! {
                                <ActionForm
                                    action=reset_action
                                    attr:class="flex flex-col gap-3"
                                >
                                    <input type="hidden" name="token" value=current_token />
                                    <input
                                        type="password"
                                        name="new_password"
                                        required
                                        minlength="8"
                                        aria-label=t("auth.new_password")
                                        placeholder=t("auth.new_password")
                                        class="lg-glass-card w-full rounded-xl px-4 py-3 text-sm outline-none"
                                        style="color: var(--lg-text-primary); border-color: var(--lg-border-default);"
                                    />
                                    <input
                                        type="password"
                                        name="confirm_password"
                                        required
                                        minlength="8"
                                        aria-label=t("auth.confirm_password")
                                        placeholder=t("auth.confirm_password")
                                        class="lg-glass-card w-full rounded-xl px-4 py-3 text-sm outline-none"
                                        style="color: var(--lg-text-primary); border-color: var(--lg-border-default);"
                                    />
                                    <button
                                        type="submit"
                                        class="lg-btn-primary squish w-full rounded-2xl px-6 py-3.5 text-base"
                                    >
                                        {t("auth.reset_password")}
                                    </button>
                                </ActionForm>
                            }.into_any())
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
