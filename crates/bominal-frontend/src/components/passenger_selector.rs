//! Passenger type selector — vertical list of counters with max total enforcement.

use leptos::prelude::*;

/// A single passenger type with count.
#[derive(Debug, Clone)]
pub struct PassengerCount {
    pub ptype: String,
    pub label: String,
    pub description: String,
    pub count: u8,
    pub min: u8,
    pub max: u8,
    /// If true, the row is greyed out and non-interactive (not yet implemented).
    pub disabled: bool,
}

/// Max total passengers across all types.
const MAX_TOTAL: u8 = 9;

/// Vertical list of passenger type counters with max total enforcement.
#[component]
pub fn PassengerSelector(
    /// Current passenger counts.
    passengers: ReadSignal<Vec<PassengerCount>>,
    /// Callback when passengers change.
    on_change: Callback<Vec<PassengerCount>>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-4">
            {move || passengers.get().into_iter().enumerate().map(|(idx, _p)| {
                let decrement = move |_| {
                    let mut current = passengers.get();
                    let total: u8 = current.iter().map(|p| p.count).sum();
                    if current[idx].count > current[idx].min && total > 1 {
                        current[idx].count -= 1;
                        on_change.run(current);
                    }
                };

                let increment = move |_| {
                    let mut current = passengers.get();
                    let total: u8 = current.iter().map(|p| p.count).sum();
                    if current[idx].count < current[idx].max && total < MAX_TOTAL {
                        current[idx].count += 1;
                        on_change.run(current);
                    }
                };

                let is_disabled = move || passengers.get()[idx].disabled;

                view! {
                    <div class=move || if is_disabled() { "flex items-center justify-between opacity-40" } else { "flex items-center justify-between" }>
                        <div class="flex flex-col">
                            <div class="flex items-center gap-2">
                                <span class="font-semibold text-sm text-[var(--color-text-primary)]">
                                    {move || passengers.get()[idx].label.clone()}
                                </span>
                                {move || is_disabled().then(|| view! {
                                    <span class="text-[10px] px-1.5 py-0.5 rounded bg-[var(--color-bg-sunken)] text-[var(--color-text-disabled)] border border-[var(--color-border-subtle)]">
                                        "—"
                                    </span>
                                })}
                            </div>
                            <span class="text-xs text-[var(--color-text-disabled)]">
                                {move || passengers.get()[idx].description.clone()}
                            </span>
                        </div>
                        <div class="flex items-center gap-3">
                            <button
                                class="w-8 h-8 rounded-full flex items-center justify-center border border-[var(--color-border-default)] text-[var(--color-text-tertiary)] hover:border-[var(--color-border-focus)] disabled:opacity-30 transition-colors bg-[var(--color-bg-elevated)] shadow-sm"
                                on:click=decrement
                                disabled=move || {
                                    let current = passengers.get();
                                    let total: u8 = current.iter().map(|p| p.count).sum();
                                    is_disabled() || current[idx].count <= current[idx].min || total <= 1
                                }
                            >"-"</button>
                            <span class="w-4 text-center text-sm font-semibold text-[var(--color-text-primary)]">
                                {move || passengers.get()[idx].count}
                            </span>
                            <button
                                class="w-8 h-8 rounded-full flex items-center justify-center border border-[var(--color-border-default)] text-[var(--color-text-tertiary)] hover:border-[var(--color-border-focus)] disabled:opacity-30 transition-colors bg-[var(--color-bg-elevated)] shadow-sm"
                                on:click=increment
                                disabled=move || {
                                    let current = passengers.get();
                                    let total: u8 = current.iter().map(|p| p.count).sum();
                                    is_disabled() || current[idx].count >= current[idx].max || total >= MAX_TOTAL
                                }
                            >"+"</button>
                        </div>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }.into_any()
}
