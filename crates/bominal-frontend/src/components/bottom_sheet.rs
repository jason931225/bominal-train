//! Bottom sheet modal — slides up from bottom with drag handle.

use leptos::prelude::*;

/// A bottom sheet that slides up from the bottom of the screen.
#[component]
pub fn BottomSheet(
    /// Whether the sheet is open.
    open: ReadSignal<bool>,
    /// Callback to close the sheet.
    on_close: Callback<()>,
    /// Optional title.
    #[prop(into, optional)]
    title: Option<String>,
    children: Children,
) -> impl IntoView {
    let title_view = title.map(|t| {
        view! {
            <div class="px-6 pb-3">
                <h3 class="text-lg font-semibold text-[var(--theme-text-strong)]">{t}</h3>
            </div>
        }
    });

    let rendered_children = children();

    view! {
        <div class="fixed inset-0 z-50"
             style=move || if open.get() { "display: block;" } else { "display: none;" }>
            // Backdrop
            <div class="absolute inset-0 bg-black/40 backdrop-blur-sm fade-in"
                 on:click=move |_| on_close.run(())></div>
            // Sheet
            <div class="absolute bottom-0 left-0 right-0 glass-panel rounded-t-3xl sheet-enter safe-area-pb">
                // Drag handle
                <div class="flex justify-center pt-3 pb-2">
                    <div class="w-10 h-1 rounded-full bg-[var(--theme-text-subtle)] opacity-40"></div>
                </div>
                // Title
                {title_view}
                // Content
                <div class="px-6 pb-6 max-h-[70vh] overflow-y-auto no-scrollbar">
                    {rendered_children}
                </div>
            </div>
        </div>
    }
}
