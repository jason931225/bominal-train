//! App-local i18n helpers built on top of the canonical `bominal-domain`
//! translation and station registries.

use bominal_domain::i18n::{Locale, station_name as domain_station_name, t as domain_t};
use leptos::prelude::{provide_context, use_context};

pub const LOCALE_COOKIE: &str = "bominal-locale";

/// Parse the preferred locale from a cookie header.
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

/// Resolve the request locale from the incoming cookie header when available.
pub fn request_locale() -> Locale {
    #[cfg(feature = "ssr")]
    {
        if let Some(parts) = use_context::<axum::http::request::Parts>() {
            let cookie_header = parts
                .headers
                .get("cookie")
                .and_then(|value| value.to_str().ok());
            return locale_from_cookie_header(cookie_header);
        }
    }

    current_locale()
}

/// Provide a locale into Leptos context for the current request/view tree.
pub fn provide_locale(locale: Locale) {
    provide_context(locale);
}

/// Get the currently active locale from context, falling back to Korean.
pub fn current_locale() -> Locale {
    use_context::<Locale>().unwrap_or_default()
}

/// Translate a key using the active locale.
pub fn t(key: &str) -> &'static str {
    domain_t(current_locale(), key)
}

/// Translate a key for a specific locale.
pub fn t_locale(locale: Locale, key: &str) -> &'static str {
    domain_t(locale, key)
}

/// Locale-aware station name helper for provider-specific display differences.
pub fn station_name(locale: Locale, korean_name: &str, provider: &str) -> &'static str {
    domain_station_name(locale, korean_name, provider)
}

/// Available locale options for selectors and menus.
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
    fn invalid_locale_cookie_falls_back_to_default() {
        assert_eq!(
            locale_from_cookie_header(Some("bominal-locale=unknown")),
            Locale::Ko
        );
    }

    #[test]
    fn locale_options_cover_all_supported_locales() {
        assert_eq!(locale_options().len(), 3);
        assert_eq!(locale_options()[0].0, Locale::Ko);
        assert_eq!(locale_options()[1].0, Locale::En);
        assert_eq!(locale_options()[2].0, Locale::Ja);
    }

    #[test]
    fn translation_bridge_uses_domain_messages() {
        assert_eq!(t_locale(Locale::En, "nav.home"), "Home");
        assert_eq!(t_locale(Locale::Ja, "nav.home"), "ホーム");
    }
}
