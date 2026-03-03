use std::{
    env,
    fmt::Write as _,
    sync::{Arc, OnceLock},
    time::Instant,
};

use argon2::{
    Algorithm, Argon2, Params, Version,
    password_hash::{
        Ident, PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
    },
};
use axum::http::HeaderMap;
use bominal_shared::config::AdminRole;
use chrono::{DateTime, Duration, Utc};
use redis::AsyncCommands;
use sha2::{Digest, Sha256};
use tokio::{sync::Semaphore, task};
use tracing::info;
use uuid::Uuid;

use super::super::AppState;

const SESSION_KEY_PREFIX: &str = "auth:session:";
const INVITE_TTL_SECONDS: u64 = 86_400;
const ARGON2_MEMORY_KIB: u32 = 16 * 1024;
const ARGON2_TIME_COST: u32 = 1;
const ARGON2_PARALLELISM: u32 = 1;
const SIGNIN_FAIL_WINDOW_SECONDS: u64 = 900;
const SIGNIN_LOCKOUT_SECONDS: u64 = 900;
const SIGNIN_MAX_FAILS_PER_EMAIL: u64 = 6;
const SIGNIN_MAX_FAILS_PER_IP: u64 = 20;
const SIGNIN_FAIL_EMAIL_PREFIX: &str = "auth:signin:fail:email:";
const SIGNIN_FAIL_IP_PREFIX: &str = "auth:signin:fail:ip:";
const SIGNIN_LOCK_EMAIL_PREFIX: &str = "auth:signin:lock:email:";
const SIGNIN_LOCK_IP_PREFIX: &str = "auth:signin:lock:ip:";
const ADMIN_EMAILS_ENV: &str = "ADMIN_EMAILS";

fn password_hash_concurrency_limit() -> usize {
    static PASSWORD_HASH_CONCURRENCY: OnceLock<usize> = OnceLock::new();
    *PASSWORD_HASH_CONCURRENCY.get_or_init(|| {
        env::var("PASSWORD_HASH_CONCURRENCY")
            .ok()
            .and_then(|raw| raw.parse::<usize>().ok())
            .map(|value| value.clamp(1, 32))
            .unwrap_or(1)
    })
}

