//! Provider credential repository — CRUD for the provider_credentials table.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Row returned from the provider_credentials table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProviderCredentialRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub login_id: String,
    pub encrypted_password: String,
    pub status: String,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Upsert provider credentials (insert or update on conflict).
pub async fn upsert_credential(
    pool: &PgPool,
    user_id: Uuid,
    provider: &str,
    login_id: &str,
    encrypted_password: &str,
    status: &str,
) -> Result<ProviderCredentialRow, sqlx::Error> {
    sqlx::query_as::<_, ProviderCredentialRow>(
        r#"
        INSERT INTO provider_credentials (user_id, provider, login_id, encrypted_password, status, last_verified_at)
        VALUES ($1, $2, $3, $4, $5, CASE WHEN $5 = 'valid' THEN now() ELSE NULL END)
        ON CONFLICT (user_id, provider) DO UPDATE SET
            login_id = EXCLUDED.login_id,
            encrypted_password = EXCLUDED.encrypted_password,
            status = EXCLUDED.status,
            last_verified_at = CASE WHEN EXCLUDED.status = 'valid' THEN now() ELSE provider_credentials.last_verified_at END
        RETURNING id, user_id, provider, login_id, encrypted_password, status, last_verified_at, created_at
        "#,
    )
    .bind(user_id)
    .bind(provider)
    .bind(login_id)
    .bind(encrypted_password)
    .bind(status)
    .fetch_one(pool)
    .await
}

/// Find all credentials for a user.
pub async fn find_by_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<ProviderCredentialRow>, sqlx::Error> {
    sqlx::query_as::<_, ProviderCredentialRow>(
        r#"
        SELECT id, user_id, provider, login_id, encrypted_password, status, last_verified_at, created_at
        FROM provider_credentials
        WHERE user_id = $1
        ORDER BY provider
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Find a specific credential by user + provider.
pub async fn find_by_user_and_provider(
    pool: &PgPool,
    user_id: Uuid,
    provider: &str,
) -> Result<Option<ProviderCredentialRow>, sqlx::Error> {
    sqlx::query_as::<_, ProviderCredentialRow>(
        r#"
        SELECT id, user_id, provider, login_id, encrypted_password, status, last_verified_at, created_at
        FROM provider_credentials
        WHERE user_id = $1 AND provider = $2
        "#,
    )
    .bind(user_id)
    .bind(provider)
    .fetch_optional(pool)
    .await
}

/// Update credential status.
pub async fn update_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
) -> Result<(), sqlx::Error> {
    let verified_at_clause = if status == "valid" {
        ", last_verified_at = now()"
    } else {
        ""
    };

    let query = format!(
        "UPDATE provider_credentials SET status = $1{verified_at_clause} WHERE id = $2"
    );

    sqlx::query(&query)
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Delete a credential by user + provider.
pub async fn delete_credential(
    pool: &PgPool,
    user_id: Uuid,
    provider: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM provider_credentials WHERE user_id = $1 AND provider = $2",
    )
    .bind(user_id)
    .bind(provider)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
