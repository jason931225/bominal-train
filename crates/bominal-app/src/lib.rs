pub mod api;
pub mod i18n;
pub mod pages;

// =============================================================================
// Auth context — shared across all pages
// =============================================================================

use serde::Deserialize;

/// Client-side representation of the authenticated user.
#[derive(Debug, Clone, Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub email: String,
    pub name: String,
}

/// Convenience accessor for the auth context signal.
pub fn use_auth() -> leptos::prelude::RwSignal<Option<AuthUser>> {
    leptos::prelude::use_context::<leptos::prelude::RwSignal<Option<AuthUser>>>()
        .expect("AuthContext not provided")
}

/// WASM entry point — replaces the #app loading div with the real app.
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    use leptos::prelude::*;
    use leptos::web_sys;

    // Remove the loading placeholder
    if let Some(window) = web_sys::window()
        && let Some(document) = window.document()
        && let Some(loading) = document.get_element_by_id("app")
    {
        loading.set_inner_html("");
    }
    mount_to_body(App);
}

use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::i18n::t;
use crate::pages::auth;

/// Root application component with router.
#[component]
pub fn App() -> impl IntoView {
    // Provide auth context to the entire app
    let auth_user = RwSignal::new(None::<AuthUser>);
    let auth_checked = RwSignal::new(false);
    provide_context(auth_user);

    // Fetch current user on mount
    let _auth_check = LocalResource::new(move || async move {
        match crate::api::get_silent::<AuthUser>("/api/auth/me").await {
            Ok(resp) if resp.success => {
                auth_user.set(resp.data);
            }
            _ => {
                auth_user.set(None);
            }
        }
        auth_checked.set(true);
    });

    view! {
        // Block rendering until auth check completes
        {move || if !auth_checked.get() {
            view! {
                <div class="min-h-screen flex items-center justify-center">
                    <p class="text-sm" style="color: var(--lg-text-tertiary);">"Bominal"</p>
                </div>
            }.into_any()
        } else {
            view! {
                <Router>
                    <FlatRoutes fallback=|| view! { <NotFoundPage /> }>
                        <Route path=path!("/auth") view=auth::SignInPage />
                        <Route path=path!("/auth/login") view=auth::LoginPage />
                        <Route path=path!("/auth/signup") view=auth::SignupPage />
                        <Route path=path!("/auth/forgot") view=auth::ForgotPage />
                    </FlatRoutes>
                </Router>
            }.into_any()
        }}
    }
}

/// 404 page component.
#[component]
fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="min-h-screen flex items-center justify-center">
            <div class="text-center page-enter">
                <h1 class="text-6xl font-bold" style="color: var(--lg-text-disabled);">"404"</h1>
                <p class="mt-4 text-lg" style="color: var(--lg-text-secondary);">{t("error.not_found")}</p>
                <a
                    href="/auth"
                    class="mt-6 inline-block lg-btn-primary px-6 py-3 rounded-xl text-sm"
                >
                    {t("error.go_home")}
                </a>
            </div>
        </div>
    }
}
