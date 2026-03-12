//! Shared types for train providers.

use serde::{Deserialize, Serialize};

/// Seat preference for reservation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeatPreference {
    GeneralFirst,
    GeneralOnly,
    SpecialFirst,
    SpecialOnly,
}

impl SeatPreference {
    /// Coerce preference for standby reservations.
    /// SPECIAL_FIRST -> SPECIAL_ONLY, GENERAL_FIRST -> GENERAL_ONLY.
    ///
    /// # Examples
    ///
    /// ```
    /// use bominal_provider::types::SeatPreference;
    /// assert_eq!(
    ///     SeatPreference::GeneralFirst.coerce_for_standby(),
    ///     SeatPreference::GeneralOnly
    /// );
    /// assert_eq!(
    ///     SeatPreference::SpecialOnly.coerce_for_standby(),
    ///     SeatPreference::SpecialOnly
    /// );
    /// ```
    pub fn coerce_for_standby(self) -> Self {
        match self {
            SeatPreference::GeneralFirst => SeatPreference::GeneralOnly,
            SeatPreference::SpecialFirst => SeatPreference::SpecialOnly,
            other => other,
        }
    }
}

/// Provider identification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provider {
    Srt,
    Ktx,
}

/// Authentication method, auto-classified from login ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthType {
    Email,
    Phone,
    Membership,
}

/// Provider error types.
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Login failed: {message}")]
    LoginFailed { message: String },

    #[error("Session expired")]
    SessionExpired,

    #[error("No results found")]
    NoResults,

    #[error("Sold out")]
    SoldOut,

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("NetFunnel blocked")]
    NetFunnelBlocked,

    #[error("Duplicate reservation")]
    DuplicateReservation,

    #[error("Unexpected response: status={status}, body={body}")]
    UnexpectedResponse { status: u16, body: String },
}

/// Classify a login ID into the appropriate auth type.
///
/// # Examples
///
/// ```
/// use bominal_provider::types::{classify_auth, AuthType};
/// assert_eq!(classify_auth("user@example.com"), AuthType::Email);
/// assert_eq!(classify_auth("010-1234-5678"), AuthType::Phone);
/// assert_eq!(classify_auth("1234567890"), AuthType::Membership);
/// assert_eq!(classify_auth("01012345678"), AuthType::Phone);
/// ```
pub fn classify_auth(login_id: &str) -> AuthType {
    if login_id.contains('@') {
        return AuthType::Email;
    }

    let digits_only: String = login_id.chars().filter(|c| c.is_ascii_digit()).collect();
    let has_hyphens = login_id.contains('-');

    if has_hyphens && digits_only.len() >= 10 && digits_only.len() <= 11 {
        return AuthType::Phone;
    }

    if digits_only.len() >= 10 && digits_only.len() <= 11
        && login_id.starts_with('0')
        && login_id.chars().all(|c| c.is_ascii_digit())
    {
        return AuthType::Phone;
    }

    AuthType::Membership
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_email() {
        assert_eq!(classify_auth("user@example.com"), AuthType::Email);
        assert_eq!(classify_auth("test@naver.com"), AuthType::Email);
    }

    #[test]
    fn classify_phone() {
        assert_eq!(classify_auth("010-1234-5678"), AuthType::Phone);
        assert_eq!(classify_auth("010-123-4567"), AuthType::Phone);
    }

    #[test]
    fn classify_membership() {
        assert_eq!(classify_auth("1234567890"), AuthType::Membership);
        assert_eq!(classify_auth("ABC123"), AuthType::Membership);
    }

    #[test]
    fn classify_phone_without_hyphens() {
        assert_eq!(classify_auth("01012345678"), AuthType::Phone);
        assert_eq!(classify_auth("0101234567"), AuthType::Phone);
    }

    #[test]
    fn standby_coercion() {
        assert_eq!(
            SeatPreference::GeneralFirst.coerce_for_standby(),
            SeatPreference::GeneralOnly
        );
        assert_eq!(
            SeatPreference::SpecialFirst.coerce_for_standby(),
            SeatPreference::SpecialOnly
        );
        assert_eq!(
            SeatPreference::GeneralOnly.coerce_for_standby(),
            SeatPreference::GeneralOnly
        );
    }
}
