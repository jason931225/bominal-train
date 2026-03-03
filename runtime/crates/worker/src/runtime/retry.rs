use chrono::{DateTime, Duration, Utc};

use super::executor::ExecutionErrorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryClass {
    Retryable,
    NonRetryable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            base_delay: Duration::seconds(5),
            max_delay: Duration::seconds(300),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailureAction {
    ScheduleRetry { next_run_at: DateTime<Utc> },
    DeadLetter { failure_kind: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailurePlan {
    pub class: RetryClass,
    pub action: FailureAction,
}

pub fn classify_error(error_kind: &ExecutionErrorKind) -> RetryClass {
    match error_kind {
        ExecutionErrorKind::Transient | ExecutionErrorKind::RateLimited => RetryClass::Retryable,
        ExecutionErrorKind::Fatal
        | ExecutionErrorKind::PaymentBlocked
        | ExecutionErrorKind::UnsupportedProvider => RetryClass::NonRetryable,
    }
}

pub fn compute_backoff_delay(policy: &RetryPolicy, attempt_count: i32) -> Duration {
    let base_seconds = policy.base_delay.num_seconds().max(1);
    let max_seconds = policy.max_delay.num_seconds().max(base_seconds);
    let exponent = attempt_count.saturating_sub(1).clamp(0, 20) as u32;
    let multiplier = 2_i64.pow(exponent);
    let bounded_seconds = base_seconds.saturating_mul(multiplier).min(max_seconds);
    Duration::seconds(bounded_seconds)
}

pub fn plan_failure(
    now: DateTime<Utc>,
    attempt_count: i32,
    max_attempts: i32,
    error_kind: &ExecutionErrorKind,
    policy: &RetryPolicy,
) -> FailurePlan {
    let class = classify_error(error_kind);
    let attempt = attempt_count.max(1);
    let max = max_attempts.max(1);
    let can_retry = class == RetryClass::Retryable && attempt < max;

    if can_retry {
        let delay = compute_backoff_delay(policy, attempt);
        return FailurePlan {
            class,
            action: FailureAction::ScheduleRetry {
                next_run_at: now + delay,
            },
        };
    }

    let failure_kind = if class == RetryClass::Retryable {
        "retry_exhausted"
    } else {
        "non_retryable_error"
    };

    FailurePlan {
        class,
        action: FailureAction::DeadLetter { failure_kind },
    }
}
