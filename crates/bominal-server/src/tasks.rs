//! Reservation task route handlers.
//!
//! - POST   /api/tasks      — create task
//! - GET    /api/tasks      — list tasks
//! - GET    /api/tasks/:id  — task detail
//! - PATCH  /api/tasks/:id  — update task
//! - DELETE /api/tasks/:id  — cancel task

use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::state::SharedState;

/// Create task request.
#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub provider: String,
    pub departure_station: String,
    pub arrival_station: String,
    pub travel_date: String,
    pub departure_time: String,
    pub passengers: serde_json::Value,
    pub seat_preference: String,
    pub target_trains: serde_json::Value,
    pub auto_pay: Option<bool>,
    pub payment_card_id: Option<Uuid>,
    pub notify_enabled: Option<bool>,
}

/// Update task request (all fields optional).
#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub status: Option<String>,
    pub notify_enabled: Option<bool>,
    pub auto_retry: Option<bool>,
    pub target_trains: Option<serde_json::Value>,
}

/// POST /api/tasks — create a new reservation task.
pub async fn create_task(
    user: AuthUser,
    State(state): State<SharedState>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Json<bominal_service::tasks::TaskInfo>, AppError> {
    let result = bominal_service::tasks::create(
        &state.db,
        user.user_id,
        &req.provider,
        &req.departure_station,
        &req.arrival_station,
        &req.travel_date,
        &req.departure_time,
        &req.passengers,
        &req.seat_preference,
        &req.target_trains,
        req.auto_pay.unwrap_or(false),
        req.payment_card_id,
        req.notify_enabled.unwrap_or(false),
    )
    .await?;

    Ok(Json(result))
}

/// GET /api/tasks — list all tasks for the user.
pub async fn list_tasks(
    user: AuthUser,
    State(state): State<SharedState>,
) -> Result<Json<Vec<bominal_service::tasks::TaskInfo>>, AppError> {
    let result = bominal_service::tasks::list(&state.db, user.user_id).await?;
    Ok(Json(result))
}

/// GET /api/tasks/:id — get task detail.
pub async fn get_task(
    user: AuthUser,
    State(state): State<SharedState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<bominal_service::tasks::TaskInfo>, AppError> {
    let result = bominal_service::tasks::get(&state.db, task_id, user.user_id).await?;
    Ok(Json(result))
}

/// PATCH /api/tasks/:id — update task.
pub async fn update_task(
    user: AuthUser,
    State(state): State<SharedState>,
    Path(task_id): Path<Uuid>,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Json<bominal_service::tasks::TaskInfo>, AppError> {
    let result = bominal_service::tasks::update(
        &state.db,
        task_id,
        user.user_id,
        req.status.as_deref(),
        req.notify_enabled,
        req.auto_retry,
        req.target_trains.as_ref(),
    )
    .await?;

    Ok(Json(result))
}

/// DELETE /api/tasks/:id — cancel task.
pub async fn delete_task(
    user: AuthUser,
    State(state): State<SharedState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    bominal_service::tasks::delete(&state.db, task_id, user.user_id).await?;
    Ok(Json(serde_json::json!({ "cancelled": true })))
}

#[cfg(test)]
mod tests {
    #[test]
    fn validate_provider() {
        assert!(bominal_service::tasks::validate_provider("SRT").is_ok());
        assert!(bominal_service::tasks::validate_provider("KTX").is_ok());
        assert!(bominal_service::tasks::validate_provider("INVALID").is_err());
    }

    #[test]
    fn status_update_validation() {
        assert!(bominal_service::tasks::validate_status_update("queued").is_ok());
        assert!(bominal_service::tasks::validate_status_update("idle").is_ok());
        assert!(bominal_service::tasks::validate_status_update("running").is_err());
        assert!(bominal_service::tasks::validate_status_update("confirmed").is_err());
        assert!(bominal_service::tasks::validate_status_update("cancelled").is_err());
    }
}
