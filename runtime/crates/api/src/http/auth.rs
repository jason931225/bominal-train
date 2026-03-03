use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
    routing::{get, post},
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
        .route("/api/auth/session/logout", post(session_logout))
        .route("/api/auth/session/me", get(session_me))
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
    {
        if let Some(first) = value
            .split(',')
            .map(str::trim)
            .find(|entry| !entry.is_empty())
        {
            return first.to_string();
        }
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
