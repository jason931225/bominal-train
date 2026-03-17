-- Add serialized webauthn-rs ceremony state to passkey challenges.
ALTER TABLE passkey_challenges
    ADD COLUMN IF NOT EXISTS state TEXT;
