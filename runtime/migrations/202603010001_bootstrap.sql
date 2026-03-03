-- Minimal bootstrap table for Rust-side migration smoke checks.
create table if not exists runtime_migration_events (
    id uuid primary key,
    event_type text not null,
    payload jsonb not null,
    created_at timestamptz not null default now()
);
