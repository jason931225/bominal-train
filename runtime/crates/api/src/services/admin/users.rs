use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::AppState;

use super::{AdminServiceError, AdminUserRecord, SessionRecord};

pub(crate) async fn list_users(
    state: &AppState,
) -> Result<Vec<AdminUserRecord>, AdminServiceError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let rows = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            String,
            Option<String>,
            Option<bool>,
            Option<DateTime<Utc>>,
        ),
    >(
        "select u.id, u.email::text, u.status,
                r.role, r.access_enabled, r.updated_at
         from auth_users u
         left join auth_user_role_bindings r on r.user_id = u.id
         order by u.created_at desc",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(rows
        .into_iter()
        .map(|row| AdminUserRecord {
            user_id: row.0.to_string(),
            email: row.1,
            status: row.2,
            role: row.3.unwrap_or_else(|| "user".to_string()),
            access_enabled: row.4.unwrap_or(true),
            updated_at: row.5,
        })
        .collect())
}

pub(crate) async fn set_user_role(
    state: &AppState,
    user_id: &str,
    role: &str,
    actor_user_id: Option<&str>,
) -> Result<(), AdminServiceError> {
    let role = role.trim().to_ascii_lowercase();
    if !matches!(role.as_str(), "admin" | "operator" | "viewer" | "user") {
        return Err(AdminServiceError::InvalidRequest("invalid role"));
    }
    let user_uuid = Uuid::parse_str(user_id)
        .map_err(|_| AdminServiceError::InvalidRequest("invalid user id"))?;
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let actor_uuid = actor_user_id.and_then(|value| Uuid::parse_str(value).ok());
    sqlx::query(
        "insert into auth_user_role_bindings (user_id, role, access_enabled, updated_at, updated_by)
         values ($1, $2, true, now(), $3)
         on conflict (user_id)
         do update set role = excluded.role, updated_at = now(), updated_by = excluded.updated_by",
    )
    .bind(user_uuid)
    .bind(role)
    .bind(actor_uuid)
    .execute(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(())
}

pub(crate) async fn set_user_access(
    state: &AppState,
    user_id: &str,
    access_enabled: bool,
    actor_user_id: Option<&str>,
) -> Result<(), AdminServiceError> {
    let user_uuid = Uuid::parse_str(user_id)
        .map_err(|_| AdminServiceError::InvalidRequest("invalid user id"))?;
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let actor_uuid = actor_user_id.and_then(|value| Uuid::parse_str(value).ok());
    sqlx::query(
        "insert into auth_user_role_bindings (user_id, role, access_enabled, updated_at, updated_by)
         values ($1, 'user', $2, now(), $3)
         on conflict (user_id)
         do update set access_enabled = excluded.access_enabled, updated_at = now(), updated_by = excluded.updated_by",
    )
    .bind(user_uuid)
    .bind(access_enabled)
    .bind(actor_uuid)
    .execute(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(())
}

pub(crate) async fn list_sessions(
    state: &AppState,
) -> Result<Vec<SessionRecord>, AdminServiceError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let rows = sqlx::query_as::<_, (String, Option<Uuid>, String, String, DateTime<Utc>, DateTime<Utc>, Option<DateTime<Utc>>, Option<DateTime<Utc>>)>(
        "select session_id_hash, user_id, email, role, issued_at, last_seen_at, step_up_verified_at, revoked_at
         from auth_sessions
         order by last_seen_at desc
         limit 500",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(rows
        .into_iter()
        .map(|row| SessionRecord {
            session_id_hash: row.0,
            user_id: row.1.map(|value| value.to_string()),
            email: row.2,
            role: row.3,
            issued_at: row.4,
            last_seen_at: row.5,
            step_up_verified_at: row.6,
            revoked_at: row.7,
        })
        .collect())
}
