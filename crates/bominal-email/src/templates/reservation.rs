//! Reservation alert templates.
//!
//! Sent when a reservation task changes state (confirmed, paid, failed, etc.).

use super::base;
use super::html_escape;

/// Reservation alert type determines the visual style and messaging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertKind {
    Confirmed,
    Paid,
    PayFailed,
    Failed,
    Cancelled,
    WaitingConfirmed,
}

impl AlertKind {
    fn badge_class(&self) -> &str {
        match self {
            Self::Confirmed | Self::Paid | Self::WaitingConfirmed => "badge-success",
            Self::PayFailed => "badge-warning",
            Self::Failed | Self::Cancelled => "badge-error",
        }
    }

    fn label(&self) -> &str {
        match self {
            Self::Confirmed => "Confirmed",
            Self::Paid => "Paid",
            Self::PayFailed => "Payment Failed",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
            Self::WaitingConfirmed => "Waitlist Confirmed",
        }
    }

    fn label_ko(&self) -> &str {
        match self {
            Self::Confirmed => "예약 확정",
            Self::Paid => "결제 완료",
            Self::PayFailed => "결제 실패",
            Self::Failed => "예약 실패",
            Self::Cancelled => "예약 취소",
            Self::WaitingConfirmed => "대기 예약 확정",
        }
    }
}

/// Train/reservation details for the alert.
pub struct ReservationDetails<'a> {
    pub provider: &'a str,
    pub train_number: &'a str,
    pub dep_station: &'a str,
    pub arr_station: &'a str,
    pub dep_date: &'a str,
    pub dep_time: &'a str,
    pub pnr: Option<&'a str>,
    pub total_cost: Option<&'a str>,
}

