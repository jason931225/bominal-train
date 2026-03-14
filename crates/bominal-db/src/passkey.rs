//! Passkey credential repository — challenge storage and credential CRUD.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PasskeyCredentialRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub credential_id: String,
    pub public_key: String,
    pub label: String,
    pub aaguid: Option<Vec<u8>>,
    pub transports: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PasskeyChallengeRow {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub challenge_id: String,
    pub state: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Store a registration challenge with the serialized webauthn-rs ceremony state.
pub async fn store_challenge(
    pool: &PgPool,
    user_id: Uuid,
    challenge_id: &str,
    state: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO passkey_challenges (user_id, challenge_id, state) VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(challenge_id)
    .bind(state)
    .execute(pool)
    .await?;
    Ok(())
}

/// Consume and return a valid registration challenge.
pub async fn verify_challenge(
    pool: &PgPool,
    user_id: Uuid,
    challenge_id: &str,
) -> Result<Option<PasskeyChallengeRow>, sqlx::Error> {
    sqlx::query_as::<_, PasskeyChallengeRow>(
        "DELETE FROM passkey_challenges \
         WHERE user_id = $1 AND challenge_id = $2 \
           AND created_at > NOW() - INTERVAL '5 minutes' \
         RETURNING *",
    )
    .bind(user_id)
    .bind(challenge_id)
    .fetch_optional(pool)
    .await
}

/// Store a login challenge with the serialized webauthn-rs ceremony state.
pub async fn store_login_challenge(
    pool: &PgPool,
    challenge_id: &str,
    state: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO passkey_challenges (challenge_id, state) VALUES ($1, $2)")
        .bind(challenge_id)
        .bind(state)
        .execute(pool)
        .await?;
    Ok(())
}

/// Consume and return a valid login challenge.
pub async fn verify_login_challenge(
    pool: &PgPool,
    challenge_id: &str,
) -> Result<Option<PasskeyChallengeRow>, sqlx::Error> {
    sqlx::query_as::<_, PasskeyChallengeRow>(
        "DELETE FROM passkey_challenges \
         WHERE challenge_id = $1 \
           AND created_at > NOW() - INTERVAL '5 minutes' \
         RETURNING *",
    )
    .bind(challenge_id)
    .fetch_optional(pool)
    .await
}

/// Delete expired passkey challenges (older than 5 minutes).
pub async fn delete_expired_challenges(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM passkey_challenges WHERE created_at < NOW() - INTERVAL '5 minutes'",
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn store_credential(
    pool: &PgPool,
    user_id: Uuid,
    credential_id: &str,
    public_key: &str,
    label: &str,
) -> Result<PasskeyCredentialRow, sqlx::Error> {
    sqlx::query_as::<_, PasskeyCredentialRow>(
        r#"
        INSERT INTO passkey_credentials (user_id, credential_id, public_key, label)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(credential_id)
    .bind(public_key)
    .bind(label)
    .fetch_one(pool)
    .await
}

pub async fn find_credential_by_id(
    pool: &PgPool,
    credential_id: &str,
) -> Result<Option<PasskeyCredentialRow>, sqlx::Error> {
    sqlx::query_as::<_, PasskeyCredentialRow>(
        "SELECT * FROM passkey_credentials WHERE credential_id = $1",
    )
    .bind(credential_id)
    .fetch_optional(pool)
    .await
}

pub async fn find_credentials_by_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<PasskeyCredentialRow>, sqlx::Error> {
    sqlx::query_as::<_, PasskeyCredentialRow>(
        "SELECT * FROM passkey_credentials WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn delete_credential(
    pool: &PgPool,
    credential_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM passkey_credentials WHERE id = $1 AND user_id = $2")
        .bind(credential_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
