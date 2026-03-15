//! Task server functions.

use leptos::prelude::*;
use uuid::Uuid;

pub use bominal_service::tasks::{
    CreateTaskInput, PassengerCount, PassengerKind, PassengerList, Provider, SeatPreference,
    TargetTrain, TargetTrainList, TaskInfo, TaskStatus, UpdateTaskInput,
};

#[server(prefix = "/sfn")]
pub async fn list_tasks() -> Result<Vec<TaskInfo>, ServerFnError> {
    let (pool, user_id) = require_auth().await?;
    bominal_service::tasks::list(&pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(prefix = "/sfn")]
pub async fn create_task(input: CreateTaskInput) -> Result<TaskInfo, ServerFnError> {
    let (pool, user_id) = require_auth().await?;

    let task = bominal_service::tasks::create(&pool, user_id, &input)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    leptos_axum::redirect("/tasks");
    Ok(task)
}

#[server(prefix = "/sfn")]
pub async fn update_task(
    task_id: String,
    input: UpdateTaskInput,
) -> Result<TaskInfo, ServerFnError> {
    let (pool, user_id) = require_auth().await?;
    let id = Uuid::parse_str(&task_id)
        .map_err(|e| ServerFnError::new(format!("Bad ID: {e}")))?;

    let result = bominal_service::tasks::update(&pool, id, user_id, &input)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    leptos_axum::redirect("/tasks");
    Ok(result)
}

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
