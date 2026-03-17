//! Database layer with Postgres and compile-time checked queries.

pub mod card;
pub mod passkey;
pub mod provider;
pub mod session;
pub mod task;
pub mod user;

use sqlx::postgres::PgPoolOptions;

pub type DbPool = sqlx::PgPool;

/// Create a database connection pool.
///
/// Configured for e2-micro (max 20 connections).
pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await
}
