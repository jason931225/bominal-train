//! Phase 3 route stubs and shared shell helpers.

use leptos::prelude::*;
use leptos_router::components::{A, Redirect};

use crate::state::use_auth_state;

pub const ROUTE_PATHS: [&str; 14] = [
    "/",
    "/auth",
    "/auth/login",
    "/auth/signup",
    "/auth/forgot",
    "/auth/verify",
    "/auth/add-passkey",
    "/home",
    "/search",
    "/tasks",
    "/reservations",
    "/settings",
    "/verify-email",
    "/reset-password",
];

pub fn is_public_path(path: &str) -> bool {
    path == "/" || path == "/verify-email" || path == "/reset-password" || path.starts_with("/auth")
}

fn loading_shell(message: &'static str) -> impl leptos::prelude::IntoView {
    view! {
        <section class="lg-loading-shell">
            <div class="lg-loading-card">
                <div class="lg-spinner" aria-hidden="true"></div>
                <p class="lg-loading-eyebrow">"Bominal Train"</p>
                <p class="lg-loading-copy">{message}</p>
            </div>
        </section>
    }
}

fn public_stub(
    phase_label: &'static str,
    title: &'static str,
    description: &'static str,
    route_path: &'static str,
    primary: (&'static str, &'static str),
    secondary: Option<(&'static str, &'static str)>,
) -> impl leptos::prelude::IntoView {
    view! {
        <section data-product="auth" class="lg-auth-shell">
            <div class="lg-route-card lg-route-card--auth">
                <p class="lg-route-kicker">{phase_label}</p>
                <h1>{title}</h1>
                <p>{description}</p>
                <div class="lg-route-path">
                    <span>"Route"</span>
                    <code>{route_path}</code>
                </div>
                <div class="lg-route-actions">
                    <A href=primary.0 attr:class="lg-btn-primary">
                        {primary.1}
                    </A>
                    {secondary.map(|(href, label)| {
                        view! {
                            <A href=href attr:class="lg-btn-secondary">
                                {label}
                            </A>
                        }
                    })}
                </div>
            </div>
        </section>
    }
}

fn protected_stub(
    phase_label: &'static str,
    title: &'static str,
    description: &'static str,
    route_path: &'static str,
    primary: (&'static str, &'static str),
) -> impl leptos::prelude::IntoView {
    view! {
        <section class="lg-page-shell">
            <div class="lg-route-card lg-route-card--protected">
                <p class="lg-route-kicker">{phase_label}</p>
                <h1>{title}</h1>
                <p>{description}</p>
                <div class="lg-route-path">
                    <span>"Route"</span>
                    <code>{route_path}</code>
                </div>
                <div class="lg-route-actions">
                    <A href=primary.0 attr:class="lg-btn-primary">
                        {primary.1}
                    </A>
                </div>
            </div>
        </section>
    }
}

fn protected_route(
    phase_label: &'static str,
    title: &'static str,
    description: &'static str,
    route_path: &'static str,
    primary: (&'static str, &'static str),
) -> impl leptos::prelude::IntoView {
    let auth = use_auth_state();

    view! {
        {move || {
            if !auth.checked.get() || auth.loading.get() {
                loading_shell("Checking your session before opening the app shell.").into_any()
            } else if !auth.is_authenticated() {
                view! { <Redirect path="/auth" /> }.into_any()
            } else {
                protected_stub(phase_label, title, description, route_path, primary).into_any()
            }
        }}
    }
}

#[component]
pub fn RootRedirectPage() -> impl leptos::prelude::IntoView {
    let auth = use_auth_state();

    view! {
        {move || {
            if !auth.checked.get() || auth.loading.get() {
                loading_shell("Checking your session before routing.").into_any()
            } else if auth.is_authenticated() {
                view! { <Redirect path="/home" /> }.into_any()
            } else {
                view! { <Redirect path="/auth" /> }.into_any()
            }
        }}
    }
}

#[component]
pub fn AuthLandingPage() -> impl leptos::prelude::IntoView {
    public_stub(
        "Phase 4 route stub",
        "Passkey entry shell",
        "Phase 4 will restore the real passkey-first sign-in page and WebAuthn handoff.",
        "/auth",
        ("/auth/login", "Continue with email"),
        Some(("/auth/signup", "Create account")),
    )
}

#[component]
pub fn LoginPage() -> impl leptos::prelude::IntoView {
    public_stub(
        "Phase 4 route stub",
        "Email sign-in",
        "Phase 4 will port the login form and connect it to the typed auth proxy layer.",
        "/auth/login",
        ("/auth", "Back to auth home"),
        Some(("/auth/forgot", "Forgot password")),
    )
}

