//! Axum extractors for common request data.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;

use crate::error::AppError;
use crate::state::SharedState;

const SESSION_COOKIE_NAME: &str = "bominal_session";

/// Authenticated user extracted from the session cookie.
///
/// Use this as an extractor in any handler that requires authentication:
/// ```ignore
/// async fn handler(user: AuthUser, State(state): State<SharedState>) -> ...
/// ```
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
    pub display_name: String,
}

impl FromRequestParts<SharedState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &SharedState,
    ) -> Result<Self, Self::Rejection> {
        let session_id = extract_session_id_from_parts(parts).ok_or(AppError::Unauthorized)?;

        let session = bominal_db::session::find_valid_session(&state.db, &session_id)
            .await
            .map_err(|e| AppError::Internal(e.into()))?
            .ok_or(AppError::Unauthorized)?;

        let user = bominal_db::user::find_by_id(&state.db, session.user_id)
            .await
            .map_err(|e| AppError::Internal(e.into()))?
            .ok_or(AppError::Unauthorized)?;

        Ok(AuthUser {
            user_id: user.id,
            email: user.email,
            display_name: user.display_name,
        })
    }
}

fn extract_session_id_from_parts(parts: &Parts) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    fn extract_from_headers(headers: &HeaderMap) -> Option<String> {
        let cookie_header = headers.get("cookie")?.to_str().ok()?;
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

    #[test]
    fn extracts_session_from_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert("cookie", "bominal_session=abc-123".parse().unwrap());
        assert_eq!(extract_from_headers(&headers), Some("abc-123".to_string()));
    }

    #[test]
    fn missing_cookie_returns_none() {
        let headers = HeaderMap::new();
        assert_eq!(extract_from_headers(&headers), None);
    }
}
