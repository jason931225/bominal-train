use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init() -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(
            "bominal_server=info,bominal_provider=info,bominal_domain=info,tower_http=info,warn",
        )
    });

    let is_production =
        std::env::var("BOMINAL_ENV").unwrap_or_default() == "production";

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

    Ok(())
}
