//! Status chip component — small colored badge for task/reservation status.

use leptos::prelude::*;

/// Display status of a task or reservation.
#[component]
pub fn StatusChip(
    /// The status text to display.
    #[prop(into)]
    label: String,
    /// CSS color variable name (e.g. "success", "warning", "error").
    #[prop(into)]
    variant: String,
) -> impl IntoView {
    let class = format!(
        "inline-flex items-center px-2 py-0.5 rounded-full text-[10px] font-medium bg-[var(--color-status-{v})]/20 text-[var(--color-status-{v})]",
        v = variant
    );

    view! {
        <span class=class>{label}</span>
    }
}
