//! Pre-built HTML email templates.
//!
//! All templates share a common dark-theme layout that matches
//! the Bominal app aesthetic (glass morphism, blue accents).

mod base;
pub mod newsletter;
pub mod reservation;
pub mod reset;
pub mod verify;

/// Escape HTML special characters to prevent XSS in email templates.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
