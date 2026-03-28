//! Task event types for real-time status updates.
//!
//! These events are published by the worker and consumed by the server SSE
//! endpoint to push reservation/task updates to connected clients.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event payload sent to SSE clients and stored in the app state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEvent {
    pub task_id: Uuid,
    pub status: String,
    pub message: String,
    pub attempt_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reservation_number: Option<String>,
}
