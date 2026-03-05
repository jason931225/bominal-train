use chrono::{DateTime, Utc};

use crate::AppState;

use super::{AdminServiceError, ObservabilityEventRecord, ObservabilityTimeseriesPoint};

pub(crate) async fn list_observability_events(
    state: &AppState,
) -> Result<Vec<ObservabilityEventRecord>, AdminServiceError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let rows = sqlx::query_as::<_, (DateTime<Utc>, String, String, Option<String>, serde_json::Value)>(
        "select occurred_at, source, event_type, target_id, detail
         from (
           select e.created_at as occurred_at, 'runtime_job_events'::text as source, e.event_type, e.job_id as target_id, e.event_payload as detail
           from runtime_job_events e
           union all
           select l.created_at as occurred_at, 'provider_contract_ledger'::text as source, l.operation as event_type, l.job_id as target_id,
                  jsonb_build_object('provider', l.provider, 'outcome', l.outcome, 'status', l.response_status_code) as detail
           from provider_contract_ledger l
           union all
           select a.created_at as occurred_at, 'auth_audit_log'::text as source, a.event_type, coalesce(a.user_id::text, 'unknown') as target_id,
                  coalesce(a.metadata, '{}'::jsonb) as detail
           from auth_audit_log a
           union all
           select aa.created_at as occurred_at, 'admin_audit_log'::text as source, aa.action as event_type, aa.target_id,
                  coalesce(aa.metadata, '{}'::jsonb) as detail
           from admin_audit_log aa
         ) unified
         order by occurred_at desc
         limit 400",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(rows
        .into_iter()
        .map(|row| ObservabilityEventRecord {
            occurred_at: row.0,
            source: row.1,
            event_type: row.2,
            target_id: row.3,
            detail: row.4,
        })
        .collect())
}

pub(crate) async fn list_observability_timeseries(
    state: &AppState,
    window_minutes: Option<i64>,
) -> Result<Vec<ObservabilityTimeseriesPoint>, AdminServiceError> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or(AdminServiceError::ServiceUnavailable(
            "database unavailable",
        ))?;
    let minutes = window_minutes.unwrap_or(240).clamp(15, 1440);
    let rows = sqlx::query_as::<_, (DateTime<Utc>, i64, i64, i64, i64, i64, i64)>(
        "with bounds as (
           select now() - ($1::int * interval '1 minute') as since_at
         ),
         events as (
           select date_trunc('minute', e.created_at) as bucket,
                  1::bigint as total_events,
                  case
                    when lower(e.event_type) like '%fail%' then 1::bigint
                    when lower(e.event_type) like '%error%' then 1::bigint
                    when lower(e.event_type) in ('cancelled', 'dead_lettered') then 1::bigint
                    else 0::bigint
                  end as error_events,
                  1::bigint as runtime_events,
                  0::bigint as provider_events,
                  0::bigint as auth_events,
                  0::bigint as admin_events
             from runtime_job_events e, bounds b
            where e.created_at >= b.since_at
           union all
           select date_trunc('minute', l.created_at) as bucket,
                  1::bigint as total_events,
                  case
                    when lower(coalesce(l.outcome, '')) in ('failed', 'error', 'timeout', 'rejected') then 1::bigint
                    else 0::bigint
                  end as error_events,
                  0::bigint as runtime_events,
                  1::bigint as provider_events,
                  0::bigint as auth_events,
                  0::bigint as admin_events
             from provider_contract_ledger l, bounds b
            where l.created_at >= b.since_at
           union all
           select date_trunc('minute', a.created_at) as bucket,
                  1::bigint as total_events,
                  0::bigint as error_events,
                  0::bigint as runtime_events,
                  0::bigint as provider_events,
                  1::bigint as auth_events,
                  0::bigint as admin_events
             from auth_audit_log a, bounds b
            where a.created_at >= b.since_at
           union all
           select date_trunc('minute', aa.created_at) as bucket,
                  1::bigint as total_events,
                  0::bigint as error_events,
                  0::bigint as runtime_events,
                  0::bigint as provider_events,
                  0::bigint as auth_events,
                  1::bigint as admin_events
             from admin_audit_log aa, bounds b
            where aa.created_at >= b.since_at
         )
         select bucket,
                sum(total_events) as total_events,
                sum(error_events) as error_events,
                sum(runtime_events) as runtime_events,
                sum(provider_events) as provider_events,
                sum(auth_events) as auth_events,
                sum(admin_events) as admin_events
           from events
       group by bucket
       order by bucket asc",
    )
    .bind(minutes as i32)
    .fetch_all(pool)
    .await
    .map_err(|_| AdminServiceError::Internal)?;
    Ok(rows
        .into_iter()
        .map(|row| ObservabilityTimeseriesPoint {
            bucket: row.0,
            total_events: row.1,
            error_events: row.2,
            runtime_events: row.3,
            provider_events: row.4,
            auth_events: row.5,
            admin_events: row.6,
        })
        .collect())
}
