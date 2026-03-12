//! Auth route handlers: register, login, logout, email verification, password reset.

use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::http::HeaderMap;
use axum::Json;
use chrono::{Duration, Utc};
use serde::Deserialize;
use uuid::Uuid;

use bominal_domain::auth::{
    AuthResponse, LoginRequest, RegisterRequest, validate_display_name, validate_email,
    validate_password,
};
use bominal_domain::crypto::password::{hash_password, verify_password};

use crate::error::AppError;
use crate::state::SharedState;

const SESSION_COOKIE_NAME: &str = "bominal_session";
const SESSION_TTL_HOURS: i64 = 24;
const VERIFY_TOKEN_TTL_MINUTES: i64 = 30;
const RESET_TOKEN_TTL_MINUTES: i64 = 15;

/// POST /api/auth/register
pub async fn register(
    State(state): State<SharedState>,
    Json(req): Json<RegisterRequest>,
) -> Result<(HeaderMap, Json<AuthResponse>), AppError> {
    validate_email(&req.email).map_err(|e| AppError::BadRequest(e.to_string()))?;
    validate_password(&req.password).map_err(|e| AppError::BadRequest(e.to_string()))?;
    validate_display_name(&req.display_name).map_err(|e| AppError::BadRequest(e.to_string()))?;

    // Check for existing user
    let existing = bominal_db::user::find_by_email(&state.db, &req.email)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    if existing.is_some() {
        return Err(AppError::BadRequest("Email already registered".to_string()));
    }

    // Hash password
    let pw_hash =
        hash_password(&req.password).map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?;

    // Create user
    let user_row = bominal_db::user::create_user(&state.db, &req.email, &req.display_name, &pw_hash)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    // Send verification email (best-effort, don't block registration)
    send_verification_email(&state, user_row.id, &user_row.email, &user_row.display_name).await;

    // Create session
    let (headers, response) = create_session_response(&state, &user_row).await?;
    Ok((headers, Json(response)))
}

/// POST /api/auth/login
pub async fn login(
    State(state): State<SharedState>,
    Json(req): Json<LoginRequest>,
) -> Result<(HeaderMap, Json<AuthResponse>), AppError> {
    let user_row = bominal_db::user::find_by_email(&state.db, &req.email)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or(AppError::Unauthorized)?;

    verify_password(&req.password, &user_row.password_hash)
        .map_err(|_| AppError::Unauthorized)?;

    let (headers, response) = create_session_response(&state, &user_row).await?;
    Ok((headers, Json(response)))
}

