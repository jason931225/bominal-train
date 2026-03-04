create table if not exists train_reservation_projection (
    id bigserial primary key,
    provider text not null,
    reservation_id text not null,
    user_ref text not null,
    reservation_status text not null,
    train_no text,
    depart_station_code text,
    arrive_station_code text,
    departs_at timestamptz,
    arrives_at timestamptz,
    passenger_count integer not null default 1,
    seat_class text,
    provider_updated_at timestamptz,
    source_ledger_id bigint references provider_contract_ledger (id) on delete set null,
    redacted_snapshot jsonb not null default '{}'::jsonb,
    version bigint not null default 1,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    constraint ux_train_reservation_projection_provider_reservation unique (provider, reservation_id),
    constraint chk_train_reservation_projection_status check (
        reservation_status in (
            'unknown',
            'reserved',
            'standby',
            'cancelled',
            'refunded',
            'completed',
            'failed'
        )
    ),
    constraint chk_train_reservation_projection_passenger_count check (passenger_count > 0)
);

create index if not exists ix_train_reservation_projection_user_status_updated_at
    on train_reservation_projection (user_ref, reservation_status, updated_at desc);

create index if not exists ix_train_reservation_projection_provider_updated_at
    on train_reservation_projection (provider_updated_at desc nulls last);

drop table if exists srt_reservation_projection cascade;
