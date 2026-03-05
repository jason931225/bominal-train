use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::super::super::AppState;

use super::{AdminServiceError, AuditRecord};

pub(crate) struct AppendAdminAuditInput<'a> {
    pub(crate) actor_user_id: Option<&'a str>,
    pub(crate) actor_email: &'a str,
    pub(crate) action: &'a str,
    pub(crate) target_type: &'a str,
    pub(crate) target_id: &'a str,
    pub(crate) reason: &'a str,
    pub(crate) request_id: &'a str,
    pub(crate) metadata: serde_json::Value,
}

pub(crate) async fn append_admin_audit(
    state: &AppState,
    input: AppendAdminAuditInput<'_>,
) -> Result<(), AdminServiceError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let actor_uuid = input
        .actor_user_id
        .and_then(|value| Uuid::parse_str(value).ok());
    sqlx::query(
        "insert into admin_audit_log
         (id, actor_user_id, actor_email, action, target_type, target_id, reason, request_id, metadata, created_at)
         values ($1, $2, $3, $4, $5, $6, $7, $8, cast($9 as jsonb), now())",
    )
    .bind(Uuid::new_v4())
    .bind(actor_uuid)
    .bind(input.actor_email)
    .bind(input.action)
    .bind(input.target_type)
    .bind(input.target_id)
    .bind(input.reason.trim())
    .bind(input.request_id)
    .bind(input.metadata)
    .execute(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(())
}

pub(crate) async fn list_admin_audit(
    state: &AppState,
) -> Result<Vec<AuditRecord>, AdminServiceError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let rows = sqlx::query_as::<_, (Uuid, Option<Uuid>, String, String, String, String, String, String, serde_json::Value, DateTime<Utc>)>(
        "select id, actor_user_id, actor_email, action, target_type, target_id, reason, request_id, metadata, created_at
         from admin_audit_log
         order by created_at desc
         limit 400",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(rows
        .into_iter()
        .map(|row| AuditRecord {
            id: row.0.to_string(),
            actor_user_id: row.1.map(|value| value.to_string()),
            actor_email: row.2,
            action: row.3,
            target_type: row.4,
            target_id: row.5,
            reason: row.6,
            request_id: row.7,
            metadata: row.8,
            created_at: row.9,
        })
        .collect())
}
