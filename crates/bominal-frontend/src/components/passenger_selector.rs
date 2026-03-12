//! Passenger type selector — counter grid for different passenger types.

use leptos::prelude::*;

/// A single passenger type with count.
#[derive(Debug, Clone)]
pub struct PassengerCount {
    pub ptype: String,
    pub label: String,
    pub count: u8,
    pub min: u8,
    pub max: u8,
}

/// Grid of passenger type counters.
#[component]
pub fn PassengerSelector(
    /// Current passenger counts.
    passengers: ReadSignal<Vec<PassengerCount>>,
    /// Callback when passengers change.
    on_change: Callback<Vec<PassengerCount>>,
) -> impl IntoView {
    view! {
        <div class="grid grid-cols-2 gap-3">
            {move || passengers.get().into_iter().enumerate().map(|(idx, p)| {
                let label = p.label.clone();
                let count = p.count;
                let min = p.min;
                let max = p.max;

                let decrement = move |_| {
                    let mut current = passengers.get();
                    if current[idx].count > min {
                        current[idx].count -= 1;
                        on_change.run(current);
                    }
                };

                let increment = move |_| {
                    let mut current = passengers.get();
                    if current[idx].count < max {
                        current[idx].count += 1;
                        on_change.run(current);
                    }
                };

                view! {
                    <div class="glass-card rounded-xl p-3 flex items-center justify-between">
                        <span class="text-sm font-medium text-[var(--theme-text-primary)]">{label}</span>
                        <div class="flex items-center gap-2">
                            <button
                                class="w-7 h-7 rounded-full flex items-center justify-center text-sm font-bold bg-[var(--theme-surface-muted)] text-[var(--theme-text-muted)] hover:bg-[var(--theme-surface-hover)] disabled:opacity-30"
                                on:click=decrement
                                disabled=move || count <= min
                            >"-"</button>
                            <span class="w-6 text-center text-sm font-semibold text-[var(--theme-text-strong)]">
                                {count}
                            </span>
                            <button
                                class="w-7 h-7 rounded-full flex items-center justify-center text-sm font-bold bg-[var(--theme-accent-soft)] text-[var(--theme-accent-text)] hover:bg-[var(--theme-accent-soft-strong)] disabled:opacity-30"
                                on:click=increment
                                disabled=move || count >= max
                            >"+"</button>
                        </div>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
