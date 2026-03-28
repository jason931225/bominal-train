use leptos::prelude::*;

#[component]
pub fn Skeleton(
    #[prop(into, default = "w-full".to_string())] width: String,
    #[prop(into, default = "h-4".to_string())] height: String,
    #[prop(into, default = String::new())] class: String,
) -> impl IntoView {
    view! {
        <div class=format!("lg-skeleton-line {width} {height} {class}") aria-hidden="true"></div>
    }
}

#[component]
pub fn SkeletonCard(#[prop(default = 3)] lines: usize) -> impl IntoView {
    view! {
        <div class="lg-skeleton-card">
            <Skeleton width="w-2/3" height="h-5" />
            {(0..lines).map(|index| {
                let width = if index + 1 == lines { "w-1/2" } else { "w-full" };
                view! { <Skeleton width=width height="h-3" /> }
            }).collect::<Vec<_>>()}
        </div>
    }
}
