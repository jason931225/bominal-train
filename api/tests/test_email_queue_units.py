from __future__ import annotations

from types import SimpleNamespace

import pytest

from app.schemas.notification import EmailJobPayload, EmailTemplateBlock, EmailTemplateJobPayload
from app.services import email_queue


def test_redis_settings_uses_non_cde_resolved_url(monkeypatch):
    monkeypatch.setattr(email_queue.settings, "redis_url_non_cde", "redis://non-cde:6379/1")
    settings = email_queue._redis_settings()
    assert settings.host == "non-cde"
    assert settings.port == 6379
    assert settings.database == 1


@pytest.mark.asyncio
async def test_get_email_queue_pool_caches_create_pool(monkeypatch):
    email_queue._pool = None
    calls = {"count": 0}
    fake_pool = object()

    async def _fake_create_pool(_redis_settings, default_queue_name: str):  # noqa: ANN001
        calls["count"] += 1
        assert default_queue_name == "train:queue"
        return fake_pool

    monkeypatch.setattr(email_queue, "create_pool", _fake_create_pool)
    monkeypatch.setattr(email_queue, "_redis_settings", lambda: "redis-settings")

    assert await email_queue.get_email_queue_pool() is fake_pool
    assert await email_queue.get_email_queue_pool() is fake_pool
    assert calls["count"] == 1


class _Pool:
    def __init__(self, *, return_none: bool = False):
        self.return_none = return_none
        self.calls: list[tuple[str, dict, float | None]] = []

    async def enqueue_job(self, name: str, payload: dict, _defer_by=None):  # noqa: ANN001
        self.calls.append((name, payload, _defer_by))
        if self.return_none:
            return None
        return SimpleNamespace(job_id="job-123")


@pytest.mark.asyncio
async def test_enqueue_email_and_template_paths(monkeypatch):
    pool = _Pool(return_none=False)
    
    async def _pool_factory():
        return pool

    monkeypatch.setattr(email_queue, "get_email_queue_pool", _pool_factory)

    payload = EmailJobPayload(to_email="user@example.com", subject="Subject", text_body="body")
    job_id = await email_queue.enqueue_email(payload)
    assert job_id == "job-123"
    assert pool.calls[-1][0] == "deliver_email_job"
    assert pool.calls[-1][2] is None

    deferred_job_id = await email_queue.enqueue_email(payload, defer_seconds=3.5)
    assert deferred_job_id == "job-123"
    assert pool.calls[-1][2] == 3.5

    template_payload = EmailTemplateJobPayload(
        to_email="user@example.com",
        subject="Template Subject",
        blocks=[EmailTemplateBlock(type="paragraph", data={"text": "hello"})],
    )
    template_job_id = await email_queue.enqueue_template_email(template_payload, defer_seconds=2.0)
    assert template_job_id == "job-123"
    assert pool.calls[-1][2] == 2.0


@pytest.mark.asyncio
async def test_enqueue_returns_none_when_arq_drops_job(monkeypatch):
    pool = _Pool(return_none=True)
    
    async def _pool_factory():
        return pool

    monkeypatch.setattr(email_queue, "get_email_queue_pool", _pool_factory)

    payload = EmailJobPayload(to_email="user@example.com", subject="Subject", text_body="body")
    assert await email_queue.enqueue_email(payload) is None

    template_payload = EmailTemplateJobPayload(
        to_email="user@example.com",
        subject="Template Subject",
        blocks=[EmailTemplateBlock(type="paragraph", data={"text": "hello"})],
    )
    assert await email_queue.enqueue_template_email(template_payload) is None
