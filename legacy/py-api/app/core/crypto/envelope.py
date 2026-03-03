import base64
import json
import os
from dataclasses import dataclass
from typing import Any, Mapping

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
    - Supports optional keyring by version for rotation-safe decrypt.
    """

    def __init__(
        self,
        master_key_b64: str | None = None,
        kek_version: int = 1,
        *,
        master_keys_b64_by_version: Mapping[int, str] | None = None,
        active_kek_version: int | None = None,
    ):
        keys: dict[int, bytes] = {}

        if master_keys_b64_by_version:
            for version, encoded in master_keys_b64_by_version.items():
                decoded = _b64decode(encoded)
                if len(decoded) != 32:
                    raise ValueError(f"MASTER_KEY for version {version} must decode to 32 bytes")
                keys[int(version)] = decoded
        elif master_key_b64 is not None:
            master_key = _b64decode(master_key_b64)
            if len(master_key) != 32:
                raise ValueError("MASTER_KEY must decode to 32 bytes")
            keys[int(kek_version)] = master_key
        else:
            raise ValueError("master_key_b64 or master_keys_b64_by_version must be provided")

        resolved_active = int(active_kek_version if active_kek_version is not None else kek_version)
        if resolved_active not in keys:
            raise ValueError("active KEK version must exist in keyring")

        self._keys = keys
        self._kek_version = resolved_active

    @property
    def kek_version(self) -> int:
        return self._kek_version

    def encrypt_payload(self, payload: dict[str, Any], aad_text: str) -> EncryptedSecret:
        dek = os.urandom(32)
        payload_bytes = json.dumps(payload, separators=(",", ":"), ensure_ascii=True).encode("utf-8")

        aad_bytes = aad_text.encode("utf-8")

        payload_nonce = os.urandom(12)
        payload_ciphertext = AESGCM(dek).encrypt(payload_nonce, payload_bytes, aad_bytes)

        dek_nonce = os.urandom(12)
        wrapped_dek = AESGCM(self._keys[self._kek_version]).encrypt(dek_nonce, dek, aad_bytes)

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
        kek_version: int | None = None,
        enforce_kek_version: bool = False,
    ) -> dict[str, Any]:
        if enforce_kek_version and kek_version is None:
            raise ValueError("kek_version is required when enforce_kek_version=True")

        version = int(kek_version if kek_version is not None else self._kek_version)
        key = self._keys.get(version)
        if key is None:
            raise ValueError(f"Unknown kek_version: {version}")

        aad_bytes = _b64decode(aad)
        dek = AESGCM(key).decrypt(_b64decode(dek_nonce), _b64decode(wrapped_dek), aad_bytes)
        payload_bytes = AESGCM(dek).decrypt(_b64decode(nonce), _b64decode(ciphertext), aad_bytes)

        try:
            return json.loads(payload_bytes.decode("utf-8"))
        finally:
            # Best-effort lifetime reduction for sensitive references.
            del dek
            del payload_bytes