fn password_hash_semaphore() -> Arc<Semaphore> {
    static PASSWORD_HASH_SEMAPHORE: OnceLock<Arc<Semaphore>> = OnceLock::new();
    PASSWORD_HASH_SEMAPHORE
        .get_or_init(|| Arc::new(Semaphore::new(password_hash_concurrency_limit())))
        .clone()
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct PasswordSigninRequest {
    pub(crate) email: String,
    pub(crate) password: String,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ChangePasswordRequest {
    pub(crate) current_password: String,
    pub(crate) new_password: String,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct AcceptInviteRequest {
    pub(crate) invite_token: String,
    pub(crate) email: String,
    pub(crate) password: String,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct CreateInviteRequest {
    pub(crate) email: String,
    pub(crate) expires_in_seconds: Option<u64>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct CreateInviteResponse {
    pub(crate) invite_url: String,
    pub(crate) expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct SessionUser {
    pub(crate) user_id: String,
    pub(crate) email: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct SessionState {
    pub(crate) user_id: String,
    pub(crate) email: String,
    pub(crate) role: AdminRole,
    pub(crate) issued_at: DateTime<Utc>,
    pub(crate) last_seen_at: DateTime<Utc>,
    pub(crate) step_up_verified_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub(crate) enum AuthServiceError {
    InvalidRequest(&'static str),
    Unauthorized(&'static str),
    NotFound(&'static str),
    Conflict(&'static str),
    ServiceUnavailable(&'static str),
    Internal,
}

pub(crate) async fn signin_with_password(
    state: &AppState,
    request: PasswordSigninRequest,
    client_ip: &str,
) -> Result<SessionUser, AuthServiceError> {
    let email = request.email.trim().to_ascii_lowercase();
    if email.is_empty() || request.password.is_empty() {
        return Err(AuthServiceError::InvalidRequest(
            "email and password are required",
        ));
    }
    ensure_signin_not_locked(state, &email, client_ip).await?;

    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AuthServiceError::ServiceUnavailable("database unavailable"))?;

    let user = sqlx::query_as::<_, (Uuid, String, String)>(
        "select id, email, password_hash from auth_users where lower(email) = lower($1) and status = 'active' limit 1",
    )
    .bind(&email)
    .fetch_optional(pool)
    .await
    .map_err(|_| AuthServiceError::Internal)?;

    let Some((user_id, persisted_email, password_hash)) = user else {
        record_signin_failure(state, &email, client_ip).await?;
        return Err(AuthServiceError::Unauthorized("invalid email or password"));
    };

    match verify_password_hash(request.password, password_hash).await {
        Ok(()) => {}
        Err(AuthServiceError::Unauthorized(message)) => {
            record_signin_failure(state, &email, client_ip).await?;
            return Err(AuthServiceError::Unauthorized(message));
        }
        Err(error) => return Err(error),
    }
    clear_signin_failure_state(state, &email, client_ip).await?;

    Ok(SessionUser {
        user_id: user_id.to_string(),
        email: persisted_email,
    })
}

pub(crate) async fn accept_invite(
    state: &AppState,
    request: AcceptInviteRequest,
) -> Result<SessionUser, AuthServiceError> {
    let invite_token = request.invite_token.trim();
    let email = request.email.trim().to_ascii_lowercase();
    if invite_token.is_empty() || email.is_empty() || request.password.is_empty() {
        return Err(AuthServiceError::InvalidRequest(
            "invite token, email, and password are required",
        ));
    }
    if !looks_like_email(&email) {
        return Err(AuthServiceError::InvalidRequest("email format is invalid"));
    }
    if request.password.len() < 8 {
        return Err(AuthServiceError::InvalidRequest(
            "password must be at least 8 characters",
        ));
    }

    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AuthServiceError::ServiceUnavailable("database unavailable"))?;

    let token_hash = hash_token(&state.config.session_secret, invite_token);
    let invite = sqlx::query_as::<_, (Uuid, String, DateTime<Utc>, Option<DateTime<Utc>>)>(
        "select id, email, expires_at, accepted_at from auth_invites where token_hash = $1 limit 1",
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await
    .map_err(|_| AuthServiceError::Internal)?;

    let Some((invite_id, invite_email, expires_at, accepted_at)) = invite else {
        return Err(AuthServiceError::Unauthorized("invalid invite token"));
    };

    if accepted_at.is_some() {
        return Err(AuthServiceError::Unauthorized("invite token already used"));
    }
    if expires_at < Utc::now() {
        return Err(AuthServiceError::Unauthorized("invite token expired"));
    }
    if !invite_email.eq_ignore_ascii_case(&email) {
        return Err(AuthServiceError::Unauthorized(
            "invite token does not match email",
        ));
    }

    let existing = sqlx::query_scalar::<_, Uuid>(
        "select id from auth_users where lower(email) = lower($1) limit 1",
    )
    .bind(&email)
    .fetch_optional(pool)
    .await
    .map_err(|_| AuthServiceError::Internal)?;

    if existing.is_some() {
        return Err(AuthServiceError::Conflict(
            "account already exists for email",
        ));
    }

    let password_hash = hash_password(request.password).await?;
    let user_id = Uuid::new_v4();

    let mut tx = pool.begin().await.map_err(|_| AuthServiceError::Internal)?;

    sqlx::query(
        "insert into auth_users (id, email, password_hash, status, created_at, updated_at) values ($1, $2, $3, 'active', now(), now())",
    )
    .bind(user_id)
    .bind(&email)
    .bind(&password_hash)
    .execute(&mut *tx)
    .await
    .map_err(|_| AuthServiceError::Internal)?;

    let update_result = sqlx::query(
        "update auth_invites set accepted_at = now() where id = $1 and accepted_at is null",
    )
    .bind(invite_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| AuthServiceError::Internal)?;

    if update_result.rows_affected() != 1 {
        return Err(AuthServiceError::Unauthorized("invite token already used"));
    }

    tx.commit().await.map_err(|_| AuthServiceError::Internal)?;

    Ok(SessionUser {
        user_id: user_id.to_string(),
        email,
    })
}

pub(crate) async fn change_session_password(
    state: &AppState,
    headers: &HeaderMap,
    request: ChangePasswordRequest,
) -> Result<(), AuthServiceError> {
    let current_password = request.current_password;
    let new_password = request.new_password;

    if current_password.is_empty() || new_password.is_empty() {
        return Err(AuthServiceError::InvalidRequest(
            "current_password and new_password are required",
        ));
    }
    if current_password == new_password {
        return Err(AuthServiceError::InvalidRequest(
            "new password must differ from current password",
        ));
    }
    validate_new_password_policy(&new_password)?;

    let session = require_session_state(state, headers).await?;
    let user_uuid = Uuid::parse_str(&session.user_id)
        .map_err(|_| AuthServiceError::InvalidRequest("invalid user id"))?;
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AuthServiceError::ServiceUnavailable("database unavailable"))?;

    let stored_hash = sqlx::query_scalar::<_, String>(
        "select password_hash from auth_users where id = $1 and status = 'active' limit 1",
    )
    .bind(user_uuid)
    .fetch_optional(pool)
    .await
    .map_err(|_| AuthServiceError::Internal)?;
    let Some(stored_hash) = stored_hash else {
        return Err(AuthServiceError::NotFound("user not found"));
    };

    match verify_password_hash(current_password, stored_hash).await {
        Ok(()) => {}
        Err(AuthServiceError::Unauthorized(_)) => {
            return Err(AuthServiceError::Unauthorized(
                "current password is incorrect",
            ));
        }
        Err(err) => return Err(err),
    }

    let new_hash = hash_password(new_password).await?;
    let result = sqlx::query(
        "update auth_users
         set password_hash = $2, updated_at = now()
         where id = $1 and status = 'active'",
    )
    .bind(user_uuid)
    .bind(&new_hash)
    .execute(pool)
    .await
    .map_err(|_| AuthServiceError::Internal)?;
    if result.rows_affected() != 1 {
        return Err(AuthServiceError::NotFound("user not found"));
    }

    Ok(())
}

pub(crate) async fn create_invite(
    state: &AppState,
    request: CreateInviteRequest,
) -> Result<CreateInviteResponse, AuthServiceError> {
    let email = request.email.trim().to_ascii_lowercase();
    if email.is_empty() || !looks_like_email(&email) {
        return Err(AuthServiceError::InvalidRequest("email format is invalid"));
    }

    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AuthServiceError::ServiceUnavailable("database unavailable"))?;

    let ttl = request
        .expires_in_seconds
        .unwrap_or(INVITE_TTL_SECONDS)
        .clamp(60, 7 * 24 * 60 * 60);
    let expires_at = Utc::now() + Duration::seconds(ttl as i64);

    let raw_token = generate_token();
    let token_hash = hash_token(&state.config.session_secret, &raw_token);

    sqlx::query(
        "insert into auth_invites (id, email, token_hash, expires_at, accepted_at, created_at) values ($1, $2, $3, $4, null, now())",
    )
    .bind(Uuid::new_v4())
    .bind(&email)
    .bind(&token_hash)
    .bind(expires_at)
    .execute(pool)
    .await
    .map_err(|_| AuthServiceError::Internal)?;

    let base = state.config.invite_base_url.trim_end_matches('/');
    let invite_url = format!("{base}/auth?invite_token={raw_token}");

    Ok(CreateInviteResponse {
        invite_url,
        expires_at,
    })
}

pub(crate) async fn establish_session(
    state: &AppState,
    user: &SessionUser,
) -> Result<String, AuthServiceError> {
    let redis_client = state
        .redis_client
        .as_ref()
        .ok_or(AuthServiceError::ServiceUnavailable(
            "session store unavailable",
        ))?;

    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| AuthServiceError::ServiceUnavailable("session store unavailable"))?;

    let session_id = generate_token();
    let key = format!("{SESSION_KEY_PREFIX}{session_id}");
    let issued_at = Utc::now();
    let role = resolve_admin_role(state, user).await?;
    let session = SessionState {
        user_id: user.user_id.clone(),
        email: user.email.clone(),
        role,
        issued_at,
        last_seen_at: issued_at,
        step_up_verified_at: None,
    };
    let value = serde_json::to_string(&session).map_err(|_| AuthServiceError::Internal)?;

    conn.set_ex::<_, _, ()>(key, value, state.config.session_ttl_seconds)
        .await
        .map_err(|_| AuthServiceError::ServiceUnavailable("session store unavailable"))?;
    upsert_auth_session_row(state, &session_id, &session, headers_client_ip(None), None).await;

    Ok(session_id)
}

pub(crate) async fn revoke_session(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(), AuthServiceError> {
    let Some(session_id) = extract_session_cookie(headers, &state.config.session_cookie_name)
    else {
        return Ok(());
    };

    let redis_client = state
        .redis_client
        .as_ref()
        .ok_or(AuthServiceError::ServiceUnavailable(
            "session store unavailable",
        ))?;
    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| AuthServiceError::ServiceUnavailable("session store unavailable"))?;

    let key = format!("{SESSION_KEY_PREFIX}{session_id}");
    conn.del::<_, ()>(key)
        .await
        .map_err(|_| AuthServiceError::ServiceUnavailable("session store unavailable"))?;
    mark_auth_session_revoked(state, session_id, "logout").await;

    Ok(())
}

pub(crate) async fn require_session_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<SessionUser, AuthServiceError> {
    current_session_user(state, headers)
        .await?
        .ok_or(AuthServiceError::Unauthorized("active session required"))
}

pub(crate) async fn require_admin_session_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<SessionUser, AuthServiceError> {
    let session = require_session_state(state, headers).await?;
    if !matches!(
        session.role,
        AdminRole::Admin | AdminRole::Operator | AdminRole::Viewer
    ) {
        return Err(AuthServiceError::Unauthorized("admin access required"));
    }
    Ok(SessionUser {
        user_id: session.user_id,
        email: session.email,
    })
}

pub(crate) async fn require_session_state(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<SessionState, AuthServiceError> {
    current_session_state(state, headers)
        .await?
        .ok_or(AuthServiceError::Unauthorized("active session required"))
}

pub(crate) async fn require_admin_session_state(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<SessionState, AuthServiceError> {
    let session = require_session_state(state, headers).await?;
    if !matches!(
        session.role,
        AdminRole::Admin | AdminRole::Operator | AdminRole::Viewer
    ) {
        return Err(AuthServiceError::Unauthorized("admin access required"));
    }
    Ok(session)
}

pub(crate) fn ensure_recent_step_up(session: &SessionState, ttl_seconds: u64) -> bool {
    let Some(verified_at) = session.step_up_verified_at else {
        return false;
    };
    let ttl = Duration::seconds(ttl_seconds as i64);
    Utc::now() - verified_at <= ttl
}

pub(crate) async fn mark_step_up_verified(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<SessionState, AuthServiceError> {
    let Some(session_id) = extract_session_cookie(headers, &state.config.session_cookie_name)
    else {
        return Err(AuthServiceError::Unauthorized("active session required"));
    };
    let mut session = require_session_state(state, headers).await?;
    session.step_up_verified_at = Some(Utc::now());
    session.last_seen_at = Utc::now();

    let redis_client = state
        .redis_client
        .as_ref()
        .ok_or(AuthServiceError::ServiceUnavailable(
            "session store unavailable",
        ))?;
    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| AuthServiceError::ServiceUnavailable("session store unavailable"))?;
    let key = format!("{SESSION_KEY_PREFIX}{session_id}");
    let value = serde_json::to_string(&session).map_err(|_| AuthServiceError::Internal)?;
    conn.set_ex::<_, _, ()>(key, value, state.config.session_ttl_seconds)
        .await
        .map_err(|_| AuthServiceError::ServiceUnavailable("session store unavailable"))?;
    upsert_auth_session_row(
        state,
        session_id,
        &session,
        headers_client_ip(Some(headers)),
        headers_user_agent(Some(headers)),
    )
    .await;
    Ok(session)
}

pub(crate) async fn current_session_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<SessionUser>, AuthServiceError> {
    let Some(session_id) = extract_session_cookie(headers, &state.config.session_cookie_name)
    else {
        return Ok(None);
    };

    let redis_client = state
        .redis_client
        .as_ref()
        .ok_or(AuthServiceError::ServiceUnavailable(
            "session store unavailable",
        ))?;

    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| AuthServiceError::ServiceUnavailable("session store unavailable"))?;

    let key = format!("{SESSION_KEY_PREFIX}{session_id}");
    let raw: Option<String> = conn
        .get(key)
        .await
        .map_err(|_| AuthServiceError::ServiceUnavailable("session store unavailable"))?;
    let Some(raw) = raw else {
        return Ok(None);
    };
    let session = parse_session_state(&raw)?;

    let user = SessionUser {
        user_id: session.user_id,
        email: session.email,
    };
    Ok(Some(user))
}

pub(crate) async fn current_session_state(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<SessionState>, AuthServiceError> {
    let Some(session_id) = extract_session_cookie(headers, &state.config.session_cookie_name)
    else {
        return Ok(None);
    };
    let redis_client = state
        .redis_client
        .as_ref()
        .ok_or(AuthServiceError::ServiceUnavailable(
            "session store unavailable",
        ))?;
    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| AuthServiceError::ServiceUnavailable("session store unavailable"))?;

    let key = format!("{SESSION_KEY_PREFIX}{session_id}");
    let raw: Option<String> = conn
        .get(&key)
        .await
        .map_err(|_| AuthServiceError::ServiceUnavailable("session store unavailable"))?;
    let Some(raw) = raw else {
        return Ok(None);
    };
    let mut session = parse_session_state(&raw)?;
    session.last_seen_at = Utc::now();
    let value = serde_json::to_string(&session).map_err(|_| AuthServiceError::Internal)?;
    conn.set_ex::<_, _, ()>(key, value, state.config.session_ttl_seconds)
        .await
        .map_err(|_| AuthServiceError::ServiceUnavailable("session store unavailable"))?;
    upsert_auth_session_row(
        state,
        session_id,
        &session,
        headers_client_ip(Some(headers)),
        headers_user_agent(Some(headers)),
    )
    .await;

    Ok(Some(session))
}

pub(crate) async fn load_user_by_id(
    state: &AppState,
    user_id: &str,
) -> Result<SessionUser, AuthServiceError> {
    let user_uuid = Uuid::parse_str(user_id)
        .map_err(|_| AuthServiceError::InvalidRequest("invalid user id"))?;

    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AuthServiceError::ServiceUnavailable("database unavailable"))?;

    let row = sqlx::query_as::<_, (Uuid, String)>(
        "select id, email from auth_users where id = $1 and status = 'active' limit 1",
    )
    .bind(user_uuid)
    .fetch_optional(pool)
    .await
    .map_err(|_| AuthServiceError::Internal)?;

    let Some((id, email)) = row else {
        return Err(AuthServiceError::NotFound("user not found"));
    };

    Ok(SessionUser {
        user_id: id.to_string(),
        email,
    })
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct PasskeySummary {
    pub(crate) credential_id: String,
    pub(crate) friendly_name: Option<String>,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) last_used_at: Option<DateTime<Utc>>,
}

pub(crate) async fn list_session_passkeys(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Vec<PasskeySummary>, AuthServiceError> {
    let session = require_session_state(state, headers).await?;
    let user_uuid = Uuid::parse_str(&session.user_id)
        .map_err(|_| AuthServiceError::InvalidRequest("invalid user id"))?;
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AuthServiceError::ServiceUnavailable("database unavailable"))?;
    let rows = sqlx::query_as::<_, (String, Option<String>, DateTime<Utc>, Option<DateTime<Utc>>)>(
        "select credential_id, friendly_name, created_at, last_used_at
         from user_passkeys
         where user_id = $1
         order by coalesce(last_used_at, created_at) desc",
    )
    .bind(user_uuid)
    .fetch_all(pool)
    .await
    .map_err(|_| AuthServiceError::Internal)?;

    Ok(rows
        .into_iter()
        .map(|row| PasskeySummary {
            credential_id: row.0,
            friendly_name: row.1,
            created_at: row.2,
            last_used_at: row.3,
        })
        .collect())
}

pub(crate) async fn delete_session_passkey(
    state: &AppState,
    headers: &HeaderMap,
    credential_id: &str,
) -> Result<(), AuthServiceError> {
    let session = require_session_state(state, headers).await?;
    let user_uuid = Uuid::parse_str(&session.user_id)
        .map_err(|_| AuthServiceError::InvalidRequest("invalid user id"))?;
    if credential_id.trim().is_empty() {
        return Err(AuthServiceError::InvalidRequest(
            "credential_id is required",
        ));
    }
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AuthServiceError::ServiceUnavailable("database unavailable"))?;
    let result = sqlx::query("delete from user_passkeys where user_id = $1 and credential_id = $2")
        .bind(user_uuid)
        .bind(credential_id.trim())
        .execute(pool)
        .await
        .map_err(|_| AuthServiceError::Internal)?;
    if result.rows_affected() == 0 {
        return Err(AuthServiceError::NotFound("passkey not found"));
    }
    Ok(())
}

pub(crate) async fn revoke_user_sessions(
    state: &AppState,
    user_id: &str,
    reason: &str,
) -> Result<u64, AuthServiceError> {
    let user_uuid = Uuid::parse_str(user_id)
        .map_err(|_| AuthServiceError::InvalidRequest("invalid user id"))?;
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AuthServiceError::ServiceUnavailable("database unavailable"))?;
    let result = sqlx::query(
        "update auth_sessions
         set revoked_at = now(), revoked_reason = $2
         where user_id = $1 and revoked_at is null",
    )
    .bind(user_uuid)
    .bind(reason)
    .execute(pool)
    .await
    .map_err(|_| AuthServiceError::Internal)?;
    Ok(result.rows_affected())
}

pub(crate) fn session_set_cookie(state: &AppState, session_id: &str) -> String {
    let secure = if state.config.app_env.eq_ignore_ascii_case("production") {
        "; Secure"
    } else {
        ""
    };
    let domain = state
        .config
        .session_cookie_domain
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("; Domain={value}"))
        .unwrap_or_default();

    format!(
        "{}={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}{}{}",
        state.config.session_cookie_name,
        session_id,
        state.config.session_ttl_seconds,
        secure,
        domain
    )
}

pub(crate) fn session_clear_cookie(state: &AppState) -> String {
    let secure = if state.config.app_env.eq_ignore_ascii_case("production") {
        "; Secure"
    } else {
        ""
    };

    let domain = state
        .config
        .session_cookie_domain
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("; Domain={value}"))
        .unwrap_or_default();

    format!(
        "{}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{}{}",
        state.config.session_cookie_name, secure, domain
    )
}

async fn ensure_signin_not_locked(
    state: &AppState,
    email: &str,
    client_ip: &str,
) -> Result<(), AuthServiceError> {
    let redis_client = state
        .redis_client
        .as_ref()
        .ok_or(AuthServiceError::ServiceUnavailable(
            "authentication rate-limit store unavailable",
        ))?;
    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| {
            AuthServiceError::ServiceUnavailable("authentication rate-limit store unavailable")
        })?;

    let keys = signin_rate_keys(state, email, client_ip);
    let email_locked: bool = conn.exists(keys.email_lock).await.map_err(|_| {
        AuthServiceError::ServiceUnavailable("authentication rate-limit check failed")
    })?;
    let ip_locked: bool = conn.exists(keys.ip_lock).await.map_err(|_| {
        AuthServiceError::ServiceUnavailable("authentication rate-limit check failed")
    })?;

    if email_locked || ip_locked {
        return Err(AuthServiceError::Unauthorized(
            "too many failed sign-in attempts, try again later",
        ));
    }
    Ok(())
}

async fn record_signin_failure(
    state: &AppState,
    email: &str,
    client_ip: &str,
) -> Result<(), AuthServiceError> {
    let redis_client = state
        .redis_client
        .as_ref()
        .ok_or(AuthServiceError::ServiceUnavailable(
            "authentication rate-limit store unavailable",
        ))?;
    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| {
            AuthServiceError::ServiceUnavailable("authentication rate-limit store unavailable")
        })?;

    let keys = signin_rate_keys(state, email, client_ip);
    let email_count: u64 = conn
        .incr(keys.email_fail.clone(), 1_u64)
        .await
        .map_err(|_| {
            AuthServiceError::ServiceUnavailable("authentication failure tracking failed")
        })?;
    let _: bool = conn
        .expire(keys.email_fail.clone(), SIGNIN_FAIL_WINDOW_SECONDS as i64)
        .await
        .map_err(|_| {
            AuthServiceError::ServiceUnavailable("authentication failure tracking failed")
        })?;

    let ip_count: u64 = conn.incr(keys.ip_fail.clone(), 1_u64).await.map_err(|_| {
        AuthServiceError::ServiceUnavailable("authentication failure tracking failed")
    })?;
    let _: bool = conn
        .expire(keys.ip_fail.clone(), SIGNIN_FAIL_WINDOW_SECONDS as i64)
        .await
        .map_err(|_| {
            AuthServiceError::ServiceUnavailable("authentication failure tracking failed")
        })?;

    if email_count >= SIGNIN_MAX_FAILS_PER_EMAIL {
        conn.set_ex::<_, _, ()>(keys.email_lock, "1", SIGNIN_LOCKOUT_SECONDS)
            .await
            .map_err(|_| {
                AuthServiceError::ServiceUnavailable("authentication lockout activation failed")
            })?;
    }

    if ip_count >= SIGNIN_MAX_FAILS_PER_IP {
        conn.set_ex::<_, _, ()>(keys.ip_lock, "1", SIGNIN_LOCKOUT_SECONDS)
            .await
            .map_err(|_| {
                AuthServiceError::ServiceUnavailable("authentication lockout activation failed")
            })?;
    }

    Ok(())
}

async fn clear_signin_failure_state(
    state: &AppState,
    email: &str,
    client_ip: &str,
) -> Result<(), AuthServiceError> {
    let redis_client = state
        .redis_client
        .as_ref()
        .ok_or(AuthServiceError::ServiceUnavailable(
            "authentication rate-limit store unavailable",
        ))?;
    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| {
            AuthServiceError::ServiceUnavailable("authentication rate-limit store unavailable")
        })?;

    let keys = signin_rate_keys(state, email, client_ip);
    let _: u64 = conn
        .del(&[
            keys.email_fail.as_str(),
            keys.ip_fail.as_str(),
            keys.email_lock.as_str(),
            keys.ip_lock.as_str(),
        ])
        .await
        .map_err(|_| AuthServiceError::ServiceUnavailable("authentication failure reset failed"))?;

    Ok(())
}

struct SigninRateKeys {
    email_fail: String,
    ip_fail: String,
    email_lock: String,
    ip_lock: String,
}

fn signin_rate_keys(state: &AppState, email: &str, client_ip: &str) -> SigninRateKeys {
    let email_id = hash_token(&state.config.session_secret, &email.to_ascii_lowercase());
    let ip_id = hash_token(&state.config.session_secret, client_ip);

    SigninRateKeys {
        email_fail: format!("{SIGNIN_FAIL_EMAIL_PREFIX}{email_id}"),
        ip_fail: format!("{SIGNIN_FAIL_IP_PREFIX}{ip_id}"),
        email_lock: format!("{SIGNIN_LOCK_EMAIL_PREFIX}{email_id}"),
        ip_lock: format!("{SIGNIN_LOCK_IP_PREFIX}{ip_id}"),
    }
}

fn is_admin_email(email: &str) -> bool {
    let configured = env::var(ADMIN_EMAILS_ENV).unwrap_or_default();
    is_admin_email_with_list(&configured, email)
}

fn is_admin_email_with_list(admin_emails: &str, email: &str) -> bool {
    let normalized_email = email.trim().to_ascii_lowercase();
    if normalized_email.is_empty() {
        return false;
    }

    admin_emails
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase)
        .any(|value| value == normalized_email)
}

fn parse_admin_role(raw: &str) -> Option<AdminRole> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "admin" => Some(AdminRole::Admin),
        "operator" => Some(AdminRole::Operator),
        "viewer" => Some(AdminRole::Viewer),
        "user" => Some(AdminRole::User),
        _ => None,
    }
}

