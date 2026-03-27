//! Post-signup verification handoff page.

use leptos::prelude::*;

use crate::{api, i18n::t, state::use_auth_state};

use super::auth_shell;

const MAIL_ICON: &str = r#"M224,56v120a16,16,0,0,1-16,16H48a16,16,0,0,1-16-16V56A16,16,0,0,1,48,40H208A16,16,0,0,1,224,56Zm-16.6.11-78.06,58.55a4,4,0,0,1-4.68,0L48.6,56.11A8,8,0,0,0,48,56v120H208V56A8,8,0,0,0,207.4,56.11ZM195.2,56H60.8L128,106.4Z"#;

#[component]
pub fn AuthVerifyPage() -> impl IntoView {
    let auth = use_auth_state();
    let resend_action = ServerAction::<api::ResendVerification>::new();
    let resend_pending = resend_action.pending();

    let email = move || {
        auth.user
            .get()
            .map(|user| user.email)
            .unwrap_or_else(|| "your email".to_string())
    };

    auth_shell(
        view! {
            <div class="lg-glass-panel flex flex-col gap-6 p-6">
                <div class="text-center">
                    <div
                        class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-2xl"
                        style="background: var(--lg-accent-bg); color: var(--lg-accent-text);"
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="28"
                            height="28"
                            viewBox="0 0 256 256"
                            fill="currentColor"
                            aria-hidden="true"
                        >
                            <path d=MAIL_ICON />
                        </svg>
                    </div>
                    <h1
                        class="text-2xl font-bold tracking-tight"
                        style="color: var(--lg-text-primary);"
                    >
                        {t("auth.check_email")}
                    </h1>
                    <p
                        class="mt-2 text-sm leading-relaxed"
                        style="color: var(--lg-text-tertiary);"
                    >
                        {t("auth.verify_sent_to")}
                        " "
                        <span style="color: var(--lg-text-primary);" class="font-semibold">
                            {email}
                        </span>
                        ". "
                        {t("auth.verify_click_link")}
                    </p>
                </div>

                <div
                    class="rounded-2xl border px-4 py-3 text-center text-sm font-medium"
                    style="border-color: var(--lg-warning); background: var(--lg-warning-bg); color: var(--lg-warning);"
                >
                    {t("auth.resend_prompt")}
                    " "
                    <ActionForm action=resend_action attr:class="inline">
                        <button
                            type="submit"
                            class="font-semibold underline"
                            disabled=resend_pending
                        >
                            {move || if resend_pending.get() { t("common.loading") } else { t("auth.resend_link") }}
                        </button>
                    </ActionForm>
                </div>

                <a
                    href="/auth/add-passkey"
                    class="lg-btn-primary squish block w-full rounded-2xl px-6 py-3.5 text-center text-base"
                >
                    {t("auth.verified_continue")}
                </a>

                <a
                    href="/auth/signup"
                    class="block w-full py-2 text-center text-sm font-medium"
                    style="color: var(--lg-text-tertiary);"
                >
                    {t("auth.back_to_signup")}
                </a>
            </div>
        }
        .into_any(),
    )
}
