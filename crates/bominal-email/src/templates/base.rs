//! Shared base layout for all email templates.
//!
//! Uses table-based layout for maximum email client compatibility.
//! Dark theme with blue accents matching the Bominal app.

use crate::templates::html_escape;

/// Wrap inner content in the standard Bominal email layout.
///
/// - `preheader`: Hidden preview text shown in inbox list
/// - `body_html`: Inner content (already formatted as HTML)
pub fn layout(preheader: &str, body_html: &str) -> String {
    let preheader = html_escape(preheader);
    format!(
        r##"<!DOCTYPE html>
<html lang="ko" xmlns="http://www.w3.org/1999/xhtml">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta http-equiv="X-UA-Compatible" content="IE=edge">
  <meta name="color-scheme" content="dark">
  <meta name="supported-color-schemes" content="dark">
  <title>Bominal</title>
  <!--[if mso]>
  <noscript><xml>
    <o:OfficeDocumentSettings>
      <o:PixelsPerInch>96</o:PixelsPerInch>
    </o:OfficeDocumentSettings>
  </xml></noscript>
  <![endif]-->
  <style>
    body, table, td {{ margin: 0; padding: 0; }}
    body {{ background-color: #0c1222; color: #f1f5f9; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif; }}
    img {{ border: 0; display: block; max-width: 100%; }}
    a {{ color: #60a5fa; text-decoration: none; }}
    a:hover {{ text-decoration: underline; }}
    .preheader {{ display: none !important; font-size: 1px; line-height: 1px; max-height: 0; max-width: 0; mso-hide: all; overflow: hidden; }}
    .cta-btn {{ background-color: #3b82f6; color: #ffffff !important; border-radius: 8px; display: inline-block; font-size: 16px; font-weight: 600; line-height: 1; padding: 14px 32px; text-decoration: none !important; }}
    .cta-btn:hover {{ background-color: #2563eb; }}
    .card {{ background-color: #1e293b; border: 1px solid rgba(59,130,246,0.15); border-radius: 12px; }}
    .detail-row td {{ padding: 8px 0; border-bottom: 1px solid rgba(148,163,184,0.1); }}
    .detail-row:last-child td {{ border-bottom: none; }}
    .badge {{ display: inline-block; padding: 4px 12px; border-radius: 999px; font-size: 12px; font-weight: 600; letter-spacing: 0.5px; text-transform: uppercase; }}
    .badge-success {{ background-color: rgba(16,185,129,0.15); color: #34d399; }}
    .badge-warning {{ background-color: rgba(245,158,11,0.15); color: #fbbf24; }}
    .badge-error {{ background-color: rgba(239,68,68,0.15); color: #f87171; }}
    .badge-info {{ background-color: rgba(59,130,246,0.15); color: #60a5fa; }}
    @media only screen and (max-width: 600px) {{
      .container {{ width: 100% !important; padding: 16px !important; }}
      .inner {{ padding: 24px 20px !important; }}
    }}
  </style>
</head>
<body style="margin:0; padding:0; background-color:#0c1222;">
  <span class="preheader">{preheader}</span>
  <table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="background-color:#0c1222;">
    <tr>
      <td align="center" style="padding:40px 16px;">
        <table role="presentation" class="container" width="560" cellpadding="0" cellspacing="0" style="max-width:560px; width:100%;">
          <!-- Logo -->
          <tr>
            <td align="center" style="padding-bottom:32px;">
              <table role="presentation" cellpadding="0" cellspacing="0">
                <tr>
                  <td style="background: linear-gradient(135deg, #3b82f6, #6366f1); border-radius:12px; padding:10px 14px;">
                    <span style="font-size:24px; line-height:1;">&#128646;</span>
                  </td>
                  <td style="padding-left:12px;">
                    <span style="font-size:22px; font-weight:700; color:#f1f5f9; letter-spacing:-0.5px;">Bominal</span>
                  </td>
                </tr>
              </table>
            </td>
          </tr>
          <!-- Content card -->
          <tr>
            <td class="card" style="background-color:#1e293b; border:1px solid rgba(59,130,246,0.15); border-radius:12px;">
              <table role="presentation" width="100%" cellpadding="0" cellspacing="0">
                <tr>
                  <td class="inner" style="padding:36px 32px;">
                    {body_html}
                  </td>
                </tr>
              </table>
            </td>
          </tr>
          <!-- Footer -->
          <tr>
            <td style="padding-top:32px; text-align:center;">
              <p style="margin:0 0 8px; font-size:13px; color:#64748b;">
                Bominal &mdash; Train Reservation Assistant
              </p>
              <p style="margin:0; font-size:12px; color:#475569;">
                &copy; 2026 Bominal. All rights reserved.
              </p>
            </td>
          </tr>
        </table>
      </td>
    </tr>
  </table>
</body>
</html>"##
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_contains_preheader() {
        let html = layout("Preview text", "<p>Body</p>");
        assert!(html.contains("Preview text"));
        assert!(html.contains("<p>Body</p>"));
    }

    #[test]
    fn layout_has_dark_background() {
        let html = layout("", "<p>test</p>");
        assert!(html.contains("#0c1222"));
    }

    #[test]
    fn layout_escapes_preheader() {
        let html = layout("<script>alert(1)</script>", "<p>safe</p>");
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
