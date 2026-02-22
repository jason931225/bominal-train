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
    task_id = "11111111-1111-1111-1111-111111111111"

    async def _fake_get_pool():
        return pool

    monkeypatch.setattr("app.modules.train.queue.get_queue_pool", _fake_get_pool)

    await enqueue_train_task(task_id)

    assert len(pool.calls) == 1
    assert pool.calls[0]["args"] == ("run_train_task", task_id)
    assert pool.calls[0]["kwargs"]["_job_id"] == f"train:{task_id}"


@pytest.mark.asyncio
async def test_enqueue_train_task_preserves_defer_by(monkeypatch):
    pool = _FakeQueuePool()
    task_id = "22222222-2222-2222-2222-222222222222"

    async def _fake_get_pool():
        return pool

    monkeypatch.setattr("app.modules.train.queue.get_queue_pool", _fake_get_pool)

    await enqueue_train_task(task_id, defer_seconds=2.5)

    assert len(pool.calls) == 1
    assert pool.calls[0]["args"] == ("run_train_task", task_id)
    assert pool.calls[0]["kwargs"]["_job_id"] == f"train:{task_id}"
    assert pool.calls[0]["kwargs"]["_defer_by"] == 2.5
