//! Time slider component — range input for 30-min intervals.

use leptos::prelude::*;

/// Format a slot index (0-47) as HH:MM.
fn format_time_slot(slot: u32) -> String {
    let hours = slot / 2;
    let minutes = if slot.is_multiple_of(2) { "00" } else { "30" };
    format!("{hours:02}:{minutes}")
}

/// A time range slider with 30-minute intervals (0-47 → 00:00–23:30).
#[component]
pub fn TimeSlider(
    /// Current slot value (0-47).
    value: ReadSignal<u32>,
    /// Callback when value changes.
    on_change: Callback<u32>,
    /// Label text.
    #[prop(into, default = "Time".to_string())]
    label: String,
) -> impl IntoView {
    let aria_label = label.clone();
    view! {
        <div class="space-y-2">
            <div class="flex items-center justify-between">
                <label class="text-sm font-medium text-[var(--color-text-primary)]">{label}</label>
                <span class="text-sm font-semibold text-[var(--color-brand-text)] bg-[var(--color-brand-primary)]/20 px-2 py-0.5 rounded-md">
                    {move || format_time_slot(value.get())}
                </span>
            </div>
            <input
                type="range"
                min="0"
                max="47"
                step="1"
                aria-label=aria_label
                aria-valuetext=move || format_time_slot(value.get())
                class="w-full h-2 rounded-full appearance-none cursor-pointer theme-range"
                prop:value=move || value.get().to_string()
                on:input=move |ev| {
                    if let Ok(v) = event_target_value(&ev).parse::<u32>() {
                        on_change.run(v);
                    }
                }
                style=move || format!("--slider-progress: {}%", (value.get() as f64 / 47.0) * 100.0)
            />
            <div class="flex justify-between text-[10px] text-[var(--color-text-disabled)] font-medium">
                <span>"00:00"</span>
                <span>"12:00"</span>
                <span>"23:30"</span>
            </div>
        </div>
    }.into_any()
}
