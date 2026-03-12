//! Authentication server functions.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User info returned after successful auth.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub preferred_locale: String,
}

/// Login with email and password.
#[server(prefix = "/sfn")]
pub async fn login(email: String, password: String) -> Result<(), ServerFnError> {
    use bominal_domain::crypto::password::verify_password;

    let pool = use_context::<bominal_db::DbPool>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    let user = bominal_db::user::find_by_email(&pool, &email)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
        .ok_or_else(|| ServerFnError::new("Invalid email or password"))?;

    verify_password(&password, &user.password_hash)
        .map_err(|_| ServerFnError::new("Invalid email or password"))?;

    create_session(&pool, &user).await?;
    leptos_axum::redirect("/home");
    Ok(())
}

/// Register a new account.
#[server(prefix = "/sfn")]
pub async fn register(
    email: String,
    password: String,
    display_name: String,
) -> Result<(), ServerFnError> {
    use bominal_domain::auth::{validate_display_name, validate_email, validate_password};
    use bominal_domain::crypto::password::hash_password;

    validate_email(&email).map_err(|e| ServerFnError::new(e.to_string()))?;
    validate_password(&password).map_err(|e| ServerFnError::new(e.to_string()))?;
    validate_display_name(&display_name).map_err(|e| ServerFnError::new(e.to_string()))?;

    let pool = use_context::<bominal_db::DbPool>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    let existing = bominal_db::user::find_by_email(&pool, &email)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    if existing.is_some() {
        leptos_axum::redirect("/?mode=register&error=email_exists");
        return Err(ServerFnError::new("Email already registered"));
    }

    let pw_hash =
        hash_password(&password).map_err(|e| ServerFnError::new(format!("Hash error: {e}")))?;

    let user = bominal_db::user::create_user(&pool, &email, &display_name, &pw_hash)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    // Send verification email (best-effort)
    if let Some(email_client) = use_context::<bominal_email::EmailClient>()
        && let Some(app_base_url) = use_context::<AppBaseUrl>()
    {
        let token = Uuid::new_v4().to_string();
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(30);
        let _ = bominal_db::user::set_verification_token(&pool, user.id, &token, expires_at).await;
        let verify_url = format!("{}/verify-email?token={}", app_base_url.0, token);
        let (subject, html) =
            bominal_email::templates::verify::render(&user.display_name, &verify_url, 30);
        email_client
            .send_best_effort(&user.email, &subject, &html)
            .await;
    }

    create_session(&pool, &user).await?;
    leptos_axum::redirect("/home");
    Ok(())
}

/// Logout — clear session cookie.
#[server(prefix = "/sfn")]
pub async fn logout() -> Result<(), ServerFnError> {
    let pool = use_context::<bominal_db::DbPool>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    if let Some(session_id) = extract_session_id() {
        let _ = bominal_db::session::delete_session(&pool, &session_id).await;
    }

    let response = use_context::<leptos_axum::ResponseOptions>()
        .ok_or_else(|| ServerFnError::new("No response context"))?;
    response.insert_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(
            "bominal_session=; Path=/; HttpOnly; SameSite=Lax; Secure; Max-Age=0",
        )
        .unwrap(),
    );

    leptos_axum::redirect("/");
    Ok(())
}

/// Get current authenticated user info (used for auth checks on page load).
#[server(prefix = "/sfn")]
pub async fn get_current_user() -> Result<Option<UserInfo>, ServerFnError> {
    let pool = use_context::<bominal_db::DbPool>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    let session_id = match extract_session_id() {
        Some(id) => id,
        None => return Ok(None),
    };

    let session = match bominal_db::session::find_valid_session(&pool, &session_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
    {
        Some(s) => s,
        None => return Ok(None),
    };

    let user = bominal_db::user::find_by_id(&pool, session.user_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?;

    Ok(user.map(|u| UserInfo {
        user_id: u.id,
        email: u.email,
        display_name: u.display_name,
        preferred_locale: u.preferred_locale,
    }))
}

// ── Context wrapper types ──────────────────────────────────────────

/// Wrapper for app base URL provided via context.
#[derive(Clone, Debug)]
pub struct AppBaseUrl(pub String);

// ── Helpers ────────────────────────────────────────────────────────

const SESSION_COOKIE_NAME: &str = "bominal_session";
const SESSION_TTL_HOURS: i64 = 24;

pub(crate) fn extract_session_id() -> Option<String> {
    let parts = use_context::<axum::http::request::Parts>()?;
    let cookie_header = parts.headers.get("cookie")?.to_str().ok()?;
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

async fn create_session(
    pool: &bominal_db::DbPool,
    user: &bominal_db::user::UserRow,
) -> Result<(), ServerFnError> {
    let session_id = Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(SESSION_TTL_HOURS);

    bominal_db::session::create_session(pool, &session_id, user.id, expires_at)
        .await
        .map_err(|e| ServerFnError::new(format!("Session error: {e}")))?;

    let response = use_context::<leptos_axum::ResponseOptions>()
        .ok_or_else(|| ServerFnError::new("No response context"))?;
    response.insert_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&format!(
            "{SESSION_COOKIE_NAME}={session_id}; Path=/; HttpOnly; SameSite=Lax; Secure; Max-Age={}",
            SESSION_TTL_HOURS * 3600
        ))
        .unwrap(),
    );

    Ok(())
}
