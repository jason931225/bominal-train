from __future__ import annotations

from typing import Any

# Exact key matches (case-insensitive)
SENSITIVE_KEYS = {
    "password",
    "password_hash",
    "token",
    "token_hash",
    "card_number",
    "card_no",
    "cvv",
    "pin2",
    "card_password",
    "card_pw",
    "birth_date",
    "phone",
    "email",
    "secret",
    "api_key",
    "apikey",
    "access_token",
    "refresh_token",
    "private_key",
    "secret_key",
    "auth_token",
    "bearer_token",
    "session_token",
    "ciphertext",
    "wrapped_dek",
    "dek_nonce",
    "nonce",
}

# Key suffix patterns that indicate sensitive data
SENSITIVE_SUFFIXES = (
    "_token",
    "_password",
    "_secret",
    "_credential",
    "_credentials",
    "_key",
    "_hash",
    "_pin",
    "_cvv",
)


def _is_sensitive_key(key: str) -> bool:
    """Check if a key represents sensitive data."""
    lowered = key.lower()
    if lowered in SENSITIVE_KEYS:
        return True
    return any(lowered.endswith(suffix) for suffix in SENSITIVE_SUFFIXES)


def redact_sensitive(data: Any) -> Any:
    """Recursively redact sensitive data from dicts and lists.
    
    Replaces values for keys matching SENSITIVE_KEYS or SENSITIVE_SUFFIXES
    with "[REDACTED]".
    """
    if isinstance(data, dict):
        redacted: dict[str, Any] = {}
        for key, value in data.items():
            if _is_sensitive_key(key):
                redacted[key] = "[REDACTED]"
            else:
                redacted[key] = redact_sensitive(value)
        return redacted

    if isinstance(data, list):
        return [redact_sensitive(item) for item in data]

    return data
