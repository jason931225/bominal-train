//! Train provider clients and credential management.
//!
//! This module contains the HTTP client implementations for SRT and KTX
//! providers, plus the service-layer credential CRUD operations.

pub mod credentials;
pub mod ktx;
pub mod netfunnel;
pub mod retry;
pub mod srt;
pub mod types;

pub use types::*;

// Re-export credential management functions at module level for backward compatibility.
pub use credentials::{ProviderInfo, add, delete, list, mask_login_id, verify_login};
