-- Index for passkey challenge cleanup query that deletes expired rows.
CREATE INDEX IF NOT EXISTS idx_passkey_challenges_created_at
    ON passkey_challenges(created_at);
