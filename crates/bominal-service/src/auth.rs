//! Auth service — authenticate, register, session management.
//!
//! Cookie/header/redirect handling stays in the callers (transport layer).
//! This module handles the pure business logic: validate, hash, query, email.

use uuid::Uuid;

use crate::error::ServiceError;
use crate::{DbPool, ServiceContext};

/// User info returned after auth operations.
pub use bominal_domain::auth::AuthResponse as UserInfo;

pub const SESSION_COOKIE_NAME: &str = "bominal_session";
pub const SESSION_TTL_HOURS: i64 = 24;

/// Authenticate a user by email and password. Returns the user row on success.
pub async fn authenticate(
    db: &DbPool,
    email: &str,
    password: &str,
) -> Result<bominal_db::user::UserRow, ServiceError> {
    let user = bominal_db::user::find_by_email(db, email)
        .await?
        .ok_or(ServiceError::Unauthorized)?;

    bominal_domain::crypto::password::verify_password(password, &user.password_hash)
        .map_err(|_| ServiceError::Unauthorized)?;

    Ok(user)
}

/// Register a new user. Returns the user row on success.
pub async fn register(
    db: &DbPool,
    email: &str,
    password: &str,
    display_name: &str,
) -> Result<bominal_db::user::UserRow, ServiceError> {
    use bominal_domain::auth::{validate_display_name, validate_email, validate_password};

    validate_email(email).map_err(ServiceError::validation)?;
    validate_password(password).map_err(ServiceError::validation)?;
    validate_display_name(display_name).map_err(ServiceError::validation)?;

    let existing = bominal_db::user::find_by_email(db, email).await?;
    if existing.is_some() {
        return Err(ServiceError::validation("Email already registered"));
    }

    let pw_hash = bominal_domain::crypto::password::hash_password(password)
        .map_err(ServiceError::internal)?;

    let user = bominal_db::user::create_user(db, email, display_name, &pw_hash).await?;

    Ok(user)
}

/// Create a session and return the session ID.
pub async fn create_session(db: &DbPool, user_id: Uuid) -> Result<String, ServiceError> {
    let session_id = Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(SESSION_TTL_HOURS);

    bominal_db::session::create_session(db, &session_id, user_id, expires_at).await?;

    Ok(session_id)
}

/// Delete a session.
pub async fn delete_session(db: &DbPool, session_id: &str) -> Result<(), ServiceError> {
    bominal_db::session::delete_session(db, session_id).await?;
    Ok(())
}

/// Get user from a valid session. Returns None if session is invalid or expired.
pub async fn get_user_from_session(
    db: &DbPool,
    session_id: &str,
) -> Result<Option<UserInfo>, ServiceError> {
    let session = match bominal_db::session::find_valid_session(db, session_id).await? {
        Some(s) => s,
        None => return Ok(None),
    };

    let user = bominal_db::user::find_by_id(db, session.user_id).await?;

    Ok(user.map(|u| UserInfo {
        user_id: u.id,
        email: u.email,
        display_name: u.display_name,
        preferred_locale: u.preferred_locale,
    }))
}

/// Send a verification email (best-effort, never fails the caller).
pub async fn send_verification_email(
    ctx: &ServiceContext,
    user_id: Uuid,
    email: &str,
    display_name: &str,
) {
    let token = Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(30);

    if let Err(e) =
        bominal_db::user::set_verification_token(&ctx.db, user_id, &token, expires_at).await
    {
        tracing::error!(user_id = %user_id, error = %e, "Failed to set verification token");
        return;
    }

    let verify_url = format!("{}/verify-email?token={}", ctx.app_base_url, token);
    let (subject, html) = bominal_email::templates::verify::render(display_name, &verify_url, 30);

    ctx.email.send_best_effort(email, &subject, &html).await;
}

/// Verify email with token.
pub async fn verify_email(db: &DbPool, token: &str) -> Result<(), ServiceError> {
    let user = bominal_db::user::verify_email(db, token)
        .await?
        .ok_or_else(|| ServiceError::validation("Invalid or expired verification link"))?;

    tracing::info!(user_id = %user.id, "Email verified");
    Ok(())
}

