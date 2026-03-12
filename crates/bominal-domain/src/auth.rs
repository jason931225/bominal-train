//! Authentication domain types and password hashing.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCredentialStatus {
    pub provider: String,
    pub login_id_masked: String,
    pub status: CredentialStatus,
    pub last_verified_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CredentialStatus {
    Valid,
    Invalid,
    Unverified,
    Disabled,
}

/// Registration request payload.
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
}

/// Login request payload.
#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Session info returned after successful auth.
#[derive(Debug, Clone, Serialize)]
pub struct AuthResponse {
    pub user_id: uuid::Uuid,
    pub email: String,
    pub display_name: String,
    pub preferred_locale: String,
}

/// Validate email format (basic check).
pub fn validate_email(email: &str) -> Result<(), &'static str> {
    if email.is_empty() {
        return Err("Email is required");
    }
    let parts: Vec<&str> = email.splitn(2, '@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].len() < 3 || !parts[1].contains('.') {
        return Err("Invalid email format");
    }
    if email.len() > 254 {
        return Err("Email too long");
    }
    Ok(())
}

/// Validate password strength.
pub fn validate_password(password: &str) -> Result<(), &'static str> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters");
    }
    if password.len() > 128 {
        return Err("Password too long");
    }
    Ok(())
}

/// Validate display name.
pub fn validate_display_name(name: &str) -> Result<(), &'static str> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Display name is required");
    }
    if trimmed.len() > 100 {
        return Err("Display name too long");
    }
    Ok(())
}

/// Mask a login ID for display (e.g., "user@example.com" -> "us***@example.com").
pub fn mask_login_id(login_id: &str) -> String {
    if let Some(at_pos) = login_id.find('@') {
        let local = &login_id[..at_pos];
        let domain = &login_id[at_pos..];
        if local.len() <= 2 {
            format!("{}***{}", local, domain)
        } else {
            format!("{}***{}", &local[..2], domain)
        }
    } else if login_id.len() > 4 {
        format!("{}****{}", &login_id[..2], &login_id[login_id.len() - 2..])
    } else {
        "****".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_email_valid() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("a@b.c").is_ok());
    }

    #[test]
    fn validate_email_invalid() {
        assert!(validate_email("").is_err());
        assert!(validate_email("nope").is_err());
        assert!(validate_email("@.").is_err());
    }

    #[test]
    fn validate_password_valid() {
        assert!(validate_password("12345678").is_ok());
        assert!(validate_password("strongpassword123").is_ok());
    }

    #[test]
    fn validate_password_too_short() {
        assert!(validate_password("short").is_err());
        assert!(validate_password("").is_err());
    }

    #[test]
    fn mask_email() {
        assert_eq!(mask_login_id("user@example.com"), "us***@example.com");
        assert_eq!(mask_login_id("ab@x.com"), "ab***@x.com");
    }

    #[test]
    fn mask_membership() {
        assert_eq!(mask_login_id("1234567890"), "12****90");
    }

    #[test]
    fn mask_short() {
        assert_eq!(mask_login_id("abc"), "****");
    }
}
