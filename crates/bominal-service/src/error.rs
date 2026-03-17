//! Unified service error type.
//!
//! Both REST handlers and Leptos server functions map from `ServiceError`
//! to their own transport-specific error types.

use std::fmt;

/// Errors produced by service-layer operations.
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    /// Input validation failure (bad provider, empty field, format error, etc.).
    #[error("{0}")]
    Validation(String),

    /// Requested resource was not found.
    #[error("{0}")]
    NotFound(String),

    /// Authentication required or session expired.
    #[error("Unauthorized")]
    Unauthorized,

    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Encryption or decryption failure.
    #[error("Crypto error: {0}")]
    Crypto(String),

    /// Provider API error (SRT/KTX).
    #[error("{0}")]
    Provider(#[from] bominal_provider::types::ProviderError),

    /// Catch-all for unexpected internal failures.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl ServiceError {
    pub fn validation(msg: impl fmt::Display) -> Self {
        Self::Validation(msg.to_string())
    }

    pub fn not_found(msg: impl fmt::Display) -> Self {
        Self::NotFound(msg.to_string())
    }

    pub fn internal(msg: impl fmt::Display) -> Self {
        Self::Internal(msg.to_string())
    }
}
