//! Small colored status badge used across the core pages.

use leptos::prelude::*;

#[component]
pub fn StatusChip(#[prop(into)] label: String, #[prop(into)] variant: String) -> impl IntoView {
    view! {
        <span class=format!("lg-status-chip lg-status-chip--{}", variant.to_lowercase())>
            {label}
        </span>
    }
}
