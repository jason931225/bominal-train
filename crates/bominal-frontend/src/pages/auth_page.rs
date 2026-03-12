//! Authentication page — login and registration forms with passkey support.
//!
//! Uses server functions via `ActionForm` for SSR-compatible form submission.
//! Error handling via URL query parameters (PRG pattern).

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::api::auth::{Login, Register};
use crate::i18n::t;

/// Auth page with login/register toggle and passkey sign-in.
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
        <div class="min-h-screen flex flex-col items-center justify-center px-4 page-enter">
            // Background blobs
            <div class="absolute top-1/4 -left-32 w-96 h-96 rounded-full blur-[120px] pointer-events-none"
                 style="background: var(--color-brand-primary)"></div>
            <div class="absolute bottom-1/4 -right-32 w-96 h-96 rounded-full blur-[120px] pointer-events-none"
                 style="background: var(--color-mesh-2)"></div>

            // Logo
            <div class="mb-8 text-center flex flex-col items-center">
                <h1 class="app-brand-wordmark text-3xl font-bold bg-clip-text text-transparent tracking-tight">"Bominal"</h1>
                <p class="text-sm text-[var(--color-text-tertiary)] mt-1">{t("auth.login_subtitle")}</p>
            </div>

            // Glass card form
            <div class="glass-panel w-full max-w-sm p-6 rounded-3xl">
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
                        <div class="mb-4 px-3 py-2 rounded-xl"
                             style="background: var(--color-status-error-bg); border: 1px solid var(--color-status-error-bg)">
                            <p class="text-sm text-[var(--color-status-error)]">{msg}</p>
                        </div>
                    }
                })}

                // Login form
                <Show when=move || !is_register()>
                    <ActionForm action=login_action>
                        <div class="space-y-4">
                            // Email field with icon
                            <div>
                                <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1.5">{t("auth.email")}</label>
                                <div class="relative">
                                    <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-[var(--color-text-disabled)]" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                        <path stroke-linecap="round" stroke-linejoin="round" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                                    </svg>
                                    <input
                                        type="email"
                                        name="email"
                                        required
                                        class="w-full pl-10 pr-3 py-2.5 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors"
                                        placeholder="your@email.com"
                                    />
                                </div>
                            </div>
                            // Password field with icon
                            <div>
                                <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1.5">{t("auth.password")}</label>
                                <div class="relative">
                                    <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-[var(--color-text-disabled)]" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                        <path stroke-linecap="round" stroke-linejoin="round" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                                    </svg>
                                    <input
                                        type="password"
                                        name="password"
                                        required
                                        class="w-full pl-10 pr-3 py-2.5 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors"
                                        placeholder="••••••••"
                                    />
                                </div>
                            </div>
                        </div>
                        <button
                            type="submit"
                            class="w-full mt-6 py-2.5 btn-glass font-medium rounded-xl text-sm disabled:opacity-50 transition-all"
                            disabled=loading
                        >
                            {move || if loading() { t("common.loading") } else { t("auth.login") }}
                        </button>
                    </ActionForm>

                    // Divider
                    <div class="flex items-center gap-2 my-4">
                        <div class="flex-1 h-px bg-[var(--color-border-subtle)]"></div>
                        <span class="text-[10px] text-[var(--color-text-disabled)] font-medium">"or"</span>
                        <div class="flex-1 h-px bg-[var(--color-border-subtle)]"></div>
                    </div>

                    // Passkey sign-in button
                    <button
                        class="w-full py-2.5 bg-[var(--color-bg-elevated)] border border-[var(--color-border-default)] text-[var(--color-text-primary)] font-medium rounded-xl text-sm flex items-center justify-center gap-2 hover:bg-[var(--color-interactive-hover)] active:scale-95 transition-all"
                        on:click=move |_| {
                            #[cfg(target_arch = "wasm32")]
                            {
                                use wasm_bindgen::prelude::*;
                                if let Some(window) = web_sys::window() {
                                    if let Ok(func) = js_sys::Reflect::get(&window, &JsValue::from_str("__startPasskeyLogin")) {
                                        if let Some(f) = func.dyn_ref::<js_sys::Function>() {
                                            let _ = f.call1(&JsValue::NULL, &JsValue::from_str("{}"));
                                        }
                                    }
                                }
                            }
                        }
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
                        </svg>
                        {t("auth.passkey_signin")}
                    </button>
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
                            class="w-full mt-6 py-2.5 btn-glass font-medium rounded-xl text-sm disabled:opacity-50 transition-all"
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