/// Render a reservation alert email.
///
/// - `display_name`: User's display name
/// - `kind`: Alert type (confirmed, paid, failed, etc.)
/// - `details`: Train and reservation info
/// - `app_url`: Link back to the app
pub fn render(
    display_name: &str,
    kind: AlertKind,
    details: &ReservationDetails<'_>,
    app_url: &str,
) -> (String, String) {
    let name = html_escape(display_name);
    let provider = html_escape(details.provider);
    let train_no = html_escape(details.train_number);
    let dep = html_escape(details.dep_station);
    let arr = html_escape(details.arr_station);
    let date = html_escape(details.dep_date);
    let time = html_escape(details.dep_time);
    let url = html_escape(app_url);

    let subject = format!("Bominal - {} {}", kind.label_ko(), train_no);
    let badge = kind.badge_class();
    let label = kind.label();

    let heading = match kind {
        AlertKind::Confirmed | AlertKind::WaitingConfirmed => "Reservation Confirmed!",
        AlertKind::Paid => "Payment Complete!",
        AlertKind::PayFailed => "Payment Failed",
        AlertKind::Failed => "Reservation Failed",
        AlertKind::Cancelled => "Reservation Cancelled",
    };

    let message = match kind {
        AlertKind::Confirmed => {
            format!("Great news, {name}! Your {provider} train reservation has been confirmed.")
        }
        AlertKind::WaitingConfirmed => {
            format!("Your waitlisted {provider} reservation has been confirmed, {name}!")
        }
        AlertKind::Paid => {
            format!("Your {provider} reservation has been paid successfully, {name}.")
        }
        AlertKind::PayFailed => format!(
            "We couldn't process the auto-payment for your {provider} reservation, {name}. Please pay manually before the deadline.",
        ),
        AlertKind::Failed => {
            format!("Sorry {name}, we couldn't secure a seat on your requested {provider} train.")
        }
        AlertKind::Cancelled => {
            format!("Your {provider} reservation task has been cancelled, {name}.")
        }
    };

    // PNR row
    let pnr_row = details.pnr.map(|pnr| {
        let pnr = html_escape(pnr);
        format!(
            r#"<tr class="detail-row">
  <td style="padding:8px 0; border-bottom:1px solid rgba(148,163,184,0.1); font-size:14px; color:#94a3b8; width:120px;">PNR</td>
  <td style="padding:8px 0; border-bottom:1px solid rgba(148,163,184,0.1); font-size:14px; color:#f1f5f9; font-weight:600; font-family:monospace;">{pnr}</td>
</tr>"#
        )
    }).unwrap_or_default();

    // Cost row
    let cost_row = details
        .total_cost
        .map(|cost| {
            let cost = html_escape(cost);
            format!(
                r#"<tr class="detail-row">
  <td style="padding:8px 0; font-size:14px; color:#94a3b8; width:120px;">Total</td>
  <td style="padding:8px 0; font-size:14px; color:#f1f5f9; font-weight:600;">{cost}</td>
</tr>"#
            )
        })
        .unwrap_or_default();

    // CTA button for actionable states
    let cta = match kind {
        AlertKind::PayFailed => format!(
            r#"<table role="presentation" width="100%" cellpadding="0" cellspacing="0">
  <tr>
    <td align="center" style="padding:24px 0 0;">
      <a href="{url}" class="cta-btn" style="background-color:#f59e0b; color:#ffffff !important; border-radius:8px; display:inline-block; font-size:16px; font-weight:600; line-height:1; padding:14px 32px; text-decoration:none !important;">
        Pay Now
      </a>
    </td>
  </tr>
</table>"#
        ),
        AlertKind::Confirmed | AlertKind::WaitingConfirmed | AlertKind::Paid => format!(
            r#"<table role="presentation" width="100%" cellpadding="0" cellspacing="0">
  <tr>
    <td align="center" style="padding:24px 0 0;">
      <a href="{url}" class="cta-btn" style="background-color:#3b82f6; color:#ffffff !important; border-radius:8px; display:inline-block; font-size:16px; font-weight:600; line-height:1; padding:14px 32px; text-decoration:none !important;">
        View Reservation
      </a>
    </td>
  </tr>
</table>"#
        ),
        _ => String::new(),
    };

    let body = format!(
        r##"<h1 style="margin:0 0 8px; font-size:24px; font-weight:700; color:#f1f5f9;">
  {heading}
</h1>
<p style="margin:0 0 24px; font-size:15px; color:#94a3b8; line-height:1.6;">
  {message}
</p>
<!-- Status badge -->
<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="margin-bottom:20px;">
  <tr>
    <td>
      <span class="badge {badge}" style="display:inline-block; padding:4px 12px; border-radius:999px; font-size:12px; font-weight:600; letter-spacing:0.5px; text-transform:uppercase;">{label}</span>
    </td>
  </tr>
</table>
<!-- Train details card -->
<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="background-color:rgba(15,23,42,0.5); border:1px solid rgba(148,163,184,0.1); border-radius:8px; margin-bottom:8px;">
  <tr>
    <td style="padding:16px 20px;">
      <table role="presentation" width="100%" cellpadding="0" cellspacing="0">
        <tr class="detail-row">
          <td style="padding:8px 0; border-bottom:1px solid rgba(148,163,184,0.1); font-size:14px; color:#94a3b8; width:120px;">Provider</td>
          <td style="padding:8px 0; border-bottom:1px solid rgba(148,163,184,0.1); font-size:14px; color:#f1f5f9; font-weight:600;">{provider}</td>
        </tr>
        <tr class="detail-row">
          <td style="padding:8px 0; border-bottom:1px solid rgba(148,163,184,0.1); font-size:14px; color:#94a3b8; width:120px;">Train</td>
          <td style="padding:8px 0; border-bottom:1px solid rgba(148,163,184,0.1); font-size:14px; color:#f1f5f9; font-weight:600;">{train_no}</td>
        </tr>
        <tr class="detail-row">
          <td style="padding:8px 0; border-bottom:1px solid rgba(148,163,184,0.1); font-size:14px; color:#94a3b8; width:120px;">Route</td>
          <td style="padding:8px 0; border-bottom:1px solid rgba(148,163,184,0.1); font-size:14px; color:#f1f5f9; font-weight:600;">{dep} &#10140; {arr}</td>
        </tr>
        <tr class="detail-row">
          <td style="padding:8px 0; border-bottom:1px solid rgba(148,163,184,0.1); font-size:14px; color:#94a3b8; width:120px;">Date</td>
          <td style="padding:8px 0; border-bottom:1px solid rgba(148,163,184,0.1); font-size:14px; color:#f1f5f9; font-weight:600;">{date}</td>
        </tr>
        <tr class="detail-row">
          <td style="padding:8px 0; border-bottom:1px solid rgba(148,163,184,0.1); font-size:14px; color:#94a3b8; width:120px;">Departure</td>
          <td style="padding:8px 0; border-bottom:1px solid rgba(148,163,184,0.1); font-size:14px; color:#f1f5f9; font-weight:600;">{time}</td>
        </tr>
        {pnr_row}
        {cost_row}
      </table>
    </td>
  </tr>
</table>
{cta}"##
    );

    let preheader = format!("{} - {} {} {}", kind.label_ko(), provider, train_no, dep);
    let html = base::layout(&preheader, &body);

    (subject, html)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_details() -> ReservationDetails<'static> {
        ReservationDetails {
            provider: "SRT",
            train_number: "305",
            dep_station: "수서",
            arr_station: "부산",
            dep_date: "20260315",
            dep_time: "090000",
            pnr: Some("12345678"),
            total_cost: Some("59,800원"),
        }
    }

    #[test]
    fn confirmed_email() {
        let (subject, html) = render(
            "Jason",
            AlertKind::Confirmed,
            &sample_details(),
            "https://bominal.com",
        );
        assert!(subject.contains("예약 확정"));
        assert!(html.contains("Reservation Confirmed"));
        assert!(html.contains("305"));
        assert!(html.contains("수서"));
        assert!(html.contains("12345678"));
        assert!(html.contains("View Reservation"));
    }

    #[test]
    fn pay_failed_shows_pay_now() {
        let (_, html) = render(
            "User",
            AlertKind::PayFailed,
            &sample_details(),
            "https://bominal.com",
        );
        assert!(html.contains("Pay Now"));
        assert!(html.contains("pay manually"));
    }

    #[test]
    fn failed_no_cta() {
        let (_, html) = render(
            "User",
            AlertKind::Failed,
            &sample_details(),
            "https://bominal.com",
        );
        assert!(!html.contains("View Reservation"));
        assert!(!html.contains("Pay Now"));
    }

    #[test]
    fn no_pnr_omits_row() {
        let details = ReservationDetails {
            pnr: None,
            total_cost: None,
            ..sample_details()
        };
        let (_, html) = render("User", AlertKind::Failed, &details, "https://bominal.com");
        assert!(!html.contains("PNR"));
        assert!(!html.contains("Total"));
    }

    #[test]
    fn all_alert_kinds_render() {
        let details = sample_details();
        for kind in [
            AlertKind::Confirmed,
            AlertKind::Paid,
            AlertKind::PayFailed,
            AlertKind::Failed,
            AlertKind::Cancelled,
            AlertKind::WaitingConfirmed,
        ] {
            let (subject, html) = render("User", kind, &details, "https://bominal.com");
            assert!(!subject.is_empty());
            assert!(html.contains("Bominal"));
        }
    }
}
