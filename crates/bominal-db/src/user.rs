//! User repository — CRUD operations for the users table.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Row returned from the users table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub password_hash: String,
    pub preferred_locale: String,
    pub email_verified: bool,
    pub email_verification_token: Option<String>,
    pub email_verification_expires_at: Option<DateTime<Utc>>,
    pub password_reset_token: Option<String>,
    pub password_reset_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Insert a new user. Returns the created row.
pub async fn create_user(
    pool: &PgPool,
    email: &str,
    display_name: &str,
    password_hash: &str,
) -> Result<UserRow, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        r#"
        INSERT INTO users (email, display_name, password_hash)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(email)
    .bind(display_name)
    .bind(password_hash)
    .fetch_one(pool)
    .await
}

/// Find a user by email.
pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await
}

/// Find a user by ID.
pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Set email verification token and expiry.
pub async fn set_verification_token(
    pool: &PgPool,
    user_id: Uuid,
    token: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE users SET email_verification_token = $1, email_verification_expires_at = $2 WHERE id = $3",
    )
    .bind(token)
    .bind(expires_at)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Verify email using the token. Returns the user if valid.
pub async fn verify_email(pool: &PgPool, token: &str) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        r#"
        UPDATE users
        SET email_verified = true,
            email_verification_token = NULL,
            email_verification_expires_at = NULL
        WHERE email_verification_token = $1
          AND email_verification_expires_at > now()
        RETURNING *
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await
}

/// Set password reset token and expiry.
pub async fn set_reset_token(
    pool: &PgPool,
    user_id: Uuid,
    token: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE users SET password_reset_token = $1, password_reset_expires_at = $2 WHERE id = $3",
    )
    .bind(token)
    .bind(expires_at)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Consume a password reset token and update the password. Returns the user if valid.
pub async fn reset_password(
    pool: &PgPool,
    token: &str,
    new_password_hash: &str,
) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        r#"
        UPDATE users
        SET password_hash = $1,
            password_reset_token = NULL,
            password_reset_expires_at = NULL
        WHERE password_reset_token = $2
          AND password_reset_expires_at > now()
        RETURNING *
        "#,
    )
    .bind(new_password_hash)
    .bind(token)
    .fetch_optional(pool)
    .await
}

/// Find all user emails (for internal broadcast).
pub async fn all_emails(pool: &PgPool) -> Result<Vec<(Uuid, String, String)>, sqlx::Error> {
    sqlx::query_as::<_, (Uuid, String, String)>(
        "SELECT id, email, display_name FROM users WHERE email_verified = true ORDER BY created_at",
    )
    .fetch_all(pool)
    .await
}
