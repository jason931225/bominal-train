from __future__ import annotations


def test_worker_settings_have_isolated_queue_names():
    from app.worker import WorkerSettings as TrainSettings
    from app.worker_restaurant import WorkerRestaurantSettings as RestaurantSettings

    assert getattr(TrainSettings, "queue_name", "arq:queue") != RestaurantSettings.queue_name
    assert RestaurantSettings.queue_name == "restaurant:queue"

