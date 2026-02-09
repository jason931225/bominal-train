from __future__ import annotations

import pytest


class _FakePool:
    def __init__(self) -> None:
        self.calls: list[tuple[tuple, dict]] = []

    async def enqueue_job(self, *args, **kwargs) -> None:  # pragma: no cover
        self.calls.append((args, kwargs))


@pytest.mark.asyncio
async def test_enqueue_train_task_passes_deterministic_job_id(monkeypatch):
    from app.modules.train.queue import enqueue_train_task

    pool = _FakePool()

    async def _fake_get_pool():
        return pool

    monkeypatch.setattr("app.modules.train.queue.get_queue_pool", _fake_get_pool)

    await enqueue_train_task("abc123")

    assert len(pool.calls) == 1
    args, kwargs = pool.calls[0]
    assert args == ("run_train_task", "abc123")
    assert kwargs["_job_id"] == "train:abc123"
    assert "_defer_by" not in kwargs


@pytest.mark.asyncio
async def test_enqueue_train_task_preserves_defer_by(monkeypatch):
    from app.modules.train.queue import enqueue_train_task

    pool = _FakePool()

    async def _fake_get_pool():
        return pool

    monkeypatch.setattr("app.modules.train.queue.get_queue_pool", _fake_get_pool)

    await enqueue_train_task("task-1", defer_seconds=2.5)

    assert len(pool.calls) == 1
    args, kwargs = pool.calls[0]
    assert args == ("run_train_task", "task-1")
    assert kwargs["_job_id"] == "train:task-1"
    assert kwargs["_defer_by"] == 2.5

