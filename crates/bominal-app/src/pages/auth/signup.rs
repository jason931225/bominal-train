//! Registration page (/auth/signup) — stub with full form layout.

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::{api, i18n::t};

use super::auth_shell;

/// Sign-up page with display name, email, and password fields.
#[component]
pub fn SignupPage() -> impl IntoView {
    let query = use_query_map();
    let register_action = ServerAction::<api::RegisterSubmit>::new();
    let error = move || query.get().get("error");

    auth_shell(view! {
        <div class="lg-glass-panel flex flex-col gap-5 p-6">
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

            {move || error().map(|message| {
                view! {
                    <p class="text-center text-sm" style="color: var(--lg-error);">{message}</p>
                }
            })}

            <ActionForm
                action=register_action
                attr:class="flex flex-col gap-5"
            >
                <div class="flex flex-col gap-3">
                    <input
                        type="text"
                        name="display_name"
                        placeholder=t("auth.display_name")
                        autocomplete="name"
                        class="lg-glass-card w-full rounded-xl px-4 py-3 text-sm outline-none"
                        style="color: var(--lg-text-primary); border-color: var(--lg-border-default);"
                        required
                    />

                    <input
                        type="email"
                        name="email"
                        placeholder=t("auth.email_placeholder")
                        autocomplete="email"
                        class="lg-glass-card w-full rounded-xl px-4 py-3 text-sm outline-none"
                        style="color: var(--lg-text-primary); border-color: var(--lg-border-default);"
                        required
                    />

                    <input
                        type="password"
                        name="password"
                        placeholder=t("auth.password")
                        autocomplete="new-password"
                        minlength="8"
                        class="lg-glass-card w-full rounded-xl px-4 py-3 text-sm outline-none"
                        style="color: var(--lg-text-primary); border-color: var(--lg-border-default);"
                        required
                    />
                </div>

                <button
                    type="submit"
                    class="lg-btn-primary squish w-full rounded-2xl px-6 py-3.5 text-base"
                >
                    {t("auth.create_account")}
                </button>
            </ActionForm>

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
    }
    .into_any())
}
