from __future__ import annotations

import pytest

from app.modules.train.queue import enqueue_train_task


class _FakeQueuePool:
    def __init__(self, *, enqueue_returns_job: bool = True) -> None:
        self.calls: list[dict] = []
        self.deleted: list[str] = []
        self.enqueue_returns_job = enqueue_returns_job

    async def delete(self, *keys: str) -> int:
        self.deleted.extend(keys)
        return len(keys)

    async def enqueue_job(self, *args, **kwargs):
        self.calls.append({"args": args, "kwargs": kwargs})
        return object() if self.enqueue_returns_job else None


@pytest.mark.asyncio
async def test_enqueue_train_task_passes_deterministic_job_id(monkeypatch):
    pool = _FakeQueuePool()

    async def _fake_get_pool():
        return pool

    monkeypatch.setattr("app.modules.train.queue.get_queue_pool", _fake_get_pool)

    enqueued = await enqueue_train_task("abc123")

    assert enqueued is True
    assert len(pool.calls) == 1
    assert pool.calls[0]["args"] == ("run_train_task", "abc123")
    assert pool.calls[0]["kwargs"]["_job_id"] == "train:abc123"
    assert pool.deleted == ["arq:result:train:abc123"]


@pytest.mark.asyncio
async def test_enqueue_train_task_preserves_defer_by(monkeypatch):
    pool = _FakeQueuePool()

    async def _fake_get_pool():
        return pool

    monkeypatch.setattr("app.modules.train.queue.get_queue_pool", _fake_get_pool)

    enqueued = await enqueue_train_task("task-1", defer_seconds=2.5)

    assert enqueued is True
    assert len(pool.calls) == 1
    assert pool.calls[0]["args"] == ("run_train_task", "task-1")
    assert pool.calls[0]["kwargs"]["_defer_by"] == 2.5
    assert "_job_id" not in pool.calls[0]["kwargs"]
    assert pool.deleted == []


@pytest.mark.asyncio
async def test_enqueue_train_task_returns_false_when_job_not_accepted(monkeypatch):
    pool = _FakeQueuePool(enqueue_returns_job=False)

    async def _fake_get_pool():
        return pool

    monkeypatch.setattr("app.modules.train.queue.get_queue_pool", _fake_get_pool)

    enqueued = await enqueue_train_task("task-dup")

    assert enqueued is False
    assert pool.deleted == ["arq:result:train:task-dup"]
