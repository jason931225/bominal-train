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
use leptos_meta::{Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::use_location,
    path,
};

use crate::pages::{auth, home, reservations, reset_password, search, tasks, verify_email};

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    leptos::mount::hydrate_islands();
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
            <Route path=path!("/settings") view=shell_pages::SettingsPage />
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

    let auth = app_state.auth;
    let theme = app_state.theme;

    let auth_bootstrap = Resource::new(|| (), |_| api::get_me());

    Effect::new(move |_| match auth_bootstrap.get() {
        Some(Ok(user)) => auth.set_user(user),
        Some(Err(_)) => auth.set_user(None),
        None => auth.loading.set(true),
    });

    Effect::new(move |_| {
        browser::sync_theme_attrs(theme.theme.get().as_str(), theme.mode.get().as_str());
    });

    view! {
        <Title text="Bominal Train" />
        <Stylesheet id="bominal-app" href="/pkg/bominal-app.css" />

        <Router>
            <ShellChrome />
        </Router>
    }
}
