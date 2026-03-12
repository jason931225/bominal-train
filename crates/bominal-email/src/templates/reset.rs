//! Password reset template.
//!
//! Sent when a user requests a password reset.

use super::base;
use super::html_escape;

/// Render the password reset email.
///
/// - `display_name`: User's display name
/// - `reset_url`: Full URL with token
/// - `expires_minutes`: Link validity period
pub fn render(display_name: &str, reset_url: &str, expires_minutes: u32) -> (String, String) {
    let name = html_escape(display_name);
    let url = html_escape(reset_url);

    let subject = "Bominal - Reset your password".to_string();

    let body = format!(
        r##"<h1 style="margin:0 0 8px; font-size:24px; font-weight:700; color:#f1f5f9;">
  Reset Your Password
</h1>
<p style="margin:0 0 24px; font-size:15px; color:#94a3b8; line-height:1.6;">
  Hi {name}, we received a request to reset your password. Click the button
  below to choose a new one.
</p>
<table role="presentation" width="100%" cellpadding="0" cellspacing="0">
  <tr>
    <td align="center" style="padding:8px 0 24px;">
      <a href="{url}" class="cta-btn" style="background-color:#3b82f6; color:#ffffff !important; border-radius:8px; display:inline-block; font-size:16px; font-weight:600; line-height:1; padding:14px 32px; text-decoration:none !important;">
        Reset Password
      </a>
    </td>
  </tr>
</table>
<p style="margin:0 0 16px; font-size:13px; color:#64748b; line-height:1.5;">
  Or copy and paste this link:
</p>
<p style="margin:0 0 24px; font-size:13px; color:#60a5fa; word-break:break-all; line-height:1.5;">
  {url}
</p>
<table role="presentation" width="100%" cellpadding="0" cellspacing="0">
  <tr>
    <td style="background-color:rgba(239,68,68,0.08); border:1px solid rgba(239,68,68,0.2); border-radius:8px; padding:12px 16px;">
      <p style="margin:0; font-size:13px; color:#f87171; line-height:1.5;">
        &#128274; This link expires in {expires_minutes} minutes. If you did not
        request a password reset, please ignore this email or contact support
        if you suspect unauthorized access.
      </p>
    </td>
  </tr>
</table>"##
    );

    let html = base::layout(
        "Reset your Bominal password",
        &body,
    );

    (subject, html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_with_name_and_url() {
        let (subject, html) = render("Jason", "https://bominal.com/reset?token=xyz", 15);
        assert_eq!(subject, "Bominal - Reset your password");
        assert!(html.contains("Jason"));
        assert!(html.contains("https://bominal.com/reset?token=xyz"));
        assert!(html.contains("15 minutes"));
    }

    #[test]
    fn contains_security_warning() {
        let (_, html) = render("User", "https://example.com/reset?t=x", 30);
        // "did not" is in the raw HTML template
        assert!(html.contains("did not"));
        assert!(html.contains("unauthorized"));
    }

    #[test]
    fn contains_reset_button() {
        let (_, html) = render("User", "https://example.com/reset?t=x", 30);
        assert!(html.contains("Reset Password"));
    }
}
