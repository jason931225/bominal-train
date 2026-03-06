use std::sync::Arc;

use async_stream::stream;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{
        IntoResponse,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{get, post, put},
};
use bominal_shared::error::{ApiError, ApiErrorCode, ApiErrorStatus};
use std::{convert::Infallible, time::Duration};
use tracing::{error, warn};

use super::super::{
    AppState, request_id_from_headers,
    services::{auth_service, payment_method_service, train_service},
};
use super::sse as canonical_sse;

pub(super) fn register(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route("/api/train/preflight", get(get_preflight))
        .route("/api/train/stations/regions", get(get_station_regions))
        .route("/api/train/stations/suggest", get(get_station_suggestions))
        .route(
            "/api/train/stations/favorites",
            put(update_station_favorites),
        )
        .route(
            "/api/train/search",
            post(create_search).get(list_search_history),
        )
        .route("/api/train/search/{search_id}", get(get_search))
        .route(
            "/api/train/providers/{provider}/search",
            post(search_provider_direct),
        )
        .route(
            "/api/train/tasks",
            post(create_train_task).get(list_train_tasks),
        )
        .route("/api/train/tasks/{task_id}", get(get_train_task))
        .route(
            "/api/train/tasks/{task_id}/state",
            post(update_train_task_state),
        )
        .route(
            "/api/train/tasks/{task_id}/stream",
            get(stream_train_task_events),
        )
        .route(
            "/api/train/providers/{provider}/credentials",
            get(get_provider_credentials)
                .put(put_provider_credentials)
                .delete(delete_provider_credentials),
        )
        .route(
            "/api/train/providers/{provider}/payment-method",
            get(get_provider_payment_method).put(put_provider_payment_method),
        )
        .route(
            "/api/train/payment-methods",
            get(list_payment_methods).put(put_universal_payment_method),
        )
        .route(
            "/api/train/payment-methods/{payment_method_ref}",
            get(get_payment_method).delete(delete_payment_method),
        )
}

