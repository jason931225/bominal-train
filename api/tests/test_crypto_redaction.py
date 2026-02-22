from __future__ import annotations

from app.core.crypto.redaction import redact_sensitive


def test_masks_authorization_cookie_and_set_cookie_headers() -> None:
    payload = {
        "headers": {
            "Authorization": "Bearer secret-token",
            "Cookie": "bominal_session=abc123",
            "Set-Cookie": "session=abc; HttpOnly",
        }
    }

    redacted = redact_sensitive(payload)

    assert redacted["headers"]["Authorization"] == "[REDACTED]"
    assert redacted["headers"]["Cookie"] == "[REDACTED]"
    assert redacted["headers"]["Set-Cookie"] == "[REDACTED]"


def test_masks_pan_pattern_in_unkeyed_string() -> None:
    payload = {"message": "payment failed for card 4111 1111 1111 1111"}

    redacted = redact_sensitive(payload)
    message = redacted["message"]

    assert "4111 1111 1111 1111" not in message
    assert "[REDACTED_PAN_****1111]" in message


def test_does_not_mask_non_luhn_numeric_strings() -> None:
    payload = {"message": "random digits 1234567890123456"}

    redacted = redact_sensitive(payload)

    assert redacted["message"] == "random digits 1234567890123456"


def test_masks_nested_json_string_payload() -> None:
    payload = {"raw": '{"number":"4111111111111111","cvv":"123"}'}

    redacted = redact_sensitive(payload)

    assert isinstance(redacted["raw"], dict)
    assert redacted["raw"]["number"] == "[REDACTED_PAN_****1111]"
    assert redacted["raw"]["cvv"] == "[REDACTED]"


def test_masks_sensitive_bytes_payload() -> None:
    payload = b"Authorization: Bearer secret-token"

    redacted = redact_sensitive(payload)

    assert "secret-token" not in redacted
    assert "[REDACTED_TOKEN]" in redacted


def test_depth_limit_returns_redacted_depth_marker() -> None:
    payload = {}
    current = payload
    for idx in range(10):
        current["next"] = {"i": idx}
        current = current["next"]

    redacted = redact_sensitive(payload, max_depth=4)

    # Walk until depth marker appears.
    cursor = redacted
    for _ in range(5):
        cursor = cursor["next"]
    assert cursor == "[REDACTED_DEPTH_LIMIT]"
