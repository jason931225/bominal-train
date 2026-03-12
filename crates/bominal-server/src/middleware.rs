//! Request ID and tracing middleware.

use axum::http::{HeaderName, HeaderValue, Request};
use tower_http::request_id::{MakeRequestId, RequestId};
use uuid::Uuid;

/// Generates UUID v7 request IDs (time-ordered for log correlation).
#[derive(Clone, Copy)]
pub struct RequestIdGenerator;

impl MakeRequestId for RequestIdGenerator {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let id = Uuid::now_v7().to_string();
        HeaderValue::from_str(&id).ok().map(RequestId::new)
    }
}

/// Header name for request IDs.
pub fn request_id_header() -> HeaderName {
    HeaderName::from_static("x-request-id")
}
