//! Review modal — confirmation dialog for selected trains.

use leptos::prelude::*;

/// Summary of a selected train for review.
#[derive(Debug, Clone)]
pub struct TrainSummary {
    pub train_number: String,
    pub departure: String,
    pub arrival: String,
    pub time: String,
}

/// A glass-panel modal showing selected trains before confirmation.
#[component]
pub fn ReviewModal(
    /// Whether the modal is open.
    open: ReadSignal<bool>,
    /// Selected trains to review.
    trains: ReadSignal<Vec<TrainSummary>>,
    /// Callback on confirm.
    on_confirm: Callback<()>,
    /// Callback on cancel/close.
    on_cancel: Callback<()>,
) -> impl IntoView {
    view! {
        <Show when=move || open.get()>
            <div class="fixed inset-0 z-50 flex items-center justify-center"
                 on:click=move |_| on_cancel.run(())>
                <div class="absolute inset-0 bg-black/40 backdrop-blur-sm fade-in"></div>
                <div class="glass-panel rounded-3xl p-6 w-[90%] max-w-md relative z-10 modal-enter"
                     on:click=move |e| e.stop_propagation()>
                    <h3 class="text-lg font-semibold text-[var(--theme-text-strong)] mb-4">
                        "Review Selection"
                    </h3>
                    <div class="space-y-3 mb-6 max-h-60 overflow-y-auto no-scrollbar">
                        {move || trains.get().into_iter().enumerate().map(|(idx, train)| {
                            view! {
                                <div class="glass-card rounded-xl p-3 page-enter"
                                     style=format!("animation-delay: {}ms", idx * 60)>
                                    <div class="flex items-center justify-between">
                                        <div class="flex items-center gap-2">
                                            <span class="w-6 h-6 rounded-full bg-[var(--theme-accent-soft)] text-[var(--theme-accent-text)] flex items-center justify-center text-xs font-bold">
                                                {idx + 1}
                                            </span>
                                            <span class="font-medium text-sm text-[var(--theme-text-strong)]">
                                                {train.train_number.clone()}
                                            </span>
                                        </div>
                                        <span class="text-xs text-[var(--theme-text-muted)]">{train.time.clone()}</span>
                                    </div>
                                    <div class="mt-1 text-xs text-[var(--theme-text-muted)]">
                                        {train.departure.clone()} " \u{2192} " {train.arrival.clone()}
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                    <div class="flex gap-3">
                        <button
                            class="flex-1 py-2.5 rounded-xl text-sm font-medium bg-[var(--theme-surface-muted)] text-[var(--theme-text-muted)] hover:bg-[var(--theme-surface-hover)]"
                            on:click=move |_| on_cancel.run(())
                        >"Cancel"</button>
                        <button
                            class="flex-1 py-2.5 rounded-xl text-sm font-medium bg-[var(--theme-accent-solid)] text-white hover:bg-[var(--theme-accent-solid-hover)]"
                            on:click=move |_| on_confirm.run(())
                        >"Confirm"</button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
