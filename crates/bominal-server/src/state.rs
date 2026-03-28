//! Shared application state passed to all handlers.

use std::sync::Arc;
use std::time::Instant;

use bominal_db::DbPool;
use bominal_domain::crypto::encryption::EncryptionKey;
use bominal_email::EmailClient;
use webauthn_rs::Webauthn;

use axum::extract::FromRef;
use leptos::config::LeptosOptions;

use crate::evervault::EvervaultConfig;
use crate::sse::EventBus;

/// Shared state available to all Axum handlers.
#[derive(Clone)]
pub struct SharedState {
    pub db: DbPool,
    pub start_time: Instant,
    pub event_bus: EventBus,
    pub email: EmailClient,
    pub encryption_key: EncryptionKey,
    pub evervault: EvervaultConfig,
    pub app_base_url: String,
    pub prometheus_handle: metrics_exporter_prometheus::PrometheusHandle,
    pub webauthn: Arc<Webauthn>,
    pub leptos_options: LeptosOptions,
}

impl FromRef<SharedState> for LeptosOptions {
    fn from_ref(state: &SharedState) -> Self {
        state.leptos_options.clone()
    }
}
