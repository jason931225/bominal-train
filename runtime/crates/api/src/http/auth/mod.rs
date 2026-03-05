use std::sync::Arc;

use axum::{
    Router,
    http::HeaderMap,
    routing::{delete, get, post},
};
use bominal_shared::error::{ApiError, ApiErrorCode, ApiErrorStatus};

use super::super::{
    AppState, request_id_from_headers,
    services::{auth_service, passkey_service},
};

mod callbacks;
mod pages;
mod passkeys;
mod sessions;

#[derive(Debug, serde::Serialize)]
pub(super) struct SessionEnvelope {
    authenticated: bool,
    user: Option<auth_service::SessionUser>,
}

#[derive(Debug, serde::Deserialize)]
pub(super) struct SetThemePreferenceRequest {
    mode: String,
}

#[derive(Debug, serde::Deserialize)]
pub(super) struct SetLocalePreferenceRequest {
    locale: String,
}

pub(super) fn register(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route("/api/auth/password/signin", post(sessions::password_signin))
        .route("/api/auth/password/change", post(sessions::password_change))
        .route("/api/auth/session/logout", post(sessions::session_logout))
        .route("/api/auth/session/me", get(sessions::session_me))
        .route("/api/auth/passkeys", get(passkeys::passkeys_list))
        .route(
            "/api/auth/passkeys/{credential_id}",
            get(passkeys::passkey_get)
                .delete(passkeys::passkey_delete)
                .patch(passkeys::passkey_update),
        )
        .route("/api/auth/invite/accept", post(callbacks::invite_accept))
        .route(
            "/api/auth/passkeys/register/start",
            post(passkeys::passkey_register_start),
        )
        .route(
            "/api/auth/passkeys/register/finish",
            post(passkeys::passkey_register_finish),
        )
        .route(
            "/api/auth/passkeys/auth/start",
            post(passkeys::passkey_auth_start),
        )
        .route(
            "/api/auth/passkeys/auth/finish",
            post(passkeys::passkey_auth_finish),
        )
        .route(
            "/api/auth/step-up/passkey/start",
            post(passkeys::passkey_step_up_start),
        )
        .route(
            "/api/auth/step-up/passkey/finish",
            post(passkeys::passkey_step_up_finish),
        )
        .route("/api/ui/theme", post(pages::set_theme_preference))
        .route("/api/ui/locale", post(pages::set_locale_preference))
}

pub(super) fn map_auth_error(
    error: auth_service::AuthServiceError,
    headers: &HeaderMap,
) -> ApiError {
    match error {
        auth_service::AuthServiceError::InvalidRequest(message)
        | auth_service::AuthServiceError::Conflict(message) => ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            message,
            request_id_from_headers(headers),
        ),
        auth_service::AuthServiceError::Unauthorized(message)
        | auth_service::AuthServiceError::NotFound(message) => ApiError::new(
            ApiErrorStatus::Unauthorized,
            ApiErrorCode::Unauthorized,
            message,
            request_id_from_headers(headers),
        ),
        auth_service::AuthServiceError::ServiceUnavailable(message) => ApiError::new(
            ApiErrorStatus::ServiceUnavailable,
            ApiErrorCode::ServiceUnavailable,
            message,
            request_id_from_headers(headers),
        ),
        auth_service::AuthServiceError::Internal => ApiError::new(
            ApiErrorStatus::InternalServerError,
            ApiErrorCode::InternalError,
            "authentication service failure",
            request_id_from_headers(headers),
        ),
    }
}

pub(super) fn map_passkey_error(
    error: passkey_service::PasskeyFlowError,
    headers: &HeaderMap,
) -> ApiError {
    match error {
        passkey_service::PasskeyFlowError::Disabled => ApiError::new(
            ApiErrorStatus::ServiceUnavailable,
            ApiErrorCode::ServiceUnavailable,
            "passkey provider disabled",
            request_id_from_headers(headers),
        ),
        passkey_service::PasskeyFlowError::Unauthorized(message)
        | passkey_service::PasskeyFlowError::NotFound(message) => ApiError::new(
            ApiErrorStatus::Unauthorized,
            ApiErrorCode::Unauthorized,
            message,
            request_id_from_headers(headers),
        ),
        passkey_service::PasskeyFlowError::InvalidRequest(message) => ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            message,
            request_id_from_headers(headers),
        ),
        passkey_service::PasskeyFlowError::ServiceUnavailable(message) => ApiError::new(
            ApiErrorStatus::ServiceUnavailable,
            ApiErrorCode::ServiceUnavailable,
            message,
            request_id_from_headers(headers),
        ),
        passkey_service::PasskeyFlowError::Internal => ApiError::new(
            ApiErrorStatus::InternalServerError,
            ApiErrorCode::InternalError,
            "passkey flow failure",
            request_id_from_headers(headers),
        ),
    }
}

pub(super) fn client_ip_from_headers(headers: &HeaderMap) -> String {
    if let Some(value) = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        && let Some(first) = value
            .split(',')
            .map(str::trim)
            .find(|entry| !entry.is_empty())
    {
        return first.to_string();
    }

    if let Some(value) = headers
        .get("x-real-ip")
        .and_then(|value| value.to_str().ok())
    {
        let value = value.trim();
        if !value.is_empty() {
            return value.to_string();
        }
    }

    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderMap;

    use super::client_ip_from_headers;

    #[test]
    fn client_ip_prefers_x_forwarded_for_first_hop() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            "203.0.113.7, 10.0.0.4".parse().expect("valid header"),
        );
        headers.insert("x-real-ip", "198.51.100.9".parse().expect("valid header"));

        assert_eq!(client_ip_from_headers(&headers), "203.0.113.7");
    }

    #[test]
    fn client_ip_falls_back_to_unknown_when_missing() {
        let headers = HeaderMap::new();
        assert_eq!(client_ip_from_headers(&headers), "unknown");
    }
}
