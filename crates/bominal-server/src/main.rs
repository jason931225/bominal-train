use std::net::SocketAddr;

use bominal_server::{config, routes, telemetry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let prometheus_handle = telemetry::init()?;
    any_spawner::Executor::init_tokio().expect("Failed to initialize Tokio executor");
    tracing::info!("Starting Bominal server");

    let config = config::AppConfig::from_env()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    let app = routes::create_router(&config, prometheus_handle).await?;

    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
