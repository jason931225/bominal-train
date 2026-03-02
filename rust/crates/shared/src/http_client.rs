use std::time::Duration;

use anyhow::Result;
use reqwest::Client;

pub fn build_http_client(timeout: Duration) -> Result<Client> {
    let builder = Client::builder()
        .connect_timeout(timeout)
        .timeout(timeout)
        .pool_idle_timeout(Duration::from_secs(30))
        .user_agent("bominal-rust/0.1.0");

    #[cfg(feature = "curl-transport")]
    let builder = {
        // The `curl-transport` feature allows environments that require native TLS stacks.
        builder.use_native_tls()
    };

    Ok(builder.build()?)
}
