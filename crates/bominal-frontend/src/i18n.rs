//! Frontend i18n helpers — locale detection and translation convenience.

use bominal_domain::i18n::{Locale, t as domain_t};
use leptos::prelude::use_context;

pub const LOCALE_COOKIE: &str = "bominal-locale";

pub fn locale_from_cookie_header(cookie_header: Option<&str>) -> Locale {
    cookie_header
        .and_then(|header| {
            header.split(';').find_map(|entry| {
                let (key, value) = entry.trim().split_once('=')?;
                (key == LOCALE_COOKIE).then_some(Locale::from_code(value))
            })
        })
        .unwrap_or_default()
}

pub fn current_locale() -> Locale {
    use_context::<Locale>().unwrap_or_default()
}

/// Translate a key using the current request locale.
pub fn t(key: &str) -> &'static str {
    domain_t(current_locale(), key)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_cookie_is_parsed() {
        assert_eq!(
            locale_from_cookie_header(Some("bominal-locale=ja; other=value")),
            Locale::Ja
        );
    }

    #[test]
    fn invalid_locale_cookie_falls_back() {
        assert_eq!(
            locale_from_cookie_header(Some("bominal-locale=unknown")),
            Locale::Ko
        );
    }
}
