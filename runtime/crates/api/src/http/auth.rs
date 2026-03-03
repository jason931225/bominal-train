use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
    routing::{delete, get, post},
};
use bominal_shared::error::{ApiError, ApiErrorCode, ApiErrorStatus};

use super::super::{
    AppState, request_id_from_headers,
    services::{auth_service, passkey_service},
};

#[derive(Debug, serde::Serialize)]
struct SessionEnvelope {
    authenticated: bool,
    user: Option<auth_service::SessionUser>,
}

pub(super) fn register(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route("/api/auth/password/signin", post(password_signin))
        .route("/api/auth/password/change", post(password_change))
        .route("/api/auth/session/logout", post(session_logout))
        .route("/api/auth/session/me", get(session_me))
        .route("/api/auth/passkeys", get(passkeys_list))
        .route("/api/auth/passkeys/{credential_id}", delete(passkey_delete))
        .route("/api/auth/invite/accept", post(invite_accept))
        .route(
            "/api/auth/passkeys/register/start",
            post(passkey_register_start),
        )
        .route(
            "/api/auth/passkeys/register/finish",
            post(passkey_register_finish),
        )
        .route("/api/auth/passkeys/auth/start", post(passkey_auth_start))
        .route("/api/auth/passkeys/auth/finish", post(passkey_auth_finish))
        .route(
            "/api/auth/step-up/passkey/start",
            post(passkey_step_up_start),
        )
        .route(
            "/api/auth/step-up/passkey/finish",
            post(passkey_step_up_finish),
        )
        .route("/api/ui/theme", post(set_theme_preference))
}

#[derive(Debug, serde::Deserialize)]
struct SetThemePreferenceRequest {
    mode: String,
}

async fn password_signin(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<auth_service::PasswordSigninRequest>,
) -> impl IntoResponse {
    let client_ip = client_ip_from_headers(&headers);
    match auth_service::signin_with_password(state.as_ref(), payload, &client_ip).await {
        Ok(user) => match auth_service::establish_session(state.as_ref(), &user).await {
            Ok(session_id) => {
                let set_cookie = auth_service::session_set_cookie(state.as_ref(), &session_id);
                let mut response = (
                    StatusCode::OK,
                    Json(SessionEnvelope {
                        authenticated: true,
                        user: Some(user),
                    }),
                )
                    .into_response();

                match HeaderValue::from_str(&set_cookie) {
                    Ok(value) => {
                        response.headers_mut().append(SET_COOKIE, value);
                        response
                    }
                    Err(_) => ApiError::new(
                        ApiErrorStatus::InternalServerError,
                        ApiErrorCode::InternalError,
                        "failed to construct session cookie",
                        request_id_from_headers(&headers),
                    )
                    .into_response(),
                }
            }
            Err(err) => map_auth_error(err, &headers).into_response(),
        },
        Err(err) => map_auth_error(err, &headers).into_response(),
    }
}

async fn password_change(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<auth_service::ChangePasswordRequest>,
) -> impl IntoResponse {
    match auth_service::change_session_password(state.as_ref(), &headers, payload).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "updated": true }))).into_response(),
        Err(err) => map_auth_error(err, &headers).into_response(),
    }
}

async fn invite_accept(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<auth_service::AcceptInviteRequest>,
) -> impl IntoResponse {
    match auth_service::accept_invite(state.as_ref(), payload).await {
        Ok(user) => match auth_service::establish_session(state.as_ref(), &user).await {
            Ok(session_id) => {
                let set_cookie = auth_service::session_set_cookie(state.as_ref(), &session_id);
                let mut response = (
                    StatusCode::OK,
                    Json(SessionEnvelope {
                        authenticated: true,
                        user: Some(user),
                    }),
                )
                    .into_response();

                match HeaderValue::from_str(&set_cookie) {
                    Ok(value) => {
                        response.headers_mut().append(SET_COOKIE, value);
                        response
                    }
                    Err(_) => ApiError::new(
                        ApiErrorStatus::InternalServerError,
                        ApiErrorCode::InternalError,
                        "failed to construct session cookie",
                        request_id_from_headers(&headers),
                    )
                    .into_response(),
                }
            }
            Err(err) => map_auth_error(err, &headers).into_response(),
        },
        Err(err) => map_auth_error(err, &headers).into_response(),
    }
}

async fn session_logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(err) = auth_service::revoke_session(state.as_ref(), &headers).await {
        return map_auth_error(err, &headers).into_response();
    }

    let clear_cookie = auth_service::session_clear_cookie(state.as_ref());
    let mut response = StatusCode::NO_CONTENT.into_response();
    match HeaderValue::from_str(&clear_cookie) {
        Ok(value) => {
            response.headers_mut().append(SET_COOKIE, value);
            response
        }
        Err(_) => ApiError::new(
            ApiErrorStatus::InternalServerError,
            ApiErrorCode::InternalError,
            "failed to clear session cookie",
            request_id_from_headers(&headers),
        )
        .into_response(),
    }
}

