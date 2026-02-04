"""Security utilities for authentication and session management.

Provides:
- Password hashing with Argon2id (OWASP recommended)
- Session token generation and hashing
- Session expiry calculation

Security notes:
- Passwords are hashed with Argon2id (memory-hard, GPU-resistant)
- Session tokens use cryptographically secure randomness
- Token hashes stored in DB, not raw tokens
"""

import hashlib
import secrets
from datetime import datetime, timedelta, timezone

from argon2 import PasswordHasher
from argon2.exceptions import VerificationError

from app.core.config import get_settings

settings = get_settings()

# Argon2id with conservative interactive defaults per OWASP guidelines.
# time_cost=3, memory_cost=64MB provides good security/performance balance.
password_hasher = PasswordHasher(
    time_cost=3,
    memory_cost=65536,
    parallelism=4,
    hash_len=32,
    salt_len=16,
)


def hash_password(password: str) -> str:
    """Hash a password using Argon2id.
    
    Args:
        password: Plain text password to hash.
        
    Returns:
        Argon2id hash string including salt and parameters.
    """
    return password_hasher.hash(password)


def verify_password(password: str, password_hash: str) -> bool:
    """Verify a password against its Argon2id hash.
    
    Args:
        password: Plain text password to verify.
        password_hash: Argon2id hash to verify against.
        
    Returns:
        True if password matches, False otherwise.
    """
    try:
        return password_hasher.verify(password_hash, password)
    except VerificationError:
        return False


def new_session_token() -> str:
    """Generate a cryptographically secure session token.
    
    Returns:
        64-character URL-safe base64 token (384 bits of entropy).
    """
    return secrets.token_urlsafe(48)


def hash_token(token: str) -> str:
    """Hash a session token for storage.
    
    Uses SHA-256 since session tokens already have high entropy.
    
    Args:
        token: Session token to hash.
        
    Returns:
        Hex-encoded SHA-256 hash.
    """
    return hashlib.sha256(token.encode("utf-8")).hexdigest()


def session_expiry(remember_me: bool) -> datetime:
    """Calculate session expiry timestamp.
    
    Args:
        remember_me: If True, use extended session duration.
        
    Returns:
        UTC datetime when session should expire.
    """
    days = settings.session_days_remember if remember_me else settings.session_days_default
    return datetime.now(timezone.utc) + timedelta(days=days)