async fn get_preflight(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::load_preflight(state.as_ref(), &session.user_id).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn get_station_suggestions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<train_service::StationSuggestQuery>,
) -> impl IntoResponse {
    if let Err(err) = auth_service::require_session_state(state.as_ref(), &headers).await {
        return map_auth_error(err, &headers).into_response();
    }

    match train_service::suggest_stations(state.as_ref(), query).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn get_station_regions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<train_service::StationRegionsQuery>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::load_station_regions(state.as_ref(), &session.user_id, query).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn update_station_favorites(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<train_service::UpdateStationFavoritesRequest>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::replace_station_favorites(state.as_ref(), &session.user_id, payload).await
    {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn create_search(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<train_service::CreateTrainSearchRequest>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::create_search(state.as_ref(), &session.user_id, payload).await {
        Ok(result) => (StatusCode::ACCEPTED, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn get_search(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(search_id): Path<String>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::get_search(state.as_ref(), &session.user_id, &search_id).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn search_provider_direct(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(provider): Path<String>,
    Json(payload): Json<train_service::DirectTrainSearchRequest>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::search_provider_direct(
        state.as_ref(),
        &session.user_id,
        &provider,
        payload,
    )
    .await
    {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn create_train_task(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<train_service::CreateTrainTaskRequest>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };
    match train_service::create_train_task(state.as_ref(), &session.user_id, payload).await {
        Ok(result) => (StatusCode::ACCEPTED, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn list_train_tasks(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<SearchHistoryQuery>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };
    let limit = query.limit.unwrap_or(20);
    match train_service::list_train_tasks(state.as_ref(), &session.user_id, limit).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

#[derive(Debug, serde::Deserialize)]
struct SearchHistoryQuery {
    #[serde(default)]
    limit: Option<usize>,
}

async fn get_train_task(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };
    match train_service::get_train_task(state.as_ref(), &session.user_id, &task_id).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn update_train_task_state(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    Json(payload): Json<train_service::TrainTaskStateUpdateRequest>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };
    match train_service::update_train_task_state(
        state.as_ref(),
        &session.user_id,
        &task_id,
        &payload.action,
    )
    .await
    {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

#[derive(Debug, serde::Deserialize)]
struct TaskEventStreamQuery {
    #[serde(default)]
    after_id: Option<i64>,
    #[serde(default)]
    limit: Option<usize>,
}

async fn stream_train_task_events(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    Query(query): Query<TaskEventStreamQuery>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };
    let snapshot =
        match train_service::get_train_task(state.as_ref(), &session.user_id, &task_id).await {
            Ok(task) => task,
            Err(err) => return map_train_error(err, &headers).into_response(),
        };
    let after_id = query.after_id.unwrap_or(0).max(0);
    let limit = query.limit.unwrap_or(100).clamp(1, 200);
    let session_user_id = session.user_id;
    let event_stream = stream! {
        let sync_id = uuid::Uuid::new_v4().to_string();
        let mut seq: u64 = 0;
        let mut cursor = after_id;
        yield Ok::<Event, Infallible>(
            canonical_sse::sync_event(
                "train.task_events",
                "train_task",
                task_id.as_str(),
                &sync_id,
                seq,
                "train_task.v1",
                serde_json::json!({
                    "task": snapshot,
                    "events": [],
                }),
            ),
        );
        loop {
            match train_service::list_train_task_events_page(
                state.as_ref(),
                &session_user_id,
                &task_id,
                cursor,
                limit,
            )
            .await {
                Ok(events) => {
                    if !events.is_empty() {
                        if let Some(last) = events.last() {
                            cursor = last.id;
                        }
                        match train_service::get_train_task(state.as_ref(), &session_user_id, &task_id).await {
                            Ok(task) => {
                                seq = seq.saturating_add(1);
                                let mut ops = vec![canonical_sse::op_upsert("/task", serde_json::json!(task))];
                                for event in events {
                                    ops.push(canonical_sse::op_append("/events", serde_json::json!(event)));
                                }
                                yield Ok::<Event, Infallible>(
                                    canonical_sse::delta_event(
                                        "train.task_events",
                                        "train_task",
                                        task_id.as_str(),
                                        &sync_id,
                                        seq,
                                        "train_task.v1",
                                        ops,
                                    ),
                                );
                            }
                            Err(_) => {
                                yield Ok::<Event, Infallible>(
                                    canonical_sse::error_event("train task stream snapshot unavailable"),
                                );
                            }
                        }
                    }
                }
                Err(_) => {
                    yield Ok::<Event, Infallible>(
                        canonical_sse::error_event("train task stream temporarily unavailable"),
                    );
                }
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    };

    Sse::new(event_stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("heartbeat"),
        )
        .into_response()
}

async fn list_search_history(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<train_service::SearchHistoryQuery>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    let limit = query.limit.unwrap_or(20);
    match train_service::list_search_history(state.as_ref(), &session.user_id, limit).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn put_provider_credentials(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(provider): Path<String>,
    Json(payload): Json<train_service::PutTrainProviderCredentialsRequest>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::put_provider_credentials_for_user(
        state.as_ref(),
        &session.user_id,
        &provider,
        payload,
    )
    .await
    {
        Ok(result) => (StatusCode::ACCEPTED, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn get_provider_credentials(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(provider): Path<String>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::get_provider_credentials_for_user(
        state.as_ref(),
        &session.user_id,
        &provider,
    )
    .await
    {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn delete_provider_credentials(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(provider): Path<String>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::delete_provider_credentials_for_user(
        state.as_ref(),
        &session.user_id,
        &provider,
    )
    .await
    {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn get_provider_payment_method(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(provider): Path<String>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::get_provider_payment_method_for_user(
        state.as_ref(),
        &session.user_id,
        &provider,
    )
    .await
    {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn put_provider_payment_method(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(provider): Path<String>,
    Json(payload): Json<train_service::PutTrainPaymentMethodRequest>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::put_payment_method_for_user(
        state.as_ref(),
        &session.user_id,
        &provider,
        payload,
    )
    .await
    {
        Ok(result) => (StatusCode::ACCEPTED, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn put_universal_payment_method(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<train_service::PutTrainPaymentMethodRequest>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::put_payment_method_for_user(
        state.as_ref(),
        &session.user_id,
        payment_method_service::UNIVERSAL_PAYMENT_PROVIDER,
        payload,
    )
    .await
    {
        Ok(result) => (StatusCode::ACCEPTED, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn list_payment_methods(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::list_payment_methods_for_user(state.as_ref(), &session.user_id).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn get_payment_method(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(payment_method_ref): Path<String>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::get_payment_method_for_user(
        state.as_ref(),
        &session.user_id,
        &payment_method_ref,
    )
    .await
    {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
    }
}

async fn delete_payment_method(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(payment_method_ref): Path<String>,
) -> impl IntoResponse {
    let session = match auth_service::require_session_state(state.as_ref(), &headers).await {
        Ok(session) => session,
        Err(err) => return map_auth_error(err, &headers).into_response(),
    };

    match train_service::delete_payment_method_for_user(
        state.as_ref(),
        &session.user_id,
        &payment_method_ref,
    )
    .await
    {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => map_train_error(err, &headers).into_response(),
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

fn map_train_error(error: train_service::TrainServiceError, headers: &HeaderMap) -> ApiError {
    let request_id = request_id_from_headers(headers);
    match &error {
        train_service::TrainServiceError::ServiceUnavailable(message) => {
            warn!(message = %message, "train service unavailable");
        }
        train_service::TrainServiceError::Internal => {
            error!("train service internal failure");
        }
        _ => {}
    }

    match error {
        train_service::TrainServiceError::InvalidRequest(message) => ApiError::new(
            ApiErrorStatus::BadRequest,
            ApiErrorCode::InvalidRequest,
            message,
            request_id.clone(),
        ),
        train_service::TrainServiceError::Unauthorized(message) => ApiError::new(
            ApiErrorStatus::Unauthorized,
            ApiErrorCode::Unauthorized,
            message,
            request_id.clone(),
        ),
        train_service::TrainServiceError::NotFound(message) => ApiError::new(
            ApiErrorStatus::Unauthorized,
            ApiErrorCode::Unauthorized,
            message,
            request_id.clone(),
        ),
        train_service::TrainServiceError::ServiceUnavailable(message) => ApiError::new(
            ApiErrorStatus::ServiceUnavailable,
            ApiErrorCode::ServiceUnavailable,
            message,
            request_id.clone(),
        ),
        train_service::TrainServiceError::Internal => ApiError::new(
            ApiErrorStatus::InternalServerError,
            ApiErrorCode::InternalError,
            "train service failure",
            request_id,
        ),
    }
}
