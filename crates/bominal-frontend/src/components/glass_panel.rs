//! Glass morphism panel component — reusable container with backdrop blur.

use leptos::prelude::*;

/// Glass panel style variant.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum GlassPanelVariant {
    /// Standard panel background.
    #[default]
    Base,
    /// Card-style with tighter backdrop.
    Card,
    /// Active/selected state with accent tint.
    Active,
}

/// A glass-morphism styled container panel.
///
/// Wraps child content in a translucent card with backdrop blur,
/// matching the design system's `.glass-panel` CSS class.
#[component]
pub fn GlassPanel(
    /// Visual variant of the panel.
    #[prop(default = GlassPanelVariant::Base)]
    variant: GlassPanelVariant,
    /// Additional CSS classes.
    #[prop(into, default = String::new())]
    class: String,
    /// Whether to show hover lift effect.
    #[prop(default = false)]
    hover: bool,
    children: Children,
) -> impl IntoView {
    let base = match variant {
        GlassPanelVariant::Base => "glass-panel",
        GlassPanelVariant::Card => "glass-card",
        GlassPanelVariant::Active => "glass-card glass-active",
    };

    let hover_class = if hover { " glass-card-hover" } else { "" };

    let full_class = format!("{base}{hover_class} rounded-3xl overflow-hidden {class}");

    view! {
        <div class=full_class>
            {children()}
        </div>
    }
}
