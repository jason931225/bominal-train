create table if not exists train_tasks (
    id bigserial primary key,
    task_id text not null unique,
    user_id text not null,
    provider text not null,
    state_code smallint not null,
    state_name text not null,
    state_reason_code smallint,
    state_reason_name text,
    dep_station_code text not null,
    arr_station_code text not null,
    dep_date text not null,
    dep_time text not null,
    departure_at timestamptz not null,
    passengers_json jsonb not null default '[]'::jsonb,
    seat_preference text not null default 'general_first',
    auto_pay boolean not null default false,
    notify_email boolean not null default false,
    retry_on_expiry boolean not null default false,
    retry_count integer not null default 0,
    max_retry_count integer not null default 3,
    payment_method_ref text,
    last_tried_at timestamptz,
    last_search_tried_at timestamptz,
    last_reservation_tried_at timestamptz,
    last_payment_tried_at timestamptz,
    pay_by_at timestamptz,
    next_poll_at timestamptz not null default now(),
    selected_train_json jsonb,
    reservation_json jsonb,
    terminal_payload jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    completed_at timestamptz,
    constraint chk_train_tasks_provider check (provider in ('srt', 'ktx')),
    constraint chk_train_tasks_state_code check (state_code between 0 and 7),
    constraint chk_train_tasks_state_name check (
        state_name in (
            'queued',
            'running',
            'paused',
            'awaiting_payment',
            'completed',
            'failed',
            'cancelled',
            'expired'
        )
    ),
    constraint chk_train_tasks_retry_count check (retry_count >= 0),
    constraint chk_train_tasks_max_retry_count check (max_retry_count > 0)
);

create index if not exists ix_train_tasks_user_created_at
    on train_tasks (user_id, created_at desc);

create index if not exists ix_train_tasks_state_next_poll
    on train_tasks (state_code, next_poll_at, updated_at desc);

create index if not exists ix_train_tasks_provider_state_next_poll
    on train_tasks (provider, state_code, next_poll_at);

create table if not exists train_task_candidates (
    id bigserial primary key,
    task_id text not null references train_tasks (task_id) on delete cascade,
    priority_index integer not null,
    provider text not null,
    candidate_json jsonb not null,
    departs_at timestamptz,
    created_at timestamptz not null default now(),
    constraint ux_train_task_candidates_priority unique (task_id, priority_index),
    constraint chk_train_task_candidates_priority check (priority_index > 0),
    constraint chk_train_task_candidates_provider check (provider in ('srt', 'ktx'))
);

create index if not exists ix_train_task_candidates_task_priority
    on train_task_candidates (task_id, priority_index);

create table if not exists train_task_events (
    id bigserial primary key,
    task_id text not null references train_tasks (task_id) on delete cascade,
    event_type text not null,
    event_payload jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now()
);

create index if not exists ix_train_task_events_task_id_id
    on train_task_events (task_id, id asc);

create table if not exists worker_scheduled_tasks (
    id bigserial primary key,
    task_key text not null unique,
    task_type text not null,
    payload jsonb not null default '{}'::jsonb,
    status text not null default 'queued',
    run_at timestamptz not null,
    last_run_at timestamptz,
    last_error text,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    constraint chk_worker_scheduled_tasks_status check (status in ('queued', 'running'))
);

create index if not exists ix_worker_scheduled_tasks_due
    on worker_scheduled_tasks (status, run_at);

insert into worker_scheduled_tasks (
    task_key,
    task_type,
    payload,
    status,
    run_at,
    created_at,
    updated_at
)
values (
    'provider_auth_recheck_daily',
    'provider_auth_recheck_daily',
    '{}'::jsonb,
    'queued',
    (date_trunc('day', now() at time zone 'utc') at time zone 'utc') + interval '1 day',
    now(),
    now()
)
on conflict (task_key) do nothing;
