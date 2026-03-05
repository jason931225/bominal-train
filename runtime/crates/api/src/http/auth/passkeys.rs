use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
};
use bominal_shared::error::{ApiError, ApiErrorCode, ApiErrorStatus};

use super::super::super::{
    AppState, request_id_from_headers,
    services::{auth_service, passkey_service},
};
use super::{SessionEnvelope, map_auth_error, map_passkey_error};

pub(super) async fn passkeys_list(
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

pub(super) async fn passkey_delete(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(credential_id): Path<String>,
) -> impl IntoResponse {
    match auth_service::delete_session_passkey(state.as_ref(), &headers, &credential_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_auth_error(err, &headers).into_response(),
    }
}

pub(super) async fn passkey_register_start(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<passkey_service::StartPasskeyRegistrationRequest>,
) -> impl IntoResponse {
    match passkey_service::start_passkey_registration(state.as_ref(), &headers, payload).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(err) => map_passkey_error(err, &headers).into_response(),
    }
}

pub(super) async fn passkey_register_finish(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<passkey_service::FinishPasskeyRegistrationRequest>,
) -> impl IntoResponse {
    match passkey_service::finish_passkey_registration(state.as_ref(), &headers, payload).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(err) => map_passkey_error(err, &headers).into_response(),
    }
}

pub(super) async fn passkey_auth_start(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    match passkey_service::start_passkey_authentication(state.as_ref()).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(err) => map_passkey_error(err, &headers).into_response(),
    }
}

pub(super) async fn passkey_auth_finish(
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

pub(super) async fn passkey_step_up_start(
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

pub(super) async fn passkey_step_up_finish(
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
