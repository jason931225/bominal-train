from __future__ import annotations

from typing import Any
from uuid import UUID

from app.core.crypto.service import get_envelope_crypto
from app.db.models import Secret


def build_encrypted_secret(*, user_id: UUID, kind: str, payload: dict[str, Any]) -> Secret:
    aad_text = f"{kind}:{user_id}"
    encrypted = get_envelope_crypto().encrypt_payload(payload=payload, aad_text=aad_text)
    return Secret(
        user_id=user_id,
        kind=kind,
        ciphertext=encrypted.ciphertext,
        nonce=encrypted.nonce,
        wrapped_dek=encrypted.wrapped_dek,
        dek_nonce=encrypted.dek_nonce,
        aad=encrypted.aad,
        kek_version=encrypted.kek_version,
    )


def decrypt_secret(secret: Secret) -> dict[str, Any]:
    return get_envelope_crypto().decrypt_payload(
        ciphertext=secret.ciphertext,
        nonce=secret.nonce,
        wrapped_dek=secret.wrapped_dek,
        dek_nonce=secret.dek_nonce,
        aad=secret.aad,
        kek_version=secret.kek_version,
        enforce_kek_version=True,
    )
