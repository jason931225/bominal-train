use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::AppState;

use super::{AdminServiceError, IncidentRecord};

fn clamp_limit(limit: Option<usize>, default_value: usize, max_value: usize) -> i64 {
    let value = limit.unwrap_or(default_value);
    value.clamp(1, max_value) as i64
}

fn normalize_incident_severity(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "sev1" => Some("sev1"),
        "sev2" => Some("sev2"),
        "sev3" => Some("sev3"),
        "sev4" => Some("sev4"),
        _ => None,
    }
}

fn normalize_incident_status(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "open" => Some("open"),
        "monitoring" => Some("monitoring"),
        "resolved" => Some("resolved"),
        _ => None,
    }
}

pub(crate) async fn list_incidents(
    state: &AppState,
    status: Option<&str>,
    severity: Option<&str>,
    limit: Option<usize>,
) -> Result<Vec<IncidentRecord>, AdminServiceError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let status_filter = match status {
        Some(value) if !value.trim().is_empty() => Some(
            normalize_incident_status(value)
                .ok_or(AdminServiceError::InvalidRequest("invalid incident status"))?
                .to_string(),
        ),
        _ => None,
    };
    let severity_filter = match severity {
        Some(value) if !value.trim().is_empty() => Some(
            normalize_incident_severity(value)
                .ok_or(AdminServiceError::InvalidRequest(
                    "invalid incident severity",
                ))?
                .to_string(),
        ),
        _ => None,
    };
    let rows = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            String,
            String,
            Option<String>,
            serde_json::Value,
            DateTime<Utc>,
            Option<DateTime<Utc>>,
            Option<Uuid>,
        ),
    >(
        "select id, title, severity, status, summary, context_json, opened_at, resolved_at, created_by
           from admin_incidents
          where ($1::text is null or status = $1::text)
            and ($2::text is null or severity = $2::text)
       order by opened_at desc
          limit $3",
    )
    .bind(status_filter)
    .bind(severity_filter)
    .bind(clamp_limit(limit, 120, 500))
    .fetch_all(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(rows
        .into_iter()
        .map(|row| IncidentRecord {
            id: row.0.to_string(),
            title: row.1,
            severity: row.2,
            status: row.3,
            summary: row.4,
            context: row.5,
            opened_at: row.6,
            resolved_at: row.7,
            created_by: row.8.map(|value| value.to_string()),
        })
        .collect())
}

pub(crate) async fn create_incident(
    state: &AppState,
    title: &str,
    severity: &str,
    summary: Option<&str>,
    context: serde_json::Value,
    created_by: Option<&str>,
) -> Result<IncidentRecord, AdminServiceError> {
    let title = title.trim();
    if title.len() < 3 {
        return Err(AdminServiceError::InvalidRequest(
            "incident title must be at least 3 characters",
        ));
    }
    if title.len() > 140 {
        return Err(AdminServiceError::InvalidRequest("incident title too long"));
    }
    let severity = normalize_incident_severity(severity).ok_or(
        AdminServiceError::InvalidRequest("invalid incident severity"),
    )?;
    let summary = summary
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());
    if let Some(value) = summary.as_ref()
        && value.len() > 600
    {
        return Err(AdminServiceError::InvalidRequest(
            "incident summary too long",
        ));
    }
    let actor_uuid = match created_by {
        Some(value) => Some(
            Uuid::parse_str(value)
                .map_err(|_| AdminServiceError::InvalidRequest("invalid actor user id"))?,
        ),
        None => None,
    };

    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let row = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            String,
            String,
            Option<String>,
            serde_json::Value,
            DateTime<Utc>,
            Option<DateTime<Utc>>,
            Option<Uuid>,
        ),
    >(
        "insert into admin_incidents
         (id, title, severity, status, summary, context_json, opened_at, created_by)
         values ($1, $2, $3, 'open', $4, cast($5 as jsonb), now(), $6)
         returning id, title, severity, status, summary, context_json, opened_at, resolved_at, created_by",
    )
    .bind(Uuid::new_v4())
    .bind(title)
    .bind(severity)
    .bind(summary)
    .bind(context)
    .bind(actor_uuid)
    .fetch_one(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;

    Ok(IncidentRecord {
        id: row.0.to_string(),
        title: row.1,
        severity: row.2,
        status: row.3,
        summary: row.4,
        context: row.5,
        opened_at: row.6,
        resolved_at: row.7,
        created_by: row.8.map(|value| value.to_string()),
    })
}

pub(crate) async fn update_incident_status(
    state: &AppState,
    incident_id: &str,
    status: &str,
    summary: Option<&str>,
) -> Result<IncidentRecord, AdminServiceError> {
    let incident_uuid = Uuid::parse_str(incident_id.trim())
        .map_err(|_| AdminServiceError::InvalidRequest("invalid incident id"))?;
    let status = normalize_incident_status(status)
        .ok_or(AdminServiceError::InvalidRequest("invalid incident status"))?;
    let summary = summary
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());
    if let Some(value) = summary.as_ref()
        && value.len() > 600
    {
        return Err(AdminServiceError::InvalidRequest(
            "incident summary too long",
        ));
    }
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let row = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            String,
            String,
            Option<String>,
            serde_json::Value,
            DateTime<Utc>,
            Option<DateTime<Utc>>,
            Option<Uuid>,
        ),
    >(
        "update admin_incidents
            set status = $2,
                summary = coalesce($3, summary),
                resolved_at = case
                  when $2 = 'resolved' then coalesce(resolved_at, now())
                  else null
                end
          where id = $1
      returning id, title, severity, status, summary, context_json, opened_at, resolved_at, created_by",
    )
    .bind(incident_uuid)
    .bind(status)
    .bind(summary)
    .fetch_optional(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    let Some(row) = row else {
        return Err(AdminServiceError::NotFound("incident not found"));
    };
    Ok(IncidentRecord {
        id: row.0.to_string(),
        title: row.1,
        severity: row.2,
        status: row.3,
        summary: row.4,
        context: row.5,
        opened_at: row.6,
        resolved_at: row.7,
        created_by: row.8.map(|value| value.to_string()),
    })
}
