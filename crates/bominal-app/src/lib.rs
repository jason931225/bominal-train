#![recursion_limit = "256"]

pub mod api;
mod browser;
pub mod components;
pub mod i18n;
pub mod pages;
pub mod shell_pages;
pub mod state;
pub mod types;
pub mod utils;

use leptos::prelude::*;
use leptos_meta::{Meta, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::use_location,
    path,
};

use crate::pages::{
    auth, home, reservations, reset_password, search, settings, tasks, verify_email,
};

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    leptos::mount::hydrate_islands();
}

#[cfg(feature = "ssr")]
pub fn shell(options: leptos::config::LeptosOptions) -> impl IntoView {
    use leptos::hydration::{AutoReload, HydrationScripts};
    use leptos_meta::MetaTags;

    let locale = i18n::request_locale();
    let (theme, mode) = state::request_theme_prefs();

    view! {
        <!DOCTYPE html>
        <html lang=locale.code() data-theme=theme.as_str() data-mode=mode.as_str()>
            <head>
                <meta charset="utf-8" />
                <meta
                    name="viewport"
                    content="width=device-width, initial-scale=1.0, viewport-fit=cover"
                />
                <meta name="apple-mobile-web-app-capable" content="yes" />
                <meta name="color-scheme" content="light dark" />
                <meta name="theme-color" content="#000000" />
                <link rel="preconnect" href="https://cdn.jsdelivr.net" />
                <link
                    rel="stylesheet"
                    href="https://cdn.jsdelivr.net/gh/orioncactus/pretendard@v1.3.9/dist/web/variable/pretendardvariable-dynamic-subset.min.css"
                />
                <script src="https://js.evervault.com/v2"></script>
                <script src="/assets/interop.js"></script>
                <AutoReload options=options.clone() />
                <HydrationScripts options=options islands=true />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
fn ShellChrome() -> impl IntoView {
    let auth = state::use_auth_state();
    let location = use_location();

    let show_navigation =
        move || auth.is_authenticated() && !shell_pages::is_public_path(&location.pathname.get());

    view! {
        <div
            data-product="train"
            class=move || {
                if show_navigation() {
                    "lg-app-shell lg-app-shell--protected"
                } else {
                    "lg-app-shell lg-app-shell--public"
                }
            }
        >
            <Show when=show_navigation>
                <components::Sidebar />
            </Show>

            <main
                class=move || {
                    if show_navigation() {
                        "lg-shell-main lg-shell-main--protected"
                    } else {
                        "lg-shell-main lg-shell-main--public"
                    }
                }
            >
                <AppRoutes />
            </main>

            <Show when=show_navigation>
                <components::BottomNav />
            </Show>
        </div>
    }
}

#[component]
fn AppRoutes() -> impl IntoView {
    view! {
        <Routes fallback=|| view! { <shell_pages::NotFoundPage /> }>
            <Route path=path!("/") view=shell_pages::RootRedirectPage />
            <Route path=path!("/auth") view=auth::SignInPage />
            <Route path=path!("/auth/login") view=auth::LoginPage />
            <Route path=path!("/auth/signup") view=auth::SignupPage />
            <Route path=path!("/auth/forgot") view=auth::ForgotPage />
            <Route path=path!("/auth/verify") view=auth::AuthVerifyPage />
            <Route path=path!("/auth/add-passkey") view=auth::AddPasskeyPage />
            <Route path=path!("/home") view=home::HomePage />
            <Route path=path!("/search") view=search::SearchPage />
            <Route path=path!("/tasks") view=tasks::TasksPage />
            <Route path=path!("/reservations") view=reservations::ReservationsPage />
            <Route path=path!("/settings") view=settings::SettingsPage />
            <Route path=path!("/verify-email") view=verify_email::VerifyEmailPage />
            <Route path=path!("/reset-password") view=reset_password::ResetPasswordPage />
        </Routes>
    }
}

/// Phase 3 shell with router, guarded layout, and responsive navigation.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_context(i18n::request_locale());
    let app_state = state::provide_app_state();
    #[cfg(feature = "ssr")]
    let evervault_ids = std::env::var("EV_TEAM_ID")
        .ok()
        .zip(std::env::var("EV_APP_ID").ok());
    #[cfg(not(feature = "ssr"))]
    let evervault_ids: Option<(String, String)> = None;

    let auth = app_state.auth;
    let theme = app_state.theme;

    let auth_bootstrap = Resource::new_blocking(|| (), |_| api::get_me());

    Effect::new_isomorphic(move |_| match auth_bootstrap.get() {
        Some(Ok(user)) => auth.set_user(user),
        Some(Err(_)) => auth.set_user(None),
        None => auth.loading.set(true),
    });

    Effect::new(move |_| {
        browser::sync_theme_attrs(theme.theme.get().as_str(), theme.mode.get().as_str());
    });

    view! {
        <Title text="Bominal Train" />
        {evervault_ids.map(|(team_id, app_id)| {
            view! {
                <>
                    <Meta name="ev-team-id" content=team_id />
                    <Meta name="ev-app-id" content=app_id />
                </>
            }
        })}
        <Stylesheet id="bominal-app" href="/pkg/bominal-app.css" />

        <Router>
            <ShellChrome />
        </Router>
    }
}
