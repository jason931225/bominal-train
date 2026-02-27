from __future__ import annotations

import pytest

from app.schemas.notification import EmailJobPayload
from app.services import supabase_edge


@pytest.mark.asyncio
async def test_send_task_notification_via_edge_returns_false_when_disabled(monkeypatch):
    monkeypatch.setattr(supabase_edge.settings, "edge_task_notify_enabled", False, raising=False)
    payload = EmailJobPayload(
        to_email="user@example.com",
        subject="Test",
        text_body="hello",
    )
    assert await supabase_edge.send_task_notification_via_edge(payload) is False


@pytest.mark.asyncio
async def test_send_task_notification_via_edge_posts_payload(monkeypatch):
    monkeypatch.setattr(supabase_edge.settings, "edge_task_notify_enabled", True, raising=False)
    monkeypatch.setattr(supabase_edge.settings, "supabase_service_role_key", "service-key", raising=False)
    monkeypatch.setattr(
        supabase_edge.settings,
        "supabase_edge_functions_base_url",
        "https://project-ref.supabase.co/functions/v1",
        raising=False,
    )
    monkeypatch.setattr(
        supabase_edge.settings,
        "supabase_edge_task_notify_function_name",
        "task-notify",
        raising=False,
    )
    monkeypatch.setattr(supabase_edge.settings, "supabase_edge_timeout_seconds", 4.0, raising=False)

    captured: dict[str, object] = {}

    class _FakeResponse:
        status_code = 200
        content = b'{"ok":true}'

        def json(self):  # noqa: ANN201
            return {"ok": True}

    class _FakeClient:
        def __init__(self, *, timeout: float):
            captured["timeout"] = timeout

        async def __aenter__(self):  # noqa: ANN204
            return self

        async def __aexit__(self, exc_type, exc, tb):  # noqa: ANN001, ANN204
            return False

        async def post(self, url: str, *, headers: dict[str, str], json: dict):  # noqa: ANN003
            captured["url"] = url
            captured["headers"] = headers
            captured["json"] = json
            return _FakeResponse()

    monkeypatch.setattr(
        supabase_edge.httpx,
        "AsyncClient",
        lambda timeout: _FakeClient(timeout=timeout),
    )

    payload = EmailJobPayload(
        to_email="user@example.com",
        subject="Train Completed",
        text_body="completed",
        html_body="<p>completed</p>",
        metadata={"module": "train"},
        idempotency_key="task:1:state:COMPLETED:v1",
    )

    ok = await supabase_edge.send_task_notification_via_edge(payload)

    assert ok is True
    assert captured["timeout"] == 4.0
    assert captured["url"] == "https://project-ref.supabase.co/functions/v1/task-notify"
    headers = captured["headers"]
    assert isinstance(headers, dict)
    assert headers["Authorization"] == "Bearer service-key"
    body = captured["json"]
    assert isinstance(body, dict)
    assert body["to_email"] == "user@example.com"
    assert body["subject"] == "Train Completed"


@pytest.mark.asyncio
async def test_send_task_notification_via_edge_rejects_invalid_json_response(monkeypatch):
    monkeypatch.setattr(supabase_edge.settings, "edge_task_notify_enabled", True, raising=False)
    monkeypatch.setattr(supabase_edge.settings, "supabase_service_role_key", "service-key", raising=False)
    monkeypatch.setattr(
        supabase_edge.settings,
        "supabase_edge_functions_base_url",
        "https://project-ref.supabase.co/functions/v1",
        raising=False,
    )
    monkeypatch.setattr(
        supabase_edge.settings,
        "supabase_edge_task_notify_function_name",
        "task-notify",
        raising=False,
    )

    class _FakeResponse:
        status_code = 200
        content = b"not-json"

        def json(self):  # noqa: ANN201
            raise ValueError("invalid json")

    class _FakeClient:
        def __init__(self, *, timeout: float):  # noqa: ARG002
            pass

        async def __aenter__(self):  # noqa: ANN204
            return self

        async def __aexit__(self, exc_type, exc, tb):  # noqa: ANN001, ANN204
            return False

        async def post(self, url: str, *, headers: dict[str, str], json: dict):  # noqa: ANN003, ARG002
            return _FakeResponse()

    monkeypatch.setattr(
        supabase_edge.httpx,
        "AsyncClient",
        lambda timeout: _FakeClient(timeout=timeout),
    )

    payload = EmailJobPayload(
        to_email="user@example.com",
        subject="Train Completed",
        text_body="completed",
    )

    ok = await supabase_edge.send_task_notification_via_edge(payload)

    assert ok is False
