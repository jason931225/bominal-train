use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
};
use bominal_shared::error::{ApiError, ApiErrorCode, ApiErrorStatus};

use super::super::super::{AppState, request_id_from_headers, services::auth_service};

use super::{SessionEnvelope, client_ip_from_headers, map_auth_error};

pub(super) async fn password_signin(
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

pub(super) async fn password_change(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<auth_service::ChangePasswordRequest>,
) -> impl IntoResponse {
    match auth_service::change_session_password(state.as_ref(), &headers, payload).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "updated": true }))).into_response(),
        Err(err) => map_auth_error(err, &headers).into_response(),
    }
}

pub(super) async fn session_logout(
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

pub(super) async fn session_me(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
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
