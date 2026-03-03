use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MetricsSummary {
    pub(crate) request_total: f64,
    pub(crate) error_total: f64,
    pub(crate) error_rate: f64,
    pub(crate) avg_latency_ms: f64,
    pub(crate) readiness_ok: bool,
    pub(crate) liveness_ok: bool,
    pub(crate) raw_excerpt: Vec<String>,
}

pub(crate) fn summarize_metrics(
    prometheus_text: &str,
    liveness_ok: bool,
    readiness_ok: bool,
) -> MetricsSummary {
    let request_total = sum_metric(prometheus_text, "http_requests_total");
    let error_total = sum_metric(prometheus_text, "http_errors_total");
    let duration_sum = sum_metric(prometheus_text, "http_request_duration_seconds_sum");
    let duration_count = sum_metric(prometheus_text, "http_request_duration_seconds_count");
    let error_rate = if request_total > 0.0 {
        error_total / request_total
    } else {
        0.0
    };
    let avg_latency_ms = if duration_count > 0.0 {
        (duration_sum / duration_count) * 1000.0
    } else {
        0.0
    };
    let raw_excerpt = prometheus_text
        .lines()
        .filter(|line| !line.starts_with('#'))
        .take(12)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    MetricsSummary {
        request_total,
        error_total,
        error_rate,
        avg_latency_ms,
        readiness_ok,
        liveness_ok,
        raw_excerpt,
    }
}

fn sum_metric(text: &str, metric_name: &str) -> f64 {
    let mut total = 0.0_f64;
    for line in text.lines() {
        if line.starts_with(metric_name) {
            let Some(raw_value) = line.split_whitespace().last() else {
                continue;
            };
            if let Ok(value) = raw_value.parse::<f64>() {
                total += value;
            }
        }
    }
    total
}
