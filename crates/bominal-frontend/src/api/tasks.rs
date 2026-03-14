//! Task server functions.

use leptos::prelude::*;
use uuid::Uuid;

pub use bominal_service::tasks::TaskInfo;

/// List all tasks for the current user.
#[server(prefix = "/sfn")]
pub async fn list_tasks() -> Result<Vec<TaskInfo>, ServerFnError> {
    let (pool, user_id) = require_auth().await?;
    bominal_service::tasks::list(&pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Create a new reservation task.
#[allow(clippy::too_many_arguments)]
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

    let passengers_json: serde_json::Value = serde_json::from_str(&passengers)
        .map_err(|e| ServerFnError::new(format!("Bad passengers JSON: {e}")))?;
    let trains_json: serde_json::Value = serde_json::from_str(&target_trains)
        .map_err(|e| ServerFnError::new(format!("Bad trains JSON: {e}")))?;

    let card_id = payment_card_id
        .filter(|s| !s.is_empty())
        .map(|s| Uuid::parse_str(&s))
        .transpose()
        .map_err(|e| ServerFnError::new(format!("Bad card ID: {e}")))?;

    let task = bominal_service::tasks::create(
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
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    leptos_axum::redirect("/tasks");
    Ok(task)
}

/// Update a task (status, notify, auto_retry, target_trains).
#[server(prefix = "/sfn")]
pub async fn update_task(
    task_id: String,
    status: Option<String>,
    notify_enabled: Option<bool>,
    auto_retry: Option<bool>,
    target_trains: Option<String>,
) -> Result<TaskInfo, ServerFnError> {
    let (pool, user_id) = require_auth().await?;
    let id = Uuid::parse_str(&task_id)
        .map_err(|e| ServerFnError::new(format!("Bad ID: {e}")))?;

    let trains = target_trains
        .filter(|s| !s.is_empty())
        .map(|s| serde_json::from_str::<serde_json::Value>(&s))
        .transpose()
        .map_err(|e| ServerFnError::new(format!("Bad trains JSON: {e}")))?;

    let result = bominal_service::tasks::update(
        &pool,
        id,
        user_id,
        status.as_deref(),
        notify_enabled,
        auto_retry,
        trains.as_ref(),
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    leptos_axum::redirect("/tasks");
    Ok(result)
}

/// Cancel (delete) a task.
#[server(prefix = "/sfn")]
pub async fn cancel_task(task_id: String) -> Result<(), ServerFnError> {
    let (pool, user_id) = require_auth().await?;
    let id = Uuid::parse_str(&task_id).map_err(|e| ServerFnError::new(format!("Bad ID: {e}")))?;

    bominal_service::tasks::delete(&pool, id, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    leptos_axum::redirect("/tasks");
    Ok(())
}

// ── Helpers ────────────────────────────────────────────────────────

pub(crate) async fn require_auth() -> Result<(bominal_service::DbPool, Uuid), ServerFnError> {
    let pool = use_context::<bominal_service::DbPool>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    let session_id =
        super::auth::extract_session_id().ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let user_id = bominal_service::auth::require_user_id(&pool, &session_id)
        .await
        .map_err(|_| ServerFnError::new("Session expired"))?;

    Ok((pool, user_id))
}