#[component]
pub fn SignupPage() -> impl leptos::prelude::IntoView {
    public_stub(
        "Phase 4 route stub",
        "Account creation",
        "Phase 4 will replace this card with the full registration flow and password guidance.",
        "/auth/signup",
        ("/auth/login", "Open sign-in"),
        Some(("/auth/add-passkey", "Preview passkey setup")),
    )
}

#[component]
pub fn ForgotPasswordPage() -> impl leptos::prelude::IntoView {
    public_stub(
        "Phase 4 route stub",
        "Password reset request",
        "Phase 4 will restore the email reset request flow on this route.",
        "/auth/forgot",
        ("/auth/login", "Back to sign-in"),
        None,
    )
}

#[component]
pub fn AuthVerifyPage() -> impl leptos::prelude::IntoView {
    public_stub(
        "Phase 4 route stub",
        "Verify your email",
        "Phase 4 will reconnect the post-signup verification handoff and resend flow.",
        "/auth/verify",
        ("/auth/add-passkey", "Open add-passkey"),
        Some(("/auth/signup", "Back to signup")),
    )
}

#[component]
pub fn AddPasskeyPage() -> impl leptos::prelude::IntoView {
    public_stub(
        "Phase 4 route stub",
        "Add a passkey",
        "Phase 4 will wire this route to the real browser credential registration flow.",
        "/auth/add-passkey",
        ("/auth", "Back to auth home"),
        Some(("/auth/login", "Use email instead")),
    )
}

#[component]
pub fn VerifyEmailPage() -> impl leptos::prelude::IntoView {
    public_stub(
        "Phase 4 route stub",
        "Email verification link",
        "Phase 4 will read the verification token here and confirm the user's email address.",
        "/verify-email",
        ("/auth", "Return to auth"),
        None,
    )
}

#[component]
pub fn ResetPasswordPage() -> impl leptos::prelude::IntoView {
    public_stub(
        "Phase 4 route stub",
        "Reset password",
        "Phase 4 will restore the token-based password reset form on this route.",
        "/reset-password",
        ("/auth/login", "Return to sign-in"),
        None,
    )
}

#[component]
pub fn HomePage() -> impl leptos::prelude::IntoView {
    protected_route(
        "Phase 5 route stub",
        "Home dashboard shell",
        "Phase 5 will replace this stub with the live task summary and dashboard hero.",
        "/home",
        ("/search", "Open search"),
    )
}

#[component]
pub fn SearchPage() -> impl leptos::prelude::IntoView {
    protected_route(
        "Phase 5 route stub",
        "Train search shell",
        "Phase 5 will port station autocomplete, date controls, and multi-provider results here.",
        "/search",
        ("/tasks", "Open tasks"),
    )
}

#[component]
pub fn TasksPage() -> impl leptos::prelude::IntoView {
    protected_route(
        "Phase 5 route stub",
        "Tasks shell",
        "Phase 5 will restore live task monitoring, tabs, and SSE-driven updates on this page.",
        "/tasks",
        ("/reservations", "View reservations"),
    )
}

#[component]
pub fn ReservationsPage() -> impl leptos::prelude::IntoView {
    protected_route(
        "Phase 5 route stub",
        "Reservations shell",
        "Phase 5 will replace this placeholder with ticket details and payment/cancel/refund actions.",
        "/reservations",
        ("/settings", "Open settings"),
    )
}

#[component]
pub fn SettingsPage() -> impl leptos::prelude::IntoView {
    protected_route(
        "Phase 6 route stub",
        "Settings shell",
        "Phase 6 will restore provider credentials, saved cards, appearance controls, and logout.",
        "/settings",
        ("/home", "Back home"),
    )
}

#[component]
pub fn NotFoundPage() -> impl leptos::prelude::IntoView {
    public_stub(
        "Route missing",
        "Page not found",
        "This route is outside the migration inventory. Use the shell navigation to jump back onto a tracked page.",
        "/404",
        ("/auth", "Go to auth"),
        Some(("/home", "Go to home")),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_inventory_matches_phase_three_scope() {
        assert_eq!(ROUTE_PATHS.len(), 14);
        assert!(ROUTE_PATHS.contains(&"/settings"));
        assert!(ROUTE_PATHS.contains(&"/reset-password"));
    }

    #[test]
    fn public_path_detection_matches_shell_contract() {
        assert!(is_public_path("/"));
        assert!(is_public_path("/auth"));
        assert!(is_public_path("/auth/login"));
        assert!(is_public_path("/verify-email"));
        assert!(is_public_path("/reset-password"));
        assert!(!is_public_path("/home"));
        assert!(!is_public_path("/tasks"));
    }
}
