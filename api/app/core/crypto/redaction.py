from __future__ import annotations

from typing import Any

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
}


def redact_sensitive(data: Any) -> Any:
    if isinstance(data, dict):
        redacted: dict[str, Any] = {}
        for key, value in data.items():
            lowered = key.lower()
            if lowered in SENSITIVE_KEYS or lowered.endswith("_token") or lowered.endswith("_password"):
                redacted[key] = "[REDACTED]"
            else:
                redacted[key] = redact_sensitive(value)
        return redacted

    if isinstance(data, list):
        return [redact_sensitive(item) for item in data]

    return data
