//! Leptos SSR frontend for Bominal.
//!
//! Provides server-side rendered pages with selective WASM hydration.
//! All UI strings use the centralized i18n system from `bominal-domain`.

pub mod api;
pub mod app;
pub mod components;
pub mod i18n;
pub mod pages;
