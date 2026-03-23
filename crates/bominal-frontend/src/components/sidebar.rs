use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::i18n::t;

/// Desktop sidebar navigation.
///
/// Hidden on mobile (`hidden md:flex flex-col ...`).
#[component]
pub fn Sidebar() -> impl IntoView {
    let location = use_location();
    let pathname = move || location.pathname.get();

    // Hide on auth pages
    let is_visible = move || {
        let p = pathname();
        !p.starts_with("/auth")
            && p != "/"
            && p != "/forgot-password"
            && p != "/verify-email"
            && p != "/reset-password"
    };

    let is_active = move |path: &'static str| {
        let current = pathname();
        if path == "/home" {
            current == "/home"
        } else {
            current.starts_with(path)
        }
    };

    let item_class = move |path: &'static str| {
        if is_active(path) {
            "flex items-center gap-3 px-4 py-3 rounded-xl bg-[var(--color-brand-primary)] text-[var(--color-brand-text)] font-semibold transition-all squish"
        } else {
            "flex items-center gap-3 px-4 py-3 rounded-xl text-[var(--color-text-secondary)] hover:bg-[var(--color-interactive-hover)] transition-all squish"
        }
    };

    view! {
        <Show when=is_visible>
            <aside class="hidden md:flex flex-col w-64 h-[100dvh] shrink-0 border-r border-[var(--color-border-subtle)] glass-panel rounded-none">
                <div class="px-6 pt-10 pb-6">
                    <span class="app-brand-wordmark text-3xl font-bold bg-clip-text text-transparent tracking-tight">"Bominal"</span>
                </div>
                
                <nav class="flex-1 px-4 space-y-2">
                    <a href="/home" class=move || item_class("/home")>
                        <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
                        </svg>
                        <span>{t("nav.home")}</span>
                    </a>
                    <a href="/search" class=move || item_class("/search")>
                        <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                        </svg>
                        <span>{t("nav.search")}</span>
                    </a>
                    <a href="/tasks" class=move || item_class("/tasks")>
                        <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
                        </svg>
                        <span>{t("nav.tasks")}</span>
                    </a>
                    <a href="/reservations" class=move || item_class("/reservations")>
                        <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M15 5v2m0 4v2m0 4v2M5 5a2 2 0 00-2 2v3a2 2 0 110 4v3a2 2 0 002 2h14a2 2 0 002-2v-3a2 2 0 110-4V7a2 2 0 00-2-2H5z" />
                        </svg>
                        <span>{t("nav.reservations")}</span>
                    </a>
                </nav>

                <div class="p-4 mt-auto">
                    <a href="/settings" class=move || item_class("/settings")>
                        <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                            <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                        </svg>
                        <span>{t("nav.settings")}</span>
                    </a>
                </div>
            </aside>
        </Show>
    }
}
