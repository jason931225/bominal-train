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
    from arq.connections import RedisSettings

    monkeypatch.setenv("REDIS_URL", "redis://legacy:6379/0")
    monkeypatch.setenv("REDIS_URL_NON_CDE", "redis://non-cde:6380/2")
    get_settings.cache_clear()

    captured: list[str] = []
    original_from_dsn = RedisSettings.from_dsn

    def _capture_from_dsn(*args, **kwargs):  # noqa: ANN001, ANN002, ANN003
        dsn = kwargs.get("dsn")
        if dsn is None and args:
            dsn = args[-1]
        dsn = str(dsn)
        captured.append(dsn)
        return original_from_dsn(dsn)

    monkeypatch.setattr(RedisSettings, "from_dsn", _capture_from_dsn)

    import app.worker as worker_module
    import app.worker_restaurant as worker_restaurant_module

    importlib.reload(worker_module)
    importlib.reload(worker_restaurant_module)

    assert "redis://non-cde:6380/2" in captured

    get_settings.cache_clear()
