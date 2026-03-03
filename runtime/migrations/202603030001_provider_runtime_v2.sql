create table if not exists provider_auth_secrets (
    id bigserial primary key,
    provider text not null,
    subject_ref text not null,
    credential_kind text not null,
    secret_envelope_ciphertext bytea not null,
    secret_envelope_dek_ciphertext bytea not null,
    secret_envelope_kek_version integer not null,
    secret_envelope_aad_scope text not null,
    secret_envelope_aad_subject text not null,
    secret_envelope_aad_hash bytea not null,
    redacted_metadata jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    rotated_at timestamptz,
    revoked_at timestamptz,
    constraint ux_provider_auth_secrets_scope unique (provider, subject_ref, credential_kind),
    constraint chk_provider_auth_secrets_kek_version check (secret_envelope_kek_version > 0)
);

create index if not exists ix_provider_auth_secrets_lookup
    on provider_auth_secrets (provider, subject_ref, credential_kind);

create index if not exists ix_provider_auth_secrets_active_lookup
    on provider_auth_secrets (provider, subject_ref, updated_at desc)
    where revoked_at is null;

create table if not exists payment_method_secrets (
    id bigserial primary key,
    provider text not null,
    owner_ref text not null,
    payment_method_ref text not null,
    method_kind text not null default 'card',
    card_brand text,
    card_last4 text,
    card_exp_month smallint,
    card_exp_year integer,
    payment_payload_envelope_ciphertext bytea not null,
    payment_payload_envelope_dek_ciphertext bytea not null,
    payment_payload_envelope_kek_version integer not null,
    payment_payload_envelope_aad_scope text not null,
    payment_payload_envelope_aad_subject text not null,
    payment_payload_envelope_aad_hash bytea not null,
    redacted_metadata jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    revoked_at timestamptz,
    constraint ux_payment_method_secrets_scope unique (provider, owner_ref, payment_method_ref),
    constraint chk_payment_method_secrets_last4 check (
        card_last4 is null
        or card_last4 ~ '^[0-9]{4}$'
    ),
    constraint chk_payment_method_secrets_exp_month check (
        card_exp_month is null
        or card_exp_month between 1 and 12
    ),
    constraint chk_payment_method_secrets_exp_year check (
        card_exp_year is null
        or card_exp_year between 2000 and 9999
    ),
    constraint chk_payment_method_secrets_kek_version check (payment_payload_envelope_kek_version > 0)
);

create index if not exists ix_payment_method_secrets_lookup
    on payment_method_secrets (provider, owner_ref, payment_method_ref);

create index if not exists ix_payment_method_secrets_active_lookup
    on payment_method_secrets (provider, owner_ref, updated_at desc)
    where revoked_at is null;

create table if not exists provider_contract_ledger (
    id bigserial primary key,
    job_id text references runtime_jobs (job_id) on delete set null,
    provider text not null,
    operation text not null,
    request_idempotency_key text,
    request_fingerprint text,
    request_redacted jsonb not null default '{}'::jsonb,
    response_redacted jsonb not null default '{}'::jsonb,
    response_status_code integer,
    outcome text not null,
    error_class text,
    error_message_redacted text,
    created_at timestamptz not null default now(),
    constraint chk_provider_contract_ledger_outcome
        check (outcome in ('success', 'retryable_error', 'terminal_error')),
    constraint chk_provider_contract_ledger_response_status
        check (
            response_status_code is null
            or response_status_code between 100 and 599
        )
);

create index if not exists ix_provider_contract_ledger_job_id_created_at
    on provider_contract_ledger (job_id, created_at desc);

create index if not exists ix_provider_contract_ledger_provider_operation_created_at
    on provider_contract_ledger (provider, operation, created_at desc);

create unique index if not exists ux_provider_contract_ledger_provider_fingerprint
    on provider_contract_ledger (provider, request_fingerprint)
    where request_fingerprint is not null;

create or replace function provider_contract_ledger_prevent_mutation()
returns trigger
language plpgsql
as $$
begin
    raise exception 'provider_contract_ledger is append-only';
end;
$$;

drop trigger if exists tr_provider_contract_ledger_immutable
    on provider_contract_ledger;

create trigger tr_provider_contract_ledger_immutable
    before update or delete on provider_contract_ledger
    for each row
    execute function provider_contract_ledger_prevent_mutation();

