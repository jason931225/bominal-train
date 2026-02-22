import pytest

from app.core.crypto.safe_metadata import validate_safe_metadata


def test_validate_safe_metadata_redacts_sensitive_values() -> None:
    payload = {
        "provider_http": {
            "headers": "Authorization: Bearer abc Cookie: sid=xyz",
            "cvv": "123",
        }
    }

    safe = validate_safe_metadata(payload)

    serialized = str(safe)
    assert "Bearer abc" not in serialized
    assert "sid=xyz" not in serialized
    assert "[REDACTED" in serialized


def test_validate_safe_metadata_rejects_pan_surviving_redaction() -> None:
    # The payload is already marked "safe" but still includes PAN-like digits.
    with pytest.raises(ValueError):
        validate_safe_metadata({"safe_value": "4111111111111111"})
