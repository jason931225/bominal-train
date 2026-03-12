//! Main Leptos application shell with routing.
//!
//! Routes: `/` (auth), `/home`, `/search`, `/search/results`, `/tasks`, `/settings`

use leptos::prelude::*;
use leptos_meta::{Meta, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::components::bottom_nav::BottomNav;
use crate::i18n::t;
use crate::pages::{
    auth_page::AuthPage, home_view::HomeView, reservations_view::ReservationsView,
    schedule_results::ScheduleResults, search_panel::SearchPanel,
    settings_view::SettingsView, tasks_view::TasksView,
};

/// HTML shell for SSR — renders the full `<html>` document.
pub fn shell() -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="ko" data-theme="dark" data-palette="current" data-colorblind="false">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover" />
                <meta name="color-scheme" content="dark light" />
                <meta name="theme-color" content="#0f172a" />
                <meta name="apple-mobile-web-app-capable" content="yes" />
                <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent" />
                <link rel="stylesheet" href="/style.css" />
                // Theme init script — runs before paint to prevent FOUC
                <script>{r#"
(function(){
  var h=document.documentElement;
  var t=localStorage.getItem('bominal-theme')||'dark';
  var p=localStorage.getItem('bominal-palette')||'current';
  var c=localStorage.getItem('bominal-colorblind')||'false';
  h.setAttribute('data-theme',t);
  h.setAttribute('data-palette',p);
  h.setAttribute('data-colorblind',c);
  window.__bSetTheme=function(v){h.setAttribute('data-theme',v);localStorage.setItem('bominal-theme',v);};
  window.__bSetPalette=function(v){h.setAttribute('data-palette',v);localStorage.setItem('bominal-palette',v);};
  window.__bSetColorblind=function(v){h.setAttribute('data-colorblind',v);localStorage.setItem('bominal-colorblind',v);};
})();
"#}</script>
            </head>
            <body class="min-h-screen bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] font-[var(--font-sans)] antialiased">
                <App />
            </body>
        </html>
    }
}

/// Root application component with meta context and routing.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="Bominal" />
        <Meta name="description" content="Korean train reservation assistant" />
        <Stylesheet href="/style.css" />

        <Router>
            <div class="relative min-h-screen pb-16">
                <main>
                    <Routes fallback=|| view! {
                        <div class="flex items-center justify-center min-h-screen">
                            <p class="text-[var(--color-text-secondary)]">{t("error.not_found")}</p>
                        </div>
                    }>
                        <Route path=path!("/") view=AuthPage />
                        <Route path=path!("/home") view=HomeView />
                        <Route path=path!("/search") view=SearchPanel />
                        <Route path=path!("/search/results") view=ScheduleResults />
                        <Route path=path!("/tasks") view=TasksView />
                        <Route path=path!("/reservations") view=ReservationsView />
                        <Route path=path!("/settings") view=SettingsView />
                    </Routes>
                </main>
                <BottomNav />
            </div>
        </Router>
    }
}
