use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiErrorCode {
    InvalidRequest,
    Unauthorized,
    ServiceUnavailable,
    InternalError,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiErrorEnvelope {
    pub code: ApiErrorCode,
    pub message: String,
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApiError {
    status: ApiErrorStatus,
    envelope: ApiErrorEnvelope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiErrorStatus {
    BadRequest,
    Unauthorized,
    ServiceUnavailable,
    InternalServerError,
}

impl ApiErrorStatus {
    pub fn as_u16(self) -> u16 {
        match self {
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::ServiceUnavailable => 503,
            Self::InternalServerError => 500,
        }
    }
}

impl From<ApiErrorStatus> for StatusCode {
    fn from(value: ApiErrorStatus) -> Self {
        match value {
            ApiErrorStatus::BadRequest => StatusCode::BAD_REQUEST,
            ApiErrorStatus::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiErrorStatus::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            ApiErrorStatus::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl ApiError {
    pub fn new(
        status: ApiErrorStatus,
        code: ApiErrorCode,
        message: impl Into<String>,
        request_id: impl Into<String>,
    ) -> Self {
        Self {
            status,
            envelope: ApiErrorEnvelope {
                code,
                message: message.into(),
                request_id: request_id.into(),
                details: None,
            },
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.envelope.details = Some(details);
        self
    }

    pub fn status(&self) -> ApiErrorStatus {
        self.status
    }

    pub fn into_envelope(self) -> ApiErrorEnvelope {
        self.envelope
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let Self { status, envelope } = self;
        (StatusCode::from(status), Json(envelope)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::{ApiError, ApiErrorCode, ApiErrorStatus};
    use axum::{http::StatusCode, response::IntoResponse};

    #[test]
    fn serializes_required_envelope_fields() {
        let error = ApiError::new(
            ApiErrorStatus::Unauthorized,
            ApiErrorCode::InvalidRequest,
            "missing bearer token",
            "req-1",
        );
        assert_eq!(error.status(), ApiErrorStatus::Unauthorized);
        assert_eq!(error.status().as_u16(), 401);

        let envelope = error.into_envelope();
        let body = match serde_json::to_value(envelope) {
            Ok(value) => value,
            Err(err) => panic!("failed to serialize api error envelope: {err}"),
        };

        assert_eq!(body["code"], "invalid_request");
        assert_eq!(body["message"], "missing bearer token");
        assert_eq!(body["request_id"], "req-1");
        assert!(body.get("details").is_none());
    }

    #[test]
    fn serializes_optional_details_when_present() {
        let envelope = ApiError::new(
            ApiErrorStatus::ServiceUnavailable,
            ApiErrorCode::ServiceUnavailable,
            "queue push failed",
            "req-2",
        )
        .with_details(serde_json::json!({"component": "redis"}))
        .into_envelope();
        let body = match serde_json::to_value(envelope) {
            Ok(value) => value,
            Err(err) => panic!("failed to serialize api error envelope: {err}"),
        };

        assert_eq!(body["details"]["component"], "redis");
    }

    #[test]
    fn api_error_status_maps_to_expected_codes() {
        assert_eq!(ApiErrorStatus::BadRequest.as_u16(), 400);
        assert_eq!(ApiErrorStatus::Unauthorized.as_u16(), 401);
        assert_eq!(ApiErrorStatus::ServiceUnavailable.as_u16(), 503);
        assert_eq!(ApiErrorStatus::InternalServerError.as_u16(), 500);
    }

    #[test]
    fn into_response_uses_expected_status_code() {
        let response = ApiError::new(
            ApiErrorStatus::ServiceUnavailable,
            ApiErrorCode::ServiceUnavailable,
            "queue push failed",
            "req-3",
        )
        .into_response();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
