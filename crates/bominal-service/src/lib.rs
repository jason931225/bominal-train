//! Bominal service layer — shared business logic for REST and Leptos server functions.
//!
//! This crate contains all domain operations (validate, query, encrypt, call provider)
//! so that both the REST API (`bominal-server`) and the Leptos app/server layer
//! (`bominal-app`) delegate here instead of duplicating logic.

pub mod auth;
pub mod cards;
pub mod error;
pub mod providers;
pub mod reservations;
pub mod search;
pub mod tasks;

pub use error::ServiceError;

// Re-export infrastructure types so consumers depend only on bominal-service.
pub use bominal_db::DbPool;
pub use bominal_domain::crypto::encryption::EncryptionKey;
pub use bominal_email::EmailClient;

/// Bundles the infrastructure dependencies needed by service functions.
///
/// Both Leptos server functions (`use_context::<ServiceContext>()`) and REST
/// handlers (`ServiceContext::from(&state)`) construct this to call service ops.
#[derive(Clone)]
pub struct ServiceContext {
    pub db: DbPool,
    pub encryption_key: EncryptionKey,
    pub email: EmailClient,
    pub app_base_url: String,
}
