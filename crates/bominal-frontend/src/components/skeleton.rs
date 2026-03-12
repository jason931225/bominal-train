//! Skeleton loading component — shimmer placeholders.

use leptos::prelude::*;

/// A skeleton loading placeholder with shimmer animation.
#[component]
pub fn Skeleton(
    /// Width class (e.g., "w-full", "w-32").
    #[prop(into, default = "w-full".to_string())]
    width: String,
    /// Height class (e.g., "h-4", "h-12").
    #[prop(into, default = "h-4".to_string())]
    height: String,
    /// Additional classes.
    #[prop(into, default = String::new())]
    class: String,
) -> impl IntoView {
    view! {
        <div class=format!("{width} {height} rounded-lg shimmer {class}")></div>
    }
}

/// A skeleton card placeholder.
#[component]
pub fn SkeletonCard(
    /// Number of text lines to show.
    #[prop(default = 3)]
    lines: usize,
) -> impl IntoView {
    view! {
        <div class="glass-card rounded-3xl p-4 space-y-3">
            <Skeleton width="w-2/3" height="h-5" />
            {(0..lines).map(|i| {
                let w = if i == lines - 1 { "w-1/2" } else { "w-full" };
                view! { <Skeleton width=w height="h-3" /> }
            }).collect::<Vec<_>>()}
        </div>
    }
}
