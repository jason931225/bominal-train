//! Frontend i18n helpers — locale detection and translation convenience.
//!
//! In the current SSR-only mode, locale defaults to Korean.
//! When WASM hydration is added, this will read from a signal or cookie.

use bominal_domain::i18n::{Locale, t as domain_t};

/// Translate a key using the current locale (defaults to Korean in SSR-only mode).
pub fn t(key: &str) -> &'static str {
    domain_t(Locale::Ko, key)
}

/// Translate a key for a specific locale.
pub fn t_locale(locale: Locale, key: &str) -> &'static str {
    domain_t(locale, key)
}

/// All supported locale options for the language selector.
pub fn locale_options() -> &'static [(Locale, &'static str)] {
    &[
        (Locale::Ko, "한국어"),
        (Locale::En, "English"),
        (Locale::Ja, "日本語"),
    ]
}
