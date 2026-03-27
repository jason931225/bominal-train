//! Token-driven password reset page.

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::{api, i18n::t, pages::auth::format_server_error};

#[component]
pub fn ResetPasswordPage() -> impl IntoView {
    let query = use_query_map();
    let token = move || query.get().get("token").unwrap_or_default().to_string();
    let reset_action = ServerAction::<api::ResetPassword>::new();
    let (new_password, set_new_password) = signal(String::new());
    let (confirm_password, set_confirm_password) = signal(String::new());

    let passwords_match = move || {
        let password = new_password.get();
        let confirm = confirm_password.get();
        !password.is_empty() && !confirm.is_empty() && password == confirm
    };

    let passwords_mismatch = move || {
        let confirm = confirm_password.get();
        !confirm.is_empty() && new_password.get() != confirm
    };

    let reset_succeeded = move || {
        reset_action
            .value()
            .get()
            .is_some_and(|result| result.is_ok())
    };

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

                    {move || {
                        reset_action
                            .value()
                            .get()
                            .and_then(|result| result.err())
                            .map(|error| {
                                view! {
                                    <p class="text-sm" style="color: var(--lg-error);">
                                        {format_server_error(&error)}
                                    </p>
                                }
                            })
                    }}

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
                                    on:submit=move |ev| {
                                        if passwords_mismatch() {
                                            ev.prevent_default();
                                        }
                                    }
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
                                        prop:value=move || new_password.get()
                                        on:input=move |ev| set_new_password.set(event_target_value(&ev))
                                        class="lg-glass-card w-full rounded-xl px-4 py-3 text-sm outline-none"
                                        style="color: var(--lg-text-primary); border-color: var(--lg-border-default);"
                                    />
                                    <input
                                        type="password"
                                        required
                                        minlength="8"
                                        aria-label=t("auth.confirm_password")
                                        placeholder=t("auth.confirm_password")
                                        prop:value=move || confirm_password.get()
                                        on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                                        class=move || {
                                            if passwords_mismatch() {
                                                "lg-glass-card w-full rounded-xl border px-4 py-3 text-sm outline-none".to_string()
                                            } else {
                                                "lg-glass-card w-full rounded-xl px-4 py-3 text-sm outline-none".to_string()
                                            }
                                        }
                                        style=move || {
                                            if passwords_mismatch() {
                                                "color: var(--lg-text-primary); border-color: var(--lg-error);".to_string()
                                            } else if passwords_match() {
                                                "color: var(--lg-text-primary); border-color: var(--lg-success);".to_string()
                                            } else {
                                                "color: var(--lg-text-primary); border-color: var(--lg-border-default);".to_string()
                                            }
                                        }
                                    />
                                    {move || if passwords_mismatch() {
                                        Some(view! {
                                            <p class="text-xs font-medium" style="color: var(--lg-error);">
                                                {t("auth.passwords_mismatch")}
                                            </p>
                                        })
                                    } else if passwords_match() {
                                        Some(view! {
                                            <p class="text-xs font-medium" style="color: var(--lg-success);">
                                                {t("auth.passwords_match")}
                                            </p>
                                        })
                                    } else {
                                        None
                                    }}
                                    <button
                                        type="submit"
                                        class="lg-btn-primary squish w-full rounded-2xl px-6 py-3.5 text-base"
                                        disabled=move || reset_action.pending().get() || passwords_mismatch()
                                    >
                                        {move || if reset_action.pending().get() {
                                            t("common.loading")
                                        } else {
                                            t("auth.reset_password")
                                        }}
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
