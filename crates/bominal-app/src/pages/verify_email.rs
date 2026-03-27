//! Token-driven email verification page.

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::{api, i18n::t, pages::auth::format_server_error};

const CHECK_ICON: &str = r#"M229.66,77.66l-128,128a8,8,0,0,1-11.32,0l-56-56a8,8,0,0,1,11.32-11.32L96,188.69,218.34,66.34a8,8,0,0,1,11.32,11.32Z"#;
const X_ICON: &str = r#"M205.66,194.34a8,8,0,0,1-11.32,11.32L128,139.31,61.66,205.66A8,8,0,0,1,50.34,194.34L116.69,128,50.34,61.66A8,8,0,0,1,61.66,50.34L128,116.69l66.34-66.35a8,8,0,0,1,11.32,11.32L139.31,128Z"#;

#[component]
pub fn VerifyEmailPage() -> impl IntoView {
    let query = use_query_map();
    let token = move || query.get().get("token").unwrap_or_default().to_string();
    let missing_token_message = t("auth.missing_token").to_string();

    let result = Resource::new(token, move |token| {
        let missing_token_message = missing_token_message.clone();
        async move {
            if token.is_empty() {
                Err(missing_token_message)
            } else {
                api::verify_email(token)
                    .await
                    .map_err(|error| format_server_error(&error))
            }
        }
    });

    view! {
        <div class="flex min-h-screen items-center justify-center px-4">
            <div class="page-enter w-full max-w-sm">
                <div class="lg-glass-panel p-6 text-center">
                    <Suspense fallback=move || view! {
                        <p class="text-sm" style="color: var(--lg-text-tertiary);">{t("auth.verifying")}</p>
                    }>
                        {move || result.get().map(|result| match result {
                            Ok(()) => view! {
                                <div class="flex flex-col gap-4">
                                    <div
                                        class="mx-auto flex h-12 w-12 items-center justify-center rounded-full"
                                        style="background: var(--lg-success-bg); color: var(--lg-success);"
                                    >
                                        <svg
                                            xmlns="http://www.w3.org/2000/svg"
                                            width="24"
                                            height="24"
                                            viewBox="0 0 256 256"
                                            fill="currentColor"
                                        >
                                            <path d=CHECK_ICON />
                                        </svg>
                                    </div>
                                    <h1 class="text-xl font-semibold" style="color: var(--lg-text-primary);">
                                        {t("auth.email_verified")}
                                    </h1>
                                    <p class="text-sm" style="color: var(--lg-text-tertiary);">
                                        {t("auth.email_verified_desc")}
                                    </p>
                                    <a href="/auth/login" class="lg-btn-primary squish rounded-2xl px-5 py-3 text-sm">
                                        {t("auth.go_to_login")}
                                    </a>
                                </div>
                            }
                            .into_any(),
                            Err(error) => view! {
                                <div class="flex flex-col gap-4">
                                    <div
                                        class="mx-auto flex h-12 w-12 items-center justify-center rounded-full"
                                        style="background: var(--lg-error-bg); color: var(--lg-error);"
                                    >
                                        <svg
                                            xmlns="http://www.w3.org/2000/svg"
                                            width="24"
                                            height="24"
                                            viewBox="0 0 256 256"
                                            fill="currentColor"
                                        >
                                            <path d=X_ICON />
                                        </svg>
                                    </div>
                                    <h1 class="text-xl font-semibold" style="color: var(--lg-text-primary);">
                                        {t("auth.verify_failed")}
                                    </h1>
                                    <p class="text-sm" style="color: var(--lg-error);">{error}</p>
                                    <a href="/auth/login" class="lg-btn-secondary squish rounded-2xl px-5 py-3 text-sm">
                                        {t("auth.go_to_login")}
                                    </a>
                                </div>
                            }
                            .into_any(),
                        })}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}
