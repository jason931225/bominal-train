import base64
import json
import os
from dataclasses import dataclass
from typing import Any

from cryptography.hazmat.primitives.ciphers.aead import AESGCM


def _b64encode(data: bytes) -> str:
    return base64.b64encode(data).decode("utf-8")


def _b64decode(data: str) -> bytes:
    return base64.b64decode(data.encode("utf-8"))


@dataclass(frozen=True)
class EncryptedSecret:
    ciphertext: str
    nonce: str
    wrapped_dek: str
    dek_nonce: str
    aad: str
    kek_version: int


class EnvelopeCrypto:
    """Envelope encryption helper.

    - Generates a random DEK for each record.
    - Encrypts payload with DEK using AES-256-GCM.
    - Wraps DEK using KEK (MASTER_KEY) with AES-256-GCM.
    """

    def __init__(self, master_key_b64: str, kek_version: int = 1):
        master_key = _b64decode(master_key_b64)
        if len(master_key) != 32:
            raise ValueError("MASTER_KEY must decode to 32 bytes")

        self._master_key = master_key
        self._kek_version = kek_version

    def encrypt_payload(self, payload: dict[str, Any], aad_text: str) -> EncryptedSecret:
        dek = os.urandom(32)
        payload_bytes = json.dumps(payload, separators=(",", ":"), ensure_ascii=True).encode("utf-8")

        aad_bytes = aad_text.encode("utf-8")

        payload_nonce = os.urandom(12)
        payload_ciphertext = AESGCM(dek).encrypt(payload_nonce, payload_bytes, aad_bytes)

        dek_nonce = os.urandom(12)
        wrapped_dek = AESGCM(self._master_key).encrypt(dek_nonce, dek, aad_bytes)

        return EncryptedSecret(
            ciphertext=_b64encode(payload_ciphertext),
            nonce=_b64encode(payload_nonce),
            wrapped_dek=_b64encode(wrapped_dek),
            dek_nonce=_b64encode(dek_nonce),
            aad=_b64encode(aad_bytes),
            kek_version=self._kek_version,
        )

    def decrypt_payload(
        self,
        *,
        ciphertext: str,
        nonce: str,
        wrapped_dek: str,
        dek_nonce: str,
        aad: str,
    ) -> dict[str, Any]:
        aad_bytes = _b64decode(aad)
        dek = AESGCM(self._master_key).decrypt(_b64decode(dek_nonce), _b64decode(wrapped_dek), aad_bytes)
        payload_bytes = AESGCM(dek).decrypt(_b64decode(nonce), _b64decode(ciphertext), aad_bytes)
        return json.loads(payload_bytes.decode("utf-8"))
