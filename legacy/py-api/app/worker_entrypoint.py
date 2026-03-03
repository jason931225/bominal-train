from __future__ import annotations

import argparse
import asyncio
import importlib
from typing import Any

from arq.worker import run_worker


def _load_settings(settings_path: str) -> Any:
    if "." not in settings_path:
        raise ValueError("settings path must be in the form module.Class")

    module_path, class_name = settings_path.rsplit(".", 1)
    try:
        module = importlib.import_module(module_path)
    except ImportError as exc:
        raise ValueError(f"Could not import module '{module_path}'") from exc

    try:
        return getattr(module, class_name)
    except AttributeError as exc:
        raise ValueError(
            f"Could not load settings class '{class_name}' from '{module_path}'"
        ) from exc


def run(settings_path: str) -> None:
    settings_cls = _load_settings(settings_path)
    loop = asyncio.new_event_loop()
    try:
        asyncio.set_event_loop(loop)
        run_worker(settings_cls)
    finally:
        asyncio.set_event_loop(None)
        loop.close()


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Run arq worker with explicit event loop bootstrap for Python 3.14+.",
    )
    parser.add_argument(
        "settings_path",
        help="Worker settings class path, e.g. app.worker_train.WorkerTrainSettings",
    )
    args = parser.parse_args(argv)

    try:
        run(args.settings_path)
    except ValueError as exc:
        parser.error(str(exc))

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
