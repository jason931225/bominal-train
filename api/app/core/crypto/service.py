from functools import lru_cache

from app.core.config import get_settings
from app.core.crypto.envelope import EnvelopeCrypto


@lru_cache
def get_envelope_crypto() -> EnvelopeCrypto:
    settings = get_settings()
    keyring = dict(settings.master_keys_by_version or {})
    if keyring:
        keyring.setdefault(settings.kek_version, settings.master_key)
        return EnvelopeCrypto(
            master_keys_b64_by_version=keyring,
            active_kek_version=settings.kek_version,
        )
    return EnvelopeCrypto(master_key_b64=settings.master_key, kek_version=settings.kek_version)
