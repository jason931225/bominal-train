from __future__ import annotations

from uuid import uuid4

import pytest


async def _register_and_login(client, *, email: str) -> str:
    register = await client.post(
        "/api/auth/register",
        json={"email": email, "password": "SuperSecret123", "display_name": "Notify User"},
    )
    assert register.status_code == 201

    login = await client.post(
        "/api/auth/login",
        json={"email": email, "password": "SuperSecret123", "remember_me": False},
    )
    assert login.status_code == 200
    cookie = login.cookies.get("bominal_session")
    assert cookie
    return cookie


@pytest.mark.asyncio
async def test_email_status_requires_auth(client):
    response = await client.get("/api/notifications/email/status")
    assert response.status_code == 401


@pytest.mark.asyncio
async def test_send_test_email_queues_job(client, monkeypatch):
    cookie = await _register_and_login(client, email=f"notify-{uuid4().hex[:8]}@example.com")

    captured: dict[str, object] = {}

    async def _fake_enqueue(payload, defer_seconds: float = 0.0):
        captured["to"] = str(payload.to_email)
        captured["subject"] = payload.subject
        captured["tags"] = payload.tags
        captured["defer_seconds"] = defer_seconds
        return "job-test-123"

    monkeypatch.setattr("app.http.routes.notifications.enqueue_email", _fake_enqueue)

    response = await client.post(
        "/api/notifications/email/test",
        cookies={"bominal_session": cookie},
        json={},
    )
    assert response.status_code == 200
    payload = response.json()

    assert payload["queued"] is True
    assert payload["job_id"] == "job-test-123"
    assert payload["detail"] == "Test email queued"
    assert payload["recipient"] == captured["to"]
    assert payload["provider"] in {"smtp", "log", "resend"}

    assert captured["subject"] == "bominal test email"
    assert "test" in captured["tags"]


@pytest.mark.asyncio
async def test_send_test_email_returns_503_when_disabled(client, monkeypatch):
    cookie = await _register_and_login(client, email=f"notify-disabled-{uuid4().hex[:8]}@example.com")

    monkeypatch.setattr("app.http.routes.notifications.settings.email_provider", "disabled")

    response = await client.post(
        "/api/notifications/email/test",
        cookies={"bominal_session": cookie},
        json={},
    )

    assert response.status_code == 503
    assert response.json()["detail"] == "Email delivery is disabled"
