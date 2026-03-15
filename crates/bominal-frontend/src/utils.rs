//! Shared formatting and display utilities used across frontend pages.

use crate::api::tasks::TaskStatus;

/// Format a raw time string (e.g. "0830") into "08:30".
pub fn format_time(raw: &str) -> String {
    if raw.len() >= 4 {
        format!("{}:{}", &raw[..2], &raw[2..4])
    } else {
        raw.to_string()
    }
}

/// Format a raw date string (e.g. "20260312") into "2026-03-12".
pub fn format_date(raw: &str) -> String {
    if raw.len() >= 8 {
        format!("{}-{}-{}", &raw[..4], &raw[4..6], &raw[6..8])
    } else {
        raw.to_string()
    }
}

/// Format a cost string with thousands separators (e.g. "59800" -> "59,800").
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

/// Map a task status to a UI variant name for `StatusChip`.
pub fn status_variant(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Queued => "queued",
        TaskStatus::Running => "running",
        TaskStatus::Idle => "warning",
        TaskStatus::AwaitingPayment => "info",
        TaskStatus::Confirmed => "success",
        TaskStatus::Failed => "error",
        TaskStatus::Cancelled => "warning",
    }
}

/// Convert a time slot (0-47) to "HHMMSS" format for the server.
/// Slot 0 = "000000", slot 1 = "003000", slot 16 = "080000", slot 47 = "233000".
pub fn slot_to_time_string(slot: u32) -> String {
    let hours = slot / 2;
    let minutes = if slot.is_multiple_of(2) { 0 } else { 30 };
    format!("{hours:02}{minutes:02}00")
}

/// Format a time slot (0-47) for display as "HH:MM".
/// Slot 0 = "00:00", slot 16 = "08:00", slot 47 = "23:30".
pub fn format_time_slot(slot: u32) -> String {
    let hours = slot / 2;
    let minutes = if slot.is_multiple_of(2) { "00" } else { "30" };
    format!("{hours:02}:{minutes}")
}
