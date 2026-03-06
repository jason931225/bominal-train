create table if not exists train_station_favorites (
    id bigserial primary key,
    user_id text not null,
    station_code text not null,
    position integer not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    constraint ux_train_station_favorites_user_station unique (user_id, station_code),
    constraint ux_train_station_favorites_user_position unique (user_id, position),
    constraint chk_train_station_favorites_position check (position > 0)
);

create index if not exists ix_train_station_favorites_user_position
    on train_station_favorites (user_id, position asc);