/// POST /api/auth/logout
pub async fn logout(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Result<HeaderMap, AppError> {
    if let Some(session_id) = extract_session_id(&headers) {
        bominal_db::session::delete_session(&state.db, &session_id)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
    }

    let domain_attr = cookie_domain_attr(&state.app_base_url);
    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(
        SET_COOKIE,
        format!(
            "{}=; Path=/; HttpOnly; SameSite=Lax; Secure; Max-Age=0{}",
            SESSION_COOKIE_NAME, domain_attr
        )
        .parse()
        .unwrap(),
    );
    Ok(resp_headers)
}

/// GET /api/auth/me — return current user info from session.
pub async fn me(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Result<Json<AuthResponse>, AppError> {
    let session_id = extract_session_id(&headers).ok_or(AppError::Unauthorized)?;

    let session = bominal_db::session::find_valid_session(&state.db, &session_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or(AppError::Unauthorized)?;

    let user = bominal_db::user::find_by_id(&state.db, session.user_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or(AppError::Unauthorized)?;

    Ok(Json(AuthResponse {
        user_id: user.id,
        email: user.email,
        display_name: user.display_name,
        preferred_locale: user.preferred_locale,
    }))
}

// ── Email verification ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}

/// POST /api/auth/verify-email — confirm email with token.
pub async fn verify_email(
    State(state): State<SharedState>,
    Json(req): Json<VerifyEmailRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = bominal_db::user::verify_email(&state.db, &req.token)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::BadRequest("Invalid or expired verification link".to_string()))?;

    tracing::info!(user_id = %user.id, "Email verified");

    Ok(Json(serde_json::json!({ "verified": true })))
}

/// POST /api/auth/resend-verification — resend verification email.
pub async fn resend_verification(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    let session_id = extract_session_id(&headers).ok_or(AppError::Unauthorized)?;
    let session = bominal_db::session::find_valid_session(&state.db, &session_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or(AppError::Unauthorized)?;

    let user = bominal_db::user::find_by_id(&state.db, session.user_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or(AppError::Unauthorized)?;

    if user.email_verified {
        return Err(AppError::BadRequest("Email already verified".to_string()));
    }

    send_verification_email(&state, user.id, &user.email, &user.display_name).await;

    Ok(Json(serde_json::json!({ "sent": true })))
}

// ── Password reset ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

/// POST /api/auth/forgot-password — request a password reset email.
///
/// Always returns 200 to avoid leaking whether the email exists.
pub async fn forgot_password(
    State(state): State<SharedState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Always return success (don't leak email existence)
    let user = bominal_db::user::find_by_email(&state.db, &req.email)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    if let Some(user) = user {
        let token = generate_token();
        let expires_at = Utc::now() + Duration::minutes(RESET_TOKEN_TTL_MINUTES);

        bominal_db::user::set_reset_token(&state.db, user.id, &token, expires_at)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        let reset_url = format!("{}/reset-password?token={}", state.app_base_url, token);
        let (subject, html) = bominal_email::templates::reset::render(
            &user.display_name,
            &reset_url,
            RESET_TOKEN_TTL_MINUTES as u32,
        );

        state.email.send_best_effort(&user.email, &subject, &html).await;
    }

    Ok(Json(serde_json::json!({ "sent": true })))
}

/// POST /api/auth/reset-password — set a new password using the reset token.
pub async fn reset_password(
    State(state): State<SharedState>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_password(&req.new_password).map_err(|e| AppError::BadRequest(e.to_string()))?;

    let pw_hash = hash_password(&req.new_password)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?;

    let user = bominal_db::user::reset_password(&state.db, &req.token, &pw_hash)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or_else(|| AppError::BadRequest("Invalid or expired reset link".to_string()))?;

    tracing::info!(user_id = %user.id, "Password reset completed");

    Ok(Json(serde_json::json!({ "reset": true })))
}

// ── Helpers ──────────────────────────────────────────────────────────

async fn send_verification_email(state: &SharedState, user_id: Uuid, email: &str, display_name: &str) {
    let token = generate_token();
    let expires_at = Utc::now() + Duration::minutes(VERIFY_TOKEN_TTL_MINUTES);

    if let Err(e) = bominal_db::user::set_verification_token(&state.db, user_id, &token, expires_at).await {
        tracing::error!(user_id = %user_id, error = %e, "Failed to set verification token");
        return;
    }

    let verify_url = format!("{}/verify-email?token={}", state.app_base_url, token);
    let (subject, html) = bominal_email::templates::verify::render(
        display_name,
        &verify_url,
        VERIFY_TOKEN_TTL_MINUTES as u32,
    );

    state.email.send_best_effort(email, &subject, &html).await;
}

fn generate_token() -> String {
    Uuid::new_v4().to_string()
}

pub(crate) fn cookie_domain_attr(app_base_url: &str) -> String {
    if app_base_url.starts_with("https://") {
        url::Url::parse(app_base_url)
            .ok()
            .and_then(|u| u.host_str().map(|h| format!("; Domain={h}")))
            .unwrap_or_default()
    } else {
        String::new()
    }
}

async fn create_session_response(
    state: &SharedState,
    user: &bominal_db::user::UserRow,
) -> Result<(HeaderMap, AuthResponse), AppError> {
    let session_id = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(SESSION_TTL_HOURS);

    bominal_db::session::create_session(&state.db, &session_id, user.id, expires_at)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let domain_attr = cookie_domain_attr(&state.app_base_url);

    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        format!(
            "{}={}; Path=/; HttpOnly; SameSite=Lax; Secure; Max-Age={}{}",
            SESSION_COOKIE_NAME,
            session_id,
            SESSION_TTL_HOURS * 3600,
            domain_attr
        )
        .parse()
        .unwrap(),
    );

    let response = AuthResponse {
        user_id: user.id,
        email: user.email.clone(),
        display_name: user.display_name.clone(),
        preferred_locale: user.preferred_locale.clone(),
    };

    Ok((headers, response))
}

fn extract_session_id(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get("cookie")?.to_str().ok()?;
    for part in cookie_header.split(';') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix(&format!("{}=", SESSION_COOKIE_NAME)) {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_session_from_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "cookie",
            "bominal_session=abc-123; other=val".parse().unwrap(),
        );
        assert_eq!(extract_session_id(&headers), Some("abc-123".to_string()));
    }

    #[test]
    fn extract_session_missing() {
        let headers = HeaderMap::new();
        assert_eq!(extract_session_id(&headers), None);
    }

    #[test]
    fn extract_session_no_match() {
        let mut headers = HeaderMap::new();
        headers.insert("cookie", "other=val".parse().unwrap());
        assert_eq!(extract_session_id(&headers), None);
    }

    #[test]
    fn generate_token_is_uuid() {
        let token = generate_token();
        assert!(Uuid::parse_str(&token).is_ok());
    }
}
