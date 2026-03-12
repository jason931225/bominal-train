//! Email delivery via Resend API.
//!
//! Provides a client for sending transactional emails and
//! pre-built HTML templates for all notification types.

pub mod client;
pub mod templates;

pub use client::{EmailClient, EmailError};
