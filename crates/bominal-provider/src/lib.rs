//! Train provider clients for SRT and KTX/Korail.
//!
//! This crate implements the HTTP client contracts for both Korean train
//! reservation providers, faithfully porting the Python `srtgo` implementation.

pub mod ktx;
pub mod netfunnel;
pub mod retry;
pub mod srt;
pub mod types;

pub use types::*;
