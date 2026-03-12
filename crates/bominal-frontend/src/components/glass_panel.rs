//! Glass morphism panel component — reusable container with backdrop blur.

use leptos::prelude::*;

/// A glass-morphism styled container panel.
///
/// Wraps child content in a translucent card with backdrop blur,
/// matching the design system's `.glass-panel` CSS class.
#[component]
pub fn GlassPanel(children: Children) -> impl IntoView {
    view! {
        <div class="glass-panel rounded-2xl overflow-hidden">
            {children()}
        </div>
    }
}
