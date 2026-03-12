//! Bottom navigation bar — persistent tab bar across all authenticated pages.

use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::i18n::t;

/// Bottom tab navigation bar.
///
/// Tabs: Home, Search, Tasks, Settings.
/// Hidden on the auth page (`/`).
#[component]
pub fn BottomNav() -> impl IntoView {
    let location = use_location();
    let pathname = move || location.pathname.get();

    // Hide on auth page
    let is_visible = move || {
        let p = pathname();
        p != "/"
    };

    let tab_class = move |path: &'static str| {
        let current = pathname();
        let is_active = if path == "/home" {
            current == "/home"
        } else {
            current.starts_with(path)
        };

        if is_active {
            "flex flex-col items-center gap-0.5 py-1.5 px-3 text-[var(--color-brand-primary)] transition-colors"
        } else {
            "flex flex-col items-center gap-0.5 py-1.5 px-3 text-[var(--color-text-tertiary)] hover:text-[var(--color-text-secondary)] transition-colors"
        }
    };

    view! {
        <Show when=is_visible>
            <nav class="fixed bottom-0 left-0 right-0 z-50 glass-panel border-t border-[var(--color-border-subtle)]">
                <div class="flex items-center justify-around max-w-lg mx-auto px-4 safe-area-pb">
                    <a href="/home" class=move || tab_class("/home")>
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
                        </svg>
                        <span class="text-[10px] font-medium">{t("nav.home")}</span>
                    </a>
                    <a href="/search" class=move || tab_class("/search")>
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                        </svg>
                        <span class="text-[10px] font-medium">{t("nav.search")}</span>
                    </a>
                    <a href="/tasks" class=move || tab_class("/tasks")>
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
                        </svg>
                        <span class="text-[10px] font-medium">{t("nav.tasks")}</span>
                    </a>
                    <a href="/settings" class=move || tab_class("/settings")>
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                            <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                        </svg>
                        <span class="text-[10px] font-medium">{t("nav.settings")}</span>
                    </a>
                </div>
            </nav>
        </Show>
    }
}
