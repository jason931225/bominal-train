//! Exponential backoff retry for transient provider errors.

use std::future::Future;
use std::time::Duration;

use crate::types::ProviderError;

/// Retry an async expression with exponential backoff.
///
/// Use this macro instead of [`retry_with_backoff`] when the expression borrows
/// `&mut self` — a `Fn` closure cannot capture a mutable reference, but the
/// macro re-evaluates the expression each iteration, sidestepping the issue.
///
/// The expression **must** include `.await` (e.g. `client.search(...).await`).
///
/// Backoff schedule: 500ms, 1s, 2s, 4s, ...
#[macro_export]
macro_rules! retry_with_backoff {
    ($max_retries:expr, $op:expr) => {{
        let __max: u32 = $max_retries;
        let mut __attempt: u32 = 0;
        loop {
            match $op {
                Ok(val) => break Ok(val),
                Err(e) if e.is_retryable() && __attempt < __max => {
                    let __delay = ::std::time::Duration::from_millis(500 * 2u64.pow(__attempt));
                    ::tokio::time::sleep(__delay).await;
                    __attempt += 1;
                }
                Err(e) => break Err(e),
            }
        }
    }};
}

/// Retry an async operation with exponential backoff.
///
/// Retries up to `max_retries` times for transient errors (network failures,
/// server errors, NetFunnel blocks). Non-retryable errors are returned immediately.
///
/// Backoff schedule: 500ms, 1s, 2s, 4s, ...
pub async fn retry_with_backoff<F, Fut, T>(max_retries: u32, f: F) -> Result<T, ProviderError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, ProviderError>>,
{
    let mut last_err = None;
    for attempt in 0..=max_retries {
        match f().await {
            Ok(val) => return Ok(val),
            Err(e) if e.is_retryable() && attempt < max_retries => {
                let delay = Duration::from_millis(500 * 2u64.pow(attempt));
                tokio::time::sleep(delay).await;
                last_err = Some(e);
            }
            Err(e) => return Err(e),
        }
    }
    Err(last_err.unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn succeeds_on_first_try() {
        let result = retry_with_backoff(3, || async { Ok::<_, ProviderError>(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn retries_on_server_error() {
        tokio::time::pause();
        let attempts = AtomicU32::new(0);
        let result = retry_with_backoff(3, || {
            let attempt = attempts.fetch_add(1, Ordering::SeqCst);
            async move {
                if attempt < 2 {
                    Err(ProviderError::UnexpectedResponse {
                        status: 500,
                        body: "Internal Server Error".to_string(),
                    })
                } else {
                    Ok(42)
                }
            }
        })
        .await;
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn retries_on_netfunnel_blocked() {
        tokio::time::pause();
        let attempts = AtomicU32::new(0);
        let result = retry_with_backoff(3, || {
            let attempt = attempts.fetch_add(1, Ordering::SeqCst);
            async move {
                if attempt < 1 {
                    Err(ProviderError::NetFunnelBlocked)
                } else {
                    Ok("success")
                }
            }
        })
        .await;
        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn does_not_retry_non_retryable() {
        let attempts = AtomicU32::new(0);
        let result = retry_with_backoff(3, || {
            attempts.fetch_add(1, Ordering::SeqCst);
            async { Err::<i32, _>(ProviderError::SoldOut) }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn does_not_retry_client_error() {
        let attempts = AtomicU32::new(0);
        let result = retry_with_backoff(3, || {
            attempts.fetch_add(1, Ordering::SeqCst);
            async {
                Err::<i32, _>(ProviderError::UnexpectedResponse {
                    status: 400,
                    body: "Bad Request".to_string(),
                })
            }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn exhausts_retries() {
        tokio::time::pause();
        let attempts = AtomicU32::new(0);
        let result = retry_with_backoff(2, || {
            attempts.fetch_add(1, Ordering::SeqCst);
            async { Err::<i32, _>(ProviderError::NetFunnelBlocked) }
        })
        .await;
        assert!(matches!(result, Err(ProviderError::NetFunnelBlocked)));
        assert_eq!(attempts.load(Ordering::SeqCst), 3); // initial + 2 retries
    }
}
