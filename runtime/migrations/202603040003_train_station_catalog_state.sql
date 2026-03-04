create table if not exists train_station_catalog_state (
    id smallint primary key,
    snapshot_sha256 text not null,
    snapshot_version integer not null,
    applied_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    constraint chk_train_station_catalog_state_singleton check (id = 1),
    constraint chk_train_station_catalog_state_version_positive check (snapshot_version > 0)
);
