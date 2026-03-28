use leptos::prelude::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GlassPanelVariant {
    #[default]
    Base,
    Card,
    Active,
}

#[component]
pub fn GlassPanel(
    #[prop(default = GlassPanelVariant::Base)] variant: GlassPanelVariant,
    #[prop(into, default = String::new())] class: String,
    #[prop(default = false)] hover: bool,
    children: Children,
) -> impl IntoView {
    let base = match variant {
        GlassPanelVariant::Base => "lg-glass-panel",
        GlassPanelVariant::Card => "lg-glass-panel lg-glass-panel--card",
        GlassPanelVariant::Active => "lg-glass-panel lg-glass-panel--card lg-glass-panel--active",
    };
    let hover_class = if hover { " lg-glass-panel--hover" } else { "" };

    view! {
        <div class=format!("{base}{hover_class} {class}")>{children()}</div>
    }
}
