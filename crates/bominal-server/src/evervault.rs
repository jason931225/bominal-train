//! Evervault configuration for payment card encryption.
//!
//! Architecture:
//! - **Frontend**: Evervault JS SDK encrypts card fields client-side before
//!   POST to `/api/cards`. The server never sees plaintext card data.
//! - **Storage**: Server stores `ev:`-prefixed encrypted strings in the DB.
//! - **Provider payment**: HTTP requests to SRT/KTX go through the Evervault
//!   Outbound Relay, which decrypts `ev:` values before forwarding.
//!
//! Provider credentials (login passwords) still use AES-256-GCM via
//! `ENCRYPTION_KEY` since those don't need Evervault's PCI-scoped protection.

/// Evervault configuration with per-provider relay domains.
#[derive(Clone, Debug)]
pub struct EvervaultConfig {
    /// Evervault Team ID — exposed to frontend for JS SDK initialization.
    pub team_id: String,
    /// Evervault App ID — exposed to frontend for JS SDK initialization.
    pub app_id: String,
    /// Evervault API key — used for Outbound Relay authentication.
    pub api_key: String,
    /// Relay domain for SRT provider (e.g. `app-srail-or-kr-<app_id>.relay.evervault.app`).
    pub srt_relay_domain: String,
    /// Relay domain for KTX provider (e.g. `smart-letskorail-com-<app_id>.relay.evervault.app`).
    pub ktx_relay_domain: String,
}

impl EvervaultConfig {
    /// Create a new Evervault configuration.
    pub fn new(team_id: &str, app_id: &str, api_key: &str, srt_relay_domain: &str, ktx_relay_domain: &str) -> Self {
        Self {
            team_id: team_id.to_string(),
            app_id: app_id.to_string(),
            api_key: api_key.to_string(),
            srt_relay_domain: srt_relay_domain.to_string(),
            ktx_relay_domain: ktx_relay_domain.to_string(),
        }
    }

    /// Relay URL for SRT provider payment requests.
    pub fn srt_relay_url(&self) -> String {
        format!("https://{}", self.srt_relay_domain)
    }

    /// Relay URL for KTX provider payment requests.
    pub fn ktx_relay_url(&self) -> String {
        format!("https://{}", self.ktx_relay_domain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relay_urls() {
        let ev = EvervaultConfig::new(
            "team_test456",
            "app_test123",
            "ev:key:test",
            "app-srail-or-kr-app-test123.relay.evervault.app",
            "smart-letskorail-com-app-test123.relay.evervault.app",
        );
        assert_eq!(
            ev.srt_relay_url(),
            "https://app-srail-or-kr-app-test123.relay.evervault.app"
        );
        assert_eq!(
            ev.ktx_relay_url(),
            "https://smart-letskorail-com-app-test123.relay.evervault.app"
        );
    }

    #[test]
    fn config_fields() {
        let ev = EvervaultConfig::new("team_id", "app_id", "api_key", "srt.relay", "ktx.relay");
        assert_eq!(ev.team_id, "team_id");
        assert_eq!(ev.app_id, "app_id");
        assert_eq!(ev.api_key, "api_key");
        assert_eq!(ev.srt_relay_domain, "srt.relay");
        assert_eq!(ev.ktx_relay_domain, "ktx.relay");
    }
}
