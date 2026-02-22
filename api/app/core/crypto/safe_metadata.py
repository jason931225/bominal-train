from __future__ import annotations

import json
import re
from typing import Any

from app.core.crypto.redaction import redact_sensitive

# 13-19 digit candidates with optional separators.
_PAN_CANDIDATE_RE = re.compile(r"(?<!\d)(?:\d[ -]?){13,19}(?!\d)")


def _luhn_ok(num: str) -> bool:
    total = 0
    digits = [int(ch) for ch in reversed(num)]
    for idx, value in enumerate(digits):
        if idx % 2 == 1:
            value *= 2
            if value > 9:
                value -= 9
        total += value
    return total % 10 == 0


def _contains_luhn_pan(text: str) -> bool:
    for match in _PAN_CANDIDATE_RE.finditer(text):
        digits = re.sub(r"\D", "", match.group(0))
        if 13 <= len(digits) <= 19 and _luhn_ok(digits):
            return True
    return False


def validate_safe_metadata(payload: Any) -> Any:
    """Sanitize metadata and fail closed if sensitive payload survives redaction."""

    raw_serialized = json.dumps(payload, default=str)
    if _contains_luhn_pan(raw_serialized):
        raise ValueError("safe metadata contains PAN-like value before redaction")

    sanitized = redact_sensitive(payload)

    serialized = json.dumps(sanitized, default=str)

    if _contains_luhn_pan(serialized):
        raise ValueError("safe metadata contains a PAN-like value")

    lowered = serialized.lower()
    if '"cvv"' in lowered and "[redacted" not in lowered:
        raise ValueError("safe metadata contains unredacted cvv field")
    if "authorization" in lowered and "[redacted" not in lowered:
        raise ValueError("safe metadata contains unredacted authorization field")
    if "cookie" in lowered and "[redacted" not in lowered:
        raise ValueError("safe metadata contains unredacted cookie field")

    return sanitized
