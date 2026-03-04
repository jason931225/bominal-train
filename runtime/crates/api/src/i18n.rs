use axum::http::{HeaderMap, header};

pub(crate) const UI_LOCALE_COOKIE_NAME: &str = "bominal_locale";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UiLocale {
    En,
    Ko,
    Ja,
}

impl UiLocale {
    pub(crate) fn as_cookie_value(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ko => "ko",
            Self::Ja => "ja",
        }
    }

    pub(crate) fn as_html_lang(self) -> &'static str {
        self.as_cookie_value()
    }
}

pub(crate) fn parse_locale(raw: &str) -> Option<UiLocale> {
    let token = raw.trim();
    if token.is_empty() {
        return None;
    }

    let token = token
        .split(';')
        .next()
        .unwrap_or(token)
        .split(',')
        .next()
        .unwrap_or(token)
        .trim()
        .to_ascii_lowercase();
    if token.is_empty() {
        return None;
    }

    // Accept both IETF-style tags (`ja-JP`) and underscore variants (`ja_JP`).
    let normalized = token.replace('_', "-");
    let primary = normalized.split('-').next().unwrap_or(normalized.as_str());
    match primary {
        "en" => Some(UiLocale::En),
        "ko" | "kr" => Some(UiLocale::Ko),
        "ja" | "jp" => Some(UiLocale::Ja),
        _ => None,
    }
}

pub(crate) fn locale_from_headers(headers: &HeaderMap) -> UiLocale {
    if let Some(locale) = parse_locale_cookie(headers) {
        return locale;
    }

    if let Some(locale) = parse_accept_language(headers) {
        return locale;
    }

    UiLocale::En
}

fn parse_locale_cookie(headers: &HeaderMap) -> Option<UiLocale> {
    let raw_cookie = headers.get(header::COOKIE)?.to_str().ok()?;
    for pair in raw_cookie.split(';') {
        let mut parts = pair.trim().splitn(2, '=');
        let Some(key) = parts.next() else {
            continue;
        };
        let Some(value) = parts.next() else {
            continue;
        };
        if key == UI_LOCALE_COOKIE_NAME
            && let Some(locale) = parse_locale(value)
        {
            return Some(locale);
        }
    }
    None
}

fn parse_accept_language(headers: &HeaderMap) -> Option<UiLocale> {
    let raw = headers.get(header::ACCEPT_LANGUAGE)?.to_str().ok()?;
    for token in raw.split(',') {
        if let Some(locale) = parse_locale(token) {
            return Some(locale);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_locale_supports_legacy_and_regional_aliases() {
        assert_eq!(parse_locale("ja"), Some(UiLocale::Ja));
        assert_eq!(parse_locale("ja-JP"), Some(UiLocale::Ja));
        assert_eq!(parse_locale("ja_JP"), Some(UiLocale::Ja));
        assert_eq!(parse_locale("jp"), Some(UiLocale::Ja));
        assert_eq!(parse_locale("ko"), Some(UiLocale::Ko));
        assert_eq!(parse_locale("ko-KR"), Some(UiLocale::Ko));
        assert_eq!(parse_locale("ko_KR"), Some(UiLocale::Ko));
        assert_eq!(parse_locale("kr"), Some(UiLocale::Ko));
        assert_eq!(parse_locale("en-US"), Some(UiLocale::En));
    }
}