create table if not exists srt_reservation_projection (
    id bigserial primary key,
    provider text not null default 'srt',
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
    constraint ux_srt_reservation_projection_provider_reservation unique (provider, reservation_id),
    constraint chk_srt_reservation_projection_status check (
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
    constraint chk_srt_reservation_projection_passenger_count check (passenger_count > 0)
);

create index if not exists ix_srt_reservation_projection_user_status_updated_at
    on srt_reservation_projection (user_ref, reservation_status, updated_at desc);

create index if not exists ix_srt_reservation_projection_provider_updated_at
    on srt_reservation_projection (provider_updated_at desc nulls last);

create table if not exists runtime_job_dead_letters (
    id bigserial primary key,
    job_id text not null references runtime_jobs (job_id) on delete cascade,
    dead_letter_key text,
    failure_kind text not null,
    error_message_redacted text,
    error_context_redacted jsonb not null default '{}'::jsonb,
    payload_redacted jsonb not null default '{}'::jsonb,
    attempt_count integer not null,
    created_at timestamptz not null default now(),
    constraint ux_runtime_job_dead_letters_job unique (job_id),
    constraint chk_runtime_job_dead_letters_attempt_count check (attempt_count >= 0)
);

create unique index if not exists ux_runtime_job_dead_letters_dead_letter_key
    on runtime_job_dead_letters (dead_letter_key)
    where dead_letter_key is not null;

create index if not exists ix_runtime_job_dead_letters_created_at
    on runtime_job_dead_letters (created_at desc);

create index if not exists ix_runtime_job_dead_letters_failure_kind_created_at
    on runtime_job_dead_letters (failure_kind, created_at desc);

alter table runtime_jobs
    add column if not exists idempotency_scope text,
    add column if not exists idempotency_key text,
    add column if not exists max_attempts integer not null default 5;

alter table runtime_jobs
    drop constraint if exists chk_runtime_jobs_status;

alter table runtime_jobs
    add constraint chk_runtime_jobs_status
        check (status in ('queued', 'running', 'completed', 'failed', 'dead_lettered'));

alter table runtime_jobs
    drop constraint if exists chk_runtime_jobs_attempt_count_non_negative;

alter table runtime_jobs
    add constraint chk_runtime_jobs_attempt_count_non_negative
        check (attempt_count >= 0);

alter table runtime_jobs
    drop constraint if exists chk_runtime_jobs_max_attempts_positive;

alter table runtime_jobs
    add constraint chk_runtime_jobs_max_attempts_positive
        check (max_attempts > 0);

alter table runtime_jobs
    drop constraint if exists chk_runtime_jobs_idempotency_pair;

alter table runtime_jobs
    add constraint chk_runtime_jobs_idempotency_pair
        check (
            (idempotency_scope is null and idempotency_key is null)
            or (idempotency_scope is not null and idempotency_key is not null)
        );

create unique index if not exists ux_runtime_jobs_idempotency_scope_key
    on runtime_jobs (idempotency_scope, idempotency_key)
    where idempotency_scope is not null
      and idempotency_key is not null;

create index if not exists ix_runtime_jobs_ready_scan
    on runtime_jobs (status, next_run_at, created_at)
    where status = 'queued';

create index if not exists ix_runtime_jobs_status_updated_at
    on runtime_jobs (status, updated_at desc);

alter table runtime_job_leases
    drop constraint if exists chk_runtime_job_leases_expiry;

alter table runtime_job_leases
    add constraint chk_runtime_job_leases_expiry
        check (expires_at > acquired_at);

create index if not exists ix_runtime_job_leases_expiry_scan
    on runtime_job_leases (expires_at, job_id);

create index if not exists ix_runtime_job_events_job_id_event_type_created_at
    on runtime_job_events (job_id, event_type, created_at desc);

create or replace function runtime_jobs_validate_status_transition()
returns trigger
language plpgsql
as $$
begin
    if new.status = old.status then
        return new;
    end if;

    if old.status = 'queued' and new.status in ('running', 'failed', 'dead_lettered') then
        return new;
    end if;

    if old.status = 'running' and new.status in ('queued', 'completed', 'failed', 'dead_lettered') then
        return new;
    end if;

    if old.status = 'failed' and new.status in ('queued', 'dead_lettered') then
        return new;
    end if;

    raise exception 'runtime_jobs status transition from % to % is not allowed', old.status, new.status;
end;
$$;

drop trigger if exists tr_runtime_jobs_status_transition_guard
    on runtime_jobs;

create trigger tr_runtime_jobs_status_transition_guard
    before update of status on runtime_jobs
    for each row
    execute function runtime_jobs_validate_status_transition();
