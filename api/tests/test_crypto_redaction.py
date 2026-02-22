from app.core.crypto.redaction import redact_sensitive


def test_redacts_sensitive_key_values() -> None:
    payload = {"password": "secret", "nested": {"token": "abc", "ok": "yes"}}
    redacted = redact_sensitive(payload)

    assert redacted["password"] == "[REDACTED]"
    assert redacted["nested"]["token"] == "[REDACTED]"
    assert redacted["nested"]["ok"] == "yes"


def test_redacts_pan_in_unknown_string_field() -> None:
    payload = {"data": "card=4111 1111 1111 1111"}

    redacted = redact_sensitive(payload)

    assert "4111 1111 1111 1111" not in str(redacted)
    assert "REDACTED_PAN" in str(redacted)


def test_redacts_authorization_and_cookie_strings() -> None:
    payload = {
        "headers": "Authorization: Bearer token123 Cookie: sessionid=abc123",
    }

    redacted = redact_sensitive(payload)

    assert "token123" not in str(redacted)
    assert "sessionid=abc123" not in str(redacted)
    assert "[REDACTED]" in str(redacted)


def test_redacts_nested_json_string_payload() -> None:
    payload = {
        "blob": '{"payment":{"number":"4111111111111111","cvv":"123"}}',
    }

    redacted = redact_sensitive(payload)

    assert "4111111111111111" not in str(redacted)
    assert "123" not in str(redacted)
    assert "REDACTED" in str(redacted)
