create table if not exists runtime_jobs (
    id bigserial primary key,
    job_id text not null unique,
    status text not null,
    attempt_count integer not null default 0,
    next_run_at timestamptz,
    last_error text,
    processed_at timestamptz,
    payload jsonb not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    constraint chk_runtime_jobs_status
        check (status in ('queued', 'running', 'completed', 'failed'))
);

create index if not exists ix_runtime_jobs_status_next_run_at
    on runtime_jobs (status, next_run_at);

create index if not exists ix_runtime_jobs_next_run_at
    on runtime_jobs (next_run_at);

create table if not exists runtime_job_events (
    id bigserial primary key,
    job_id text not null references runtime_jobs (job_id) on delete cascade,
    event_type text not null,
    event_payload jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now()
);

create index if not exists ix_runtime_job_events_job_id_created_at
    on runtime_job_events (job_id, created_at desc);

create table if not exists runtime_job_leases (
    job_id text primary key references runtime_jobs (job_id) on delete cascade,
    lease_owner text not null,
    lease_token text not null,
    acquired_at timestamptz not null default now(),
    expires_at timestamptz not null
);

create unique index if not exists ux_runtime_job_leases_lease_token
    on runtime_job_leases (lease_token);

create index if not exists ix_runtime_job_leases_expires_at
    on runtime_job_leases (expires_at);
