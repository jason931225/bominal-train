use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
};
use bominal_shared::error::{ApiError, ApiErrorCode, ApiErrorStatus};

use super::{SetLocalePreferenceRequest, SetThemePreferenceRequest};
use super::super::super::{AppState, i18n, request_id_from_headers};

pub(super) async fn set_theme_preference(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<SetThemePreferenceRequest>,
) -> impl IntoResponse {
    let mode = payload.mode.trim().to_ascii_lowercase();
    if !matches!(mode.as_str(), "light" | "dark") {
        return ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            "mode must be one of: light, dark",
            request_id_from_headers(&headers),
        )
        .into_response();
    }
    let secure = if state.config.app_env.eq_ignore_ascii_case("production") {
        "; Secure"
    } else {
        ""
    };
    let domain = state
        .config
        .session_cookie_domain
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("; Domain={value}"))
        .unwrap_or_default();
    let cookie = format!(
        "{}={}; Path=/; SameSite=Lax; Max-Age=31536000{}{}",
        state.config.ui_theme_cookie_name, mode, secure, domain
    );
    let mut response = (StatusCode::OK, Json(serde_json::json!({ "mode": mode }))).into_response();
    match HeaderValue::from_str(&cookie) {
        Ok(value) => {
            response.headers_mut().append(SET_COOKIE, value);
            response
        }
        Err(_) => ApiError::new(
            ApiErrorStatus::InternalServerError,
            ApiErrorCode::InternalError,
            "failed to set theme cookie",
            request_id_from_headers(&headers),
        )
        .into_response(),
    }
}

pub(super) async fn set_locale_preference(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<SetLocalePreferenceRequest>,
) -> impl IntoResponse {
    let Some(locale) = i18n::parse_locale(&payload.locale) else {
        return ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            "locale must resolve to one of: en, ko, ja",
            request_id_from_headers(&headers),
        )
        .into_response();
    };

    let secure = if state.config.app_env.eq_ignore_ascii_case("production") {
        "; Secure"
    } else {
        ""
    };
    let domain = state
        .config
        .session_cookie_domain
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("; Domain={value}"))
        .unwrap_or_default();
    let locale_value = locale.as_cookie_value();
    let cookie = format!(
        "{}={}; Path=/; SameSite=Lax; Max-Age=31536000{}{}",
        i18n::UI_LOCALE_COOKIE_NAME,
        locale_value,
        secure,
        domain
    );

    let mut response = (
        StatusCode::OK,
        Json(serde_json::json!({ "locale": locale_value })),
    )
        .into_response();
    match HeaderValue::from_str(&cookie) {
        Ok(value) => {
            response.headers_mut().append(SET_COOKIE, value);
            response
        }
        Err(_) => ApiError::new(
            ApiErrorStatus::InternalServerError,
            ApiErrorCode::InternalError,
            "failed to set locale cookie",
            request_id_from_headers(&headers),
        )
        .into_response(),
    }
}
