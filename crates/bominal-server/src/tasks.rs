//! Reservation task route handlers.
//!
//! - POST   /api/tasks      — create task
//! - GET    /api/tasks      — list tasks
//! - GET    /api/tasks/:id  — task detail
//! - PATCH  /api/tasks/:id  — update task
//! - DELETE /api/tasks/:id  — cancel task

use axum::extract::{Path, State};
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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
    pub target_trains: Option<serde_json::Value>,
}

/// Task response (API representation).
#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: Uuid,
    pub provider: String,
    pub departure_station: String,
    pub arrival_station: String,
    pub travel_date: String,
    pub departure_time: String,
    pub passengers: serde_json::Value,
    pub seat_preference: String,
    pub target_trains: serde_json::Value,
    pub auto_pay: bool,
    pub payment_card_id: Option<Uuid>,
    pub notify_enabled: bool,
    pub status: String,
    pub reservation_number: Option<String>,
    pub attempt_count: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// POST /api/tasks — create a new reservation task.
pub async fn create_task(
    user: AuthUser,
    State(state): State<SharedState>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    validate_task_request(&req)?;

    // Verify user has valid provider credentials
    let cred = bominal_db::provider::find_by_user_and_provider(
        &state.db,
        user.user_id,
        &req.provider,
    )
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    match &cred {
        Some(c) if c.status == "valid" => {}
        _ => {
            return Err(AppError::BadRequest(format!(
                "Valid {} credentials required to create a task",
                req.provider
            )));
        }
    }

    let row = bominal_db::task::create_task(
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
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(task_to_response(&row)))
}

/// GET /api/tasks — list all tasks for the user.
pub async fn list_tasks(
    user: AuthUser,
    State(state): State<SharedState>,
) -> Result<Json<Vec<TaskResponse>>, AppError> {
    let rows = bominal_db::task::find_by_user(&state.db, user.user_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(rows.iter().map(task_to_response).collect()))
}

/// GET /api/tasks/:id — get task detail.
pub async fn get_task(
    user: AuthUser,
    State(state): State<SharedState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TaskResponse>, AppError> {
    let row = bominal_db::task::find_by_id(&state.db, task_id, user.user_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

    Ok(Json(task_to_response(&row)))
}

/// PATCH /api/tasks/:id — update task.
pub async fn update_task(
    user: AuthUser,
    State(state): State<SharedState>,
    Path(task_id): Path<Uuid>,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    // Validate status transition if provided
    if let Some(status) = &req.status {
        validate_status_update(status)?;
    }

    let row = bominal_db::task::update_task(
        &state.db,
        task_id,
        user.user_id,
        req.status.as_deref(),
        req.notify_enabled,
        req.target_trains.as_ref(),
    )
    .await
    .map_err(|e| AppError::Internal(e.into()))?
    .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

    Ok(Json(task_to_response(&row)))
}

/// DELETE /api/tasks/:id — cancel task.
pub async fn delete_task(
    user: AuthUser,
    State(state): State<SharedState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let cancelled = bominal_db::task::delete_task(&state.db, task_id, user.user_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    if !cancelled {
        return Err(AppError::NotFound(
            "Task not found or already in terminal state".to_string(),
        ));
    }

    Ok(Json(serde_json::json!({ "cancelled": true })))
}

// ── Helpers ──────────────────────────────────────────────────────────

fn validate_task_request(req: &CreateTaskRequest) -> Result<(), AppError> {
    if req.provider != "SRT" && req.provider != "KTX" {
        return Err(AppError::BadRequest(format!(
            "Invalid provider: {}",
            req.provider
        )));
    }
    if req.departure_station.is_empty() || req.arrival_station.is_empty() {
        return Err(AppError::BadRequest(
            "Departure and arrival stations are required".to_string(),
        ));
    }
    if req.departure_station == req.arrival_station {
        return Err(AppError::BadRequest(
            "Departure and arrival stations must be different".to_string(),
        ));
    }
    if req.travel_date.len() != 8 || !req.travel_date.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::BadRequest(
            "Travel date must be YYYYMMDD format".to_string(),
        ));
    }
    if req.departure_time.len() != 6 || !req.departure_time.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::BadRequest(
            "Departure time must be HHMMSS format".to_string(),
        ));
    }
    if req.target_trains.as_array().map(|a| a.is_empty()).unwrap_or(true) {
        return Err(AppError::BadRequest(
            "At least one target train is required".to_string(),
        ));
    }
    Ok(())
}

fn validate_status_update(status: &str) -> Result<(), AppError> {
    match status {
        "queued" | "idle" => Ok(()),
        _ => Err(AppError::BadRequest(format!(
            "Cannot manually set status to '{status}'. Use DELETE to cancel."
        ))),
    }
}

