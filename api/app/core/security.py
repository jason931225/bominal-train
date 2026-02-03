import hashlib
import secrets
from datetime import datetime, timedelta, timezone

from argon2 import PasswordHasher
from argon2.exceptions import VerificationError

from app.core.config import get_settings

settings = get_settings()

# Argon2id with conservative interactive defaults.
password_hasher = PasswordHasher(
    time_cost=3,
    memory_cost=65536,
    parallelism=4,
    hash_len=32,
    salt_len=16,
)


def hash_password(password: str) -> str:
    return password_hasher.hash(password)


def verify_password(password: str, password_hash: str) -> bool:
    try:
        return password_hasher.verify(password_hash, password)
    except VerificationError:
        return False


def new_session_token() -> str:
    return secrets.token_urlsafe(48)


def hash_token(token: str) -> str:
    return hashlib.sha256(token.encode("utf-8")).hexdigest()


def session_expiry(remember_me: bool) -> datetime:
    days = settings.session_days_remember if remember_me else settings.session_days_default
    return datetime.now(timezone.utc) + timedelta(days=days)
