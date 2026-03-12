//! Broadcast email template.
//!
//! A generic template for internal announcements, product updates,
//! and important notices sent to all verified users.

use super::base;
use super::html_escape;

/// A content section in the broadcast.
pub struct Section<'a> {
    pub title: &'a str,
    pub body: &'a str,
}

/// Render a broadcast email.
///
/// - `display_name`: Recipient's display name
/// - `subject_line`: Email subject (also used as heading)
/// - `intro`: Opening paragraph
/// - `sections`: Content sections (can be empty)
/// - `app_url`: Link to the app
pub fn render(
    display_name: &str,
    subject_line: &str,
    intro: &str,
    sections: &[Section<'_>],
    app_url: &str,
) -> (String, String) {
    let name = html_escape(display_name);
    let title = html_escape(subject_line);
    let intro = html_escape(intro);
    let url = html_escape(app_url);

    let subject = format!("Bominal - {}", subject_line);

    let sections_html: String = sections
        .iter()
        .map(|s| {
            let sec_title = html_escape(s.title);
            let sec_body = html_escape(s.body);
            format!(
                r#"<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="margin-bottom:20px;">
  <tr>
    <td style="background-color:rgba(15,23,42,0.5); border:1px solid rgba(148,163,184,0.1); border-radius:8px; padding:20px;">
      <h3 style="margin:0 0 8px; font-size:16px; font-weight:600; color:#f1f5f9;">{sec_title}</h3>
      <p style="margin:0; font-size:14px; color:#94a3b8; line-height:1.6;">{sec_body}</p>
    </td>
  </tr>
</table>"#
            )
        })
        .collect();

    let body = format!(
        r##"<!-- Header gradient bar -->
<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="margin-bottom:24px;">
  <tr>
    <td style="height:4px; background:linear-gradient(90deg, #3b82f6, #8b5cf6, #3b82f6); border-radius:2px;"></td>
  </tr>
</table>
<h1 style="margin:0 0 8px; font-size:24px; font-weight:700; color:#f1f5f9;">
  {title}
</h1>
<p style="margin:0 0 24px; font-size:15px; color:#94a3b8; line-height:1.6;">
  Hi {name}, {intro}
</p>
{sections_html}
<table role="presentation" width="100%" cellpadding="0" cellspacing="0">
  <tr>
    <td align="center" style="padding:12px 0 0;">
      <a href="{url}" class="cta-btn" style="background-color:#3b82f6; color:#ffffff !important; border-radius:8px; display:inline-block; font-size:16px; font-weight:600; line-height:1; padding:14px 32px; text-decoration:none !important;">
        Open Bominal
      </a>
    </td>
  </tr>
</table>"##
    );

    let preheader = format!("{} - {}", subject_line, intro);
    let html = base::layout(&preheader, &body);

    (subject, html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_broadcast() {
        let sections = vec![
            Section {
                title: "KTX Integration",
                body: "We've added full KTX support alongside SRT.",
            },
            Section {
                title: "Auto-Pay",
                body: "Save your card and pay automatically when seats are found.",
            },
        ];
        let (subject, html) = render(
            "Jason",
            "March Update",
            "here's what's new this month.",
            &sections,
            "https://bominal.com",
        );
        assert_eq!(subject, "Bominal - March Update");
        assert!(html.contains("Jason"));
        assert!(html.contains("KTX Integration"));
        assert!(html.contains("Auto-Pay"));
        assert!(html.contains("Open Bominal"));
    }

    #[test]
    fn empty_sections() {
        let (_, html) = render(
            "User",
            "Update",
            "not much to report.",
            &[],
            "https://bominal.com",
        );
        assert!(html.contains("Bominal"));
        assert!(html.contains("not much to report"));
    }

    #[test]
    fn escapes_user_content() {
        let (_, html) = render(
            "<b>hacker</b>",
            "Title <script>",
            "body &amp;",
            &[Section {
                title: "Sec <img>",
                body: "Content \"quotes\"",
            }],
            "https://bominal.com",
        );
        assert!(!html.contains("<b>hacker</b>"));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;b&gt;"));
    }
}
