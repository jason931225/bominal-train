//! Valkey (Redis-compatible) pub/sub event publisher.
//!
//! Publishes `TaskEvent` payloads to per-user Valkey channels so the
//! server's SSE endpoint can fan them out to connected browser clients.

use anyhow::Result;
use fred::prelude::*;
use uuid::Uuid;

use bominal_domain::task_event::TaskEvent;

/// Channel name for a specific user's task events.
fn channel_key(user_id: Uuid) -> String {
    format!("task_events:{user_id}")
}

/// Valkey-backed event publisher.
///
/// Each `publish` call serializes the event to JSON and PUBLISHes it
/// to `task_events:{user_id}`. The server subscribes with a pattern
/// (`task_events:*`) and routes events into per-user SSE streams.
#[derive(Clone)]
pub struct EventPublisher {
    client: Client,
}

impl EventPublisher {
    /// Connect to Valkey and return a ready publisher.
    pub async fn connect(valkey_url: &str) -> Result<Self> {
        let config = Config::from_url(valkey_url)?;
        let client = Client::new(config, None, None, None);
        client.init().await?;
        Ok(Self { client })
    }

    /// Publish a task event for a user. Best-effort: logs on failure.
    pub async fn publish(&self, user_id: Uuid, event: &TaskEvent) {
        let channel = channel_key(user_id);
        let payload = match serde_json::to_string(event) {
            Ok(json) => json,
            Err(e) => {
                tracing::error!(error = %e, "Failed to serialize TaskEvent");
                return;
            }
        };

        if let Err(e) = self.client.publish::<(), _, _>(&channel, payload).await {
            tracing::warn!(
                channel = %channel,
                error = %e,
                "Failed to publish event to Valkey"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_key_format() {
        let id = Uuid::nil();
        assert_eq!(
            channel_key(id),
            "task_events:00000000-0000-0000-0000-000000000000"
        );
    }
}
