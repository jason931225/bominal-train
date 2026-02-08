from __future__ import annotations

import pytest

from app.modules.train.queue import enqueue_train_task


class _FakeQueuePool:
    def __init__(self) -> None:
        self.calls: list[dict] = []

    async def enqueue_job(self, *args, **kwargs):
        self.calls.append({"args": args, "kwargs": kwargs})
        return None


@pytest.mark.asyncio
async def test_enqueue_train_task_passes_deterministic_job_id(monkeypatch):
    pool = _FakeQueuePool()

    async def _fake_get_pool():
        return pool

    monkeypatch.setattr("app.modules.train.queue.get_queue_pool", _fake_get_pool)

    await enqueue_train_task("abc123")

    assert len(pool.calls) == 1
    assert pool.calls[0]["args"] == ("run_train_task", "abc123")
    assert pool.calls[0]["kwargs"]["_job_id"] == "train:abc123"


@pytest.mark.asyncio
async def test_enqueue_train_task_preserves_defer_by(monkeypatch):
    pool = _FakeQueuePool()

    async def _fake_get_pool():
        return pool

    monkeypatch.setattr("app.modules.train.queue.get_queue_pool", _fake_get_pool)

    await enqueue_train_task("task-1", defer_seconds=2.5)

    assert len(pool.calls) == 1
    assert pool.calls[0]["args"] == ("run_train_task", "task-1")
    assert pool.calls[0]["kwargs"]["_job_id"] == "train:task-1"
    assert pool.calls[0]["kwargs"]["_defer_by"] == 2.5

