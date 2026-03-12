//! Selection prompt — floating bar when trains are selected.

use leptos::prelude::*;

/// Floating action bar shown when trains are selected.
#[component]
pub fn SelectionPrompt(
    /// Number of selected items.
    count: ReadSignal<usize>,
    /// Callback on confirm.
    on_confirm: Callback<()>,
    /// Callback on clear.
    on_clear: Callback<()>,
) -> impl IntoView {
    view! {
        <Show when=move || { count.get() > 0 }>
            <div class="fixed bottom-20 left-1/2 -translate-x-1/2 z-40 glass-panel rounded-3xl px-5 py-3 flex items-center gap-4 sheet-enter safe-area-pb">
                <span class="text-sm font-medium text-[var(--theme-text-strong)]">
                    {move || format!("{} selected", count.get())}
                </span>
                <div class="flex gap-2">
                    <button
                        class="px-4 py-1.5 rounded-xl text-xs font-medium bg-[var(--theme-surface-muted)] text-[var(--theme-text-muted)] hover:bg-[var(--theme-surface-hover)]"
                        on:click=move |_| on_clear.run(())
                    >"Clear"</button>
                    <button
                        class="px-4 py-1.5 rounded-xl text-xs font-medium bg-[var(--theme-accent-solid)] text-white hover:bg-[var(--theme-accent-solid-hover)]"
                        on:click=move |_| on_confirm.run(())
                    >"Confirm"</button>
                </div>
            </div>
        </Show>
    }
}
