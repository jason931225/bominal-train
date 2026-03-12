//! Sortable list component — reorder items with up/down buttons.

use leptos::prelude::*;

/// A list item that can be reordered.
#[derive(Debug, Clone)]
pub struct SortableItem {
    pub id: String,
    pub label: String,
}

/// A sortable list with up/down/remove controls.
#[component]
pub fn SortableList(
    /// Current items in order.
    items: ReadSignal<Vec<SortableItem>>,
    /// Callback when order changes.
    on_change: Callback<Vec<SortableItem>>,
) -> impl IntoView {
    let move_up = move |idx: usize| {
        let mut current = items.get();
        if idx > 0 {
            current.swap(idx, idx - 1);
            on_change.run(current);
        }
    };

    let move_down = move |idx: usize| {
        let mut current = items.get();
        if idx < current.len() - 1 {
            current.swap(idx, idx + 1);
            on_change.run(current);
        }
    };

    let remove = move |idx: usize| {
        let mut current = items.get();
        current.remove(idx);
        on_change.run(current);
    };

    view! {
        <div class="space-y-2">
            {move || {
                let list = items.get();
                let len = list.len();
                list.into_iter().enumerate().map(|(idx, item)| {
                    let label = item.label.clone();
                    view! {
                        <div class="glass-card rounded-xl p-3 flex items-center justify-between page-enter"
                             style=format!("animation-delay: {}ms", idx * 50)>
                            <div class="flex items-center gap-3">
                                <span class="w-6 h-6 rounded-full bg-[var(--theme-accent-soft)] text-[var(--theme-accent-text)] flex items-center justify-center text-xs font-bold">
                                    {idx + 1}
                                </span>
                                <span class="text-sm font-medium text-[var(--theme-text-primary)]">{label}</span>
                            </div>
                            <div class="flex items-center gap-1">
                                <button
                                    class="p-1.5 rounded-lg hover:bg-[var(--theme-surface-hover)] text-[var(--theme-text-muted)] disabled:opacity-30"
                                    on:click=move |_| move_up(idx)
                                    disabled=move || idx == 0
                                >"\u{2191}"</button>
                                <button
                                    class="p-1.5 rounded-lg hover:bg-[var(--theme-surface-hover)] text-[var(--theme-text-muted)] disabled:opacity-30"
                                    on:click=move |_| move_down(idx)
                                    disabled=move || { idx >= len - 1 }
                                >"\u{2193}"</button>
                                <button
                                    class="p-1.5 rounded-lg hover:bg-[var(--theme-danger-soft)] text-[var(--theme-danger-text)]"
                                    on:click=move |_| remove(idx)
                                >"\u{00D7}"</button>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()
            }}
        </div>
    }
}