pub(crate) fn admin_role_as_str(role: &AdminRole) -> &'static str {
    match role {
        AdminRole::Admin => "admin",
        AdminRole::Operator => "operator",
        AdminRole::Viewer => "viewer",
        AdminRole::User => "user",
    }
}

pub(crate) fn admin_role_can_mutate(role: &AdminRole) -> bool {
    matches!(role, AdminRole::Admin | AdminRole::Operator)
}

pub(crate) fn parse_session_state(raw: &str) -> Result<SessionState, AuthServiceError> {
    match serde_json::from_str::<SessionState>(raw) {
        Ok(session) => Ok(session),
        Err(_) => {
            let legacy =
                serde_json::from_str::<SessionUser>(raw).map_err(|_| AuthServiceError::Internal)?;
            let now = Utc::now();
            Ok(SessionState {
                user_id: legacy.user_id,
                email: legacy.email,
                role: AdminRole::User,
                issued_at: now,
                last_seen_at: now,
                step_up_verified_at: None,
            })
        }
    }
}

async fn resolve_admin_role(
    state: &AppState,
    user: &SessionUser,
) -> Result<AdminRole, AuthServiceError> {
    let user_id = match Uuid::parse_str(&user.user_id) {
        Ok(value) => value,
        Err(_) => return Ok(AdminRole::User),
    };

    let Some(pool) = state.db_pool.as_ref() else {
        if is_admin_email(&user.email) {
            return Ok(AdminRole::Admin);
        }
        return Ok(AdminRole::User);
    };

    let row = sqlx::query_as::<_, (String, bool)>(
        "select role, access_enabled from auth_user_role_bindings where user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await;

    match row {
        Ok(Some((role_raw, access_enabled))) => {
            if !access_enabled {
                return Err(AuthServiceError::Unauthorized("account access disabled"));
            }
            match parse_admin_role(&role_raw) {
                Some(role) => Ok(role),
                None => Ok(AdminRole::User),
            }
        }
        Ok(None) => {
            if is_admin_email(&user.email) {
                Ok(AdminRole::Admin)
            } else {
                Ok(AdminRole::User)
            }
        }
        Err(_) => {
            if is_admin_email(&user.email) {
                Ok(AdminRole::Admin)
            } else {
                Ok(AdminRole::User)
            }
        }
    }
}

fn hash_session_id(state: &AppState, session_id: &str) -> String {
    hash_token(&state.config.session_secret, session_id)
}

fn headers_client_ip(headers: Option<&HeaderMap>) -> Option<String> {
    let Some(headers) = headers else {
        return None;
    };
    let ip = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|value| value.to_str().ok())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
        });
    ip
}

