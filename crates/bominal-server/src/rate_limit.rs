//! Simple in-memory token bucket rate limiter per IP.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use tokio::sync::Mutex;

/// Per-IP bucket state.
struct Bucket {
    tokens: f64,
    last_refill: Instant,
}

/// Token bucket rate limiter.
#[derive(Clone)]
pub struct RateLimiter {
    inner: Arc<Mutex<HashMap<IpAddr, Bucket>>>,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
}

impl RateLimiter {
    /// Create a rate limiter with the given requests per minute.
    pub fn new(requests_per_minute: u32) -> Self {
        let max_tokens = requests_per_minute as f64;
        let refill_rate = max_tokens / 60.0;
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            max_tokens,
            refill_rate,
        }
    }

    /// Try to consume one token for the given IP. Returns true if allowed.
    async fn try_acquire(&self, ip: IpAddr) -> bool {
        let mut map = self.inner.lock().await;
        let now = Instant::now();

        let bucket = map.entry(ip).or_insert(Bucket {
            tokens: self.max_tokens,
            last_refill: now,
        });

        // Refill tokens based on elapsed time
        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        bucket.last_refill = now;

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Spawn a background task to periodically clean up stale entries.
    pub fn spawn_cleanup(self) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(300)).await;
                let mut map = self.inner.lock().await;
                let now = Instant::now();
                map.retain(|_, bucket| {
                    now.duration_since(bucket.last_refill) < Duration::from_secs(600)
                });
            }
        });
    }
}

/// Axum middleware that rate-limits by client IP.
pub async fn rate_limit_middleware(
    limiter: RateLimiter,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract IP from ConnectInfo or X-Forwarded-For header
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse::<IpAddr>().ok())
        .unwrap_or_else(|| {
            req.extensions()
                .get::<ConnectInfo<std::net::SocketAddr>>()
                .map(|ci| ci.0.ip())
                .unwrap_or(IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))
        });

    if !limiter.try_acquire(ip).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(req).await)
}
