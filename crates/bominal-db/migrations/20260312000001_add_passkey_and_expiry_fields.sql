-- Add encrypted_expiry_yymm column for SRT (which expects YYMM format)
ALTER TABLE payment_cards
    ADD COLUMN IF NOT EXISTS encrypted_expiry_yymm TEXT;

-- Extend passkey_credentials for full WebAuthn ceremony support and add
-- the passkey_challenges table for challenge storage.

-- Convert credential_id and public_key from BYTEA to TEXT for simpler
-- base64url-encoded storage coming from the browser API.
ALTER TABLE passkey_credentials
    ALTER COLUMN credential_id TYPE TEXT USING encode(credential_id, 'base64'),
    ALTER COLUMN public_key TYPE TEXT USING encode(public_key, 'base64');

-- Add label, aaguid, and transports columns.
ALTER TABLE passkey_credentials
    ADD COLUMN IF NOT EXISTS label TEXT NOT NULL DEFAULT 'My Passkey',
    ADD COLUMN IF NOT EXISTS aaguid BYTEA,
    ADD COLUMN IF NOT EXISTS transports TEXT[] NOT NULL DEFAULT '{}';

-- Drop the counter column (signature count tracking is handled application-side).
ALTER TABLE passkey_credentials
    DROP COLUMN IF EXISTS counter;

-- Passkey challenges — short-lived rows consumed during register/login ceremonies.
CREATE TABLE IF NOT EXISTS passkey_challenges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    challenge_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_passkey_challenges_challenge_id
    ON passkey_challenges(challenge_id);
