//! Schedule results page — displays train search results for task creation.
//!
//! This page is navigated to after searching on SearchPanel.
//! For SSR, results are passed via URL query parameters.

use leptos::prelude::*;

use crate::components::glass_panel::GlassPanel;
use crate::i18n::t;

/// Search results listing available trains.
///
/// In the current SSR-only mode, search results are displayed inline
/// on the SearchPanel. This page serves as a placeholder for
/// future enhanced flow (multi-step selection with review modal).
#[component]
pub fn ScheduleResults() -> impl IntoView {
    view! {
        <div class="px-4 pt-6 pb-4 space-y-4 max-w-xl lg:max-w-2xl mx-auto page-enter">
            <div class="flex items-center gap-3">
                <a href="/search" class="p-2 rounded-lg hover:bg-[var(--color-interactive-hover)] transition-colors">
                    <svg class="w-5 h-5 text-[var(--color-text-primary)]" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M15 19l-7-7 7-7" />
                    </svg>
                </a>
                <h1 class="text-xl font-bold text-[var(--color-text-primary)]">{t("search.title")}</h1>
            </div>

            <GlassPanel>
                <div class="p-4 text-center py-12">
                    <p class="text-[var(--color-text-tertiary)] text-sm">{t("search.no_results")}</p>
                    <a href="/search" class="inline-block mt-3 text-sm text-[var(--color-brand-text)] font-medium hover:underline">
                        {t("search.go_to_search")}
                    </a>
                </div>
            </GlassPanel>
        </div>
    }
}
