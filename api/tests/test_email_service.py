from __future__ import annotations

from types import SimpleNamespace

import httpx
import pytest

from app.schemas.notification import EmailJobPayload, EmailTag
from app.services import email as email_service


def _patch_settings(monkeypatch, **values):
    for key, value in values.items():
        monkeypatch.setattr(email_service.settings, key, value)


@pytest.mark.asyncio
async def test_send_email_log_provider(monkeypatch):
    _patch_settings(
        monkeypatch,
        email_provider="log",
    )

    payload = EmailJobPayload(
        to_email="user@example.com",
        subject="Test",
        text_body="hello",
        tags=["unit"],
    )

    result = await email_service.send_email(payload)
    assert result.status == "sent"
    assert result.provider == "log"
    assert result.metadata["mode"] == "log"


@pytest.mark.asyncio
async def test_send_email_resend_provider(monkeypatch):
    _patch_settings(
        monkeypatch,
        email_provider="resend",
        resend_api_key="re_test_key",
        resend_api_base_url="https://api.resend.com",
        smtp_timeout_seconds=2.0,
    )
    monkeypatch.setattr(email_service.settings, "resend_timeout_seconds", 12.5, raising=False)

    captured: dict[str, object] = {}

    class FakeAsyncClient:
        def __init__(self, timeout: float):
            captured["timeout"] = timeout

        async def __aenter__(self):
            return self

        async def __aexit__(self, exc_type, exc, tb):
            return False

        async def post(self, url: str, headers: dict, json: dict):
            captured["url"] = url
            captured["headers"] = headers
            captured["json"] = json
            return SimpleNamespace(status_code=200, content=b'{"id":"email_123"}', json=lambda: {"id": "email_123"})

    monkeypatch.setattr(email_service.httpx, "AsyncClient", FakeAsyncClient)

    payload = EmailJobPayload(
        to_email="user@example.com",
        subject="Test Resend",
        text_body="hello resend",
        tags=["unit", {"name": "module", "value": "auth"}],
        headers={"X-Custom": "true"},
        message_id="msg-123",
        idempotency_key="idem-123",
    )

    result = await email_service.send_email(payload)
    assert result.status == "sent"
    assert result.provider == "resend"
    assert result.metadata["provider_message_id"] == "email_123"
    assert captured["url"] == "https://api.resend.com/emails"
    assert captured["timeout"] == 12.5
    assert captured["headers"]["Idempotency-Key"] == "idem-123"
    assert captured["json"]["headers"]["X-Bominal-Message-Id"] == "msg-123"
    assert captured["json"]["headers"]["X-Custom"] == "true"
    assert captured["json"]["tags"] == [
        {"name": "unit", "value": "true"},
        {"name": "module", "value": "auth"},
    ]


@pytest.mark.asyncio
async def test_send_email_resend_rejected(monkeypatch):
    _patch_settings(
        monkeypatch,
        email_provider="resend",
        resend_api_key="re_test_key",
        resend_api_base_url="https://api.resend.com",
    )

    class FakeAsyncClient:
        def __init__(self, timeout: float):
            self.timeout = timeout

        async def __aenter__(self):
            return self

        async def __aexit__(self, exc_type, exc, tb):
            return False

        async def post(self, url: str, headers: dict, json: dict):
            return SimpleNamespace(status_code=422, content=b'{}', json=lambda: {})

    monkeypatch.setattr(email_service.httpx, "AsyncClient", FakeAsyncClient)

    payload = EmailJobPayload(
        to_email="user@example.com",
        subject="Test Resend",
        text_body="hello resend",
    )

    with pytest.raises(email_service.EmailDeliveryError, match="Resend rejected email"):
        await email_service.send_email(payload)


def test_build_message_and_tag_helpers(monkeypatch):
    _patch_settings(
        monkeypatch,
        email_from_name="Bominal Ops",
        email_from_address="ops@bominal.local",
        email_reply_to="reply@bominal.local",
    )
    payload = EmailJobPayload(
        to_email="user@example.com",
        subject="Subject",
        text_body="plain text",
        html_body="<b>hello</b>",
        message_id="msg-abc",
        headers={"X-Custom": "1", "X-Bominal-Message-Id": "do-not-override"},
    )
    message = email_service._build_message(payload)
    assert message["From"] == "Bominal Ops <ops@bominal.local>"
    assert message["Reply-To"] == "reply@bominal.local"
    assert message["X-Bominal-Message-Id"] == "msg-abc"
    assert message["X-Custom"] == "1"
    assert message.get_content_type() == "multipart/alternative"

    assert email_service._tags_for_log(["a", EmailTag(name="module", value="auth")]) == "a,module:auth"
    assert email_service._resend_tag_payload(
        ["a", EmailTag(name="module", value="auth"), {"name": "x", "value": "y"}, {"bad": "skip"}]
    ) == [
        {"name": "a", "value": "true"},
        {"name": "module", "value": "auth"},
        {"name": "x", "value": "y"},
    ]


