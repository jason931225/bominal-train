from __future__ import annotations

import runpy
import sys

import pytest

from app import worker_entrypoint
from app.worker_train import WorkerTrainSettings


def test_load_settings_resolves_valid_path() -> None:
    loaded = worker_entrypoint._load_settings("app.worker_train.WorkerTrainSettings")
    assert loaded is WorkerTrainSettings


def test_load_settings_rejects_invalid_paths() -> None:
    with pytest.raises(ValueError, match="module.Class"):
        worker_entrypoint._load_settings("invalid")

    with pytest.raises(ValueError, match="Could not import module"):
        worker_entrypoint._load_settings("app.not_a_module.WorkerTrainSettings")

    with pytest.raises(ValueError, match="Could not load settings class"):
        worker_entrypoint._load_settings("app.worker_train.DoesNotExist")


def test_run_bootstraps_event_loop_and_calls_run_worker(monkeypatch: pytest.MonkeyPatch) -> None:
    calls: dict[str, object] = {}

    class _Loop:
        def __init__(self) -> None:
            self.closed = False

        def close(self) -> None:
            self.closed = True

    loop = _Loop()
    settings_sentinel = object()

    monkeypatch.setattr(worker_entrypoint, "_load_settings", lambda _path: settings_sentinel)
    monkeypatch.setattr(worker_entrypoint.asyncio, "new_event_loop", lambda: loop)
    monkeypatch.setattr(
        worker_entrypoint.asyncio,
        "set_event_loop",
        lambda value: calls.setdefault("set_event_loop_calls", []).append(value),
    )
    monkeypatch.setattr(worker_entrypoint, "run_worker", lambda value: calls.setdefault("run_worker_arg", value))

    worker_entrypoint.run("app.worker_train.WorkerTrainSettings")

    assert calls["run_worker_arg"] is settings_sentinel
    assert calls["set_event_loop_calls"] == [loop, None]
    assert loop.closed is True


def test_main_passes_settings_path_to_run(monkeypatch: pytest.MonkeyPatch) -> None:
    observed: dict[str, str] = {}
    monkeypatch.setattr(worker_entrypoint, "run", lambda value: observed.setdefault("settings_path", value))
    assert worker_entrypoint.main(["app.worker_train.WorkerTrainSettings"]) == 0
    assert observed["settings_path"] == "app.worker_train.WorkerTrainSettings"


def test_main_reports_invalid_settings(monkeypatch: pytest.MonkeyPatch) -> None:
    def _boom(_value: str) -> None:
        raise ValueError("boom")

    monkeypatch.setattr(worker_entrypoint, "run", _boom)
    with pytest.raises(SystemExit):
        worker_entrypoint.main(["app.worker_train.WorkerTrainSettings"])


def test_module_entrypoint_requires_args(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(sys, "argv", ["worker_entrypoint"])
    sys.modules.pop("app.worker_entrypoint", None)
    with pytest.raises(SystemExit):
        runpy.run_module("app.worker_entrypoint", run_name="__main__")
