create table if not exists auth_sessions (
    session_id_hash text primary key,
    user_id uuid references auth_users (id) on delete cascade,
    email text not null,
    role text not null default 'user',
    issued_at timestamptz not null default now(),
    last_seen_at timestamptz not null default now(),
    step_up_verified_at timestamptz,
    revoked_at timestamptz,
    revoked_reason text,
    ip inet,
    user_agent text,
    constraint chk_auth_sessions_role check (role in ('admin', 'operator', 'viewer', 'user'))
);

create index if not exists ix_auth_sessions_user_id_last_seen
    on auth_sessions (user_id, last_seen_at desc);

create index if not exists ix_auth_sessions_revoked_at
    on auth_sessions (revoked_at);

create table if not exists auth_user_role_bindings (
    user_id uuid primary key references auth_users (id) on delete cascade,
    role text not null default 'user',
    access_enabled boolean not null default true,
    updated_at timestamptz not null default now(),
    updated_by uuid references auth_users (id) on delete set null,
    constraint chk_auth_user_role_bindings_role check (role in ('admin', 'operator', 'viewer', 'user'))
);

create index if not exists ix_auth_user_role_bindings_role
    on auth_user_role_bindings (role);

create table if not exists admin_runtime_flags (
    flag text primary key,
    enabled boolean not null default false,
    reason text not null,
    updated_at timestamptz not null default now(),
    updated_by uuid references auth_users (id) on delete set null
);

create table if not exists admin_incidents (
    id uuid primary key,
    title text not null,
    severity text not null,
    status text not null,
    summary text,
    context_json jsonb not null default '{}'::jsonb,
    opened_at timestamptz not null default now(),
    resolved_at timestamptz,
    created_by uuid references auth_users (id) on delete set null,
    constraint chk_admin_incidents_severity check (severity in ('sev1', 'sev2', 'sev3', 'sev4')),
    constraint chk_admin_incidents_status check (status in ('open', 'monitoring', 'resolved'))
);

create index if not exists ix_admin_incidents_status_opened_at
    on admin_incidents (status, opened_at desc);

create table if not exists admin_audit_log (
    id uuid primary key,
    actor_user_id uuid references auth_users (id) on delete set null,
    actor_email text not null,
    action text not null,
    target_type text not null,
    target_id text not null,
    reason text not null,
    request_id text not null,
    metadata jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now()
);

create index if not exists ix_admin_audit_log_created_at
    on admin_audit_log (created_at desc);

create index if not exists ix_admin_audit_log_actor_created_at
    on admin_audit_log (actor_user_id, created_at desc);

create index if not exists ix_admin_audit_log_target_created_at
    on admin_audit_log (target_type, target_id, created_at desc);

create or replace function admin_audit_log_prevent_mutation()
returns trigger
language plpgsql
as $$
begin
    raise exception 'admin_audit_log is append-only';
end;
$$;

drop trigger if exists tr_admin_audit_log_immutable on admin_audit_log;

create trigger tr_admin_audit_log_immutable
    before update or delete on admin_audit_log
    for each row
    execute function admin_audit_log_prevent_mutation();
