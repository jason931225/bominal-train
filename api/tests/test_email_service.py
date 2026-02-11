from __future__ import annotations

from types import SimpleNamespace

import pytest

from app.schemas.notification import EmailJobPayload
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
