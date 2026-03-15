//! Main Leptos application shell with routing.
//!
//! Routes: `/` (auth), `/home`, `/search`, `/search/results`, `/tasks`, `/settings`

use leptos::prelude::*;
use leptos_meta::{Meta, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::components::bottom_nav::BottomNav;
use crate::i18n::t;
use crate::pages::{
    auth::{
        AddPasskeyPage, AuthVerifyPage, ForgotPage, LoginPage, PasskeyPage, SignupPage,
    },
    home_view::HomeView,
    reservations_view::ReservationsView,
    reset_password_page::ResetPasswordPage,
    schedule_results::ScheduleResults,
    search_panel::SearchPanel,
    settings_view::SettingsView,
    tasks_view::TasksView,
    verify_email_page::VerifyEmailPage,
};

/// HTML shell for SSR — renders the full `<html>` document.
pub fn shell() -> impl IntoView {
    let locale = use_context::<bominal_domain::i18n::Locale>().unwrap_or_default();
    let theme_prefs = use_context::<crate::theme::ThemePrefs>().unwrap_or_default();

    view! {
        <!DOCTYPE html>
        <html
            lang=locale.code()
            data-theme=theme_prefs.theme.as_str()
            data-mode=theme_prefs.mode.as_str()
        >
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover" />
                <meta name="color-scheme" content="light dark" />
                <meta name="theme-color" content="#f2f2f7" />
                <meta name="apple-mobile-web-app-capable" content="yes" />
                <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent" />
                <link rel="stylesheet" href="/style.css" />
                // Evervault JS SDK + interop
                <script src="https://js.evervault.com/v2"></script>
                <script src="/interop.js" defer></script>
            </head>
            <body class="min-h-screen bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] font-[var(--font-sans)] antialiased">
                <App />
            </body>
        </html>
    }
}

/// Root application component with meta context and routing.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let ev_ids = use_context::<crate::EvervaultIds>();

    view! {
        <Title text="Bominal" />
        <Meta name="description" content="Korean train reservation assistant" />
        <Stylesheet href="/style.css" />

        {move || ev_ids.as_ref().map(|ids| view! {
            <Meta name="ev-team-id" content=ids.team_id.clone() />
            <Meta name="ev-app-id" content=ids.app_id.clone() />
        })}

        <Router>
            <div class="relative min-h-screen pb-16">
                <main>
                    <Routes fallback=|| view! {
                        <div class="flex items-center justify-center min-h-screen">
                            <p class="text-[var(--color-text-secondary)]">{t("error.not_found")}</p>
                        </div>
                    }>
                        // Root and auth entry point both serve PasskeyPage
                        <Route path=path!("/") view=PasskeyPage />
                        <Route path=path!("/auth") view=PasskeyPage />
                        <Route path=path!("/auth/login") view=LoginPage />
                        <Route path=path!("/auth/signup") view=SignupPage />
                        <Route path=path!("/auth/forgot") view=ForgotPage />
                        <Route path=path!("/forgot-password") view=ForgotPage />
                        <Route path=path!("/auth/verify") view=AuthVerifyPage />
                        <Route path=path!("/auth/add-passkey") view=AddPasskeyPage />
                        // Authenticated app routes
                        <Route path=path!("/home") view=HomeView />
                        <Route path=path!("/search") view=SearchPanel />
                        <Route path=path!("/search/results") view=ScheduleResults />
                        <Route path=path!("/tasks") view=TasksView />
                        <Route path=path!("/reservations") view=ReservationsView />
                        <Route path=path!("/settings") view=SettingsView />
                        // Email link landing pages
                        <Route path=path!("/verify-email") view=VerifyEmailPage />
                        <Route path=path!("/reset-password") view=ResetPasswordPage />
                    </Routes>
                </main>
                <BottomNav />
            </div>
        </Router>
    }
}
