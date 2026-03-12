//! Reservation task domain model and state machine.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Task status with valid state transitions enforced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Queued,
    Running,
    Idle,
    AwaitingPayment,
    Confirmed,
    Failed,
    Cancelled,
}

impl TaskStatus {
    /// Returns whether transitioning from `self` to `next` is valid.
    ///
    /// # Examples
    ///
    /// ```
    /// use bominal_domain::task::TaskStatus;
    /// assert!(TaskStatus::Queued.can_transition_to(TaskStatus::Running));
    /// assert!(!TaskStatus::Confirmed.can_transition_to(TaskStatus::Running));
    /// ```
    pub fn can_transition_to(self, next: TaskStatus) -> bool {
        matches!(
            (self, next),
            (TaskStatus::Queued, TaskStatus::Running)
                | (TaskStatus::Queued, TaskStatus::Cancelled)
                | (TaskStatus::Running, TaskStatus::Idle)
                | (TaskStatus::Running, TaskStatus::AwaitingPayment)
                | (TaskStatus::Running, TaskStatus::Confirmed)
                | (TaskStatus::Running, TaskStatus::Failed)
                | (TaskStatus::Running, TaskStatus::Cancelled)
                | (TaskStatus::Idle, TaskStatus::Running)
                | (TaskStatus::Idle, TaskStatus::Cancelled)
                | (TaskStatus::AwaitingPayment, TaskStatus::Confirmed)
                | (TaskStatus::AwaitingPayment, TaskStatus::Failed)
                | (TaskStatus::AwaitingPayment, TaskStatus::Cancelled)
        )
    }

    /// Returns the i18n key for this status.
    pub fn i18n_key(self) -> &'static str {
        match self {
            TaskStatus::Queued => "task.queued",
            TaskStatus::Running => "task.running",
            TaskStatus::Idle => "task.idle",
            TaskStatus::AwaitingPayment => "task.awaiting_payment",
            TaskStatus::Confirmed => "task.confirmed",
            TaskStatus::Failed => "task.failed",
            TaskStatus::Cancelled => "task.cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservationTask {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub departure_station: String,
    pub arrival_station: String,
    pub travel_date: String,
    pub departure_time: String,
    pub passengers: serde_json::Value,
    pub seat_preference: String,
    pub target_trains: serde_json::Value,
    pub auto_pay: bool,
    pub payment_card_id: Option<Uuid>,
    pub notify_enabled: bool,
    pub auto_retry: bool,
    pub status: TaskStatus,
    pub reservation_number: Option<String>,
    pub reservation_data: Option<serde_json::Value>,
    pub started_at: Option<DateTime<Utc>>,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub attempt_count: i32,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_transitions() {
        assert!(TaskStatus::Queued.can_transition_to(TaskStatus::Running));
        assert!(TaskStatus::Running.can_transition_to(TaskStatus::Confirmed));
        assert!(TaskStatus::Running.can_transition_to(TaskStatus::AwaitingPayment));
        assert!(TaskStatus::Idle.can_transition_to(TaskStatus::Running));
        assert!(TaskStatus::AwaitingPayment.can_transition_to(TaskStatus::Confirmed));
    }

    #[test]
    fn invalid_transitions() {
        assert!(!TaskStatus::Confirmed.can_transition_to(TaskStatus::Running));
        assert!(!TaskStatus::Failed.can_transition_to(TaskStatus::Running));
        assert!(!TaskStatus::Cancelled.can_transition_to(TaskStatus::Running));
        assert!(!TaskStatus::Queued.can_transition_to(TaskStatus::Confirmed));
    }

    #[test]
    fn i18n_keys() {
        assert_eq!(TaskStatus::Running.i18n_key(), "task.running");
        assert_eq!(TaskStatus::Confirmed.i18n_key(), "task.confirmed");
    }
}
