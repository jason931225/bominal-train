use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{Postgres, postgres::PgArguments, query::Query};

pub const SRT_RESERVATION_PROJECTION_UPSERT_SQL: &str = "insert into srt_reservation_projection (provider, reservation_id, user_ref, reservation_status, train_no, depart_station_code, arrive_station_code, departs_at, arrives_at, passenger_count, seat_class, provider_updated_at, source_ledger_id, redacted_snapshot, updated_at) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, cast($14 as jsonb), $15) on conflict (provider, reservation_id) do update set user_ref = excluded.user_ref, reservation_status = excluded.reservation_status, train_no = excluded.train_no, depart_station_code = excluded.depart_station_code, arrive_station_code = excluded.arrive_station_code, departs_at = excluded.departs_at, arrives_at = excluded.arrives_at, passenger_count = excluded.passenger_count, seat_class = excluded.seat_class, provider_updated_at = excluded.provider_updated_at, source_ledger_id = excluded.source_ledger_id, redacted_snapshot = excluded.redacted_snapshot, updated_at = excluded.updated_at, version = srt_reservation_projection.version + 1 returning id, provider, reservation_id, user_ref, reservation_status, train_no, depart_station_code, arrive_station_code, departs_at, arrives_at, passenger_count, seat_class, provider_updated_at, source_ledger_id, redacted_snapshot::text, version, created_at, updated_at";
pub const SRT_RESERVATION_PROJECTION_SELECT_ONE_SQL: &str = "select id, provider, reservation_id, user_ref, reservation_status, train_no, depart_station_code, arrive_station_code, departs_at, arrives_at, passenger_count, seat_class, provider_updated_at, source_ledger_id, redacted_snapshot::text, version, created_at, updated_at from srt_reservation_projection where provider = $1 and reservation_id = $2";
pub const SRT_RESERVATION_PROJECTION_SELECT_BY_USER_SQL: &str = "select id, provider, reservation_id, user_ref, reservation_status, train_no, depart_station_code, arrive_station_code, departs_at, arrives_at, passenger_count, seat_class, provider_updated_at, source_ledger_id, redacted_snapshot::text, version, created_at, updated_at from srt_reservation_projection where user_ref = $1 order by updated_at desc limit $2";

#[derive(Debug, Clone)]
pub struct UpsertSrtReservationProjectionParams<'a> {
    pub provider: &'a str,
    pub reservation_id: &'a str,
    pub user_ref: &'a str,
    pub reservation_status: &'a str,
    pub train_no: Option<&'a str>,
    pub depart_station_code: Option<&'a str>,
    pub arrive_station_code: Option<&'a str>,
    pub departs_at: Option<DateTime<Utc>>,
    pub arrives_at: Option<DateTime<Utc>>,
    pub passenger_count: i32,
    pub seat_class: Option<&'a str>,
    pub provider_updated_at: Option<DateTime<Utc>>,
    pub source_ledger_id: Option<i64>,
    pub redacted_snapshot: &'a Value,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SrtReservationProjectionRecord {
    pub id: i64,
    pub provider: String,
    pub reservation_id: String,
    pub user_ref: String,
    pub reservation_status: String,
    pub train_no: Option<String>,
    pub depart_station_code: Option<String>,
    pub arrive_station_code: Option<String>,
    pub departs_at: Option<DateTime<Utc>>,
    pub arrives_at: Option<DateTime<Utc>>,
    pub passenger_count: i32,
    pub seat_class: Option<String>,
    pub provider_updated_at: Option<DateTime<Utc>>,
    pub source_ledger_id: Option<i64>,
    pub redacted_snapshot: Value,
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub trait SrtReservationProjectionRepoContract {
    fn upsert_srt_reservation_projection_query<'q>(
        params: &'q UpsertSrtReservationProjectionParams<'q>,
    ) -> Query<'q, Postgres, PgArguments>;

    fn select_srt_reservation_projection_query<'q>(
        provider: &'q str,
        reservation_id: &'q str,
    ) -> Query<'q, Postgres, PgArguments>;

    fn select_srt_reservation_projections_by_user_query<'q>(
        user_ref: &'q str,
        limit: i64,
    ) -> Query<'q, Postgres, PgArguments>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SqlSrtReservationProjectionRepoContract;

impl SrtReservationProjectionRepoContract for SqlSrtReservationProjectionRepoContract {
    fn upsert_srt_reservation_projection_query<'q>(
        params: &'q UpsertSrtReservationProjectionParams<'q>,
    ) -> Query<'q, Postgres, PgArguments> {
        sqlx::query(SRT_RESERVATION_PROJECTION_UPSERT_SQL)
            .bind(params.provider)
            .bind(params.reservation_id)
            .bind(params.user_ref)
            .bind(params.reservation_status)
            .bind(params.train_no)
            .bind(params.depart_station_code)
            .bind(params.arrive_station_code)
            .bind(params.departs_at)
            .bind(params.arrives_at)
            .bind(params.passenger_count)
            .bind(params.seat_class)
            .bind(params.provider_updated_at)
            .bind(params.source_ledger_id)
            .bind(params.redacted_snapshot)
            .bind(params.updated_at)
    }

    fn select_srt_reservation_projection_query<'q>(
        provider: &'q str,
        reservation_id: &'q str,
    ) -> Query<'q, Postgres, PgArguments> {
        sqlx::query(SRT_RESERVATION_PROJECTION_SELECT_ONE_SQL)
            .bind(provider)
            .bind(reservation_id)
    }

    fn select_srt_reservation_projections_by_user_query<'q>(
        user_ref: &'q str,
        limit: i64,
    ) -> Query<'q, Postgres, PgArguments> {
        sqlx::query(SRT_RESERVATION_PROJECTION_SELECT_BY_USER_SQL)
            .bind(user_ref)
            .bind(limit)
    }
}

pub fn upsert_srt_reservation_projection_query<'q>(
    params: &'q UpsertSrtReservationProjectionParams<'q>,
) -> Query<'q, Postgres, PgArguments> {
    <SqlSrtReservationProjectionRepoContract as SrtReservationProjectionRepoContract>::upsert_srt_reservation_projection_query(params)
}

pub fn select_srt_reservation_projection_query<'q>(
    provider: &'q str,
    reservation_id: &'q str,
) -> Query<'q, Postgres, PgArguments> {
    <SqlSrtReservationProjectionRepoContract as SrtReservationProjectionRepoContract>::select_srt_reservation_projection_query(provider, reservation_id)
}

pub fn select_srt_reservation_projections_by_user_query<'q>(
    user_ref: &'q str,
    limit: i64,
) -> Query<'q, Postgres, PgArguments> {
    <SqlSrtReservationProjectionRepoContract as SrtReservationProjectionRepoContract>::select_srt_reservation_projections_by_user_query(user_ref, limit)
}
