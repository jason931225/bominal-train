//! Password reset page — shows new password form for `/reset-password?token=...` links.

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::api::auth::ResetPassword;
use crate::components::glass_panel::GlassPanel;
use crate::i18n::t;

/// Standalone page for `/reset-password?token=...` links sent in password reset emails.
///
/// Shows a new-password form. Submits via ActionForm → reset_password server function.
#[component]
pub fn ResetPasswordPage() -> impl IntoView {
    let query = use_query_map();
    let token = move || query.get().get("token").unwrap_or_default().to_string();
    let reset_action = ServerAction::<ResetPassword>::new();

    let succeeded = move || {
        reset_action
            .value()
            .get()
            .is_some_and(|r: Result<(), _>| r.is_ok())
    };

    view! {
        <div class="flex items-center justify-center min-h-screen px-4">
            <div class="w-full max-w-sm">
                <GlassPanel>
                    <div class="p-6 space-y-4">
                        <h2 class="text-lg font-semibold text-center text-[var(--color-text-primary)]">
                            {t("auth.reset_password")}
                        </h2>

                        // Error feedback
                        {move || reset_action.value().get().and_then(|r| r.err()).map(|e| view! {
                            <p class="text-sm text-[var(--color-status-error)]">{format!("{e}")}</p>
                        })}

                        // Success state
                        {move || succeeded().then(|| view! {
                            <div class="text-center space-y-3">
                                <p class="text-sm text-[var(--color-status-success)]">{t("auth.password_reset_success")}</p>
                                <a href="/" class="inline-block text-sm text-[var(--color-brand-text)] font-medium hover:underline">
                                    {t("auth.go_to_login")}
                                </a>
                            </div>
                        })}

                        // Missing token state
                        {move || token().is_empty().then(|| view! {
                            <p class="text-sm text-[var(--color-status-error)] text-center">{t("auth.missing_token")}</p>
                        })}

                        // Reset form — hidden after success or when token missing
                        {move || {
                            let tok = token();
                            (!succeeded() && !tok.is_empty()).then(|| view! {
                                <ActionForm action=reset_action>
                                    <input type="hidden" name="token" value=tok />
                                    <div class="space-y-3">
                                        <input
                                            type="password"
                                            name="new_password"
                                            required
                                            minlength="8"
                                            placeholder=t("auth.new_password")
                                            class="w-full px-4 py-3 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors"
                                        />
                                        <input
                                            type="password"
                                            name="confirm_password"
                                            required
                                            minlength="8"
                                            placeholder=t("auth.confirm_password")
                                            class="w-full px-4 py-3 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors"
                                        />
                                        <button
                                            type="submit"
                                            class="w-full py-3 btn-glass font-semibold rounded-xl text-sm transition-all"
                                        >
                                            {t("auth.reset_password")}
                                        </button>
                                    </div>
                                </ActionForm>
                                // Client-side password match check
                                <script>{r#"
(function() {
  var form = document.currentScript.previousElementSibling;
  if (!form) return;
  form.addEventListener('submit', function(e) {
    var pw = form.querySelector('[name=new_password]');
    var confirm = form.querySelector('[name=confirm_password]');
    if (pw && confirm && pw.value !== confirm.value) {
      e.preventDefault();
      alert('Passwords do not match');
    }
  });
})();
"#}</script>
                            })
                        }}
                    </div>
                </GlassPanel>
            </div>
        </div>
    }
}