fn headers_user_agent(headers: Option<&HeaderMap>) -> Option<String> {
    let Some(headers) = headers else {
        return None;
    };
    headers
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

async fn upsert_auth_session_row(
    state: &AppState,
    session_id: &str,
    session: &SessionState,
    ip: Option<String>,
    user_agent: Option<String>,
) {
    let Some(pool) = state.db_pool.as_ref() else {
        return;
    };
    let Ok(user_uuid) = Uuid::parse_str(&session.user_id) else {
        return;
    };
    let ip_addr = ip.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });
    let _ = sqlx::query(
        "insert into auth_sessions (session_id_hash, user_id, email, role, issued_at, last_seen_at, step_up_verified_at, ip, user_agent)
         values ($1, $2, $3, $4, $5, $6, $7, cast($8 as inet), $9)
         on conflict (session_id_hash)
         do update set role = excluded.role, last_seen_at = excluded.last_seen_at, step_up_verified_at = excluded.step_up_verified_at, ip = coalesce(excluded.ip, auth_sessions.ip), user_agent = coalesce(excluded.user_agent, auth_sessions.user_agent), revoked_at = null, revoked_reason = null",
    )
    .bind(hash_session_id(state, session_id))
    .bind(user_uuid)
    .bind(&session.email)
    .bind(admin_role_as_str(&session.role))
    .bind(session.issued_at)
    .bind(session.last_seen_at)
    .bind(session.step_up_verified_at)
    .bind(ip_addr)
    .bind(user_agent)
    .execute(pool)
    .await;
}