/// Request a password reset email.
pub async fn forgot_password(ctx: &ServiceContext, email: &str) -> Result<(), ServiceError> {
    // Always return success (don't leak email existence)
    let user = bominal_db::user::find_by_email(&ctx.db, email).await?;

    if let Some(user) = user {
        let token = Uuid::new_v4().to_string();
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(15);

        bominal_db::user::set_reset_token(&ctx.db, user.id, &token, expires_at).await?;

        let reset_url = format!("{}/reset-password?token={}", ctx.app_base_url, token);
        let (subject, html) =
            bominal_email::templates::reset::render(&user.display_name, &reset_url, 15);

        ctx.email
            .send_best_effort(&user.email, &subject, &html)
            .await;
    }

    Ok(())
}

/// Reset password using a reset token.
pub async fn reset_password(
    db: &DbPool,
    token: &str,
    new_password: &str,
) -> Result<(), ServiceError> {
    bominal_domain::auth::validate_password(new_password).map_err(ServiceError::validation)?;

    let pw_hash = bominal_domain::crypto::password::hash_password(new_password)
        .map_err(ServiceError::internal)?;

    let user = bominal_db::user::reset_password(db, token, &pw_hash)
        .await?
        .ok_or_else(|| ServiceError::validation("Invalid or expired reset link"))?;

    tracing::info!(user_id = %user.id, "Password reset completed");
    Ok(())
}

/// Extract session ID from a cookie header string.
pub fn extract_session_id_from_cookie(cookie_header: &str) -> Option<String> {
    for part in cookie_header.split(';') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix(&format!("{SESSION_COOKIE_NAME}=")) {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Build a Set-Cookie header value for a new session.
pub fn session_cookie_value(session_id: &str, app_base_url: &str) -> String {
    let domain_attr = cookie_domain_attr(app_base_url);
    let secure_attr = cookie_secure_attr(app_base_url);
    format!(
        "{SESSION_COOKIE_NAME}={session_id}; Path=/; HttpOnly; SameSite=Lax{secure_attr}; Max-Age={}{domain_attr}",
        SESSION_TTL_HOURS * 3600,
    )
}

/// Build a Set-Cookie header value that clears the session.
pub fn clear_session_cookie_value(app_base_url: &str) -> String {
    let domain_attr = cookie_domain_attr(app_base_url);
    let secure_attr = cookie_secure_attr(app_base_url);
    format!(
        "{SESSION_COOKIE_NAME}=; Path=/; HttpOnly; SameSite=Lax{secure_attr}; Max-Age=0{domain_attr}",
    )
}

/// Resolve a session to a user_id, returning an error if not authenticated.
pub async fn require_user_id(db: &DbPool, session_id: &str) -> Result<Uuid, ServiceError> {
    let session = bominal_db::session::find_valid_session(db, session_id)
        .await?
        .ok_or(ServiceError::Unauthorized)?;
    Ok(session.user_id)
}

// ── Cookie helpers ───────────────────────────────────────────────────

fn cookie_secure_attr(app_base_url: &str) -> &'static str {
    if app_base_url.starts_with("https://") {
        "; Secure"
    } else {
        ""
    }
}

fn cookie_domain_attr(app_base_url: &str) -> String {
    if app_base_url.starts_with("https://") {
        url::Url::parse(app_base_url)
            .ok()
            .and_then(|u| u.host_str().map(|h| format!("; Domain={h}")))
            .unwrap_or_default()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_session_from_cookie() {
        let cookie = "bominal_session=abc-123; other=val";
        assert_eq!(
            extract_session_id_from_cookie(cookie),
            Some("abc-123".to_string())
        );
    }

    #[test]
    fn extract_session_missing() {
        assert_eq!(extract_session_id_from_cookie("other=val"), None);
    }

    #[test]
    fn extract_session_empty_value() {
        assert_eq!(
            extract_session_id_from_cookie("bominal_session=; x=1"),
            None
        );
    }
}
