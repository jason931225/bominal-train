from __future__ import annotations

import base64
from types import SimpleNamespace

import pytest

from app.core.crypto import master_key_resolver as resolver

_VALID_MASTER_KEY = base64.b64encode(b"k" * 32).decode("utf-8")


def _settings(**overrides):
    defaults = {
        "master_key_override": None,
        "master_key": _VALID_MASTER_KEY,
        "gsm_master_key_enabled": False,
        "resolved_gsm_master_key_project_id": "test-project",
        "gsm_master_key_secret_id": "bominal-master-key",
        "gsm_master_key_version": "1",
        "gsm_master_key_allow_env_fallback": False,
    }
    defaults.update(overrides)
    return SimpleNamespace(**defaults)


def test_resolve_master_key_prefers_override() -> None:
    settings = _settings(master_key_override=_VALID_MASTER_KEY, master_key="")

    resolved = resolver.resolve_master_key(settings=settings)

    assert resolved.source == "deploy_override"
    assert resolved.master_key_b64 == _VALID_MASTER_KEY


def test_resolve_master_key_uses_gsm_when_enabled(monkeypatch) -> None:
    settings = _settings(master_key="", gsm_master_key_enabled=True)

    monkeypatch.setattr(
        resolver,
        "_fetch_master_key_from_secret_manager",
        lambda *, project_id, secret_id, version: _VALID_MASTER_KEY,
    )

    resolved = resolver.resolve_master_key(settings=settings)

    assert resolved.source == "gcp_secret_manager"
    assert resolved.project_id == "test-project"
    assert resolved.secret_id == "bominal-master-key"
    assert resolved.secret_version == "1"


def test_resolve_master_key_uses_env_fallback_when_enabled(monkeypatch) -> None:
    settings = _settings(
        master_key=_VALID_MASTER_KEY,
        gsm_master_key_enabled=True,
        gsm_master_key_allow_env_fallback=True,
    )

    def _raise(*, project_id, secret_id, version):
        raise RuntimeError("gsm-down")

    monkeypatch.setattr(resolver, "_fetch_master_key_from_secret_manager", _raise)

    resolved = resolver.resolve_master_key(settings=settings)

    assert resolved.source == "env_fallback"
    assert resolved.master_key_b64 == _VALID_MASTER_KEY


def test_resolve_master_key_raises_when_gsm_fails_without_fallback(monkeypatch) -> None:
    settings = _settings(master_key=_VALID_MASTER_KEY, gsm_master_key_enabled=True)

    def _raise(*, project_id, secret_id, version):
        raise RuntimeError("gsm-down")

    monkeypatch.setattr(resolver, "_fetch_master_key_from_secret_manager", _raise)

    with pytest.raises(resolver.MasterKeyResolutionError, match="Failed to resolve MASTER_KEY"):
        resolver.resolve_master_key(settings=settings)


def test_resolve_master_key_rejects_invalid_base64() -> None:
    settings = _settings(master_key="not-base64")

    with pytest.raises(resolver.MasterKeyResolutionError, match="not valid base64"):
        resolver.resolve_master_key(settings=settings)
