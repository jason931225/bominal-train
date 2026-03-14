//! Task service — create, list, update, delete reservation tasks.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ServiceError;
use crate::DbPool;

/// Task info returned to callers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskInfo {
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
    pub auto_retry: bool,
    pub status: String,
    pub reservation_number: Option<String>,
    pub attempt_count: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// List all tasks for a user.
pub async fn list(db: &DbPool, user_id: Uuid) -> Result<Vec<TaskInfo>, ServiceError> {
    let rows = bominal_db::task::find_by_user(db, user_id).await?;
    Ok(rows.iter().map(row_to_info).collect())
}

/// Get a single task by ID.
pub async fn get(db: &DbPool, task_id: Uuid, user_id: Uuid) -> Result<TaskInfo, ServiceError> {
    let row = bominal_db::task::find_by_id(db, task_id, user_id)
        .await?
        .ok_or_else(|| ServiceError::not_found("Task not found"))?;
    Ok(row_to_info(&row))
}

/// Create a new reservation task.
#[allow(clippy::too_many_arguments)]
pub async fn create(
    db: &DbPool,
    user_id: Uuid,
    provider: &str,
    departure_station: &str,
    arrival_station: &str,
    travel_date: &str,
    departure_time: &str,
    passengers: &serde_json::Value,
    seat_preference: &str,
    target_trains: &serde_json::Value,
    auto_pay: bool,
    payment_card_id: Option<Uuid>,
    notify_enabled: bool,
) -> Result<TaskInfo, ServiceError> {
    validate_provider(provider)?;

    if departure_station.is_empty() || arrival_station.is_empty() {
        return Err(ServiceError::validation(
            "Departure and arrival stations are required",
        ));
    }
    if departure_station == arrival_station {
        return Err(ServiceError::validation(
            "Departure and arrival stations must be different",
        ));
    }
    if travel_date.len() != 8 || !travel_date.chars().all(|c| c.is_ascii_digit()) {
        return Err(ServiceError::validation(
            "Travel date must be YYYYMMDD format",
        ));
    }
    if departure_time.len() != 6 || !departure_time.chars().all(|c| c.is_ascii_digit()) {
        return Err(ServiceError::validation(
            "Departure time must be HHMMSS format",
        ));
    }
    if target_trains
        .as_array()
        .map(|a| a.is_empty())
        .unwrap_or(true)
    {
        return Err(ServiceError::validation(
            "At least one target train is required",
        ));
    }

    // Verify provider credentials exist and are valid
    let cred = bominal_db::provider::find_by_user_and_provider(db, user_id, provider).await?;

    match &cred {
        Some(c) if c.status == "valid" => {}
        _ => {
            return Err(ServiceError::validation(format!(
                "Valid {provider} credentials required to create a task"
            )));
        }
    }

    let row = bominal_db::task::create_task(
        db,
        user_id,
        provider,
        departure_station,
        arrival_station,
        travel_date,
        departure_time,
        passengers,
        seat_preference,
        target_trains,
        auto_pay,
        payment_card_id,
        notify_enabled,
    )
    .await?;

    Ok(row_to_info(&row))
}

/// Update a task (status, notify, auto_retry, target_trains).
pub async fn update(
    db: &DbPool,
    task_id: Uuid,
    user_id: Uuid,
    status: Option<&str>,
    notify_enabled: Option<bool>,
    auto_retry: Option<bool>,
    target_trains: Option<&serde_json::Value>,
) -> Result<TaskInfo, ServiceError> {
    if let Some(s) = status {
        validate_status_update(s)?;
    }

    let row = bominal_db::task::update_task(db, task_id, user_id, status, notify_enabled, auto_retry, target_trains)
        .await?
        .ok_or_else(|| ServiceError::not_found("Task not found"))?;

    Ok(row_to_info(&row))
}

/// Cancel (delete) a task. Returns true if deleted.
pub async fn delete(db: &DbPool, task_id: Uuid, user_id: Uuid) -> Result<(), ServiceError> {
    let cancelled = bominal_db::task::delete_task(db, task_id, user_id).await?;

    if !cancelled {
        return Err(ServiceError::not_found(
            "Task not found or already in terminal state",
        ));
    }

    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────

fn row_to_info(row: &bominal_db::task::TaskRow) -> TaskInfo {
    TaskInfo {
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
        auto_retry: row.auto_retry,
        status: row.status.clone(),
        reservation_number: row.reservation_number.clone(),
        attempt_count: row.attempt_count,
        started_at: row.started_at,
        last_attempt_at: row.last_attempt_at,
        created_at: row.created_at,
    }
}

/// Validate provider is SRT or KTX.
pub fn validate_provider(provider: &str) -> Result<(), ServiceError> {
    match provider {
        "SRT" | "KTX" => Ok(()),
        _ => Err(ServiceError::validation(format!(
            "Invalid provider: {provider}. Must be SRT or KTX"
        ))),
    }
}

pub fn validate_status_update(status: &str) -> Result<(), ServiceError> {
    match status {
        "queued" | "idle" => Ok(()),
        _ => Err(ServiceError::validation(format!(
            "Cannot manually set status to '{status}'. Use DELETE to cancel."
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_provider() {
        assert!(validate_provider("SRT").is_ok());
        assert!(validate_provider("KTX").is_ok());
        assert!(validate_provider("INVALID").is_err());
    }

    #[test]
    fn test_validate_status_update() {
        assert!(validate_status_update("queued").is_ok());
        assert!(validate_status_update("idle").is_ok());
        assert!(validate_status_update("running").is_err());
        assert!(validate_status_update("confirmed").is_err());
        assert!(validate_status_update("cancelled").is_err());
    }
}
