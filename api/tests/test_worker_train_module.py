from __future__ import annotations

from app.worker import WorkerSettings
from app.worker_train import WorkerTrainSettings


def test_worker_train_reexports_worker_settings() -> None:
    assert WorkerTrainSettings is WorkerSettings
