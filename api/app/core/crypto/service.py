from functools import lru_cache

from app.core.config import get_settings
from app.core.crypto.envelope import EnvelopeCrypto


@lru_cache
def get_envelope_crypto() -> EnvelopeCrypto:
    settings = get_settings()
    return EnvelopeCrypto(master_key_b64=settings.master_key, kek_version=settings.kek_version)
