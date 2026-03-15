//! Session repository — CRUD for the sessions table.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Row returned from the sessions table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SessionRow {
    pub id: String,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Create a new session.
pub async fn create_session(
    pool: &PgPool,
    session_id: &str,
    user_id: Uuid,
    expires_at: DateTime<Utc>,
) -> Result<SessionRow, sqlx::Error> {
    sqlx::query_as::<_, SessionRow>(
        r#"
        INSERT INTO sessions (id, user_id, expires_at)
        VALUES ($1, $2, $3)
        RETURNING id, user_id, expires_at, created_at
        "#,
    )
    .bind(session_id)
    .bind(user_id)
    .bind(expires_at)
    .fetch_one(pool)
    .await
}

/// Find a non-expired session by ID.
pub async fn find_valid_session(
    pool: &PgPool,
    session_id: &str,
) -> Result<Option<SessionRow>, sqlx::Error> {
    sqlx::query_as::<_, SessionRow>(
        r#"
        SELECT id, user_id, expires_at, created_at
        FROM sessions
        WHERE id = $1 AND expires_at > now()
        "#,
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await
}

/// Delete a session (logout).
pub async fn delete_session(pool: &PgPool, session_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE id = $1")
        .bind(session_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Delete all expired sessions (cleanup).
pub async fn delete_expired_sessions(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM sessions WHERE expires_at <= now()")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}