async fn session_me(State(state): State<Arc<AppState>>, headers: HeaderMap) -> impl IntoResponse {
    match auth_service::current_session_user(state.as_ref(), &headers).await {
        Ok(Some(user)) => (
            StatusCode::OK,
            Json(SessionEnvelope {
                authenticated: true,
                user: Some(user),
            }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::OK,
            Json(SessionEnvelope {
                authenticated: false,
                user: None,
            }),
        )
            .into_response(),
        Err(err) => map_auth_error(err, &headers).into_response(),
    }
}

async fn passkeys_list(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    match auth_service::list_session_passkeys(state.as_ref(), &headers).await {
        Ok(passkeys) => (
            StatusCode::OK,
            Json(serde_json::json!({ "passkeys": passkeys })),
        )
            .into_response(),
        Err(err) => map_auth_error(err, &headers).into_response(),
    }
}

async fn passkey_delete(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(credential_id): Path<String>,
) -> impl IntoResponse {
    match auth_service::delete_session_passkey(state.as_ref(), &headers, &credential_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_auth_error(err, &headers).into_response(),
    }
}

async fn passkey_register_start(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<passkey_service::StartPasskeyRegistrationRequest>,
) -> impl IntoResponse {
    match passkey_service::start_passkey_registration(state.as_ref(), &headers, payload).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(err) => map_passkey_error(err, &headers).into_response(),
    }
}

async fn passkey_register_finish(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<passkey_service::FinishPasskeyRegistrationRequest>,
) -> impl IntoResponse {
    match passkey_service::finish_passkey_registration(state.as_ref(), &headers, payload).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(err) => map_passkey_error(err, &headers).into_response(),
    }
}

async fn passkey_auth_start(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    match passkey_service::start_passkey_authentication(state.as_ref()).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(err) => map_passkey_error(err, &headers).into_response(),
    }
}

async fn passkey_auth_finish(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<passkey_service::FinishPasskeyAuthenticationRequest>,
) -> impl IntoResponse {
    match passkey_service::finish_passkey_authentication(state.as_ref(), payload).await {
        Ok(response) => {
            match auth_service::load_user_by_id(state.as_ref(), &response.user_id).await {
                Ok(user) => match auth_service::establish_session(state.as_ref(), &user).await {
                    Ok(session_id) => {
                        let set_cookie =
                            auth_service::session_set_cookie(state.as_ref(), &session_id);
                        let mut http_response = (
                            StatusCode::OK,
                            Json(SessionEnvelope {
                                authenticated: true,
                                user: Some(user),
                            }),
                        )
                            .into_response();
                        match HeaderValue::from_str(&set_cookie) {
                            Ok(value) => {
                                http_response.headers_mut().append(SET_COOKIE, value);
                                http_response
                            }
                            Err(_) => ApiError::new(
                                ApiErrorStatus::InternalServerError,
                                ApiErrorCode::InternalError,
                                "failed to construct session cookie",
                                request_id_from_headers(&headers),
                            )
                            .into_response(),
                        }
                    }
                    Err(err) => map_auth_error(err, &headers).into_response(),
                },
                Err(err) => map_auth_error(err, &headers).into_response(),
            }
        }
        Err(err) => map_passkey_error(err, &headers).into_response(),
    }
}

async fn passkey_step_up_start(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(err) = auth_service::require_session_state(state.as_ref(), &headers).await {
        return map_auth_error(err, &headers).into_response();
    }
    match passkey_service::start_passkey_authentication(state.as_ref()).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(err) => map_passkey_error(err, &headers).into_response(),
    }
}

async fn passkey_step_up_finish(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<passkey_service::FinishPasskeyAuthenticationRequest>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(value) => value,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };
    match passkey_service::finish_passkey_authentication(state.as_ref(), payload).await {
        Ok(result) => {
            if result.user_id != session.user_id {
                return ApiError::new(
                    ApiErrorStatus::Unauthorized,
                    ApiErrorCode::Unauthorized,
                    "step-up credential must match active session user",
                    request_id_from_headers(&headers),
                )
                .into_response();
            }
            match auth_service::mark_step_up_verified(state.as_ref(), &headers).await {
                Ok(updated) => (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "step_up_verified": true,
                        "verified_at": updated.step_up_verified_at,
                    })),
                )
                    .into_response(),
                Err(err) => map_auth_error(err, &headers).into_response(),
            }
        }
        Err(err) => map_passkey_error(err, &headers).into_response(),
    }
}

async fn set_theme_preference(
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

fn map_auth_error(error: auth_service::AuthServiceError, headers: &HeaderMap) -> ApiError {
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

fn client_ip_from_headers(headers: &HeaderMap) -> String {
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

fn map_passkey_error(error: passkey_service::PasskeyFlowError, headers: &HeaderMap) -> ApiError {
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
