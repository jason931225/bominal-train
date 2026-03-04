create table if not exists train_station_catalog (
    id bigserial primary key,
    provider text not null,
    station_code text not null,
    station_name_ko text not null,
    station_name_en text,
    line_code text,
    order_index integer not null default 0,
    selected boolean not null default true,
    remark text,
    normalized_name text not null,
    normalized_remark text,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    constraint ux_train_station_catalog_provider_station unique (provider, station_code),
    constraint chk_train_station_catalog_provider check (provider in ('srt', 'ktx'))
);

create index if not exists ix_train_station_catalog_provider_name
    on train_station_catalog (provider, station_name_ko);

create index if not exists ix_train_station_catalog_provider_normalized_name
    on train_station_catalog (provider, normalized_name);

create table if not exists train_search_sessions (
    id bigserial primary key,
    search_id text not null unique,
    user_id text not null,
    dep_station_code text not null,
    arr_station_code text not null,
    dep_date text not null,
    dep_time text not null,
    available_only boolean not null default true,
    passenger_count integer not null default 1,
    providers text[] not null,
    status text not null default 'queued',
    error_message text,
    completed_at timestamptz,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    constraint chk_train_search_sessions_provider_count check (cardinality(providers) > 0),
    constraint chk_train_search_sessions_status check (status in ('queued', 'running', 'completed', 'partial', 'failed')),
    constraint chk_train_search_sessions_passenger_count check (passenger_count > 0)
);

create index if not exists ix_train_search_sessions_user_created_at
    on train_search_sessions (user_id, created_at desc);

create index if not exists ix_train_search_sessions_status_updated_at
    on train_search_sessions (status, updated_at desc);

create table if not exists train_search_session_jobs (
    id bigserial primary key,
    search_id text not null references train_search_sessions (search_id) on delete cascade,
    provider text not null,
    runtime_job_id text not null references runtime_jobs (job_id) on delete cascade,
    status text not null default 'queued',
    error_message text,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    constraint ux_train_search_session_jobs_provider unique (search_id, provider),
    constraint ux_train_search_session_jobs_runtime_job unique (runtime_job_id),
    constraint chk_train_search_session_jobs_provider check (provider in ('srt', 'ktx')),
    constraint chk_train_search_session_jobs_status check (status in ('queued', 'running', 'completed', 'failed', 'dead_lettered'))
);

create index if not exists ix_train_search_session_jobs_search_status
    on train_search_session_jobs (search_id, status, updated_at desc);
