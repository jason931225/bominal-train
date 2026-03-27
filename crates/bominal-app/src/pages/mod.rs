pub mod auth;
pub mod home;
pub mod reservations;
pub mod reset_password;
pub mod search;
pub mod tasks;
pub mod verify_email;

use leptos::prelude::*;
use leptos_router::components::Redirect;
use server_fn::error::ServerFnError;

use crate::{state::use_auth_state, types::TaskStatus};

fn loading_shell(message: &'static str) -> impl IntoView {
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

#[component]
pub(crate) fn ProtectedPage(children: ChildrenFn) -> impl IntoView {
    let auth = use_auth_state();

    view! {
        {move || {
            if !auth.checked.get() || auth.loading.get() {
                loading_shell("Checking your session before opening the app shell.").into_any()
            } else if !auth.is_authenticated() {
                view! { <Redirect path="/auth" /> }.into_any()
            } else {
                children().into_any()
            }
        }}
    }
}

pub(crate) fn format_server_error(error: &ServerFnError) -> String {
    let message = error.to_string();
    message
        .strip_prefix("error running server function: ")
        .unwrap_or(&message)
        .trim()
        .to_string()
}

pub(crate) fn is_active_task(status: TaskStatus) -> bool {
    matches!(
        status,
        TaskStatus::Queued | TaskStatus::Running | TaskStatus::Idle | TaskStatus::AwaitingPayment
    )
}
