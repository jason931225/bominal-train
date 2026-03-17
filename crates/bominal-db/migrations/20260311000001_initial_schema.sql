-- Initial Bominal schema
-- Users, provider credentials, payment cards, reservation tasks, sessions

-- Enable UUID generation
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Users
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT UNIQUE NOT NULL,
    display_name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    preferred_locale TEXT NOT NULL DEFAULT 'ko'
        CHECK (preferred_locale IN ('ko', 'en', 'ja')),
    email_verified BOOLEAN NOT NULL DEFAULT false,
    email_verification_token TEXT,
    email_verification_expires_at TIMESTAMPTZ,
    password_reset_token TEXT,
    password_reset_expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Passkey credentials (WebAuthn)
CREATE TABLE passkey_credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    credential_id BYTEA UNIQUE NOT NULL,
    public_key BYTEA NOT NULL,
    counter BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_passkey_credentials_user ON passkey_credentials(user_id);

-- Provider credentials (SRT/KTX login)
CREATE TABLE provider_credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL CHECK (provider IN ('SRT', 'KTX')),
    login_id TEXT NOT NULL,
    encrypted_password TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'unverified'
        CHECK (status IN ('valid', 'invalid', 'unverified', 'disabled')),
    last_verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, provider)
);

CREATE INDEX idx_provider_credentials_user ON provider_credentials(user_id);

-- Payment cards (Evervault-encrypted PAN)
CREATE TABLE payment_cards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    label TEXT NOT NULL DEFAULT 'My Card',
    encrypted_number TEXT NOT NULL,
    encrypted_password TEXT NOT NULL,
    encrypted_birthday TEXT NOT NULL,
    encrypted_expiry TEXT NOT NULL,
    card_type TEXT NOT NULL CHECK (card_type IN ('J', 'S')),
    last_four TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_payment_cards_user ON payment_cards(user_id);

-- Reservation tasks
CREATE TABLE reservation_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL CHECK (provider IN ('SRT', 'KTX')),
    departure_station TEXT NOT NULL,
    arrival_station TEXT NOT NULL,
    travel_date TEXT NOT NULL,
    departure_time TEXT NOT NULL,
    seat_preference TEXT NOT NULL
        CHECK (seat_preference IN ('GeneralFirst', 'SpecialFirst', 'GeneralOnly', 'SpecialOnly')),
    auto_pay BOOLEAN NOT NULL DEFAULT false,
    payment_card_id UUID REFERENCES payment_cards(id),
    notify_enabled BOOLEAN NOT NULL DEFAULT false,
    auto_retry BOOLEAN NOT NULL DEFAULT true,
    status TEXT NOT NULL DEFAULT 'queued'
        CHECK (status IN ('queued', 'running', 'idle', 'awaiting_payment', 'confirmed', 'failed', 'cancelled')),
    reservation_number TEXT,
    reserved_dep_station TEXT,
    reserved_arr_station TEXT,
    reserved_dep_date TEXT,
    reserved_dep_time TEXT,
    reserved_train_number TEXT,
    reserved_total_cost TEXT,
    reservation_is_waiting BOOLEAN,
    started_at TIMESTAMPTZ,
    last_attempt_at TIMESTAMPTZ,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_reservation_tasks_user ON reservation_tasks(user_id);
CREATE INDEX idx_reservation_tasks_status ON reservation_tasks(status);

CREATE TABLE reservation_task_passengers (
    task_id UUID NOT NULL REFERENCES reservation_tasks(id) ON DELETE CASCADE,
    passenger_kind TEXT NOT NULL
        CHECK (passenger_kind IN ('adult', 'child', 'senior', 'severe', 'mild', 'infant', 'merit')),
    passenger_count SMALLINT NOT NULL CHECK (passenger_count > 0),
    PRIMARY KEY (task_id, passenger_kind)
);

CREATE INDEX idx_reservation_task_passengers_task
    ON reservation_task_passengers(task_id);

CREATE TABLE reservation_task_targets (
    task_id UUID NOT NULL REFERENCES reservation_tasks(id) ON DELETE CASCADE,
    ordinal SMALLINT NOT NULL CHECK (ordinal >= 0),
    train_number TEXT NOT NULL,
    dep_time TEXT NOT NULL,
    PRIMARY KEY (task_id, ordinal)
);

CREATE INDEX idx_reservation_task_targets_task
    ON reservation_task_targets(task_id);

-- Sessions
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_sessions_user ON sessions(user_id);
CREATE INDEX idx_sessions_expires ON sessions(expires_at);

-- Token lookup indexes
CREATE INDEX idx_users_email_verification_token ON users(email_verification_token) WHERE email_verification_token IS NOT NULL;
CREATE INDEX idx_users_password_reset_token ON users(password_reset_token) WHERE password_reset_token IS NOT NULL;

-- Row Level Security
ALTER TABLE provider_credentials ENABLE ROW LEVEL SECURITY;
ALTER TABLE payment_cards ENABLE ROW LEVEL SECURITY;
ALTER TABLE reservation_tasks ENABLE ROW LEVEL SECURITY;

-- RLS policies (enforced via app-level SET role)
CREATE POLICY user_own_creds ON provider_credentials
    FOR ALL USING (user_id = current_setting('app.current_user_id', true)::UUID);

CREATE POLICY user_own_cards ON payment_cards
    FOR ALL USING (user_id = current_setting('app.current_user_id', true)::UUID);

CREATE POLICY user_own_tasks ON reservation_tasks
    FOR ALL USING (user_id = current_setting('app.current_user_id', true)::UUID);
