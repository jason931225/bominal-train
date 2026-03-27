//! Shared formatting helpers used across the migrated Leptos app.

use bominal_domain::task::TaskStatus;
use chrono::{Datelike, NaiveDate};

/// "0830" -> "08:30"
pub fn format_time(raw: &str) -> String {
    if raw.len() < 4 {
        return raw.to_string();
    }

    format!("{}:{}", &raw[..2], &raw[2..4])
}

/// "20260312" -> "2026-03-12"
pub fn format_date(raw: &str) -> String {
    if raw.len() < 8 {
        return raw.to_string();
    }

    format!("{}-{}-{}", &raw[..4], &raw[4..6], &raw[6..8])
}

/// "59800" -> "59,800"
pub fn format_cost(raw: &str) -> String {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    let n: u64 = digits.parse().unwrap_or(0);

    if n == 0 {
        return "0".to_string();
    }

    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }

    result.chars().rev().collect()
}

/// Map task status to the UI status-chip variant used by the frontend.
pub fn status_variant(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Queued => "queued",
        TaskStatus::Running => "running",
        TaskStatus::Idle => "warning",
        TaskStatus::AwaitingPayment => "warning",
        TaskStatus::Confirmed => "success",
        TaskStatus::Failed => "error",
        TaskStatus::Cancelled => "neutral",
    }
}

/// Convert a time slot (0-47) to "HHMMSS" for API submission.
pub fn slot_to_time_string(slot: u32) -> String {
    let hours = slot / 2;
    let minutes = if slot.is_multiple_of(2) { 0 } else { 30 };
    format!("{hours:02}{minutes:02}00")
}

/// Convert a time slot (0-47) to "HH:MM" for display.
pub fn format_time_slot(slot: u32) -> String {
    let hours = slot / 2;
    let minutes = if slot.is_multiple_of(2) { "00" } else { "30" };
    format!("{hours:02}:{minutes}")
}

/// "20260326" -> "3월 26일 (목)"
pub fn format_display_date(raw: &str) -> String {
    if raw.len() < 8 {
        return raw.to_string();
    }

    let Ok(date) = NaiveDate::parse_from_str(raw, "%Y%m%d") else {
        return raw.to_string();
    };

    let weekday = match date.weekday().num_days_from_sunday() {
        0 => "일",
        1 => "월",
        2 => "화",
        3 => "수",
        4 => "목",
        5 => "금",
        _ => "토",
    };

    format!("{}월 {}일 ({weekday})", date.month(), date.day())
}

/// i18n key for a given task status.
pub fn task_status_i18n_key(status: TaskStatus) -> &'static str {
    status.i18n_key()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bominal_domain::task::TaskStatus;

    #[test]
    fn time_is_formatted() {
        assert_eq!(format_time("0830"), "08:30");
        assert_eq!(format_time("930"), "930");
    }

    #[test]
    fn date_is_formatted() {
        assert_eq!(format_date("20260312"), "2026-03-12");
        assert_eq!(format_date("2026"), "2026");
    }

    #[test]
    fn cost_is_formatted() {
        assert_eq!(format_cost("59800"), "59,800");
        assert_eq!(format_cost("0"), "0");
    }

    #[test]
    fn status_variant_matches_frontend_contract() {
        assert_eq!(status_variant(TaskStatus::AwaitingPayment), "warning");
        assert_eq!(status_variant(TaskStatus::Cancelled), "neutral");
    }

    #[test]
    fn slot_conversion_matches_existing_frontend_logic() {
        assert_eq!(slot_to_time_string(16), "080000");
        assert_eq!(format_time_slot(47), "23:30");
    }

    #[test]
    fn display_date_uses_korean_weekday_format() {
        assert_eq!(format_display_date("20260326"), "3월 26일 (목)");
    }

    #[test]
    fn task_status_key_reuses_domain_mapping() {
        assert_eq!(task_status_i18n_key(TaskStatus::Queued), "task.queued");
    }
}
