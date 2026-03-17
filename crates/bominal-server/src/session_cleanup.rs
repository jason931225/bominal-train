//! Periodic cleanup of expired sessions.

use bominal_db::DbPool;
use tracing::{error, info};

/// Spawn a background task that deletes expired sessions and stale
/// passkey challenges every hour.
pub fn spawn_cleanup(db: DbPool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            match bominal_db::session::delete_expired_sessions(&db).await {
                Ok(count) => {
                    if count > 0 {
                        info!(deleted = count, "Cleaned up expired sessions");
                    }
                }
                Err(e) => {
                    error!(error = %e, "Failed to clean up expired sessions");
                }
            }
            match bominal_db::passkey::delete_expired_challenges(&db).await {
                Ok(count) => {
                    if count > 0 {
                        info!(deleted = count, "Cleaned up expired passkey challenges");
                    }
                }
                Err(e) => {
                    error!(error = %e, "Failed to clean up passkey challenges");
                }
            }
        }
    });
}
