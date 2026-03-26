//! Server-Sent Events for real-time task status updates.
//!
//! Clients connect via `GET /api/tasks/events` and receive a stream of
//! JSON events whenever their tasks change status.
//!
//! Architecture: broadcast channel per user (lazy, created on first connect).
//! The worker publishes events via Valkey pub/sub; a subscriber task feeds
//! them into the in-process EventBus for SSE delivery.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use fred::clients::SubscriberClient;
use fred::prelude::*;
use futures_util::stream::Stream;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use uuid::Uuid;

// Re-export TaskEvent from domain so existing `use crate::sse::TaskEvent` still works.
pub use bominal_domain::task_event::TaskEvent;

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::state::SharedState;

/// Channel capacity per user. Slow clients drop older events.
const CHANNEL_CAPACITY: usize = 64;

/// Registry of per-user broadcast channels.
#[derive(Clone, Default)]
pub struct EventBus {
    channels: Arc<RwLock<HashMap<Uuid, broadcast::Sender<TaskEvent>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create a broadcast sender for a user.
    pub async fn sender_for(&self, user_id: Uuid) -> broadcast::Sender<TaskEvent> {
        // Fast path: read lock
        {
            let channels = self.channels.read().await;
            if let Some(tx) = channels.get(&user_id) {
                return tx.clone();
            }
        }

        // Slow path: write lock to create
        let mut channels = self.channels.write().await;
        channels
            .entry(user_id)
            .or_insert_with(|| broadcast::channel(CHANNEL_CAPACITY).0)
            .clone()
    }

    /// Subscribe to events for a user.
    pub async fn subscribe(&self, user_id: Uuid) -> broadcast::Receiver<TaskEvent> {
        let tx = self.sender_for(user_id).await;
        tx.subscribe()
    }

    /// Publish an event for a user. No-op if no subscribers.
    pub async fn publish(&self, user_id: Uuid, event: TaskEvent) {
        let channels = self.channels.read().await;
        if let Some(tx) = channels.get(&user_id) {
            // Ignore send errors (no active subscribers)
            let _ = tx.send(event);
        }
    }

    /// Remove channels with no active subscribers to prevent memory leaks.
    pub async fn cleanup_inactive(&self) {
        let mut channels = self.channels.write().await;
        channels.retain(|_, tx| tx.receiver_count() > 0);
    }
}

/// Spawn a background task that subscribes to Valkey `task_events:*` pattern
/// and feeds received events into the in-process EventBus.
pub fn spawn_valkey_subscriber(valkey_url: &str, event_bus: EventBus) {
    let url = valkey_url.to_string();
    tokio::spawn(async move {
        if let Err(e) = run_valkey_subscriber(&url, &event_bus).await {
            tracing::error!(error = %e, "Valkey subscriber failed");
        }
    });
}

/// Connect to Valkey, PSUBSCRIBE to `task_events:*`, and route messages
/// into the EventBus.
async fn run_valkey_subscriber(
    valkey_url: &str,
    event_bus: &EventBus,
) -> anyhow::Result<()> {
    let config = Config::from_url(valkey_url)?;
    let subscriber = SubscriberClient::new(config, None, None, None);

    let mut message_rx = subscriber.message_rx();

    subscriber.init().await?;
    subscriber.psubscribe("task_events:*").await?;

    tracing::info!("Valkey subscriber connected, subscribed to task_events:*");

    while let Ok(message) = message_rx.recv().await {
        let channel = message.channel.to_string();
        let payload: String = match message.value.convert() {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Parse user_id from channel name: "task_events:{uuid}"
        let user_id = match channel.strip_prefix("task_events:") {
            Some(id_str) => match Uuid::parse_str(id_str) {
                Ok(id) => id,
                Err(_) => continue,
            },
            None => continue,
        };

        // Parse the TaskEvent from JSON
        let event: TaskEvent = match serde_json::from_str(&payload) {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to parse TaskEvent from Valkey");
                continue;
            }
        };

        event_bus.publish(user_id, event).await;
    }

    tracing::warn!("Valkey subscriber message stream ended");
    Ok(())
}

/// GET /api/tasks/events — SSE stream for task updates.
pub async fn task_events(
    user: AuthUser,
    State(state): State<SharedState>,
) -> Result<Sse<impl Stream<Item = Result<Event, axum::Error>>>, AppError> {
    let mut rx = state.event_bus.subscribe(user.user_id).await;

    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let json = serde_json::to_string(&event).unwrap_or_default();
                    yield Ok(Event::default().event("task_update").data(json));
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    // Client was too slow, some events were dropped
                    let msg = serde_json::json!({
                        "type": "lagged",
                        "dropped": n
                    });
                    yield Ok(Event::default().event("system").data(msg.to_string()));
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn event_bus_publish_subscribe() {
        let bus = EventBus::new();
        let user_id = Uuid::new_v4();

        let mut rx = bus.subscribe(user_id).await;

        let event = TaskEvent {
            task_id: Uuid::new_v4(),
            status: "confirmed".to_string(),
            message: "Reservation confirmed".to_string(),
            attempt_count: 42,
            reservation_number: Some("PNR123".to_string()),
        };

        bus.publish(user_id, event.clone()).await;

        let received = rx.recv().await.unwrap();
        assert_eq!(received.task_id, event.task_id);
        assert_eq!(received.status, "confirmed");
        assert_eq!(received.reservation_number, Some("PNR123".to_string()));
    }

    #[tokio::test]
    async fn event_bus_no_cross_user_leak() {
        let bus = EventBus::new();
        let user_a = Uuid::new_v4();
        let user_b = Uuid::new_v4();

        let mut rx_b = bus.subscribe(user_b).await;

        let event = TaskEvent {
            task_id: Uuid::new_v4(),
            status: "running".to_string(),
            message: "Searching...".to_string(),
            attempt_count: 1,
            reservation_number: None,
        };

        bus.publish(user_a, event).await;

        // user_b should not receive user_a's event
        let result = tokio::time::timeout(std::time::Duration::from_millis(50), rx_b.recv()).await;
        assert!(result.is_err()); // timeout = no message
    }

    #[tokio::test]
    async fn event_bus_cleanup_removes_empty_channels() {
        let bus = EventBus::new();
        let user_id = Uuid::new_v4();

        // Create a channel by subscribing, then drop the receiver
        {
            let _rx = bus.subscribe(user_id).await;
        }

        // Channel exists but has no subscribers
        bus.cleanup_inactive().await;

        let channels = bus.channels.read().await;
        assert!(!channels.contains_key(&user_id));
    }

    #[test]
    fn task_event_serialization() {
        let event = TaskEvent {
            task_id: Uuid::nil(),
            status: "confirmed".to_string(),
            message: "Reserved!".to_string(),
            attempt_count: 10,
            reservation_number: Some("ABC123".to_string()),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["status"], "confirmed");
        assert_eq!(json["reservation_number"], "ABC123");
    }

    #[test]
    fn task_event_skips_none_reservation() {
        let event = TaskEvent {
            task_id: Uuid::nil(),
            status: "running".to_string(),
            message: "Searching...".to_string(),
            attempt_count: 1,
            reservation_number: None,
        };
        let json = serde_json::to_value(&event).unwrap();
        assert!(json.get("reservation_number").is_none());
    }
}
