create extension if not exists citext;

create table if not exists auth_users (
    id uuid primary key,
    email citext not null unique,
    password_hash text not null,
    status text not null default 'active',
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table if not exists auth_invites (
    id uuid primary key,
    email citext not null,
    token_hash text not null unique,
    expires_at timestamptz not null,
    accepted_at timestamptz,
    created_at timestamptz not null default now()
);

create index if not exists ix_auth_invites_email on auth_invites (email);
create index if not exists ix_auth_invites_expires_at on auth_invites (expires_at);

create table if not exists user_passkeys (
    id uuid primary key,
    user_id uuid not null references auth_users(id) on delete cascade,
    webauthn_user_uuid uuid not null,
    credential_id text not null unique,
    passkey jsonb not null,
    friendly_name text,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    last_used_at timestamptz
);

create index if not exists ix_user_passkeys_user_id on user_passkeys (user_id);
create index if not exists ix_user_passkeys_webauthn_user_uuid on user_passkeys (webauthn_user_uuid);
create index if not exists ix_user_passkeys_last_used_at on user_passkeys (last_used_at desc);

create table if not exists auth_audit_log (
    id uuid primary key,
    user_id uuid references auth_users(id) on delete set null,
    event_type text not null,
    ip inet,
    user_agent text,
    metadata jsonb,
    created_at timestamptz not null default now()
);

create index if not exists ix_auth_audit_log_user_id on auth_audit_log (user_id);
create index if not exists ix_auth_audit_log_event_type on auth_audit_log (event_type);
create index if not exists ix_auth_audit_log_created_at on auth_audit_log (created_at desc);
