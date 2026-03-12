//! Task server functions.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Task info for display.
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
    pub status: String,
    pub reservation_number: Option<String>,
    pub attempt_count: i32,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_attempt_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// List all tasks for the current user.
#[server(prefix = "/sfn")]
pub async fn list_tasks() -> Result<Vec<TaskInfo>, ServerFnError> {
    let (pool, user_id) = require_auth().await?;

    let rows = bominal_db::task::find_by_user(&pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(rows.iter().map(row_to_info).collect())
}

/// Create a new reservation task.
#[server(prefix = "/sfn")]
pub async fn create_task(
    provider: String,
    departure_station: String,
    arrival_station: String,
    travel_date: String,
    departure_time: String,
    passengers: String,
    seat_preference: String,
    target_trains: String,
    auto_pay: Option<bool>,
    payment_card_id: Option<String>,
    notify_enabled: Option<bool>,
) -> Result<TaskInfo, ServerFnError> {
    let (pool, user_id) = require_auth().await?;

    if provider != "SRT" && provider != "KTX" {
        return Err(ServerFnError::new(format!("Invalid provider: {provider}")));
    }
    if departure_station.is_empty() || arrival_station.is_empty() {
        return Err(ServerFnError::new("Stations are required"));
    }
    if departure_station == arrival_station {
        return Err(ServerFnError::new("Departure and arrival must differ"));
    }

    let passengers_json: serde_json::Value = serde_json::from_str(&passengers)
        .map_err(|e| ServerFnError::new(format!("Bad passengers JSON: {e}")))?;
    let trains_json: serde_json::Value = serde_json::from_str(&target_trains)
        .map_err(|e| ServerFnError::new(format!("Bad trains JSON: {e}")))?;

    let card_id = payment_card_id
        .filter(|s| !s.is_empty())
        .map(|s| Uuid::parse_str(&s))
        .transpose()
        .map_err(|e| ServerFnError::new(format!("Bad card ID: {e}")))?;

    // Verify provider credentials exist and are valid
    let cred = bominal_db::provider::find_by_user_and_provider(&pool, user_id, &provider)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    match &cred {
        Some(c) if c.status == "valid" => {}
        _ => {
            return Err(ServerFnError::new(format!(
                "Valid {provider} credentials required"
            )));
        }
    }

    let row = bominal_db::task::create_task(
        &pool,
        user_id,
        &provider,
        &departure_station,
        &arrival_station,
        &travel_date,
        &departure_time,
        &passengers_json,
        &seat_preference,
        &trains_json,
        auto_pay.unwrap_or(false),
        card_id,
        notify_enabled.unwrap_or(false),
    )
    .await
    .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    leptos_axum::redirect("/tasks");
    Ok(row_to_info(&row))
}

/// Cancel (delete) a task.
#[server(prefix = "/sfn")]
pub async fn cancel_task(task_id: String) -> Result<(), ServerFnError> {
    let (pool, user_id) = require_auth().await?;
    let id = Uuid::parse_str(&task_id).map_err(|e| ServerFnError::new(format!("Bad ID: {e}")))?;

    let cancelled = bominal_db::task::delete_task(&pool, id, user_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    if !cancelled {
        return Err(ServerFnError::new("Task not found or already terminal"));
    }

    Ok(())
}

// ── Helpers ────────────────────────────────────────────────────────

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
        status: row.status.clone(),
        reservation_number: row.reservation_number.clone(),
        attempt_count: row.attempt_count,
        started_at: row.started_at,
        last_attempt_at: row.last_attempt_at,
        created_at: row.created_at,
    }
}

pub(crate) async fn require_auth() -> Result<(bominal_db::DbPool, Uuid), ServerFnError> {
    let pool = use_context::<bominal_db::DbPool>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    let session_id =
        super::auth::extract_session_id().ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let session = bominal_db::session::find_valid_session(&pool, &session_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
        .ok_or_else(|| ServerFnError::new("Session expired"))?;

    Ok((pool, session.user_id))
}
