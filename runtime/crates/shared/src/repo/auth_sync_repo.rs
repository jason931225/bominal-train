use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::runtime_job_repo::RepoError;

pub const AUTH_SYNC_UPSERT_SQL: &str = "insert into supabase_auth_user_sync (user_id, email, last_event_type, last_synced_at) values ($1, $2, $3, $4) on conflict (user_id) do update set email = excluded.email, last_event_type = excluded.last_event_type, last_synced_at = excluded.last_synced_at returning user_id, email, last_event_type, last_synced_at";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthSyncRecord {
    pub user_id: String,
    pub email: Option<String>,
    pub event_type: String,
    pub synced_at: DateTime<Utc>,
}

pub async fn upsert_auth_sync(
    pool: &PgPool,
    user_id: &str,
    email: Option<&str>,
    event_type: &str,
    synced_at: DateTime<Utc>,
) -> Result<AuthSyncRecord, RepoError> {
    let row =
        sqlx::query_as::<_, (String, Option<String>, String, DateTime<Utc>)>(AUTH_SYNC_UPSERT_SQL)
            .bind(user_id)
            .bind(email)
            .bind(event_type)
            .bind(synced_at)
            .fetch_one(pool)
            .await?;

    Ok(AuthSyncRecord {
        user_id: row.0,
        email: row.1,
        event_type: row.2,
        synced_at: row.3,
    })
}
