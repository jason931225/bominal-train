use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
};
use bominal_shared::error::{ApiError, ApiErrorCode, ApiErrorStatus};

use super::super::super::{AppState, request_id_from_headers, services::auth_service};
use super::{SessionEnvelope, map_auth_error};

pub(super) async fn invite_accept(
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
