from __future__ import annotations

import pytest

from app.core.crypto.envelope import EnvelopeCrypto

MASTER_KEY_B64 = "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY="


def test_rejects_mismatched_kek_version_when_enforced() -> None:
    crypto = EnvelopeCrypto(master_key_b64=MASTER_KEY_B64, kek_version=2)
    encrypted = crypto.encrypt_payload(payload={"x": 1}, aad_text="unit:test")

    with pytest.raises(ValueError, match="kek_version"):
        crypto.decrypt_payload(
            ciphertext=encrypted.ciphertext,
            nonce=encrypted.nonce,
            wrapped_dek=encrypted.wrapped_dek,
            dek_nonce=encrypted.dek_nonce,
            aad=encrypted.aad,
            kek_version=1,
            enforce_kek_version=True,
        )


def test_accepts_matching_kek_version_when_enforced() -> None:
    crypto = EnvelopeCrypto(master_key_b64=MASTER_KEY_B64, kek_version=2)
    encrypted = crypto.encrypt_payload(payload={"x": 1}, aad_text="unit:test")

    payload = crypto.decrypt_payload(
        ciphertext=encrypted.ciphertext,
        nonce=encrypted.nonce,
        wrapped_dek=encrypted.wrapped_dek,
        dek_nonce=encrypted.dek_nonce,
        aad=encrypted.aad,
        kek_version=2,
        enforce_kek_version=True,
    )

    assert payload == {"x": 1}


def test_requires_kek_version_if_enforcement_enabled() -> None:
    crypto = EnvelopeCrypto(master_key_b64=MASTER_KEY_B64, kek_version=3)
    encrypted = crypto.encrypt_payload(payload={"x": 1}, aad_text="unit:test")

    with pytest.raises(ValueError, match="kek_version"):
        crypto.decrypt_payload(
            ciphertext=encrypted.ciphertext,
            nonce=encrypted.nonce,
            wrapped_dek=encrypted.wrapped_dek,
            dek_nonce=encrypted.dek_nonce,
            aad=encrypted.aad,
            enforce_kek_version=True,
        )


def test_backward_compatible_when_enforcement_disabled() -> None:
    crypto = EnvelopeCrypto(master_key_b64=MASTER_KEY_B64, kek_version=4)
    encrypted = crypto.encrypt_payload(payload={"x": "ok"}, aad_text="unit:test")

    payload = crypto.decrypt_payload(
        ciphertext=encrypted.ciphertext,
        nonce=encrypted.nonce,
        wrapped_dek=encrypted.wrapped_dek,
        dek_nonce=encrypted.dek_nonce,
        aad=encrypted.aad,
    )

    assert payload == {"x": "ok"}


def test_unicode_round_trip_stability() -> None:
    crypto = EnvelopeCrypto(master_key_b64=MASTER_KEY_B64, kek_version=1)
    encrypted = crypto.encrypt_payload(payload={"greeting": "안녕하세요", "emoji": "x"}, aad_text="unit:test")

    payload = crypto.decrypt_payload(
        ciphertext=encrypted.ciphertext,
        nonce=encrypted.nonce,
        wrapped_dek=encrypted.wrapped_dek,
        dek_nonce=encrypted.dek_nonce,
        aad=encrypted.aad,
    )

    assert payload == {"greeting": "안녕하세요", "emoji": "x"}
