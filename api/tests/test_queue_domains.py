from __future__ import annotations

import pytest

from app.core.queue_domains import RESTAURANT_QUEUE_NAME, TRAIN_QUEUE_NAME


@pytest.mark.asyncio
async def test_train_queue_pool_uses_train_queue_name(monkeypatch):
    import app.modules.train.queue as train_queue

    train_queue._pool = None
    captured: dict[str, object] = {}

    async def _fake_create_pool(settings, **kwargs):  # noqa: ANN001
        captured["kwargs"] = kwargs
        return object()

    monkeypatch.setattr(train_queue, "create_pool", _fake_create_pool)

    await train_queue.get_queue_pool()

    kwargs = captured.get("kwargs")
    assert isinstance(kwargs, dict)
    assert kwargs["default_queue_name"] == TRAIN_QUEUE_NAME


@pytest.mark.asyncio
async def test_email_queue_pool_uses_train_queue_name(monkeypatch):
    import app.services.email_queue as email_queue

    email_queue._pool = None
    captured: dict[str, object] = {}

    async def _fake_create_pool(settings, **kwargs):  # noqa: ANN001
        captured["kwargs"] = kwargs
        return object()

    monkeypatch.setattr(email_queue, "create_pool", _fake_create_pool)

    await email_queue.get_email_queue_pool()

    kwargs = captured.get("kwargs")
    assert isinstance(kwargs, dict)
    assert kwargs["default_queue_name"] == TRAIN_QUEUE_NAME


@pytest.mark.asyncio
async def test_restaurant_queue_pool_uses_restaurant_queue_name(monkeypatch):
    import app.modules.restaurant.queue as restaurant_queue

    restaurant_queue._pool = None
    captured: dict[str, object] = {}

    async def _fake_create_pool(settings, **kwargs):  # noqa: ANN001
        captured["kwargs"] = kwargs
        return object()

    monkeypatch.setattr(restaurant_queue, "create_pool", _fake_create_pool)

    await restaurant_queue.get_restaurant_queue_pool()

    kwargs = captured.get("kwargs")
    assert isinstance(kwargs, dict)
    assert kwargs["default_queue_name"] == RESTAURANT_QUEUE_NAME


class _FakeQueuePool:
    def __init__(self) -> None:
        self.calls: list[dict[str, object]] = []

    async def enqueue_job(self, *args, **kwargs):  # noqa: ANN002, ANN003
        self.calls.append({"args": args, "kwargs": kwargs})
        return None


@pytest.mark.asyncio
async def test_enqueue_restaurant_task_uses_deterministic_job_id(monkeypatch):
    import app.modules.restaurant.queue as restaurant_queue

    pool = _FakeQueuePool()
    task_id = "33333333-3333-3333-3333-333333333333"

    async def _fake_get_pool():
        return pool

    monkeypatch.setattr(restaurant_queue, "get_restaurant_queue_pool", _fake_get_pool)

    await restaurant_queue.enqueue_restaurant_task(task_id, defer_seconds=1.5)

    assert len(pool.calls) == 1
    assert pool.calls[0]["args"] == ("run_restaurant_task", task_id)
    kwargs = pool.calls[0]["kwargs"]
    assert isinstance(kwargs, dict)
    assert kwargs["_job_id"] == f"restaurant:{task_id}"
    assert kwargs["_defer_by"] == 1.5


def test_worker_function_sets_are_isolated():
    from app.modules.restaurant.worker import run_restaurant_task
    from app.modules.train.worker import run_train_task
    from app.services.email_worker import deliver_email_job
    from app.worker import WorkerSettings as TrainSettings
    from app.worker_restaurant import WorkerRestaurantSettings as RestaurantSettings

    assert run_train_task in TrainSettings.functions
    assert deliver_email_job in TrainSettings.functions
    assert run_restaurant_task not in TrainSettings.functions
    assert RestaurantSettings.functions == [run_restaurant_task]
    assert run_train_task not in RestaurantSettings.functions