async fn mark_auth_session_revoked(state: &AppState, session_id: &str, reason: &str) {
    let Some(pool) = state.db_pool.as_ref() else {
        return;
    };
    let _ = sqlx::query(
        "update auth_sessions set revoked_at = now(), revoked_reason = $2 where session_id_hash = $1",
    )
    .bind(hash_session_id(state, session_id))
    .bind(reason)
    .execute(pool)
    .await;
}

async fn verify_password_hash(
    password: String,
    password_hash: String,
) -> Result<(), AuthServiceError> {
    run_password_hash_work("verify", move || {
        verify_password_hash_blocking(&password, &password_hash)
    })
    .await
}

async fn hash_password(password: String) -> Result<String, AuthServiceError> {
    run_password_hash_work("hash", move || hash_password_blocking(&password)).await
}

async fn run_password_hash_work<T, F>(
    operation: &'static str,
    work: F,
) -> Result<T, AuthServiceError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, AuthServiceError> + Send + 'static,
{
    let semaphore = password_hash_semaphore();
    let limit = password_hash_concurrency_limit();

    let wait_start = Instant::now();
    let _permit = semaphore
        .acquire_owned()
        .await
        .map_err(|_| AuthServiceError::Internal)?;
    let wait_ms = wait_start.elapsed().as_millis() as u64;

    let exec_start = Instant::now();
    let result = task::spawn_blocking(work)
        .await
        .map_err(|_| AuthServiceError::Internal)?;
    let exec_ms = exec_start.elapsed().as_millis() as u64;

    info!(
        target: "auth.password_hash",
        operation,
        wait_ms,
        exec_ms,
        total_ms = wait_ms + exec_ms,
        concurrency_limit = limit,
        "password hash workload timing"
    );

    result
}

