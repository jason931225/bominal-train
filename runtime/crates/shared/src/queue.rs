use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeQueueJob {
    pub job_id: String,
    pub user_id: String,
    pub kind: String,
    pub payload: serde_json::Value,
    pub enqueued_at: DateTime<Utc>,
}
