//! Reservation task route handlers.

use axum::Json;
use axum::extract::{Path, State};
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::state::SharedState;

pub use bominal_service::tasks::{CreateTaskInput, UpdateTaskInput};

pub async fn create_task(
    user: AuthUser,
    State(state): State<SharedState>,
    Json(req): Json<CreateTaskInput>,
) -> Result<Json<bominal_service::tasks::TaskInfo>, AppError> {
    let result = bominal_service::tasks::create(&state.db, user.user_id, &req).await?;
    Ok(Json(result))
}

pub async fn list_tasks(
    user: AuthUser,
    State(state): State<SharedState>,
) -> Result<Json<Vec<bominal_service::tasks::TaskInfo>>, AppError> {
    let result = bominal_service::tasks::list(&state.db, user.user_id).await?;
    Ok(Json(result))
}

pub async fn get_task(
    user: AuthUser,
    State(state): State<SharedState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<bominal_service::tasks::TaskInfo>, AppError> {
    let result = bominal_service::tasks::get(&state.db, task_id, user.user_id).await?;
    Ok(Json(result))
}

pub async fn update_task(
    user: AuthUser,
    State(state): State<SharedState>,
    Path(task_id): Path<Uuid>,
    Json(req): Json<UpdateTaskInput>,
) -> Result<Json<bominal_service::tasks::TaskInfo>, AppError> {
    let result = bominal_service::tasks::update(&state.db, task_id, user.user_id, &req).await?;
    Ok(Json(result))
}

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
    use bominal_domain::task::TaskStatus;

    #[test]
    fn status_update_validation() {
        assert!(bominal_service::tasks::validate_status_update(TaskStatus::Queued).is_ok());
        assert!(bominal_service::tasks::validate_status_update(TaskStatus::Idle).is_ok());
        assert!(bominal_service::tasks::validate_status_update(TaskStatus::Running).is_err());
        assert!(bominal_service::tasks::validate_status_update(TaskStatus::Confirmed).is_err());
        assert!(bominal_service::tasks::validate_status_update(TaskStatus::Cancelled).is_err());
    }
}
