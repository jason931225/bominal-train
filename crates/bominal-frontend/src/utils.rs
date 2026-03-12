//! Shared formatting and display utilities used across frontend pages.

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

/// Map a task status string to a UI variant name for `StatusChip`.
pub fn status_variant(status: &str) -> &'static str {
    match status {
        "queued" => "idle",
        "running" => "running",
        "confirmed" => "success",
        "failed" | "error" => "error",
        "cancelled" => "warning",
        _ => "info",
    }
}
