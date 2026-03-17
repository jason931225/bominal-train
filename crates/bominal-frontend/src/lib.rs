//! Leptos SSR frontend for Bominal.
//!
//! Provides server-side rendered pages with selective WASM hydration.
//! All UI strings use the centralized i18n system from `bominal-domain`.

pub mod api;
pub mod app;
pub mod browser;
pub mod components;
pub mod i18n;
pub mod pages;
pub mod theme;
pub mod utils;

/// Evervault IDs for JS SDK initialization (provided via Leptos context).
#[derive(Clone, Debug)]
pub struct EvervaultIds {
    pub team_id: String,
    pub app_id: String,
}

/// Evervault Outbound Relay domains for provider API calls (provided via Leptos context).
#[derive(Clone, Debug)]
pub struct EvervaultRelay {
    pub srt_domain: String,
    pub ktx_domain: String,
}
