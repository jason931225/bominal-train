from functools import lru_cache

from app.core.config import get_settings
from app.core.crypto.envelope import EnvelopeCrypto
from app.core.crypto.master_key_resolver import resolve_master_key


@lru_cache
def get_envelope_crypto() -> EnvelopeCrypto:
    settings = get_settings()
    resolved_master_key = resolve_master_key(settings=settings)
    keyring = dict(settings.master_keys_by_version or {})
    if keyring:
        keyring.setdefault(settings.kek_version, resolved_master_key.master_key_b64)
        return EnvelopeCrypto(
            master_keys_b64_by_version=keyring,
            active_kek_version=settings.kek_version,
        )
    return EnvelopeCrypto(master_key_b64=resolved_master_key.master_key_b64, kek_version=settings.kek_version)
