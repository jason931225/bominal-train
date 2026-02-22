import pytest

from app.modules.restaurant.queue import enqueue_restaurant_task
from app.modules.train.queue import enqueue_train_task


@pytest.mark.asyncio
async def test_train_queue_rejects_non_uuid_payload(monkeypatch):
    async def _unexpected_pool():
        raise AssertionError("queue pool should not be called for invalid payload")

    monkeypatch.setattr("app.modules.train.queue.get_queue_pool", _unexpected_pool)

    with pytest.raises(ValueError):
        await enqueue_train_task("not-a-uuid")


@pytest.mark.asyncio
async def test_restaurant_queue_rejects_non_uuid_payload(monkeypatch):
    async def _unexpected_pool():
        raise AssertionError("queue pool should not be called for invalid payload")

    monkeypatch.setattr("app.modules.restaurant.queue.get_restaurant_queue_pool", _unexpected_pool)

    with pytest.raises(ValueError):
        await enqueue_restaurant_task("not-a-uuid")
