use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SrtProviderError {
    #[error("provider session is not logged in")]
    NotLoggedIn,
    #[error("provider session expired")]
    SessionExpired,
    #[error("provider request is unauthorized")]
    Unauthorized,
    #[error("provider relogin is unavailable")]
    ReloginUnavailable,
    #[error("provider operation failed: {message}")]
    OperationFailed { message: String },
    #[error("provider transport failed: {message}")]
    Transport { message: String },
    #[error("provider operation '{operation}' is not supported")]
    UnsupportedOperation { operation: &'static str },
}

impl SrtProviderError {
    pub fn is_auth_failure(&self) -> bool {
        matches!(
            self,
            Self::SessionExpired | Self::Unauthorized | Self::NotLoggedIn
        )
    }
}

pub type SrtResult<T> = Result<T, SrtProviderError>;
