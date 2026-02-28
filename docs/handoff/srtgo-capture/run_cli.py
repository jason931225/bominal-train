#!/usr/bin/env python3
from __future__ import annotations

import os
import sys
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parent
VENDOR = ROOT / "vendor"

if str(VENDOR) not in sys.path:
    sys.path.insert(0, str(VENDOR))

from srtgo_capture.capture_runtime import install_capture


def _install_keyring_fallback() -> None:
    """Wrap keyring calls with an in-memory fallback for flaky local backends."""
    try:
        import keyring
    except ModuleNotFoundError:
        return

    original_get = keyring.get_password
    original_set = keyring.set_password
    original_delete = keyring.delete_password

    memory_store: dict[tuple[str, str], str] = {}
    fallback_mode = False
    warned = False

    def activate_fallback() -> None:
        nonlocal fallback_mode, warned
        fallback_mode = True
        if not warned:
            print(
                "[capture] keyring backend error detected; using in-memory keyring fallback for this run."
            )
            warned = True

    def safe_set(service: str, username: str, password: str) -> None:
        if fallback_mode:
            memory_store[(service, username)] = password
            return
        try:
            original_set(service, username, password)
            memory_store[(service, username)] = password
        except Exception:
            activate_fallback()
            memory_store[(service, username)] = password

    def safe_get(service: str, username: str) -> str | None:
        if fallback_mode:
            return memory_store.get((service, username))
        try:
            value = original_get(service, username)
            if value is not None:
                memory_store[(service, username)] = value
                return value
            return memory_store.get((service, username))
        except Exception:
            activate_fallback()
            return memory_store.get((service, username))

    def safe_delete(service: str, username: str) -> None:
        memory_store.pop((service, username), None)
        if fallback_mode:
            return
        try:
            original_delete(service, username)
        except Exception:
            activate_fallback()

    # Fast probe to avoid first-run crash paths on known-broken backends.
    probe_service = "srtgo_capture_probe"
    probe_user = f"pid-{os.getpid()}"
    probe_value = uuid.uuid4().hex
    try:
        original_set(probe_service, probe_user, probe_value)
        original_get(probe_service, probe_user)
        original_delete(probe_service, probe_user)
    except Exception:
        activate_fallback()

    keyring.set_password = safe_set
    keyring.get_password = safe_get
    keyring.delete_password = safe_delete


def main() -> None:
    output_root = os.environ.get("SRTGO_CAPTURE_OUTPUT_DIR")
    _install_keyring_fallback()
    try:
        recorder = install_capture(output_root=output_root or ROOT / "output")
    except RuntimeError as exc:
        raise SystemExit(
            f"[capture] {exc}\n"
            "Install dependencies: "
            "python -m pip install click curl_cffi requests inquirer keyring PyCryptodome prompt_toolkit python-telegram-bot termcolor"
        ) from exc

    print(f"[capture] run_id={recorder.run_id}")
    print(f"[capture] output={recorder.run_dir}")

    try:
        from srtgo_capture.srtgo import srtgo
    except ModuleNotFoundError as exc:
        missing = exc.name or "unknown"
        raise SystemExit(
            f"[capture] Missing dependency: {missing}\n"
            "Install dependencies: "
            "python -m pip install click curl_cffi requests inquirer keyring PyCryptodome prompt_toolkit python-telegram-bot termcolor"
        ) from exc

    # Keep original srtgo click CLI UX intact.
    srtgo()


if __name__ == "__main__":
    main()
