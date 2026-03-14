//! Auth route handlers: register, login, logout, email verification, password reset.

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::header::SET_COOKIE;
use serde::Deserialize;

use bominal_domain::auth::{AuthResponse, LoginRequest, RegisterRequest};

use crate::error::AppError;
use crate::state::SharedState;

/// POST /api/auth/register
pub async fn register(
    State(state): State<SharedState>,
    Json(req): Json<RegisterRequest>,
) -> Result<(HeaderMap, Json<AuthResponse>), AppError> {
    let user = bominal_service::auth::register(
        &state.db,
        &req.email,
        &req.password,
        &req.display_name,
    )
    .await?;

    // Send verification email (best-effort, don't block registration)
    let ctx = service_context(&state);
    bominal_service::auth::send_verification_email(
        &ctx,
        user.id,
        &user.email,
        &user.display_name,
    )
    .await;

    // Create session
    let session_id = bominal_service::auth::create_session(&state.db, user.id).await?;
    let cookie = bominal_service::auth::session_cookie_value(&session_id, &state.app_base_url);

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    let response = AuthResponse {
        user_id: user.id,
        email: user.email,
        display_name: user.display_name,
        preferred_locale: user.preferred_locale,
    };

    Ok((headers, Json(response)))
}

/// POST /api/auth/login
pub async fn login(
    State(state): State<SharedState>,
    Json(req): Json<LoginRequest>,
) -> Result<(HeaderMap, Json<AuthResponse>), AppError> {
    let user = bominal_service::auth::authenticate(&state.db, &req.email, &req.password).await?;

    let session_id = bominal_service::auth::create_session(&state.db, user.id).await?;
    let cookie = bominal_service::auth::session_cookie_value(&session_id, &state.app_base_url);

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    let response = AuthResponse {
        user_id: user.id,
        email: user.email,
        display_name: user.display_name,
        preferred_locale: user.preferred_locale,
    };

    Ok((headers, Json(response)))
}

/// POST /api/auth/logout
pub async fn logout(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Result<HeaderMap, AppError> {
    if let Some(cookie_header) = headers.get("cookie").and_then(|v| v.to_str().ok()) {
        if let Some(session_id) =
            bominal_service::auth::extract_session_id_from_cookie(cookie_header)
        {
            bominal_service::auth::delete_session(&state.db, &session_id).await?;
        }
    }

    let cookie = bominal_service::auth::clear_session_cookie_value(&state.app_base_url);
    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(SET_COOKIE, cookie.parse().unwrap());
    Ok(resp_headers)
}

/// GET /api/auth/me — return current user info from session.
pub async fn me(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Result<Json<AuthResponse>, AppError> {
    let cookie_header = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let session_id = bominal_service::auth::extract_session_id_from_cookie(cookie_header)
        .ok_or(AppError::Unauthorized)?;

    let user_info = bominal_service::auth::get_user_from_session(&state.db, &session_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    Ok(Json(AuthResponse {
        user_id: user_info.user_id,
        email: user_info.email,
        display_name: user_info.display_name,
        preferred_locale: user_info.preferred_locale,
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
    bominal_service::auth::verify_email(&state.db, &req.token).await?;
    Ok(Json(serde_json::json!({ "verified": true })))
}

/// POST /api/auth/resend-verification — resend verification email.
pub async fn resend_verification(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    let cookie_header = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let session_id = bominal_service::auth::extract_session_id_from_cookie(cookie_header)
        .ok_or(AppError::Unauthorized)?;

    let user_info = bominal_service::auth::get_user_from_session(&state.db, &session_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // Need the full user row to check email_verified
    let user = bominal_db::user::find_by_id(&state.db, user_info.user_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or(AppError::Unauthorized)?;

    if user.email_verified {
        return Err(AppError::BadRequest("Email already verified".to_string()));
    }

    let ctx = service_context(&state);
    bominal_service::auth::send_verification_email(
        &ctx,
        user.id,
        &user.email,
        &user.display_name,
    )
    .await;

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
    let ctx = service_context(&state);
    bominal_service::auth::forgot_password(&ctx, &req.email).await?;
    Ok(Json(serde_json::json!({ "sent": true })))
}

/// POST /api/auth/reset-password — set a new password using the reset token.
pub async fn reset_password(
    State(state): State<SharedState>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    bominal_service::auth::reset_password(&state.db, &req.token, &req.new_password).await?;
    Ok(Json(serde_json::json!({ "reset": true })))
}

// ── Helpers ──────────────────────────────────────────────────────────

fn service_context(state: &SharedState) -> bominal_service::ServiceContext {
    bominal_service::ServiceContext {
        db: state.db.clone(),
        encryption_key: state.encryption_key.clone(),
        email: state.email.clone(),
        app_base_url: state.app_base_url.clone(),
    }
}

/// Returns `"; Secure"` when the app runs over HTTPS, or empty for HTTP/localhost.
pub(crate) fn cookie_secure_attr(app_base_url: &str) -> &'static str {
    if app_base_url.starts_with("https://") {
        "; Secure"
    } else {
        ""
    }
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

#[cfg(test)]
mod tests {
    use bominal_service::auth::extract_session_id_from_cookie;

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
    fn extract_session_no_match() {
        assert_eq!(extract_session_id_from_cookie("other=val"), None);
    }
}
