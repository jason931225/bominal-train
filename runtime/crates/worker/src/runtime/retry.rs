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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn classify_error_maps_execution_kinds_to_retry_class() {
        assert_eq!(
            classify_error(&ExecutionErrorKind::Transient),
            RetryClass::Retryable
        );
        assert_eq!(
            classify_error(&ExecutionErrorKind::RateLimited),
            RetryClass::Retryable
        );
        assert_eq!(
            classify_error(&ExecutionErrorKind::Fatal),
            RetryClass::NonRetryable
        );
        assert_eq!(
            classify_error(&ExecutionErrorKind::PaymentBlocked),
            RetryClass::NonRetryable
        );
        assert_eq!(
            classify_error(&ExecutionErrorKind::UnsupportedProvider),
            RetryClass::NonRetryable
        );
    }

    #[test]
    fn compute_backoff_delay_is_deterministic_and_capped() {
        let policy = RetryPolicy::default();

        assert_eq!(compute_backoff_delay(&policy, -3), Duration::seconds(5));
        assert_eq!(compute_backoff_delay(&policy, 1), Duration::seconds(5));
        assert_eq!(compute_backoff_delay(&policy, 2), Duration::seconds(10));
        assert_eq!(compute_backoff_delay(&policy, 3), Duration::seconds(20));
        assert_eq!(compute_backoff_delay(&policy, 8), Duration::seconds(300));
        assert_eq!(compute_backoff_delay(&policy, 40), Duration::seconds(300));
        assert_eq!(compute_backoff_delay(&policy, 40), Duration::seconds(300));
    }

    #[test]
    fn plan_failure_schedules_retry_with_expected_next_run_at() {
        let policy = RetryPolicy::default();
        let now = Utc
            .with_ymd_and_hms(2026, 1, 15, 10, 0, 0)
            .single()
            .expect("valid timestamp");

        let plan = plan_failure(now, 2, 5, &ExecutionErrorKind::Transient, &policy);
        assert_eq!(plan.class, RetryClass::Retryable);
        assert_eq!(
            plan.action,
            FailureAction::ScheduleRetry {
                next_run_at: now + Duration::seconds(10),
            }
        );
    }

    #[test]
    fn plan_failure_dead_letters_exhausted_or_non_retryable_failures() {
        let policy = RetryPolicy::default();
        let now = Utc
            .with_ymd_and_hms(2026, 1, 15, 10, 0, 0)
            .single()
            .expect("valid timestamp");

        let exhausted = plan_failure(now, 3, 3, &ExecutionErrorKind::RateLimited, &policy);
        assert_eq!(exhausted.class, RetryClass::Retryable);
        assert_eq!(
            exhausted.action,
            FailureAction::DeadLetter {
                failure_kind: "retry_exhausted"
            }
        );

        let non_retryable = plan_failure(now, 1, 5, &ExecutionErrorKind::Fatal, &policy);
        assert_eq!(non_retryable.class, RetryClass::NonRetryable);
        assert_eq!(
            non_retryable.action,
            FailureAction::DeadLetter {
                failure_kind: "non_retryable_error"
            }
        );
    }
}
