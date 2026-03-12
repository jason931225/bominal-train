//! Resend API client for sending transactional emails.

use serde::Serialize;
use tracing::{error, info};

/// Errors from the email client.
#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("Resend API error: {0}")]
    Api(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}

/// Email client backed by the Resend API.
#[derive(Clone)]
pub struct EmailClient {
    api_key: String,
    from_address: String,
    http: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct SendRequest<'a> {
    from: &'a str,
    to: &'a [&'a str],
    subject: &'a str,
    html: &'a str,
}

#[derive(Debug, serde::Deserialize)]
struct SendResponse {
    id: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ErrorResponse {
    message: Option<String>,
}

impl EmailClient {
    /// Create a new Resend email client.
    ///
    /// - `api_key`: Resend API key (`re_...`)
    /// - `from_address`: Verified sender, e.g. `"Bominal <noreply@bominal.com>"`
    pub fn new(api_key: &str, from_address: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            from_address: from_address.to_string(),
            http: reqwest::Client::new(),
        }
    }

    /// Send an email via Resend.
    pub async fn send(
        &self,
        to: &str,
        subject: &str,
        html: &str,
    ) -> Result<String, EmailError> {
        let body = SendRequest {
            from: &self.from_address,
            to: &[to],
            subject,
            html,
        };

        let resp = self
            .http
            .post("https://api.resend.com/emails")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        let status = resp.status();

        if status.is_success() {
            let data: SendResponse = resp.json().await?;
            let msg_id = data.id.unwrap_or_default();
            info!(to = to, subject = subject, msg_id = %msg_id, "Email sent");
            Ok(msg_id)
        } else {
            let err_body = resp.text().await.unwrap_or_default();
            let message = serde_json::from_str::<ErrorResponse>(&err_body)
                .ok()
                .and_then(|e| e.message)
                .unwrap_or(err_body);
            error!(to = to, subject = subject, status = %status, error = %message, "Email send failed");
            Err(EmailError::Api(message))
        }
    }

    /// Send email, logging failure without propagating the error.
    ///
    /// Use this for non-critical notifications (reservation alerts, etc.)
    /// where a delivery failure should not break the main flow.
    pub async fn send_best_effort(&self, to: &str, subject: &str, html: &str) {
        if let Err(e) = self.send(to, subject, html).await {
            error!(to = to, subject = subject, error = %e, "Best-effort email failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_stores_config() {
        let client = EmailClient::new("re_test_key", "Bominal <noreply@bominal.com>");
        assert_eq!(client.api_key, "re_test_key");
        assert_eq!(client.from_address, "Bominal <noreply@bominal.com>");
    }

    #[test]
    fn send_request_serializes() {
        let req = SendRequest {
            from: "Bominal <noreply@bominal.com>",
            to: &["user@example.com"],
            subject: "Test",
            html: "<p>Hello</p>",
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["from"], "Bominal <noreply@bominal.com>");
        assert_eq!(json["to"][0], "user@example.com");
        assert_eq!(json["subject"], "Test");
    }
}
