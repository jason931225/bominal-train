//! Status chip component — small colored badge for task/reservation status.

use leptos::prelude::*;

/// Display status of a task or reservation.
#[component]
pub fn StatusChip(
    /// The status text to display.
    #[prop(into)]
    label: String,
    /// CSS variant name (e.g. "success", "warning", "error", "queued", "running").
    #[prop(into)]
    variant: String,
) -> impl IntoView {
    let status_class = match variant.as_str() {
        "queued" => "status-queued",
        "running" => "status-running",
        "success" => "status-success",
        "error" => "status-error",
        "warning" => "status-warning",
        "info" => "status-info",
        _ => "status-neutral",
    };

    let class = format!(
        "inline-flex items-center px-2 py-0.5 rounded-full text-[10px] font-medium {status_class}"
    );

    view! {
        <span role="status" class=class>{label}</span>
    }
}
