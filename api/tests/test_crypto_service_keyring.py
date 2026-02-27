from __future__ import annotations

import base64
from types import SimpleNamespace

from app.core.crypto import service as crypto_service


def test_get_envelope_crypto_uses_default_master_key_branch(monkeypatch):
    crypto_service.get_envelope_crypto.cache_clear()
    key = base64.b64encode(b"a" * 32).decode("utf-8")
    settings = SimpleNamespace(
        kek_version=3,
        master_keys_by_version=None,
    )
    monkeypatch.setattr(crypto_service, "get_settings", lambda: settings)
    monkeypatch.setattr(
        crypto_service,
        "resolve_master_key",
        lambda *, settings: SimpleNamespace(master_key_b64=key),
    )
    crypto = crypto_service.get_envelope_crypto()
    encrypted = crypto.encrypt_payload({"x": 1}, "kind:user")
    assert encrypted.kek_version == 3
    roundtrip = crypto.decrypt_payload(
        ciphertext=encrypted.ciphertext,
        nonce=encrypted.nonce,
        wrapped_dek=encrypted.wrapped_dek,
        dek_nonce=encrypted.dek_nonce,
        aad=encrypted.aad,
        kek_version=3,
        enforce_kek_version=True,
    )
    assert roundtrip == {"x": 1}


def test_get_envelope_crypto_uses_keyring_branch_and_fallback_injection(monkeypatch):
    crypto_service.get_envelope_crypto.cache_clear()
    old_key = base64.b64encode(b"b" * 32).decode("utf-8")
    new_key = base64.b64encode(b"c" * 32).decode("utf-8")
    settings = SimpleNamespace(
        kek_version=2,
        master_keys_by_version={1: old_key},
    )
    monkeypatch.setattr(crypto_service, "get_settings", lambda: settings)
    monkeypatch.setattr(
        crypto_service,
        "resolve_master_key",
        lambda *, settings: SimpleNamespace(master_key_b64=new_key),
    )
    crypto = crypto_service.get_envelope_crypto()

    old_crypto = crypto_service.EnvelopeCrypto(  # type: ignore[attr-defined]
        master_keys_b64_by_version={1: old_key, 2: new_key},
        active_kek_version=1,
    )
    old_encrypted = old_crypto.encrypt_payload({"legacy": True}, "kind:user")
    decrypted = crypto.decrypt_payload(
        ciphertext=old_encrypted.ciphertext,
        nonce=old_encrypted.nonce,
        wrapped_dek=old_encrypted.wrapped_dek,
        dek_nonce=old_encrypted.dek_nonce,
        aad=old_encrypted.aad,
        kek_version=1,
        enforce_kek_version=True,
    )
    assert decrypted == {"legacy": True}
