from __future__ import annotations

import base64
import hashlib
import hmac
import json
import secrets
import time
from dataclasses import dataclass
from typing import Any

from app.core.config import get_settings


class InternalIdentityError(ValueError):
    """Raised when internal service token validation fails."""


def _b64url_encode(data: bytes) -> str:
    return base64.urlsafe_b64encode(data).decode("ascii").rstrip("=")


def _b64url_decode(data: str) -> bytes:
    pad = "=" * ((4 - len(data) % 4) % 4)
    return base64.urlsafe_b64decode((data + pad).encode("ascii"))


def _json_dumps(payload: dict[str, Any]) -> str:
    return json.dumps(payload, separators=(",", ":"), ensure_ascii=True, sort_keys=True)


def _sign(message: bytes, *, secret: str) -> bytes:
    return hmac.new(secret.encode("utf-8"), message, hashlib.sha256).digest()


def _now_epoch_seconds() -> int:
    return int(time.time())


@dataclass(frozen=True)
class InternalTokenClaims:
    iss: str
    sub: str
    aud: str
    iat: int
    exp: int
    jti: str


def mint_internal_service_token(
    *,
    subject: str,
    audience: str = "internal-api",
    ttl_seconds: int | None = None,
    now_epoch_seconds: int | None = None,
) -> str:
    settings = get_settings()
    secret = (settings.internal_identity_secret or "").strip()
    if not secret:
        raise InternalIdentityError("INTERNAL_IDENTITY_SECRET is not configured")

    now = int(now_epoch_seconds if now_epoch_seconds is not None else _now_epoch_seconds())
    ttl = int(ttl_seconds if ttl_seconds is not None else settings.internal_identity_ttl_seconds)
    if ttl < 1:
        raise InternalIdentityError("internal identity ttl must be >= 1 second")

    claims = {
        "iss": settings.internal_identity_issuer,
        "sub": str(subject),
        "aud": str(audience),
        "iat": now,
        "exp": now + ttl,
        "jti": secrets.token_hex(16),
    }
    header = {"alg": "HS256", "typ": "BIT"}
    header_segment = _b64url_encode(_json_dumps(header).encode("utf-8"))
    claims_segment = _b64url_encode(_json_dumps(claims).encode("utf-8"))
    signing_input = f"{header_segment}.{claims_segment}".encode("ascii")
    signature_segment = _b64url_encode(_sign(signing_input, secret=secret))
    return f"{header_segment}.{claims_segment}.{signature_segment}"


def verify_internal_service_token(
    token: str,
    *,
    expected_audience: str = "internal-api",
    now_epoch_seconds: int | None = None,
) -> InternalTokenClaims:
    settings = get_settings()
    secret = (settings.internal_identity_secret or "").strip()
    if not secret:
        raise InternalIdentityError("INTERNAL_IDENTITY_SECRET is not configured")

    parts = token.split(".")
    if len(parts) != 3:
        raise InternalIdentityError("invalid token format")
    header_segment, claims_segment, signature_segment = parts

    try:
        header = json.loads(_b64url_decode(header_segment).decode("utf-8"))
        claims = json.loads(_b64url_decode(claims_segment).decode("utf-8"))
    except Exception as exc:
        raise InternalIdentityError("invalid token encoding") from exc

    if str(header.get("alg") or "") != "HS256":
        raise InternalIdentityError("unsupported token algorithm")

    signing_input = f"{header_segment}.{claims_segment}".encode("ascii")
    expected_signature = _b64url_encode(_sign(signing_input, secret=secret))
    if not hmac.compare_digest(signature_segment, expected_signature):
        raise InternalIdentityError("invalid token signature")

    issuer = str(claims.get("iss") or "")
    subject = str(claims.get("sub") or "")
    audience = str(claims.get("aud") or "")
    jti = str(claims.get("jti") or "")
    try:
        iat = int(claims.get("iat"))
        exp = int(claims.get("exp"))
    except Exception as exc:
        raise InternalIdentityError("invalid token timing claims") from exc

    if not issuer or issuer != settings.internal_identity_issuer:
        raise InternalIdentityError("invalid token issuer")
    if not subject:
        raise InternalIdentityError("invalid token subject")
    if audience != expected_audience:
        raise InternalIdentityError("invalid token audience")
    if not jti:
        raise InternalIdentityError("invalid token identifier")

    now = int(now_epoch_seconds if now_epoch_seconds is not None else _now_epoch_seconds())
    if exp <= now:
        raise InternalIdentityError("token expired")
    if iat > now + 60:
        raise InternalIdentityError("token issued in the future")
    if exp <= iat:
        raise InternalIdentityError("invalid token expiry")

    return InternalTokenClaims(
        iss=issuer,
        sub=subject,
        aud=audience,
        iat=iat,
        exp=exp,
        jti=jti,
    )
