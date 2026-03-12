//! Email verification template.
//!
//! Sent after user registration to confirm ownership of the email address.

use super::base;
use super::html_escape;

/// Render the email verification message.
///
/// - `display_name`: User's display name
/// - `verify_url`: Full URL with token, e.g. `https://bominal.com/verify?token=abc`
/// - `expires_minutes`: How long the link is valid
pub fn render(display_name: &str, verify_url: &str, expires_minutes: u32) -> (String, String) {
    let name = html_escape(display_name);
    let url = html_escape(verify_url);

    let subject = "Bominal - Verify your email".to_string();

    let body = format!(
        r##"<h1 style="margin:0 0 8px; font-size:24px; font-weight:700; color:#f1f5f9;">
  Welcome to Bominal
</h1>
<p style="margin:0 0 24px; font-size:15px; color:#94a3b8; line-height:1.6;">
  Hi {name}, thanks for signing up! Please verify your email address
  to get started with train reservations.
</p>
<table role="presentation" width="100%" cellpadding="0" cellspacing="0">
  <tr>
    <td align="center" style="padding:8px 0 24px;">
      <a href="{url}" class="cta-btn" style="background-color:#3b82f6; color:#ffffff !important; border-radius:8px; display:inline-block; font-size:16px; font-weight:600; line-height:1; padding:14px 32px; text-decoration:none !important;">
        Verify Email Address
      </a>
    </td>
  </tr>
</table>
<p style="margin:0 0 16px; font-size:13px; color:#64748b; line-height:1.5;">
  Or copy and paste this link into your browser:
</p>
<p style="margin:0 0 24px; font-size:13px; color:#60a5fa; word-break:break-all; line-height:1.5;">
  {url}
</p>
<table role="presentation" width="100%" cellpadding="0" cellspacing="0">
  <tr>
    <td style="background-color:rgba(245,158,11,0.08); border:1px solid rgba(245,158,11,0.2); border-radius:8px; padding:12px 16px;">
      <p style="margin:0; font-size:13px; color:#fbbf24; line-height:1.5;">
        &#9888;&#65039; This link expires in {expires_minutes} minutes. If you didn't create
        an account, you can safely ignore this email.
      </p>
    </td>
  </tr>
</table>"##
    );

    let html = base::layout("Verify your email to start using Bominal", &body);

    (subject, html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_with_name_and_url() {
        let (subject, html) = render("Jason", "https://bominal.com/verify?token=abc", 30);
        assert_eq!(subject, "Bominal - Verify your email");
        assert!(html.contains("Jason"));
        assert!(html.contains("https://bominal.com/verify?token=abc"));
        assert!(html.contains("30 minutes"));
    }

    #[test]
    fn escapes_xss_in_name() {
        let (_, html) = render("<script>alert(1)</script>", "https://example.com", 30);
        assert!(!html.contains("<script>alert"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn contains_cta_button() {
        let (_, html) = render("User", "https://bominal.com/verify?t=x", 60);
        assert!(html.contains("Verify Email Address"));
        assert!(html.contains("cta-btn"));
    }
}
