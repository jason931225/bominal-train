//! Shared application state passed to all handlers.

use std::time::Instant;

use bominal_db::DbPool;
use bominal_domain::crypto::encryption::EncryptionKey;
use bominal_email::EmailClient;

use crate::sse::EventBus;

/// Shared state available to all Axum handlers.
#[derive(Clone)]
pub struct SharedState {
    pub db: DbPool,
    pub start_time: Instant,
    pub event_bus: EventBus,
    pub email: EmailClient,
    pub encryption_key: EncryptionKey,
    pub app_base_url: String,
}
