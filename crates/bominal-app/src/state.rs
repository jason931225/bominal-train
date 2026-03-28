//! Shared app state contexts for auth, theme, and SSE.

use leptos::prelude::{Get, RwSignal, Set, Update, provide_context, use_context};

use crate::types::{TaskEvent, UserInfo};

pub const THEME_COOKIE: &str = "bominal-theme";
pub const MODE_COOKIE: &str = "bominal-mode";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeName {
    Glass,
    ClearSky,
}

impl ThemeName {
    pub const fn as_str(self) -> &'static str {
        match self {
            ThemeName::Glass => "glass",
            ThemeName::ClearSky => "clear-sky",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "glass" => Some(Self::Glass),
            "clear-sky" => Some(Self::ClearSky),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
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

#[derive(Clone, Copy)]
pub struct AuthState {
    pub user: RwSignal<Option<UserInfo>>,
    pub loading: RwSignal<bool>,
    pub checked: RwSignal<bool>,
}

impl AuthState {
    pub fn is_authenticated(self) -> bool {
        self.user.get().is_some()
    }

    pub fn set_user(self, user: Option<UserInfo>) {
        self.user.set(user);
        self.checked.set(true);
        self.loading.set(false);
    }

    pub fn clear(self) {
        self.user.set(None);
        self.checked.set(false);
        self.loading.set(false);
    }
}

#[derive(Clone, Copy)]
pub struct ThemeState {
    pub theme: RwSignal<ThemeName>,
    pub mode: RwSignal<ThemeMode>,
}

impl ThemeState {
    pub fn set_theme(self, theme: ThemeName) {
        self.theme.set(theme);
    }

    pub fn set_mode(self, mode: ThemeMode) {
        self.mode.set(mode);
    }
}

#[derive(Clone, Copy)]
pub struct SseState {
    pub connected: RwSignal<bool>,
    pub event_count: RwSignal<u64>,
    pub last_event: RwSignal<Option<TaskEvent>>,
}

impl SseState {
    pub fn set_connected(self, connected: bool) {
        self.connected.set(connected);
    }

    pub fn push_event(self, event: TaskEvent) {
        self.event_count.update(|count| *count += 1);
        self.last_event.set(Some(event));
    }

    pub fn clear(self) {
        self.connected.set(false);
        self.last_event.set(None);
    }
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub auth: AuthState,
    pub theme: ThemeState,
    pub sse: SseState,
}

#[cfg(any(feature = "ssr", test))]
fn cookie_value<'a>(cookie_header: &'a str, name: &str) -> Option<&'a str> {
    cookie_header.split(';').find_map(|entry| {
        let (key, value) = entry.trim().split_once('=')?;
        (key == name).then_some(value)
    })
}

#[cfg(any(feature = "ssr", test))]
fn theme_prefs_from_cookie_header(cookie_header: Option<&str>) -> (ThemeName, ThemeMode) {
    let mut theme = ThemeName::Glass;
    let mut mode = ThemeMode::Light;

    if let Some(cookie_header) = cookie_header {
        if let Some(value) = cookie_value(cookie_header, THEME_COOKIE).and_then(ThemeName::parse) {
            theme = value;
        }
        if let Some(value) = cookie_value(cookie_header, MODE_COOKIE).and_then(ThemeMode::parse) {
            mode = value;
        }
    }

    (theme, mode)
}

pub(crate) fn request_theme_prefs() -> (ThemeName, ThemeMode) {
    #[cfg(feature = "ssr")]
    {
        if let Some(parts) = use_context::<axum::http::request::Parts>() {
            let cookie_header = parts
                .headers
                .get(axum::http::header::COOKIE)
                .and_then(|value| value.to_str().ok());
            return theme_prefs_from_cookie_header(cookie_header);
        }
    }

    (ThemeName::Glass, ThemeMode::Light)
}

pub fn provide_app_state() -> AppState {
    let (theme_name, theme_mode) = request_theme_prefs();
    let auth = AuthState {
        user: RwSignal::new(None),
        loading: RwSignal::new(true),
        checked: RwSignal::new(false),
    };
    let theme = ThemeState {
        theme: RwSignal::new(theme_name),
        mode: RwSignal::new(theme_mode),
    };
    let sse = SseState {
        connected: RwSignal::new(false),
        event_count: RwSignal::new(0),
        last_event: RwSignal::new(None),
    };
    let app_state = AppState { auth, theme, sse };

    provide_context(auth);
    provide_context(theme);
    provide_context(sse);
    provide_context(app_state);

    app_state
}

pub fn use_auth_state() -> AuthState {
    use_context::<AuthState>().expect("AuthState not provided")
}

pub fn use_theme_state() -> ThemeState {
    use_context::<ThemeState>().expect("ThemeState not provided")
}

pub fn use_sse_state() -> SseState {
    use_context::<SseState>().expect("SseState not provided")
}

pub fn use_app_state() -> AppState {
    use_context::<AppState>().expect("AppState not provided")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_prefs_are_parsed_from_cookie_header() {
        let (theme, mode) =
            theme_prefs_from_cookie_header(Some("bominal-theme=clear-sky; bominal-mode=dark"));
        assert_eq!(theme, ThemeName::ClearSky);
        assert_eq!(mode, ThemeMode::Dark);
    }

    #[test]
    fn invalid_theme_cookie_values_fall_back_to_defaults() {
        let (theme, mode) =
            theme_prefs_from_cookie_header(Some("bominal-theme=invalid; bominal-mode=unknown"));
        assert_eq!(theme, ThemeName::Glass);
        assert_eq!(mode, ThemeMode::Light);
    }
}
