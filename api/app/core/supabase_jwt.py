from __future__ import annotations

from functools import lru_cache
from typing import Any

import jwt
from jwt import PyJWKClient

from app.core.config import get_settings


class SupabaseJWTError(RuntimeError):
    pass


@lru_cache(maxsize=4)
def _jwk_client(jwks_url: str) -> PyJWKClient:
    return PyJWKClient(jwks_url)


def verify_supabase_jwt(token: str) -> dict[str, Any]:
    settings = get_settings()
    jwks_url = settings.resolved_supabase_jwks_url
    if not jwks_url:
        raise SupabaseJWTError("Supabase JWKS URL is not configured")

    if not settings.supabase_jwt_issuer:
        raise SupabaseJWTError("SUPABASE_JWT_ISSUER is not configured")

    try:
        signing_key = _jwk_client(jwks_url).get_signing_key_from_jwt(token)
        claims = jwt.decode(
            token,
            signing_key.key,
            algorithms=["RS256"],
            audience=settings.supabase_jwt_audience,
            issuer=settings.supabase_jwt_issuer,
            options={"require": ["exp", "iat", "sub"]},
        )
    except Exception as exc:
        raise SupabaseJWTError("Invalid Supabase JWT") from exc

    if not claims.get("sub"):
        raise SupabaseJWTError("Supabase JWT missing sub claim")

    return claims
