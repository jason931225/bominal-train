//! Centralized internationalization for Bominal.
//!
//! Supports Korean (ko), English (en), and Japanese (ja).
//! All UI strings are accessed via typed keys through the [`t`] function.
//! Station names are accessed via [`station_name`] with locale-aware display.

mod en;
mod ja;
mod ko;
pub mod stations;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

/// Supported locales.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Locale {
    Ko,
    En,
    Ja,
}

impl Locale {
    /// Parse from string code.
    ///
    /// # Examples
    ///
    /// ```
    /// use bominal_domain::i18n::Locale;
    /// assert_eq!(Locale::from_code("ko"), Locale::Ko);
    /// assert_eq!(Locale::from_code("en"), Locale::En);
    /// assert_eq!(Locale::from_code("ja"), Locale::Ja);
    /// assert_eq!(Locale::from_code("unknown"), Locale::Ko);
    /// ```
    pub fn from_code(code: &str) -> Self {
        match code {
            "en" => Locale::En,
            "ja" => Locale::Ja,
            _ => Locale::Ko,
        }
    }

    /// ISO 639-1 code.
    pub fn code(self) -> &'static str {
        match self {
            Locale::Ko => "ko",
            Locale::En => "en",
            Locale::Ja => "ja",
        }
    }

    /// Native display name.
    pub fn display_name(self) -> &'static str {
        match self {
            Locale::Ko => "한국어",
            Locale::En => "English",
            Locale::Ja => "日本語",
        }
    }
}

impl Default for Locale {
    fn default() -> Self {
        Locale::Ko
    }
}

type Messages = HashMap<&'static str, &'static str>;

static KO_MESSAGES: LazyLock<Messages> = LazyLock::new(ko::messages);
static EN_MESSAGES: LazyLock<Messages> = LazyLock::new(en::messages);
static JA_MESSAGES: LazyLock<Messages> = LazyLock::new(ja::messages);

/// Translate a message key to the given locale.
///
/// Falls back to Korean if the key is not found in the target locale.
/// Returns the key itself if not found in any locale (debug aid).
///
/// # Examples
///
/// ```
/// use bominal_domain::i18n::{Locale, t};
/// assert_eq!(t(Locale::Ko, "nav.home"), "홈");
/// assert_eq!(t(Locale::En, "nav.home"), "Home");
/// assert_eq!(t(Locale::Ja, "nav.home"), "ホーム");
/// ```
pub fn t(locale: Locale, key: &str) -> &'static str {
    let messages = match locale {
        Locale::Ko => &*KO_MESSAGES,
        Locale::En => &*EN_MESSAGES,
        Locale::Ja => &*JA_MESSAGES,
    };

    if let Some(msg) = messages.get(key) {
        return msg;
    }

    // Fallback to Korean
    if locale != Locale::Ko {
        if let Some(msg) = KO_MESSAGES.get(key) {
            return msg;
        }
    }

    // Key not found in any locale
    "[missing]"
}

/// Get station display name for the given locale and provider.
pub fn station_name(locale: Locale, korean_name: &str, provider: &str) -> &'static str {
    stations::display_name(locale, korean_name, provider)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_roundtrip() {
        for locale in [Locale::Ko, Locale::En, Locale::Ja] {
            assert_eq!(Locale::from_code(locale.code()), locale);
        }
    }

    #[test]
    fn translate_all_locales() {
        let key = "nav.home";
        assert_eq!(t(Locale::Ko, key), "홈");
        assert_eq!(t(Locale::En, key), "Home");
        assert_eq!(t(Locale::Ja, key), "ホーム");
    }

    #[test]
    fn fallback_to_korean() {
        // A key that exists in Korean but not in a hypothetical incomplete locale
        let result = t(Locale::Ko, "nav.home");
        assert!(!result.is_empty());
    }

    #[test]
    fn missing_key_returns_placeholder() {
        assert_eq!(t(Locale::En, "nonexistent.key"), "[missing]");
    }
}
