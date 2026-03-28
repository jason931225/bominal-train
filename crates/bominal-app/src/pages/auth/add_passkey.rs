//! Passkey upsell page before the live WebAuthn ceremony is wired.

use leptos::prelude::*;

use crate::i18n::t;

use super::{PasskeyActionButton, PasskeyActionKind, auth_shell};

const KEY_ICON: &str = r#"M168,80a40,40,0,1,0-35.77,57.89L112,158.06V184a8,8,0,0,1-8,8H88v16a8,8,0,0,1-8,8H64v24a8,8,0,0,1-16,0V208a8,8,0,0,1,8-8H72V184a8,8,0,0,1,8-8H96V154.75a8,8,0,0,1,2.34-5.66l22.57-22.57A40,40,0,0,0,168,80Zm0,24a16,16,0,1,1,16-16A16,16,0,0,1,168,104Z"#;

#[component]
pub fn AddPasskeyPage() -> impl IntoView {
    auth_shell(
        view! {
            <div class="lg-glass-panel flex flex-col gap-6 p-6">
                <div class="text-center">
                    <div
                        class="mx-auto mb-4 flex h-20 w-20 items-center justify-center rounded-2xl"
                        style="background: var(--lg-accent-bg); color: var(--lg-accent-text);"
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="40"
                            height="40"
                            viewBox="0 0 256 256"
                            fill="currentColor"
                            aria-hidden="true"
                        >
                            <path d=KEY_ICON />
                        </svg>
                    </div>
                    <h1
                        class="text-2xl font-bold tracking-tight"
                        style="color: var(--lg-text-primary);"
                    >
                        {t("auth.add_passkey")}
                    </h1>
                    <p class="mt-2 text-sm leading-relaxed" style="color: var(--lg-text-tertiary);">
                        {t("auth.passkey_subtitle")}
                    </p>
                </div>

                <div class="lg-glass-card flex flex-col gap-3 rounded-2xl px-4 py-4">
                    <p class="text-sm" style="color: var(--lg-text-secondary);">
                        "1. " {t("auth.passkey_benefit_1")}
                    </p>
                    <p class="text-sm" style="color: var(--lg-text-secondary);">
                        "2. " {t("auth.passkey_benefit_2")}
                    </p>
                    <p class="text-sm" style="color: var(--lg-text-secondary);">
                        "3. " {t("auth.passkey_benefit_3")}
                    </p>
                </div>

                <PasskeyActionButton
                    kind=PasskeyActionKind::Register
                    class_name="lg-btn-primary squish w-full rounded-2xl px-6 py-3.5 text-base".to_string()
                    label=t("auth.add_passkey_now").to_string()
                    loading_label=t("common.loading").to_string()
                    error_fallback="Passkey registration failed".to_string()
                    show_icon=false
                />

                <a
                    href="/home"
                    class="block w-full py-2 text-center text-sm font-medium"
                    style="color: var(--lg-text-tertiary);"
                >
                    {t("auth.skip_for_now")}
                </a>
            </div>
        }
        .into_any(),
    )
}
