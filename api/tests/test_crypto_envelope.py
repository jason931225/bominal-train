import base64

import pytest

from app.core.crypto.envelope import EnvelopeCrypto


@pytest.fixture
def master_key_b64() -> str:
    return base64.b64encode(b"a" * 32).decode("utf-8")


def test_encrypt_decrypt_with_matching_kek_version(master_key_b64: str) -> None:
    crypto = EnvelopeCrypto(master_key_b64=master_key_b64, kek_version=3)
    encrypted = crypto.encrypt_payload(payload={"x": "y"}, aad_text="secret:1")

    decrypted = crypto.decrypt_payload(
        ciphertext=encrypted.ciphertext,
        nonce=encrypted.nonce,
        wrapped_dek=encrypted.wrapped_dek,
        dek_nonce=encrypted.dek_nonce,
        aad=encrypted.aad,
        kek_version=encrypted.kek_version,
        enforce_kek_version=True,
    )

    assert decrypted == {"x": "y"}


def test_decrypt_requires_kek_version_when_enforced(master_key_b64: str) -> None:
    crypto = EnvelopeCrypto(master_key_b64=master_key_b64, kek_version=1)
    encrypted = crypto.encrypt_payload(payload={"x": 1}, aad_text="secret:2")

    with pytest.raises(ValueError):
        crypto.decrypt_payload(
            ciphertext=encrypted.ciphertext,
            nonce=encrypted.nonce,
            wrapped_dek=encrypted.wrapped_dek,
            dek_nonce=encrypted.dek_nonce,
            aad=encrypted.aad,
            enforce_kek_version=True,
        )


def test_decrypt_fails_on_kek_version_mismatch_single_key(master_key_b64: str) -> None:
    crypto = EnvelopeCrypto(master_key_b64=master_key_b64, kek_version=1)
    encrypted = crypto.encrypt_payload(payload={"x": 1}, aad_text="secret:3")

    with pytest.raises(ValueError):
        crypto.decrypt_payload(
            ciphertext=encrypted.ciphertext,
            nonce=encrypted.nonce,
            wrapped_dek=encrypted.wrapped_dek,
            dek_nonce=encrypted.dek_nonce,
            aad=encrypted.aad,
            kek_version=2,
            enforce_kek_version=True,
        )
