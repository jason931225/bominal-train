//! Authentication page — login and registration forms.
//!
//! Uses server functions via `ActionForm` for SSR-compatible form submission.
//! Error handling via URL query parameters (PRG pattern).

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::api::auth::{Login, Register};
use crate::i18n::t;

/// Auth page with login/register toggle.
#[component]
pub fn AuthPage() -> impl IntoView {
    let query = use_query_map();
    let error = move || query.get().get("error");
    let mode = move || query.get().get("mode").unwrap_or_default();
    let is_register = move || mode() == "register";

    let login_action = ServerAction::<Login>::new();
    let register_action = ServerAction::<Register>::new();

    let login_pending = login_action.pending();
    let register_pending = register_action.pending();
    let loading = move || login_pending.get() || register_pending.get();

    view! {
        <div class="min-h-screen flex flex-col items-center justify-center px-4">
            // Mesh gradient background
            <div class="fixed inset-0 -z-10 bg-[var(--color-bg-primary)]">
                <div class="absolute inset-0 opacity-30"
                     style="background: radial-gradient(ellipse at 20% 50%, var(--color-mesh-1) 0%, transparent 50%), radial-gradient(ellipse at 80% 20%, var(--color-mesh-2) 0%, transparent 50%), radial-gradient(ellipse at 50% 80%, var(--color-mesh-3) 0%, transparent 50%)">
                </div>
            </div>

            // Logo area
            <div class="mb-8 text-center">
                <h1 class="text-3xl font-bold text-[var(--color-brand-primary)]">"Bominal"</h1>
                <p class="text-sm text-[var(--color-text-secondary)] mt-1">{t("auth.login_subtitle")}</p>
            </div>

            // Glass card form
            <div class="glass-panel w-full max-w-sm p-6 rounded-2xl">
                // Toggle tabs
                <div class="flex mb-6 bg-[var(--color-bg-sunken)] rounded-xl p-1">
                    <a
                        href="/"
                        class=move || if !is_register() {
                            "flex-1 py-2 text-sm font-medium rounded-lg bg-[var(--color-bg-elevated)] text-[var(--color-text-primary)] shadow-sm transition-all text-center"
                        } else {
                            "flex-1 py-2 text-sm font-medium rounded-lg text-[var(--color-text-tertiary)] transition-all text-center"
                        }
                    >
                        {t("auth.login")}
                    </a>
                    <a
                        href="/?mode=register"
                        class=move || if is_register() {
                            "flex-1 py-2 text-sm font-medium rounded-lg bg-[var(--color-bg-elevated)] text-[var(--color-text-primary)] shadow-sm transition-all text-center"
                        } else {
                            "flex-1 py-2 text-sm font-medium rounded-lg text-[var(--color-text-tertiary)] transition-all text-center"
                        }
                    >
                        {t("auth.register")}
                    </a>
                </div>

                // Error message
                {move || error().map(|err| {
                    let msg = match err.as_str() {
                        "invalid" => t("error.login_failed"),
                        "email_exists" => t("auth.email_exists"),
                        _ => t("error.unexpected"),
                    };
                    view! {
                        <div class="mb-4 px-3 py-2 bg-[var(--color-status-error)]/10 border border-[var(--color-status-error)]/30 rounded-xl">
                            <p class="text-sm text-[var(--color-status-error)]">{msg}</p>
                        </div>
                    }
                })}

                // Login form
                <Show when=move || !is_register()>
                    <ActionForm action=login_action>
                        <div class="space-y-4">
                            <div>
                                <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1.5">{t("auth.email")}</label>
                                <input
                                    type="email"
                                    name="email"
                                    required
                                    class="w-full px-3 py-2.5 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors"
                                    placeholder="your@email.com"
                                />
                            </div>
                            <div>
                                <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1.5">{t("auth.password")}</label>
                                <input
                                    type="password"
                                    name="password"
                                    required
                                    class="w-full px-3 py-2.5 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors"
                                    placeholder="••••••••"
                                />
                            </div>
                        </div>
                        <button
                            type="submit"
                            class="w-full mt-6 py-2.5 bg-[var(--color-brand-primary)] text-white font-medium rounded-xl text-sm hover:opacity-90 disabled:opacity-50 transition-all"
                            disabled=loading
                        >
                            {move || if loading() { t("common.loading") } else { t("auth.login") }}
                        </button>
                    </ActionForm>
                </Show>

                // Register form
                <Show when=is_register>
                    <ActionForm action=register_action>
                        <div class="space-y-4">
                            <div>
                                <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1.5">{t("auth.display_name")}</label>
                                <input
                                    type="text"
                                    name="display_name"
                                    required
                                    class="w-full px-3 py-2.5 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors"
                                    placeholder="Your name"
                                />
                            </div>
                            <div>
                                <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1.5">{t("auth.email")}</label>
                                <input
                                    type="email"
                                    name="email"
                                    required
                                    class="w-full px-3 py-2.5 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors"
                                    placeholder="your@email.com"
                                />
                            </div>
                            <div>
                                <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1.5">{t("auth.password")}</label>
                                <input
                                    type="password"
                                    name="password"
                                    required
                                    minlength="8"
                                    class="w-full px-3 py-2.5 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors"
                                    placeholder="••••••••"
                                />
                            </div>
                        </div>
                        <button
                            type="submit"
                            class="w-full mt-6 py-2.5 bg-[var(--color-brand-primary)] text-white font-medium rounded-xl text-sm hover:opacity-90 disabled:opacity-50 transition-all"
                            disabled=loading
                        >
                            {move || if loading() { t("common.loading") } else { t("auth.register") }}
                        </button>
                    </ActionForm>
                </Show>
            </div>
        </div>
    }
}
