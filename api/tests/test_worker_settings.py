from __future__ import annotations

import importlib

from app.core.queue_domains import RESTAURANT_QUEUE_NAME, TRAIN_QUEUE_NAME


def test_worker_settings_have_isolated_queue_names():
    from app.worker import WorkerSettings as TrainSettings
    from app.worker_restaurant import WorkerRestaurantSettings as RestaurantSettings

    assert TrainSettings.queue_name == TRAIN_QUEUE_NAME
    assert RestaurantSettings.queue_name == RESTAURANT_QUEUE_NAME
    assert TrainSettings.queue_name != RestaurantSettings.queue_name


def test_worker_settings_use_non_cde_redis_dsn(monkeypatch):
    from app.core.config import get_settings

    monkeypatch.setenv("REDIS_URL", "redis://legacy:6379/0")
    monkeypatch.setenv("REDIS_URL_NON_CDE", "redis://non-cde:6380/2")
    get_settings.cache_clear()

    import app.worker as worker_module
    import app.worker_restaurant as worker_restaurant_module

    importlib.reload(worker_module)
    importlib.reload(worker_restaurant_module)

    train_redis_settings = worker_module.WorkerSettings.redis_settings
    restaurant_redis_settings = worker_restaurant_module.WorkerRestaurantSettings.redis_settings

    assert train_redis_settings.host == "non-cde"
    assert train_redis_settings.port == 6380
    assert train_redis_settings.database == 2
    assert restaurant_redis_settings.host == "non-cde"
    assert restaurant_redis_settings.port == 6380
    assert restaurant_redis_settings.database == 2

    assert train_redis_settings.retry_on_timeout is True
    assert train_redis_settings.conn_timeout >= 3
    assert train_redis_settings.conn_retries >= 5
    assert train_redis_settings.max_connections is not None and train_redis_settings.max_connections >= 100

    assert restaurant_redis_settings.retry_on_timeout is True
    assert restaurant_redis_settings.conn_timeout >= 3
    assert restaurant_redis_settings.conn_retries >= 5
    assert restaurant_redis_settings.max_connections is not None and restaurant_redis_settings.max_connections >= 50

    get_settings.cache_clear()
