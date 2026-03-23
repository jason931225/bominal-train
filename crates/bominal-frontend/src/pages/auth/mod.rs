//! Flat auth route pages — each view is an independent page component.
//!
//! Routes:
//!   /auth           → PasskeyPage (passkey-first entry)
//!   /auth/login     → LoginPage (email + password)
//!   /auth/signup    → SignupPage (registration)
//!   /auth/forgot    → ForgotPage (request password reset)
//!   /auth/verify    → AuthVerifyPage (post-signup email verification)
//!   /auth/add-passkey → AddPasskeyPage (post-signup passkey registration)

pub mod add_passkey_page;
pub mod forgot_page;
pub mod login_page;
pub mod passkey_page;
pub mod signup_page;
pub mod verify_page;

pub use add_passkey_page::AddPasskeyPage;
pub use forgot_page::ForgotPage;
pub use login_page::LoginPage;
pub use passkey_page::PasskeyPage;
pub use signup_page::SignupPage;
pub use verify_page::AuthVerifyPage;

// ── Shared icon helpers ───────────────────────────────────────────────

use leptos::prelude::*;

pub(super) fn icon_hero_key() -> impl IntoView {
    view! {
        <svg class="w-14 h-14 text-[var(--color-brand-text)]" fill="none" stroke="url(#key-grad)" viewBox="0 0 24 24" stroke-width="1.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 5.25a3 3 0 013 3m3 0a6 6 0 01-7.029 5.912c-.563-.097-1.159.026-1.563.43L10.5 17.25H8.25v2.25H6v2.25H2.25v-2.818c0-.597.237-1.17.659-1.591l6.499-6.499c.404-.404.527-1 .43-1.563A6 6 0 1121.75 8.25z" />
            <defs>
                <linearGradient id="key-grad" x1="2" y1="2" x2="22" y2="22">
                    <stop stop-color="#007AFF"/>
                    <stop offset="1" stop-color="#5AC8FA"/>
                </linearGradient>
            </defs>
        </svg>
    }
}

pub(super) fn icon_mail() -> impl IntoView {
    view! {
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round"
                d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
        </svg>
    }
}

pub(super) fn icon_lock() -> impl IntoView {
    view! {
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round"
                d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
        </svg>
    }
}

pub(super) fn icon_key() -> impl IntoView {
    view! {
        <svg class="w-4.5 h-4.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 5.25a3 3 0 013 3m3 0a6 6 0 01-7.029 5.912c-.563-.097-1.159.026-1.563.43L10.5 17.25H8.25v2.25H6v2.25H2.25v-2.818c0-.597.237-1.17.659-1.591l6.499-6.499c.404-.404.527-1 .43-1.563A6 6 0 1121.75 8.25z" />
        </svg>
    }
}

pub(super) fn icon_arrow_left() -> impl IntoView {
    view! {
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M10.5 19.5L3 12m0 0l7.5-7.5M3 12h18" />
        </svg>
    }
}

pub(super) fn icon_mail_large() -> impl IntoView {
    view! {
        <svg class="w-7 h-7 text-[var(--color-brand-text)]" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.5">
            <path stroke-linecap="round" stroke-linejoin="round"
                d="M21.75 6.75v10.5a2.25 2.25 0 01-2.25 2.25h-15a2.25 2.25 0 01-2.25-2.25V6.75m19.5 0A2.25 2.25 0 0019.5 4.5h-15a2.25 2.25 0 00-2.25 2.25m19.5 0v.243a2.25 2.25 0 01-1.07 1.916l-7.5 4.615a2.25 2.25 0 01-2.36 0L3.32 8.91a2.25 2.25 0 01-1.07-1.916V6.75" />
        </svg>
    }
}

pub(super) fn icon_eye() -> impl IntoView {
    view! {
        <svg class="w-4.5 h-4.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round"
                d="M2.036 12.322a1.012 1.012 0 010-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178z" />
            <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
    }
}

pub(super) fn icon_eye_off() -> impl IntoView {
    view! {
        <svg class="w-4.5 h-4.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round"
                d="M3.98 8.223A10.477 10.477 0 001.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.45 10.45 0 0112 4.5c4.756 0 8.773 3.162 10.065 7.498a10.523 10.523 0 01-4.293 5.774M6.228 6.228L3 3m3.228 3.228l3.65 3.65m7.894 7.894L21 21m-3.228-3.228l-3.65-3.65m0 0a3 3 0 10-4.243-4.243m4.242 4.242L9.88 9.88" />
        </svg>
    }
}

pub(super) fn icon_user_plus() -> impl IntoView {
    view! {
        <svg class="w-6 h-6 text-[var(--color-status-success)]" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round"
                d="M19 7.5v3m0 0v3m0-3h3m-3 0h-3m-2.25-4.125a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zM4 19.235v-.11a6.375 6.375 0 0112.75 0v.109A12.318 12.318 0 0110.374 21c-2.331 0-4.512-.645-6.374-1.766z" />
        </svg>
    }
}

// ── Shared auth shell ────────────────────────────────────────────────

/// Wraps the auth content in the full-screen centered layout with blobs and logo.
pub(super) fn auth_shell(content: AnyView) -> impl IntoView {
    view! {
        <div class="min-h-screen w-full flex items-center justify-center px-4 relative overflow-hidden page-enter">
            <div class="absolute top-1/4 -left-32 w-96 h-96 rounded-full blur-[120px] pointer-events-none opacity-60"
                 style="background: rgba(0,122,255,0.25)"></div>
            <div class="absolute bottom-1/4 -right-32 w-96 h-96 rounded-full blur-[120px] pointer-events-none opacity-50"
                 style="background: rgba(88,86,214,0.20)"></div>

            <div class="w-full max-w-sm relative z-10">
                <div class="flex justify-center mb-8">
                    <div class="flex items-center gap-3">
                        <div class="app-brand-badge flex h-10 w-10 items-center justify-center rounded-xl rotate-3">
                            <div class="app-brand-badge-inner h-5 w-5 rounded-md border-[2.5px]"></div>
                        </div>
                        <span class="app-brand-wordmark text-2xl font-bold bg-clip-text text-transparent tracking-tight">"Bominal"</span>
                    </div>
                </div>

                {content}
            </div>
        </div>
    }
}

// ── Password strength helpers ────────────────────────────────────────

pub(super) fn password_strength(pw: &str) -> usize {
    let mut score = 0;
    if pw.len() >= 8 {
        score += 1;
    }
    if pw.chars().any(|c| c.is_uppercase()) {
        score += 1;
    }
    if pw.chars().any(|c| c.is_ascii_digit()) {
        score += 1;
    }
    if pw.chars().any(|c| !c.is_alphanumeric()) {
        score += 1;
    }
    score
}

pub(super) fn strength_color(score: usize) -> &'static str {
    match score {
        1 => "bg-[var(--color-status-error)]",
        2 => "bg-[var(--color-status-warning)]",
        3 => "bg-[var(--color-status-warning)]",
        4 => "bg-[var(--color-status-success)]",
        _ => "bg-[var(--color-border-subtle)]",
    }
}

pub(super) fn strength_text_color(score: usize) -> &'static str {
    match score {
        1 => "text-[var(--color-status-error)]",
        2 => "text-[var(--color-status-warning)]",
        3 => "text-[var(--color-status-warning)]",
        4 => "text-[var(--color-status-success)]",
        _ => "",
    }
}

pub(super) fn strength_label(score: usize) -> &'static str {
    match score {
        1 => "auth.pw_weak",
        2 => "auth.pw_fair",
        3 => "auth.pw_good",
        4 => "auth.pw_strong",
        _ => "",
    }
}
