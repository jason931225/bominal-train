create table if not exists supabase_auth_user_sync (
    user_id text primary key,
    email text,
    last_event_type text not null,
    last_synced_at timestamptz not null default now()
);

create index if not exists ix_supabase_auth_user_sync_last_synced_at
    on supabase_auth_user_sync (last_synced_at desc);
