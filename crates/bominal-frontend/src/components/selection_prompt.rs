//! Selection prompt — floating bar when trains are selected.

use leptos::prelude::*;

use crate::i18n::t;

/// Floating action bar shown when trains are selected.
#[component]
pub fn SelectionPrompt(
    /// Number of selected items.
    count: Signal<usize>,
    /// Callback on review/confirm.
    on_confirm: Callback<()>,
    /// Callback on clear.
    on_clear: Callback<()>,
) -> impl IntoView {
    view! {
        <Show when=move || { count.get() > 0 }>
            <div class="fixed bottom-20 left-1/2 -translate-x-1/2 z-40 glass-panel rounded-3xl px-5 py-3 flex items-center gap-4 sheet-enter safe-area-pb">
                <span class="text-sm font-medium text-[var(--color-text-primary)]">
                    {move || format!("{} {}", count.get(), t("selection.selected_count"))}
                </span>
                <div class="flex gap-2">
                    <button
                        class="px-4 py-1.5 rounded-xl text-xs font-medium bg-[var(--color-bg-sunken)] text-[var(--color-text-tertiary)] hover:bg-[var(--color-interactive-hover)] transition-colors"
                        on:click=move |_| on_clear.run(())
                    >{t("selection.clear")}</button>
                    <button
                        class="px-4 py-1.5 rounded-xl text-xs font-medium btn-glass transition-all"
                        on:click=move |_| on_confirm.run(())
                    >{t("selection.review")}</button>
                </div>
            </div>
        </Show>
    }
}