fn task_to_response(row: &bominal_db::task::TaskRow) -> TaskResponse {
    TaskResponse {
        id: row.id,
        provider: row.provider.clone(),
        departure_station: row.departure_station.clone(),
        arrival_station: row.arrival_station.clone(),
        travel_date: row.travel_date.clone(),
        departure_time: row.departure_time.clone(),
        passengers: row.passengers.clone(),
        seat_preference: row.seat_preference.clone(),
        target_trains: row.target_trains.clone(),
        auto_pay: row.auto_pay,
        payment_card_id: row.payment_card_id,
        notify_enabled: row.notify_enabled,
        status: row.status.clone(),
        reservation_number: row.reservation_number.clone(),
        attempt_count: row.attempt_count,
        started_at: row.started_at,
        last_attempt_at: row.last_attempt_at,
        created_at: row.created_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_provider() {
        let req = CreateTaskRequest {
            provider: "INVALID".to_string(),
            departure_station: "수서".to_string(),
            arrival_station: "부산".to_string(),
            travel_date: "20260315".to_string(),
            departure_time: "090000".to_string(),
            passengers: serde_json::json!([{"type": "adult", "count": 1}]),
            seat_preference: "GeneralFirst".to_string(),
            target_trains: serde_json::json!([{"train_number": "305"}]),
            auto_pay: None,
            payment_card_id: None,
            notify_enabled: None,
        };
        assert!(validate_task_request(&req).is_err());
    }

    #[test]
    fn validate_date_format() {
        let req = CreateTaskRequest {
            provider: "SRT".to_string(),
            departure_station: "수서".to_string(),
            arrival_station: "부산".to_string(),
            travel_date: "2026-03-15".to_string(),
            departure_time: "090000".to_string(),
            passengers: serde_json::json!([{"type": "adult", "count": 1}]),
            seat_preference: "GeneralFirst".to_string(),
            target_trains: serde_json::json!([{"train_number": "305"}]),
            auto_pay: None,
            payment_card_id: None,
            notify_enabled: None,
        };
        assert!(validate_task_request(&req).is_err());
    }

    #[test]
    fn validate_empty_trains() {
        let req = CreateTaskRequest {
            provider: "SRT".to_string(),
            departure_station: "수서".to_string(),
            arrival_station: "부산".to_string(),
            travel_date: "20260315".to_string(),
            departure_time: "090000".to_string(),
            passengers: serde_json::json!([{"type": "adult", "count": 1}]),
            seat_preference: "GeneralFirst".to_string(),
            target_trains: serde_json::json!([]),
            auto_pay: None,
            payment_card_id: None,
            notify_enabled: None,
        };
        assert!(validate_task_request(&req).is_err());
    }

    #[test]
    fn validate_valid_request() {
        let req = CreateTaskRequest {
            provider: "SRT".to_string(),
            departure_station: "수서".to_string(),
            arrival_station: "부산".to_string(),
            travel_date: "20260315".to_string(),
            departure_time: "090000".to_string(),
            passengers: serde_json::json!([{"type": "adult", "count": 1}]),
            seat_preference: "GeneralFirst".to_string(),
            target_trains: serde_json::json!([{"train_number": "305"}]),
            auto_pay: None,
            payment_card_id: None,
            notify_enabled: None,
        };
        assert!(validate_task_request(&req).is_ok());
    }

    #[test]
    fn validate_ktx_accepted() {
        let req = CreateTaskRequest {
            provider: "KTX".to_string(),
            departure_station: "서울".to_string(),
            arrival_station: "부산".to_string(),
            travel_date: "20260315".to_string(),
            departure_time: "090000".to_string(),
            passengers: serde_json::json!([{"type": "adult", "count": 1}]),
            seat_preference: "GeneralFirst".to_string(),
            target_trains: serde_json::json!([{"train_number": "101"}]),
            auto_pay: None,
            payment_card_id: None,
            notify_enabled: None,
        };
        assert!(validate_task_request(&req).is_ok());
    }

    #[test]
    fn validate_same_station() {
        let req = CreateTaskRequest {
            provider: "SRT".to_string(),
            departure_station: "수서".to_string(),
            arrival_station: "수서".to_string(),
            travel_date: "20260315".to_string(),
            departure_time: "090000".to_string(),
            passengers: serde_json::json!([{"type": "adult", "count": 1}]),
            seat_preference: "GeneralFirst".to_string(),
            target_trains: serde_json::json!([{"train_number": "305"}]),
            auto_pay: None,
            payment_card_id: None,
            notify_enabled: None,
        };
        assert!(validate_task_request(&req).is_err());
    }

    #[test]
    fn validate_time_format() {
        let req = CreateTaskRequest {
            provider: "SRT".to_string(),
            departure_station: "수서".to_string(),
            arrival_station: "부산".to_string(),
            travel_date: "20260315".to_string(),
            departure_time: "09:00:00".to_string(),
            passengers: serde_json::json!([{"type": "adult", "count": 1}]),
            seat_preference: "GeneralFirst".to_string(),
            target_trains: serde_json::json!([{"train_number": "305"}]),
            auto_pay: None,
            payment_card_id: None,
            notify_enabled: None,
        };
        assert!(validate_task_request(&req).is_err());
    }

    #[test]
    fn status_update_validation() {
        assert!(validate_status_update("queued").is_ok());
        assert!(validate_status_update("idle").is_ok());
        assert!(validate_status_update("running").is_err());
        assert!(validate_status_update("confirmed").is_err());
        assert!(validate_status_update("cancelled").is_err());
    }
}