fn verify_password_hash_blocking(
    password: &str,
    password_hash: &str,
) -> Result<(), AuthServiceError> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|_| AuthServiceError::Unauthorized("invalid email or password"))?;
    if !is_supported_argon2_hash(&parsed_hash) {
        return Err(AuthServiceError::Unauthorized("invalid email or password"));
    }

    let argon2 = argon2_hasher()?;
    argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AuthServiceError::Unauthorized("invalid email or password"))
}

fn hash_password_blocking(password: &str) -> Result<String, AuthServiceError> {
    let argon2 = argon2_hasher()?;
    let salt = SaltString::generate(&mut OsRng);
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| AuthServiceError::Internal)?;
    Ok(hash.to_string())
}

fn argon2_hasher() -> Result<Argon2<'static>, AuthServiceError> {
    let params = Params::new(
        ARGON2_MEMORY_KIB,
        ARGON2_TIME_COST,
        ARGON2_PARALLELISM,
        None,
    )
    .map_err(|_| AuthServiceError::Internal)?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

fn is_supported_argon2_hash(hash: &PasswordHash<'_>) -> bool {
    let Ok(expected_alg) = Ident::new("argon2id") else {
        return false;
    };

    hash.algorithm == expected_alg
        && hash.version == Some(19)
        && hash.params.get_decimal("m") == Some(ARGON2_MEMORY_KIB)
        && hash.params.get_decimal("t") == Some(ARGON2_TIME_COST)
        && hash.params.get_decimal("p") == Some(ARGON2_PARALLELISM)
}

