use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_location};

use crate::i18n::t;

fn icon_home() -> impl IntoView {
    view! {
        <svg aria-hidden="true" class="lg-nav-link__icon" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.9">
            <path stroke-linecap="round" stroke-linejoin="round" d="M3 12l2-2 7-7 7 7 2 2" />
            <path stroke-linecap="round" stroke-linejoin="round" d="M5 10v10a1 1 0 001 1h3v-6h6v6h3a1 1 0 001-1V10" />
        </svg>
    }
}

fn icon_search() -> impl IntoView {
    view! {
        <svg aria-hidden="true" class="lg-nav-link__icon" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.9">
            <circle cx="11" cy="11" r="7"></circle>
            <path stroke-linecap="round" stroke-linejoin="round" d="M20 20l-3.5-3.5" />
        </svg>
    }
}

fn icon_tasks() -> impl IntoView {
    view! {
        <svg aria-hidden="true" class="lg-nav-link__icon" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.9">
            <rect x="5" y="4" width="14" height="16" rx="3"></rect>
            <path stroke-linecap="round" stroke-linejoin="round" d="M9 4h6M9 10h6M9 15l2 2 4-4" />
        </svg>
    }
}

fn icon_ticket() -> impl IntoView {
    view! {
        <svg aria-hidden="true" class="lg-nav-link__icon" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.9">
            <path stroke-linecap="round" stroke-linejoin="round" d="M5 7a2 2 0 012-2h10a2 2 0 012 2v3a2 2 0 010 4v3a2 2 0 01-2 2H7a2 2 0 01-2-2v-3a2 2 0 010-4V7z" />
            <path stroke-linecap="round" stroke-linejoin="round" d="M15 7v10" />
        </svg>
    }
}

fn icon_settings() -> impl IntoView {
    view! {
        <svg aria-hidden="true" class="lg-nav-link__icon" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.9">
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 3l1.2 2.4 2.7.4-.8 2.6 1.8 2-1.8 2 .8 2.6-2.7.4L12 21l-1.2-2.4-2.7-.4.8-2.6-1.8-2 1.8-2-.8-2.6 2.7-.4L12 3z" />
            <circle cx="12" cy="12" r="3"></circle>
        </svg>
    }
}

#[component]
pub fn Sidebar() -> impl IntoView {
    let location = use_location();
    let pathname = move || location.pathname.get();

    let is_active = move |path: &'static str, exact: bool| {
        let current = pathname();
        if exact {
            current == path
        } else {
            current.starts_with(path)
        }
    };

    let item_class = move |path: &'static str, exact: bool| {
        if is_active(path, exact) {
            "lg-nav-link lg-nav-link--active"
        } else {
            "lg-nav-link"
        }
    };

    view! {
        <aside class="lg-sidebar lg-surface-sidebar lg-navigation-sidebar">
            <div class="lg-sidebar__brand">
                <p class="lg-sidebar__eyebrow">"Bominal Train"</p>
                <span class="app-brand-wordmark">"Bominal"</span>
            </div>

            <nav aria-label="Primary navigation" class="lg-nav-list">
                <A href="/home" exact=true attr:class=move || item_class("/home", true)>
                    {icon_home()}
                    <span class="lg-nav-link__label">{t("nav.home")}</span>
                </A>
                <A href="/search" attr:class=move || item_class("/search", false)>
                    {icon_search()}
                    <span class="lg-nav-link__label">{t("nav.search")}</span>
                </A>
                <A href="/tasks" attr:class=move || item_class("/tasks", false)>
                    {icon_tasks()}
                    <span class="lg-nav-link__label">{t("nav.tasks")}</span>
                </A>
                <A href="/reservations" attr:class=move || item_class("/reservations", false)>
                    {icon_ticket()}
                    <span class="lg-nav-link__label">{t("nav.reservations")}</span>
                </A>
            </nav>

            <div class="lg-sidebar__footer">
                <A href="/settings" attr:class=move || item_class("/settings", false)>
                    {icon_settings()}
                    <span class="lg-nav-link__label">{t("nav.settings")}</span>
                </A>
            </div>
        </aside>
    }
}
