//! Authentication server functions.

use leptos::prelude::*;
use uuid::Uuid;

pub use bominal_service::auth::UserInfo;

/// Login with email and password.
#[server(prefix = "/sfn")]
pub async fn login(email: String, password: String) -> Result<(), ServerFnError> {
    let pool = use_context::<bominal_service::DbPool>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    let user = bominal_service::auth::authenticate(&pool, &email, &password)
        .await
        .map_err(|_| ServerFnError::new("Invalid email or password"))?;

    create_session_and_set_cookie(&pool, user.id).await?;
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
    let pool = use_context::<bominal_service::DbPool>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    let user = bominal_service::auth::register(&pool, &email, &password, &display_name)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Send verification email (best-effort)
    if let Some(email_client) = use_context::<bominal_service::EmailClient>()
        && let Some(base_url) = use_context::<AppBaseUrl>()
        && let Some(key) = use_context::<bominal_service::EncryptionKey>()
    {
        let ctx = bominal_service::ServiceContext {
            db: pool.clone(),
            encryption_key: key,
            email: email_client,
            app_base_url: base_url.0.clone(),
        };
        bominal_service::auth::send_verification_email(
            &ctx,
            user.id,
            &user.email,
            &user.display_name,
        )
        .await;
    }

    create_session_and_set_cookie(&pool, user.id).await?;
    leptos_axum::redirect("/auth/verify");
    Ok(())
}

/// Request a password reset email.
///
/// Always returns Ok to avoid leaking whether the email exists.
#[server(prefix = "/sfn")]
pub async fn forgot_password(email: String) -> Result<(), ServerFnError> {
    let pool = use_context::<bominal_service::DbPool>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    if let Some(email_client) = use_context::<bominal_service::EmailClient>()
        && let Some(base_url) = use_context::<AppBaseUrl>()
        && let Some(key) = use_context::<bominal_service::EncryptionKey>()
    {
        let ctx = bominal_service::ServiceContext {
            db: pool,
            encryption_key: key,
            email: email_client,
            app_base_url: base_url.0.clone(),
        };
        let _ = bominal_service::auth::forgot_password(&ctx, &email).await;
    }
    Ok(())
}

/// Resend the email verification link (requires active session).
#[server(prefix = "/sfn")]
pub async fn resend_verification() -> Result<(), ServerFnError> {
    let pool = use_context::<bominal_service::DbPool>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    let session_id = extract_session_id().ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let user_info = bominal_service::auth::get_user_from_session(&pool, &session_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Session expired"))?;

    if let Some(email_client) = use_context::<bominal_service::EmailClient>()
        && let Some(base_url) = use_context::<AppBaseUrl>()
        && let Some(key) = use_context::<bominal_service::EncryptionKey>()
    {
        let ctx = bominal_service::ServiceContext {
            db: pool,
            encryption_key: key,
            email: email_client,
            app_base_url: base_url.0.clone(),
        };
        bominal_service::auth::send_verification_email(
            &ctx,
            user_info.user_id,
            &user_info.email,
            &user_info.display_name,
        )
        .await;
    }
    Ok(())
}

/// Logout — clear session cookie.
#[server(prefix = "/sfn")]
pub async fn logout() -> Result<(), ServerFnError> {
    let pool = use_context::<bominal_service::DbPool>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    if let Some(session_id) = extract_session_id() {
        let _ = bominal_service::auth::delete_session(&pool, &session_id).await;
    }

    let response = use_context::<leptos_axum::ResponseOptions>()
        .ok_or_else(|| ServerFnError::new("No response context"))?;
    response.insert_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&format!(
            "{}=; Path=/; HttpOnly; SameSite=Lax; Secure; Max-Age=0",
            bominal_service::auth::SESSION_COOKIE_NAME,
        ))
        .unwrap(),
    );

    leptos_axum::redirect("/");
    Ok(())
}

/// Verify email address using token from verification email.
#[server(prefix = "/sfn")]
pub async fn verify_email(token: String) -> Result<(), ServerFnError> {
    let pool = use_context::<bominal_service::DbPool>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    bominal_service::auth::verify_email(&pool, &token)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Reset password using token from reset email.
#[server(prefix = "/sfn")]
pub async fn reset_password(token: String, new_password: String) -> Result<(), ServerFnError> {
    let pool = use_context::<bominal_service::DbPool>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    bominal_service::auth::reset_password(&pool, &token, &new_password)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Get current authenticated user info (used for auth checks on page load).
#[server(prefix = "/sfn")]
pub async fn get_current_user() -> Result<Option<UserInfo>, ServerFnError> {
    let pool = use_context::<bominal_service::DbPool>()
        .ok_or_else(|| ServerFnError::new("Server misconfigured"))?;

    let session_id = match extract_session_id() {
        Some(id) => id,
        None => return Ok(None),
    };

    bominal_service::auth::get_user_from_session(&pool, &session_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

// ── Context wrapper types ──────────────────────────────────────────

/// Wrapper for app base URL provided via context.
#[derive(Clone, Debug)]
pub struct AppBaseUrl(pub String);

// ── Helpers ────────────────────────────────────────────────────────

pub(crate) fn extract_session_id() -> Option<String> {
    let parts = use_context::<axum::http::request::Parts>()?;
    let cookie_header = parts.headers.get("cookie")?.to_str().ok()?;
    bominal_service::auth::extract_session_id_from_cookie(cookie_header)
}

async fn create_session_and_set_cookie(
    pool: &bominal_service::DbPool,
    user_id: Uuid,
) -> Result<(), ServerFnError> {
    let session_id = bominal_service::auth::create_session(pool, user_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Session error: {e}")))?;

    let response = use_context::<leptos_axum::ResponseOptions>()
        .ok_or_else(|| ServerFnError::new("No response context"))?;
    response.insert_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&format!(
            "{}={}; Path=/; HttpOnly; SameSite=Lax; Secure; Max-Age={}",
            bominal_service::auth::SESSION_COOKIE_NAME,
            session_id,
            bominal_service::auth::SESSION_TTL_HOURS * 3600
        ))
        .unwrap(),
    );

    Ok(())
}