fn looks_like_email(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value.contains('@')
        && value.rsplit('.').next().is_some_and(|v| !v.is_empty())
}

fn validate_new_password_policy(password: &str) -> Result<(), AuthServiceError> {
    if password.len() < 10 {
        return Err(AuthServiceError::InvalidRequest(
            "new password must be at least 10 characters",
        ));
    }
    if !password.chars().any(|ch| ch.is_ascii_lowercase()) {
        return Err(AuthServiceError::InvalidRequest(
            "new password must include a lowercase letter",
        ));
    }
    if !password.chars().any(|ch| ch.is_ascii_uppercase()) {
        return Err(AuthServiceError::InvalidRequest(
            "new password must include an uppercase letter",
        ));
    }
    if !password.chars().any(|ch| ch.is_ascii_digit()) {
        return Err(AuthServiceError::InvalidRequest(
            "new password must include a number",
        ));
    }
    if !password.chars().any(|ch| !ch.is_ascii_alphanumeric()) {
        return Err(AuthServiceError::InvalidRequest(
            "new password must include a symbol",
        ));
    }

    Ok(())
}

fn extract_session_cookie<'a>(headers: &'a HeaderMap, cookie_name: &str) -> Option<&'a str> {
    let raw_cookie = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;

    raw_cookie
        .split(';')
        .filter_map(|entry| {
            let mut parts = entry.trim().splitn(2, '=');
            let key = parts.next()?.trim();
            let value = parts.next()?.trim();
            Some((key, value))
        })
        .find_map(|(key, value)| (key == cookie_name).then_some(value))
}

#[cfg(test)]
mod tests {
    use std::{
        net::TcpListener,
        process::{Child, Command, Stdio},
        time::Duration,
    };

    use argon2::{
        Argon2,
        password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    };
    use bominal_shared::config::{
        AppConfig, EvervaultConfig, PasskeyConfig, PasskeyProvider, RedisConfig, RuntimeSchedule,
    };
    use tokio::time::sleep;

    use super::{
        ARGON2_MEMORY_KIB, ARGON2_PARALLELISM, ARGON2_TIME_COST, SIGNIN_MAX_FAILS_PER_EMAIL,
        SIGNIN_MAX_FAILS_PER_IP, clear_signin_failure_state, ensure_signin_not_locked,
        hash_password_blocking, record_signin_failure, session_clear_cookie, session_set_cookie,
        validate_new_password_policy, verify_password_hash_blocking,
    };

    #[test]
    fn password_hash_uses_expected_argon2_profile() {
        let hashed =
            hash_password_blocking("test-password").expect("password hashing should succeed");
        assert!(hashed.starts_with("$argon2id$"));
        assert!(hashed.contains(&format!(
            "m={ARGON2_MEMORY_KIB},t={ARGON2_TIME_COST},p={ARGON2_PARALLELISM}"
        )));
    }

    #[test]
    fn password_verify_is_success_and_fail_closed() {
        let password = "correct-horse-battery-staple";
        let hashed = hash_password_blocking(password).expect("password hashing should succeed");

        verify_password_hash_blocking(password, &hashed).expect("correct password should verify");

        let wrong = verify_password_hash_blocking("wrong-password", &hashed);
        assert!(wrong.is_err());
    }

    #[test]
    fn non_argon2_hash_format_is_rejected_fail_closed() {
        let non_argon2_hash = "$2b$10$YwhsW7X4M59g3mqY2aUQ1eF5QlNydPjKk7w8v8Q1E1Xq7u0wG4hZO";
        let verification = verify_password_hash_blocking("pw", non_argon2_hash);
        assert!(matches!(
            verification,
            Err(super::AuthServiceError::Unauthorized(_))
        ));
    }

    #[test]
    fn non_current_argon2_parameters_are_rejected_fail_closed() {
        let password = "argon2-legacy-password";
        let legacy_hash = Argon2::default()
            .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
            .expect("legacy hash generation should succeed")
            .to_string();

        let verification = verify_password_hash_blocking(password, &legacy_hash);
        assert!(matches!(
            verification,
            Err(super::AuthServiceError::Unauthorized(_))
        ));
    }

    #[test]
    fn admin_email_matching_uses_case_insensitive_list() {
        assert!(super::is_admin_email_with_list(
            "admin@bominal.com,ops@bominal.com",
            "ADMIN@BOMINAL.COM",
        ));
        assert!(!super::is_admin_email_with_list(
            "admin@bominal.com,ops@bominal.com",
            "user@bominal.com",
        ));
        assert!(!super::is_admin_email_with_list("", "admin@bominal.com"));
    }

    #[test]
    fn validate_new_password_policy_accepts_strong_password() {
        let result = validate_new_password_policy("StrongPass#2026");
        assert!(result.is_ok());
    }

    #[test]
    fn validate_new_password_policy_rejects_short_password() {
        let result = validate_new_password_policy("S1#short");
        assert!(matches!(
            result,
            Err(super::AuthServiceError::InvalidRequest(
                "new password must be at least 10 characters"
            ))
        ));
    }

    #[test]
    fn validate_new_password_policy_rejects_missing_symbol() {
        let result = validate_new_password_policy("StrongPass2026");
        assert!(matches!(
            result,
            Err(super::AuthServiceError::InvalidRequest(
                "new password must include a symbol"
            ))
        ));
    }

    struct RedisTestServer {
        child: Child,
        url: String,
    }

    impl RedisTestServer {
        async fn start() -> Option<Self> {
            if Command::new("redis-server")
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .ok()?
                .success()
                == false
            {
                return None;
            }

            let port = free_port()?;
            let child = Command::new("redis-server")
                .arg("--save")
                .arg("")
                .arg("--appendonly")
                .arg("no")
                .arg("--bind")
                .arg("127.0.0.1")
                .arg("--port")
                .arg(port.to_string())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .ok()?;
            let url = format!("redis://127.0.0.1:{port}/0");
            if wait_for_redis(&url).await {
                Some(Self { child, url })
            } else {
                None
            }
        }
    }

