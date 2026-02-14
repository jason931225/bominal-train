from __future__ import annotations

from app.core.queue_domains import RESTAURANT_QUEUE_NAME, TRAIN_QUEUE_NAME


def test_worker_settings_have_isolated_queue_names():
    from app.worker import WorkerSettings as TrainSettings
    from app.worker_restaurant import WorkerRestaurantSettings as RestaurantSettings

    assert TrainSettings.queue_name == TRAIN_QUEUE_NAME
    assert RestaurantSettings.queue_name == RESTAURANT_QUEUE_NAME
    assert TrainSettings.queue_name != RestaurantSettings.queue_name
