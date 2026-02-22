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
    # envelope fields
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

# Header-like keys (case-insensitive)
SENSITIVE_HEADER_KEYS = {
    "authorization",
    "proxy-authorization",
    "cookie",
    "set-cookie",
    "x-api-key",
    "x-internal-api-key",
    "x-auth-token",
    "x-csrf-token",
}

REDACTED = "[REDACTED]"

# PAN candidates: 13-19 digits allowing spaces/dashes between groups.
_PAN_CANDIDATE_RE = re.compile(r"(?<!\d)(?:\d[ -]?){13,19}(?!\d)")
_PAN_DIGITS_ONLY_RE = re.compile(r"\b\d{13,19}\b")
_CVV_RE = re.compile(r"(?i)\b(cvv|cvc|security\s*code)\b\s*[:=]\s*(\d{3,4})\b")
_BEARER_RE = re.compile(r"(?i)\bbearer\s+([A-Za-z0-9._~+/-]+=*)")
_JWT_RE = re.compile(r"\beyJ[a-zA-Z0-9_-]{10,}\.[a-zA-Z0-9_-]{10,}\.[a-zA-Z0-9_-]{10,}\b")
_LONG_B64ISH_RE = re.compile(r"\b[a-zA-Z0-9+/=_-]{40,}\b")

_DEFAULT_MAX_DEPTH = 8
_DEFAULT_MAX_STR_LEN = 20_000


def _is_sensitive_key(key: str) -> bool:
    lowered = key.lower()
    if lowered in SENSITIVE_KEYS:
        return True
    if lowered in SENSITIVE_HEADER_KEYS:
        return True
    return any(lowered.endswith(suffix) for suffix in SENSITIVE_SUFFIXES)


def _luhn_ok(num: str) -> bool:
    total = 0
    reverse_digits = list(map(int, reversed(num)))
    for index, digit in enumerate(reverse_digits):
        if index % 2 == 1:
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


def _redact_string(value: str) -> str:
    if not value:
        return value

    redacted = _JWT_RE.sub("[REDACTED_JWT]", value)
    redacted = _BEARER_RE.sub("Bearer [REDACTED_TOKEN]", redacted)
    redacted = _CVV_RE.sub(lambda match: f"{match.group(1)}={REDACTED}", redacted)
    redacted = _PAN_CANDIDATE_RE.sub(lambda match: _mask_pan(match.group(0)), redacted)
    redacted = _PAN_DIGITS_ONLY_RE.sub(lambda match: _mask_pan(match.group(0)), redacted)
    redacted = _LONG_B64ISH_RE.sub("[REDACTED_TOKEN]", redacted)
    return redacted


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


def redact_sensitive(
    data: Any,
    *,
    max_depth: int = _DEFAULT_MAX_DEPTH,
    _depth: int = 0,
) -> Any:
    """Recursively redact sensitive keys and high-risk string patterns.

    This helper is intended for logging safety.
    """
    if _depth > max_depth:
        return "[REDACTED_DEPTH_LIMIT]"

    if isinstance(data, Mapping):
        redacted: dict[str, Any] = {}
        for key, value in data.items():
            key_text = str(key)
            if _is_sensitive_key(key_text):
                redacted[key_text] = REDACTED
            else:
                redacted[key_text] = redact_sensitive(value, max_depth=max_depth, _depth=_depth + 1)
        return redacted

    if isinstance(data, Sequence) and not isinstance(data, (str, bytes, bytearray)):
        return [redact_sensitive(item, max_depth=max_depth, _depth=_depth + 1) for item in data]

    if isinstance(data, (bytes, bytearray)):
        try:
            return _redact_string(data.decode("utf-8", errors="replace")[:_DEFAULT_MAX_STR_LEN])
        except Exception:
            return "[REDACTED_BYTES]"

    if isinstance(data, str):
        truncated = data[:_DEFAULT_MAX_STR_LEN]
        parsed = _maybe_parse_json_string(truncated)
        if parsed is not None:
            return redact_sensitive(parsed, max_depth=max_depth, _depth=_depth + 1)
        return _redact_string(truncated)

    return data