def test_resend_payload_generation_uses_auto_message_id(monkeypatch):
    _patch_settings(
        monkeypatch,
        email_from_name="Bominal",
        email_from_address="noreply@bominal.local",
    )
    monkeypatch.setattr(email_service.uuid, "uuid4", lambda: "uuid-fixed")

    payload = EmailJobPayload(
        to_email="user@example.com",
        subject="Subject",
        text_body="plain",
        html_body="<b>html</b>",
        tags=["unit"],
        headers={"X-Test": "1"},
    )
    body = email_service._resend_payload(payload)
    assert body["headers"]["X-Bominal-Message-Id"] == "uuid-fixed"
    assert body["headers"]["X-Test"] == "1"
    assert body["html"] == "<b>html</b>"
    assert body["tags"] == [{"name": "unit", "value": "true"}]


@pytest.mark.asyncio
async def test_send_resend_requires_key_and_maps_http_errors(monkeypatch):
    _patch_settings(
        monkeypatch,
        resend_api_key="",
        resend_api_base_url="https://api.resend.com",
    )
    payload = EmailJobPayload(to_email="user@example.com", subject="Resend", text_body="body")

    with pytest.raises(email_service.EmailDeliveryError, match="API key"):
        await email_service._send_resend(payload)

    _patch_settings(
        monkeypatch,
        resend_api_key="re_key",
        resend_api_base_url="https://api.resend.com",
    )

    class _FailingAsyncClient:
        def __init__(self, timeout: float):  # noqa: ARG002
            pass

        async def __aenter__(self):
            return self

        async def __aexit__(self, exc_type, exc, tb):
            return False

        async def post(self, url: str, headers: dict, json: dict):  # noqa: ARG002
            raise httpx.ReadTimeout("timed out")

    monkeypatch.setattr(email_service.httpx, "AsyncClient", _FailingAsyncClient)
    with pytest.raises(email_service.EmailDeliveryError, match="ReadTimeout"):
        await email_service._send_resend(payload)


def test_send_smtp_sync_handles_starttls_login_and_quit_fallback(monkeypatch):
    _patch_settings(
        monkeypatch,
        smtp_host="mailpit",
        smtp_port=1025,
        smtp_timeout_seconds=5.0,
        smtp_use_ssl=False,
        smtp_starttls=True,
        smtp_username="user",
        smtp_password="pass",
    )
    calls: list[str] = []

    class _FakeSMTP:
        def __init__(self, host: str, port: int, timeout: float):  # noqa: ARG002
            calls.append("smtp")

        def ehlo(self):
            calls.append("ehlo")

        def starttls(self, context):  # noqa: ANN001
            calls.append("starttls")

        def login(self, username: str, password: str):
            calls.append(f"login:{username}:{password}")

        def send_message(self, _message):  # noqa: ANN001
            calls.append("send")

        def quit(self):
            calls.append("quit")
            raise RuntimeError("quit failed")

        def close(self):
            calls.append("close")

    monkeypatch.setattr(email_service.smtplib, "SMTP", _FakeSMTP)

    message = EmailJobPayload(to_email="user@example.com", subject="SMTP", text_body="body")
    email_service._send_smtp_sync(email_service._build_message(message))
    assert "starttls" in calls
    assert "login:user:pass" in calls
    assert "send" in calls
    assert "close" in calls

    _patch_settings(
        monkeypatch,
        smtp_use_ssl=True,
        smtp_starttls=False,
        smtp_username="",
        smtp_password="",
    )
    ssl_calls: list[str] = []

    class _FakeSMTPSSL(_FakeSMTP):
        def __init__(self, host: str, port: int, timeout: float, context):  # noqa: ARG002
            ssl_calls.append("smtp_ssl")

        def quit(self):
            ssl_calls.append("quit")

    monkeypatch.setattr(email_service.smtplib, "SMTP_SSL", _FakeSMTPSSL)
    email_service._send_smtp_sync(email_service._build_message(message))
    assert ssl_calls == ["smtp_ssl", "quit"]


@pytest.mark.asyncio
async def test_send_email_disabled_unsupported_and_smtp_failure(monkeypatch):
    payload = EmailJobPayload(to_email="user@example.com", subject="Subject", text_body="body")

    _patch_settings(monkeypatch, email_provider="disabled")
    with pytest.raises(email_service.EmailDeliveryError, match="disabled"):
        await email_service.send_email(payload)

    _patch_settings(monkeypatch, email_provider="unknown-provider")
    with pytest.raises(email_service.EmailDeliveryError, match="Unsupported email provider"):
        await email_service.send_email(payload)

    _patch_settings(monkeypatch, email_provider="smtp")

    async def _to_thread_fail(_fn, _arg):  # noqa: ANN001
        raise RuntimeError("smtp down")

    monkeypatch.setattr(email_service.asyncio, "to_thread", _to_thread_fail)
    with pytest.raises(email_service.EmailDeliveryError, match="SMTP delivery failed"):
        await email_service.send_email(payload)
