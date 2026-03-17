//! Shared theme preferences for SSR and browser updates.

pub const THEME_COOKIE: &str = "bominal-theme";
pub const MODE_COOKIE: &str = "bominal-mode";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeName {
    #[default]
    Rosewood,
    ClearSky,
}

impl ThemeName {
    pub const fn as_str(self) -> &'static str {
        match self {
            ThemeName::Rosewood => "rosewood",
            ThemeName::ClearSky => "clear-sky",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "rosewood" => Some(Self::Rosewood),
            "clear-sky" => Some(Self::ClearSky),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

impl ThemeMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ThemePrefs {
    pub theme: ThemeName,
    pub mode: ThemeMode,
}

impl ThemePrefs {
    pub fn from_cookie_header(cookie_header: Option<&str>) -> Self {
        let mut prefs = Self::default();

        if let Some(cookie_header) = cookie_header {
            if let Some(theme) =
                cookie_value(cookie_header, THEME_COOKIE).and_then(ThemeName::parse)
            {
                prefs.theme = theme;
            }
            if let Some(mode) = cookie_value(cookie_header, MODE_COOKIE).and_then(ThemeMode::parse)
            {
                prefs.mode = mode;
            }
        }

        prefs
    }
}

fn cookie_value<'a>(cookie_header: &'a str, name: &str) -> Option<&'a str> {
    cookie_header.split(';').find_map(|entry| {
        let (key, value) = entry.trim().split_once('=')?;
        (key == name).then_some(value)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_theme_prefs_from_cookie_header() {
        let prefs = ThemePrefs::from_cookie_header(Some(
            "bominal-theme=clear-sky; bominal-mode=dark; bominal-locale=ko",
        ));
        assert_eq!(prefs.theme, ThemeName::ClearSky);
        assert_eq!(prefs.mode, ThemeMode::Dark);
    }

    #[test]
    fn falls_back_on_invalid_cookie_values() {
        let prefs =
            ThemePrefs::from_cookie_header(Some("bominal-theme=invalid; bominal-mode=unknown"));
        assert_eq!(prefs, ThemePrefs::default());
    }
}
