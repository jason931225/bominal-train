from __future__ import annotations

import json
import re
from typing import Any, Mapping, Sequence

# Exact key matches (case-insensitive)
SENSITIVE_KEYS = {
    "password",
    "password_hash",
    "token",
    "token_hash",
    "card_number",
    "card_no",
    "cvv",
    "cvc",
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
    "authorization",
    "cookie",
    "set-cookie",
    "proxy-authorization",
    "x-api-key",
    "x-internal-api-key",
    "x-auth-token",
    "x-csrf-token",
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

REDACTED = "[REDACTED]"

# PAN candidates: 13-19 digits, allowing spaces/dashes between groups.
_PAN_CANDIDATE_RE = re.compile(r"(?<!\d)(?:\d[ -]?){13,19}(?!\d)")
# CVV candidates: "cvv=123" / "cvc: 123" etc.
_CVV_RE = re.compile(r"(?i)\b(cvv|cvc|security\s*code)\b\s*[:=]\s*(\d{3,4})\b")
# Bearer and basic auth style token headers in plain strings.
_AUTH_HEADER_RE = re.compile(r"(?i)\b(authorization|proxy-authorization)\b\s*[:=]\s*([^\n\r,;]+)")
# Cookie strings often appear as plain text in logs.
_COOKIE_HEADER_RE = re.compile(r"(?i)\b(set-cookie|cookie)\b\s*[:=]\s*([^\n\r]+)")

# Token-like strings (JWT-ish or long base64-ish) - conservative masking.
_JWT_RE = re.compile(r"\beyJ[a-zA-Z0-9_-]{8,}\.[a-zA-Z0-9_-]{8,}\.[a-zA-Z0-9_-]{8,}\b")
_LONG_B64ISH_RE = re.compile(r"\b[a-zA-Z0-9+/=_-]{40,}\b")

# Any 13-19 digit run (no separators) as last resort (used only if Luhn passes).
_PAN_DIGITS_ONLY_RE = re.compile(r"\b\d{13,19}\b")

_DEFAULT_MAX_DEPTH = 8
_DEFAULT_MAX_STR_LEN = 20_000


def _is_sensitive_key(key: str) -> bool:
    lowered = key.lower()
    if lowered in SENSITIVE_KEYS:
        return True
    return any(lowered.endswith(suffix) for suffix in SENSITIVE_SUFFIXES)


def _luhn_ok(num: str) -> bool:
    total = 0
    reverse_digits = [int(ch) for ch in reversed(num)]
    for idx, digit in enumerate(reverse_digits):
        if idx % 2 == 1:
            digit *= 2
            if digit > 9:
                digit -= 9
        total += digit
    return total % 10 == 0


def _mask_pan(value: str) -> str:
    digits = re.sub(r"\D", "", value)
    if not (13 <= len(digits) <= 19):
        return value
    if not _luhn_ok(digits):
        return value
    return f"[REDACTED_PAN_****{digits[-4:]}]"


def _maybe_parse_json_string(value: str) -> Any | None:
    text = value.strip()
    if not text:
        return None
    if (text.startswith("{") and text.endswith("}")) or (text.startswith("[") and text.endswith("]")):
        try:
            return json.loads(text)
        except Exception:
            return None
    return None


def _redact_string(value: str) -> str:
    if not value:
        return value

    redacted = value

    redacted = _JWT_RE.sub("[REDACTED_JWT]", redacted)
    redacted = _CVV_RE.sub(lambda match: f"{match.group(1)}={REDACTED}", redacted)
    redacted = _AUTH_HEADER_RE.sub(lambda match: f"{match.group(1)}={REDACTED}", redacted)
    redacted = _COOKIE_HEADER_RE.sub(lambda match: f"{match.group(1)}={REDACTED}", redacted)

    redacted = _PAN_CANDIDATE_RE.sub(lambda match: _mask_pan(match.group(0)), redacted)
    redacted = _PAN_DIGITS_ONLY_RE.sub(lambda match: _mask_pan(match.group(0)), redacted)

    redacted = _LONG_B64ISH_RE.sub("[REDACTED_TOKEN]", redacted)
    return redacted


def redact_sensitive(
    data: Any,
    *,
    max_depth: int = _DEFAULT_MAX_DEPTH,
    _depth: int = 0,
) -> Any:
    """Recursively redact sensitive data for logging/persistence safety boundaries."""

    if _depth > max_depth:
        return "[REDACTED_DEPTH_LIMIT]"

    if isinstance(data, Mapping):
        redacted: dict[str, Any] = {}
        for key, value in data.items():
            key_str = str(key)
            if _is_sensitive_key(key_str):
                redacted[key_str] = REDACTED
            else:
                redacted[key_str] = redact_sensitive(value, max_depth=max_depth, _depth=_depth + 1)
        return redacted

    if isinstance(data, Sequence) and not isinstance(data, (str, bytes, bytearray)):
        return [redact_sensitive(item, max_depth=max_depth, _depth=_depth + 1) for item in data]

    if isinstance(data, (bytes, bytearray)):
        try:
            return _redact_string(data.decode("utf-8", errors="replace")[:_DEFAULT_MAX_STR_LEN])
        except Exception:
            return "[REDACTED_BYTES]"

    if isinstance(data, str):
        clipped = data[:_DEFAULT_MAX_STR_LEN]
        parsed = _maybe_parse_json_string(clipped)
        if parsed is not None:
            return redact_sensitive(parsed, max_depth=max_depth, _depth=_depth + 1)
        return _redact_string(clipped)

    return data
