//! Time slider component — range input for 30-min intervals.

use leptos::prelude::*;

/// Format a slot index (0-48) as HH:MM.
#[allow(clippy::manual_is_multiple_of)]
fn format_time_slot(slot: u32) -> String {
    let hours = slot / 2;
    let minutes = if slot % 2 == 0 { "00" } else { "30" };
    format!("{hours:02}:{minutes}")
}

/// A time range slider with 30-minute intervals.
#[component]
pub fn TimeSlider(
    /// Current slot value (0-48).
    value: ReadSignal<u32>,
    /// Callback when value changes.
    on_change: Callback<u32>,
    /// Label text.
    #[prop(into, default = "Time".to_string())]
    label: String,
) -> impl IntoView {
    view! {
        <div class="space-y-2">
            <div class="flex items-center justify-between">
                <label class="text-sm font-medium text-[var(--theme-text-primary)]">{label}</label>
                <span class="text-sm font-semibold text-[var(--theme-accent-text)] bg-[var(--theme-accent-soft)] px-2 py-0.5 rounded-md">
                    {move || format_time_slot(value.get())}
                </span>
            </div>
            <input
                type="range"
                min="0"
                max="48"
                step="1"
                class="w-full h-2 rounded-full appearance-none cursor-pointer theme-range"
                prop:value=move || value.get().to_string()
                on:input=move |ev| {
                    if let Ok(v) = event_target_value(&ev).parse::<u32>() {
                        on_change.run(v);
                    }
                }
                style=move || format!("--slider-progress: {}%", (value.get() as f64 / 48.0) * 100.0)
            />
        </div>
    }
}
