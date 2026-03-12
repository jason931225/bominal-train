//! Inline script component that connects to SSE and reloads on task updates.
//!
//! Since the app runs in SSR-only mode (no WASM hydration), this injects a
//! small `<script>` that uses the browser's native EventSource API. When a
//! `task_update` event arrives, the page reloads after a short debounce to
//! reflect the new state.

use leptos::prelude::*;

/// Injects an SSE listener script that auto-reloads the page on task updates.
///
/// Place this component on pages that display task data (home, tasks).
/// The script debounces reloads so rapid status changes don't thrash the page.
#[component]
pub fn SseReload() -> impl IntoView {
    view! {
        <script>{r#"
(function() {
    if (typeof EventSource === 'undefined') return;
    var timer = null;
    var es = new EventSource('/api/tasks/events');
    es.addEventListener('task_update', function() {
        if (timer) clearTimeout(timer);
        timer = setTimeout(function() { location.reload(); }, 800);
    });
    es.addEventListener('error', function() {
        es.close();
        // Reconnect after 10s on error
        setTimeout(function() { location.reload(); }, 10000);
    });
})();
"#}</script>
    }
}
