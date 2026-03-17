use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl From<bominal_service::error::ServiceError> for AppError {
    fn from(e: bominal_service::error::ServiceError) -> Self {
        use bominal_service::error::ServiceError;
        match e {
            ServiceError::Validation(msg) => AppError::BadRequest(msg),
            ServiceError::NotFound(msg) => AppError::NotFound(msg),
            ServiceError::Unauthorized => AppError::Unauthorized,
            ServiceError::Database(e) => AppError::Internal(e.into()),
            ServiceError::Crypto(msg) => AppError::Internal(anyhow::anyhow!(msg)),
            ServiceError::Provider(e) => AppError::BadRequest(e.to_string()),
            ServiceError::Internal(msg) => AppError::Internal(anyhow::anyhow!(msg)),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".into()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Internal(err) => {
                tracing::error!(error = %err, "Internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".into(),
                )
            }
        };

        (status, axum::Json(ErrorResponse { error: message })).into_response()
    }
}
