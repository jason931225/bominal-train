//! Server functions for API operations.
//!
//! These run on the server during SSR and as HTTP endpoints after hydration.
//! Each function uses `use_context` to access the database pool and other services.

pub mod auth;
pub mod cards;
pub mod providers;
pub mod reservations;
pub mod search;
pub mod tasks;

/// Shared response types used across server functions and components.
pub use auth::UserInfo;
pub use cards::CardInfo;
pub use providers::ProviderInfo;
pub use search::{StationInfo, TrainInfo};
pub use tasks::TaskInfo;
