//! Email verification page — extracts token from query params, verifies on load.

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::api::auth::verify_email;
use crate::components::glass_panel::GlassPanel;
use crate::i18n::t;

/// Standalone page for `/verify-email?token=...` links sent in verification emails.
///
/// Calls the `verify_email` server function on load and shows success or error.
#[component]
pub fn VerifyEmailPage() -> impl IntoView {
    let query = use_query_map();
    let token = move || query.get().get("token").unwrap_or_default().to_string();

    let result = Resource::new(
        token,
        |tok| async move {
            if tok.is_empty() {
                Err("missing_token".to_string())
            } else {
                verify_email(tok).await.map_err(|e| e.to_string())
            }
        },
    );

    view! {
        <div class="flex items-center justify-center min-h-screen px-4">
            <div class="w-full max-w-sm">
                <GlassPanel>
                    <div class="p-6 text-center space-y-4">
                        <Suspense fallback=move || view! {
                            <p class="text-sm text-[var(--color-text-tertiary)]">{t("auth.verifying")}</p>
                        }>
                            {move || result.get().map(|r| match r {
                                Ok(()) => view! {
                                    <div class="space-y-4">
                                        <div class="w-12 h-12 mx-auto rounded-full bg-[var(--color-status-success)]/20 flex items-center justify-center">
                                            <svg class="w-6 h-6 text-[var(--color-status-success)]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                                <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7"/>
                                            </svg>
                                        </div>
                                        <h2 class="text-lg font-semibold text-[var(--color-text-primary)]">{t("auth.email_verified")}</h2>
                                        <p class="text-sm text-[var(--color-text-tertiary)]">{t("auth.email_verified_desc")}</p>
                                        <a href="/" class="inline-block text-sm text-[var(--color-brand-text)] font-medium hover:underline">
                                            {t("auth.go_to_login")}
                                        </a>
                                    </div>
                                }.into_any(),
                                Err(e) => view! {
                                    <div class="space-y-4">
                                        <div class="w-12 h-12 mx-auto rounded-full bg-[var(--color-status-error)]/20 flex items-center justify-center">
                                            <svg class="w-6 h-6 text-[var(--color-status-error)]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12"/>
                                            </svg>
                                        </div>
                                        <h2 class="text-lg font-semibold text-[var(--color-text-primary)]">{t("auth.verify_failed")}</h2>
                                        <p class="text-sm text-[var(--color-status-error)]">
                                            {if e == "missing_token" { t("auth.missing_token").to_string() } else { e }}
                                        </p>
                                        <a href="/" class="inline-block text-sm text-[var(--color-brand-text)] font-medium hover:underline">
                                            {t("auth.go_to_login")}
                                        </a>
                                    </div>
                                }.into_any(),
                            })}
                        </Suspense>
                    </div>
                </GlassPanel>
            </div>
        </div>
    }
}
