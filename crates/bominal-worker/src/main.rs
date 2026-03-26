//! Bominal Worker — standalone background task runner.
//!
//! Polls for queued train reservation tasks and spawns tokio workers to
//! search/reserve seats via SRT/KTX providers. Runs as a separate process
//! so each worker instance can hold its own provider credentials.
//!
//! Usage: `cargo run -p bominal-worker`

mod events;
mod runner;

use anyhow::{Context, Result};

use bominal_domain::crypto::encryption::EncryptionKey;
use bominal_domain::evervault::EvervaultConfig;
use bominal_email::EmailClient;

use crate::events::EventPublisher;

fn init_tracing() {
    use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(
            "bominal_worker=info,bominal_provider=info,bominal_domain=info,warn",
        )
    });

    let is_production = std::env::var("BOMINAL_ENV").unwrap_or_default() == "production";

    if is_production {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().pretty())
            .init();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    tracing::info!("Starting Bominal worker");

    // ── Config from env ──────────────────────────────────────────
    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL is required")?;
    let encryption_key_hex =
        std::env::var("ENCRYPTION_KEY").context("ENCRYPTION_KEY is required")?;
    let valkey_url =
        std::env::var("VALKEY_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    let resend_api_key = std::env::var("RESEND_API_KEY").context("RESEND_API_KEY is required")?;
    let email_from =
        std::env::var("EMAIL_FROM").unwrap_or_else(|_| "Bominal <noreply@bominal.com>".into());
    let app_base_url =
        std::env::var("APP_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".into());

    let ev_team_id = std::env::var("EV_TEAM_ID").context("EV_TEAM_ID is required")?;
    let ev_app_id = std::env::var("EV_APP_ID").context("EV_APP_ID is required")?;
    let ev_api_key = std::env::var("EV_API_KEY").context("EV_API_KEY is required")?;
    let ev_srt_domain = std::env::var("EV_SRT_DOMAIN").context("EV_SRT_DOMAIN is required")?;
    let ev_ktx_domain = std::env::var("EV_KTX_DOMAIN").context("EV_KTX_DOMAIN is required")?;

    // ── Initialize dependencies ──────────────────────────────────
    let db = bominal_db::create_pool(&database_url).await?;
    let email = EmailClient::new(&resend_api_key, &email_from);
    let encryption_key = EncryptionKey::from_hex(&encryption_key_hex)?;
    let evervault = EvervaultConfig::new(
        &ev_team_id,
        &ev_app_id,
        &ev_api_key,
        &ev_srt_domain,
        &ev_ktx_domain,
    );

    let publisher = EventPublisher::connect(&valkey_url)
        .await
        .context("Failed to connect to Valkey")?;

    tracing::info!("Connected to Valkey at {}", valkey_url);

    // ── Run the task loop (never returns) ────────────────────────
    runner::run_loop(db, publisher, email, encryption_key, evervault, app_base_url).await;

    Ok(())
}
