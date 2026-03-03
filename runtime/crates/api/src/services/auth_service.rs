use std::time::Duration;

use anyhow::{Context, Result};
use axum::http::HeaderMap;
use bominal_shared::supabase::{Jwks, SupabaseClaims, fetch_jwks, verify_supabase_jwt};
use sqlx::PgPool;
use tracing::{error, info, warn};

use super::super::{AppState, JwksCacheEntry};

#[derive(Debug, serde::Deserialize)]
pub(crate) struct SupabaseAuthWebhook {
    #[serde(rename = "type")]
    pub(crate) event_type: String,
    pub(crate) user_id: Option<String>,
    pub(crate) email: Option<String>,
}

#[derive(Debug)]
pub(crate) enum VerifySupabaseTokenError {
    MissingBearerToken,
    JwksUnavailable,
    Unauthorized,
}

#[derive(Debug)]
pub(crate) enum SupabaseAuthWebhookError {
    SecretMismatch,
    PersistenceFailure,
}

pub(crate) async fn verify_supabase_token(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<SupabaseClaims, VerifySupabaseTokenError> {
    let Some(token) = bearer_token(headers) else {
        return Err(VerifySupabaseTokenError::MissingBearerToken);
    };

    let jwks = match get_or_refresh_jwks(state).await {
        Ok(value) => value,
        Err(err) => {
            error!(error = %err, "failed to load supabase jwks");
            return Err(VerifySupabaseTokenError::JwksUnavailable);
        }
    };

    verify_supabase_jwt(
        token,
        &jwks,
        &state.config.supabase.jwt_issuer,
        state.config.supabase.jwt_audience.as_deref(),
    )
    .map_err(|err| {
        warn!(error = %err, "supabase token verification failed");
        VerifySupabaseTokenError::Unauthorized
    })
}

pub(crate) async fn process_supabase_auth_webhook(
    state: &AppState,
    headers: &HeaderMap,
    payload: &SupabaseAuthWebhook,
) -> Result<(), SupabaseAuthWebhookError> {
    if let Some(expected_secret) = state.config.supabase.auth_webhook_secret.as_deref() {
        let provided = headers
            .get("x-bominal-supabase-webhook-secret")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("");

        if provided != expected_secret {
            return Err(SupabaseAuthWebhookError::SecretMismatch);
        }
    }

    if let Some(pool) = state.db_pool.as_ref()
        && let Err(err) = persist_auth_sync(pool, payload).await
    {
        error!(error = %err, "failed to persist supabase auth sync payload");
        return Err(SupabaseAuthWebhookError::PersistenceFailure);
    }

    info!(
        event_type = %payload.event_type,
        user_id = payload.user_id.as_deref().unwrap_or("unknown"),
        has_email = payload.email.is_some(),
        "received supabase auth webhook"
    );

    Ok(())
}

async fn persist_auth_sync(pool: &PgPool, payload: &SupabaseAuthWebhook) -> Result<()> {
    let user_id = payload
        .user_id
        .as_deref()
        .context("supabase webhook payload missing user_id")?;

    sqlx::query(
        "insert into supabase_auth_user_sync (user_id, email, last_event_type, last_synced_at) values ($1, $2, $3, now()) on conflict (user_id) do update set email = excluded.email, last_event_type = excluded.last_event_type, last_synced_at = now()",
    )
    .bind(user_id)
    .bind(payload.email.as_deref())
    .bind(payload.event_type.as_str())
    .execute(pool)
    .await
    .context("failed to upsert supabase auth sync row")?;

    Ok(())
}

async fn get_or_refresh_jwks(state: &AppState) -> Result<Jwks> {
    {
        let guard = state.jwks_cache.read().await;
        if let Some(entry) = guard.as_ref()
            && entry.fetched_at.elapsed()
                < Duration::from_secs(state.config.supabase.jwks_cache_seconds)
        {
            return Ok(entry.jwks.clone());
        }
    }

    let jwks = fetch_jwks(&state.http_client, &state.config.supabase.jwks_url).await?;

    {
        let mut guard = state.jwks_cache.write().await;
        *guard = Some(JwksCacheEntry {
            fetched_at: std::time::Instant::now(),
            jwks: jwks.clone(),
        });
    }

    Ok(jwks)
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let auth_header = headers.get(axum::http::header::AUTHORIZATION)?;
    let raw = auth_header.to_str().ok()?;
    raw.strip_prefix("Bearer ")
}