    impl Drop for RedisTestServer {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }

    fn free_port() -> Option<u16> {
        let listener = TcpListener::bind("127.0.0.1:0").ok()?;
        listener.local_addr().ok().map(|addr| addr.port())
    }

    async fn wait_for_redis(url: &str) -> bool {
        for _ in 0..40 {
            if let Ok(client) = redis::Client::open(url) {
                if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                    let pong: redis::RedisResult<String> =
                        redis::cmd("PING").query_async(&mut conn).await;
                    if matches!(pong.as_deref(), Ok("PONG")) {
                        return true;
                    }
                }
            }
            sleep(Duration::from_millis(100)).await;
        }
        false
    }

    fn test_state_with_redis(redis_url: &str) -> super::AppState {
        let config = AppConfig {
            app_env: "test".to_string(),
            app_host: "127.0.0.1".to_string(),
            app_port: 0,
            log_json: false,
            session_cookie_name: "bominal_session".to_string(),
            session_cookie_domain: None,
            session_ttl_seconds: 3600,
            step_up_ttl_seconds: 600,
            session_secret: "test-session-secret".to_string(),
            invite_base_url: "http://127.0.0.1:8000".to_string(),
            user_app_host: "www.bominal.com".to_string(),
            admin_app_host: "ops.bominal.com".to_string(),
            ui_theme_cookie_name: "bominal_theme".to_string(),
            database_url: String::new(),
            redis: RedisConfig {
                url: redis_url.to_string(),
                queue_key: "test:runtime:queue".to_string(),
                queue_dlq_key: "test:runtime:queue:dlq".to_string(),
                lease_prefix: "test:runtime:lease".to_string(),
                rate_limit_prefix: "test:runtime:rate".to_string(),
            },
            evervault: EvervaultConfig {
                relay_base_url: "https://relay.evervault.com".to_string(),
                app_id: None,
            },
            resend: None,
            passkey: PasskeyConfig {
                provider: PasskeyProvider::ServerWebauthn,
                webauthn_rp_id: "localhost".to_string(),
                webauthn_rp_origin: "http://localhost:8000".to_string(),
                webauthn_rp_name: "bominal".to_string(),
                webauthn_challenge_ttl_seconds: 300,
            },
            runtime: RuntimeSchedule {
                poll_interval: Duration::from_secs(1),
                reconcile_interval: Duration::from_secs(1),
                watch_interval: Duration::from_secs(1),
                key_rotation_interval: Duration::from_secs(1),
            },
        };

        super::AppState {
            config,
            db_pool: None,
            redis_client: Some(
                redis::Client::open(redis_url).expect("redis url should be valid for test state"),
            ),
            metrics_handle: super::super::super::init_metrics_recorder()
                .expect("metrics recorder should initialize for tests"),
            http_client: reqwest::Client::new(),
            webauthn: None,
        }
    }

    #[tokio::test]
    async fn lockout_triggers_after_email_threshold_and_resets() {
        let Some(redis_server) = RedisTestServer::start().await else {
            eprintln!("redis-server not available; skipping lockout integration test");
            return;
        };

        let state = test_state_with_redis(&redis_server.url);
        let email = "lockout-email@example.com";
        let client_ip = "198.51.100.9";

        for _ in 0..SIGNIN_MAX_FAILS_PER_EMAIL {
            record_signin_failure(&state, email, client_ip)
                .await
                .expect("recording failure should succeed");
        }

        let locked = ensure_signin_not_locked(&state, email, client_ip).await;
        assert!(matches!(
            locked,
            Err(super::AuthServiceError::Unauthorized(
                "too many failed sign-in attempts, try again later"
            ))
        ));

        clear_signin_failure_state(&state, email, client_ip)
            .await
            .expect("clearing failure state should succeed");

        ensure_signin_not_locked(&state, email, client_ip)
            .await
            .expect("lockout should be cleared after reset");
    }

    #[tokio::test]
    async fn ip_lockout_threshold_applies_across_emails() {
        let Some(redis_server) = RedisTestServer::start().await else {
            eprintln!("redis-server not available; skipping lockout integration test");
            return;
        };

        let state = test_state_with_redis(&redis_server.url);
        let client_ip = "203.0.113.15";

        for index in 0..SIGNIN_MAX_FAILS_PER_IP {
            let email = format!("user-{index}@example.com");
            record_signin_failure(&state, &email, client_ip)
                .await
                .expect("recording failure should succeed");
        }

        let locked = ensure_signin_not_locked(&state, "fresh-email@example.com", client_ip).await;
        assert!(matches!(
            locked,
            Err(super::AuthServiceError::Unauthorized(
                "too many failed sign-in attempts, try again later"
            ))
        ));

        ensure_signin_not_locked(&state, "fresh-email@example.com", "203.0.113.16")
            .await
            .expect("different IP should not be locked");
    }

    #[test]
    fn session_cookie_omits_domain_when_not_configured() {
        let state = test_state_with_redis("redis://127.0.0.1:6379");
        let set_cookie = session_set_cookie(&state, "session-id-1");
        let clear_cookie = session_clear_cookie(&state);
        assert!(!set_cookie.contains("Domain="));
        assert!(!clear_cookie.contains("Domain="));
    }

    #[test]
    fn session_cookie_includes_domain_when_configured() {
        let mut state = test_state_with_redis("redis://127.0.0.1:6379");
        state.config.session_cookie_domain = Some(".bominal.com".to_string());
        let set_cookie = session_set_cookie(&state, "session-id-2");
        let clear_cookie = session_clear_cookie(&state);
        assert!(set_cookie.contains("Domain=.bominal.com"));
        assert!(clear_cookie.contains("Domain=.bominal.com"));
    }
}

fn generate_token() -> String {
    let first = Uuid::new_v4().simple().to_string();
    let second = Uuid::new_v4().simple().to_string();
    format!("{first}{second}")
}

fn hash_token(secret: &str, value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(b":");
    hasher.update(value.as_bytes());

    let digest = hasher.finalize();
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(&mut out, "{byte:02x}");
    }

    out
}
