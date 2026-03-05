use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::AppState;

use super::{AdminServiceError, KillSwitchRecord};

pub(crate) async fn list_kill_switches(
    state: &AppState,
) -> Result<Vec<KillSwitchRecord>, AdminServiceError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let rows = sqlx::query_as::<_, (String, bool, String, DateTime<Utc>)>(
        "select flag, enabled, reason, updated_at
         from admin_runtime_flags
         order by flag asc",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(rows
        .into_iter()
        .map(|row| KillSwitchRecord {
            flag: row.0,
            enabled: row.1,
            reason: row.2,
            updated_at: row.3,
        })
        .collect())
}

pub(crate) async fn upsert_kill_switch(
    state: &AppState,
    flag: &str,
    enabled: bool,
    reason: &str,
    actor_user_id: Option<&str>,
) -> Result<(), AdminServiceError> {
    let flag = flag.trim().to_ascii_lowercase();
    if !matches!(
        flag.as_str(),
        "runtime_ingest" | "runtime_dispatch" | "provider_calls"
    ) {
        return Err(AdminServiceError::InvalidRequest(
            "unknown kill switch flag",
        ));
    }
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let actor_uuid = actor_user_id.and_then(|value| Uuid::parse_str(value).ok());
    sqlx::query(
        "insert into admin_runtime_flags (flag, enabled, reason, updated_at, updated_by)
         values ($1, $2, $3, now(), $4)
         on conflict (flag)
         do update set enabled = excluded.enabled, reason = excluded.reason, updated_at = now(), updated_by = excluded.updated_by",
    )
    .bind(flag)
    .bind(enabled)
    .bind(reason.trim())
    .bind(actor_uuid)
    .execute(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(())
}
