from __future__ import annotations

import base64
import inspect
import json
import secrets
from datetime import timedelta
from typing import Any
from uuid import UUID

from fastapi import HTTPException, status
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.config import get_settings
from app.core.security import hash_token
from app.db.models import AuthChallenge, PasskeyCredential, User
from app.services.auth import utc_now

settings = get_settings()

PASSKEY_CHALLENGE_TTL_SECONDS = 300
PASSKEY_PURPOSE_REGISTER = "passkey_register"
PASSKEY_PURPOSE_AUTH = "passkey_auth"
PASSKEY_PURPOSE_STEP_UP = "passkey_step_up"
PASSKEY_STEP_UP_TTL_SECONDS = 300


class PasskeyRuntimeError(RuntimeError):
    pass


def _load_webauthn_runtime() -> Any:
    try:
        from webauthn import (  # type: ignore[import-not-found]
            base64url_to_bytes,
            generate_authentication_options,
            generate_registration_options,
            options_to_json,
            verify_authentication_response,
            verify_registration_response,
        )
        from webauthn.helpers import structs as webauthn_structs  # type: ignore[import-not-found]
    except Exception as exc:  # pragma: no cover - dependency availability varies by runtime image
        raise PasskeyRuntimeError("Passkey runtime is unavailable") from exc

    PublicKeyCredentialDescriptor = webauthn_structs.PublicKeyCredentialDescriptor
    PublicKeyCredentialType = getattr(webauthn_structs, "PublicKeyCredentialType", None)
    UserVerificationRequirement = webauthn_structs.UserVerificationRequirement
    AuthenticatorSelectionCriteria = getattr(webauthn_structs, "AuthenticatorSelectionCriteria", None)

    return {
        "base64url_to_bytes": base64url_to_bytes,
        "generate_authentication_options": generate_authentication_options,
        "generate_registration_options": generate_registration_options,
        "options_to_json": options_to_json,
        "verify_authentication_response": verify_authentication_response,
        "verify_registration_response": verify_registration_response,
        "PublicKeyCredentialDescriptor": PublicKeyCredentialDescriptor,
        "PublicKeyCredentialType": PublicKeyCredentialType,
        "UserVerificationRequirement": UserVerificationRequirement,
        "AuthenticatorSelectionCriteria": AuthenticatorSelectionCriteria,
    }


def _b64url_encode(data: bytes) -> str:
    return base64.urlsafe_b64encode(data).decode("ascii").rstrip("=")


def _b64url_decode(data: str) -> bytes:
    normalized = data + "=" * ((4 - len(data) % 4) % 4)
    return base64.urlsafe_b64decode(normalized.encode("ascii"))


def _challenge_bytes(runtime: dict[str, Any], challenge_b64url: str) -> bytes:
    decode = runtime.get("base64url_to_bytes")
    if callable(decode):
        return decode(challenge_b64url)
    return _b64url_decode(challenge_b64url)


def _bytes_to_b64url(data: bytes) -> str:
    return _b64url_encode(data)


def _effective_passkey_rp_id() -> str:
    explicit = (settings.passkey_rp_id or "").strip()
    if explicit:
        return explicit
    from urllib.parse import urlparse

    host = (urlparse(settings.app_public_base_url).hostname or "").strip()
    return host


def _effective_passkey_origin() -> str:
    explicit = (settings.passkey_origin or "").strip()
    if explicit:
        return explicit
    return settings.app_public_base_url.rstrip("/")


def ensure_passkeys_enabled() -> None:
    if not settings.passkey_enabled:
        raise HTTPException(status_code=status.HTTP_503_SERVICE_UNAVAILABLE, detail="Passkeys are disabled")
    if not _effective_passkey_rp_id():
        raise HTTPException(status_code=status.HTTP_500_INTERNAL_SERVER_ERROR, detail="PASSKEY_RP_ID is not configured")
    if not _effective_passkey_origin():
        raise HTTPException(status_code=status.HTTP_500_INTERNAL_SERVER_ERROR, detail="PASSKEY_ORIGIN is not configured")
    try:
        _load_webauthn_runtime()
    except PasskeyRuntimeError as exc:
        raise HTTPException(status_code=status.HTTP_503_SERVICE_UNAVAILABLE, detail="Passkey runtime unavailable") from exc


async def list_passkeys(db: AsyncSession, *, user_id: UUID) -> list[PasskeyCredential]:
    return (
        await db.execute(
            select(PasskeyCredential)
            .where(PasskeyCredential.user_id == user_id)
            .order_by(PasskeyCredential.created_at.desc())
        )
    ).scalars().all()


async def delete_passkey(db: AsyncSession, *, user_id: UUID, passkey_id: UUID) -> bool:
    credential = (
        await db.execute(
            select(PasskeyCredential)
            .where(PasskeyCredential.id == passkey_id)
            .where(PasskeyCredential.user_id == user_id)
        )
    ).scalar_one_or_none()
    if credential is None:
        return False
    await db.delete(credential)
    await db.commit()
    return True


def _descriptor(runtime: dict[str, Any], credential_id_b64url: str):
    descriptor_cls = runtime["PublicKeyCredentialDescriptor"]
    credential_type: Any = "public-key"
    credential_type_enum = runtime.get("PublicKeyCredentialType")
    if credential_type_enum is not None:
        credential_type = getattr(credential_type_enum, "PUBLIC_KEY", credential_type)
    return descriptor_cls(id=_challenge_bytes(runtime, credential_id_b64url), type=credential_type)


def _call_with_supported_kwargs(fn: Any, **kwargs: Any) -> Any:
    """Call third-party helpers defensively across runtime signature changes."""
    try:
        params = inspect.signature(fn).parameters
    except (TypeError, ValueError):
        return fn(**kwargs)
    if any(p.kind == inspect.Parameter.VAR_KEYWORD for p in params.values()):
        return fn(**kwargs)
    filtered = {k: v for k, v in kwargs.items() if k in params}
    return fn(**filtered)


async def _create_challenge(
    db: AsyncSession,
    *,
    purpose: str,
    challenge_b64url: str,
    user_id: UUID | None = None,
    email: str | None = None,
) -> AuthChallenge:
    challenge = AuthChallenge(
        user_id=user_id,
        email=email,
        purpose=purpose,
        challenge_hash=hash_token(challenge_b64url),
        challenge_b64url=challenge_b64url,
        expires_at=utc_now() + timedelta(seconds=PASSKEY_CHALLENGE_TTL_SECONDS),
    )
    db.add(challenge)
    await db.commit()
    await db.refresh(challenge)
    return challenge


async def _consume_challenge(
    db: AsyncSession,
    *,
    challenge_id: UUID,
    purpose: str,
    user_id: UUID | None = None,
    email: str | None = None,
) -> AuthChallenge:
    now = utc_now()
    stmt = (
        select(AuthChallenge)
        .where(AuthChallenge.id == challenge_id)
        .where(AuthChallenge.purpose == purpose)
        .where(AuthChallenge.used_at.is_(None))
        .where(AuthChallenge.expires_at > now)
    )
    if user_id is not None:
        stmt = stmt.where(AuthChallenge.user_id == user_id)
    if email is not None:
        stmt = stmt.where(AuthChallenge.email == email)
    challenge = (await db.execute(stmt)).scalar_one_or_none()
    if challenge is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Invalid or expired passkey challenge")
    return challenge


async def issue_passkey_step_up_token(db: AsyncSession, *, user_id: UUID) -> str:
    token = secrets.token_urlsafe(32)
    challenge = AuthChallenge(
        user_id=user_id,
        email=None,
        purpose=PASSKEY_PURPOSE_STEP_UP,
        challenge_hash=hash_token(token),
        challenge_b64url="[step-up-redacted]",
        expires_at=utc_now() + timedelta(seconds=PASSKEY_STEP_UP_TTL_SECONDS),
    )
    db.add(challenge)
    await db.commit()
    return token


async def consume_passkey_step_up_token(
    db: AsyncSession,
    *,
    user_id: UUID,
    token: str,
) -> bool:
    token_raw = token.strip()
    if not token_raw:
        return False
    now = utc_now()
    challenge = (
        await db.execute(
            select(AuthChallenge)
            .where(AuthChallenge.user_id == user_id)
            .where(AuthChallenge.purpose == PASSKEY_PURPOSE_STEP_UP)
            .where(AuthChallenge.challenge_hash == hash_token(token_raw))
            .where(AuthChallenge.used_at.is_(None))
            .where(AuthChallenge.expires_at > now)
        )
    ).scalar_one_or_none()
    if challenge is None:
        return False
    challenge.used_at = now
    return True


async def begin_passkey_registration(
    db: AsyncSession,
    *,
    user: User,
) -> tuple[UUID, dict[str, Any]]:
    runtime = _load_webauthn_runtime()
    existing_credentials = await list_passkeys(db, user_id=user.id)
    registration_kwargs: dict[str, Any] = {
        "rp_id": _effective_passkey_rp_id(),
        "rp_name": settings.app_name,
        "user_id": str(user.id).encode("utf-8"),
        "user_name": user.email,
        "user_display_name": user.display_name or user.email,
        "timeout": max(settings.passkey_timeout_ms, 1),
        "exclude_credentials": [_descriptor(runtime, item.credential_id) for item in existing_credentials],
    }
    uv_preferred = runtime["UserVerificationRequirement"].PREFERRED
    authenticator_selection_cls = runtime.get("AuthenticatorSelectionCriteria")
    if authenticator_selection_cls is not None:
        registration_kwargs["authenticator_selection"] = authenticator_selection_cls(user_verification=uv_preferred)
    else:
        # Older `webauthn` releases accepted this directly.
        registration_kwargs["user_verification"] = uv_preferred

    options = _call_with_supported_kwargs(runtime["generate_registration_options"], **registration_kwargs)
    public_key = json.loads(runtime["options_to_json"](options))
    challenge_b64url = str(public_key.get("challenge") or "").strip()
    if not challenge_b64url:
        raise HTTPException(status_code=status.HTTP_500_INTERNAL_SERVER_ERROR, detail="Passkey challenge generation failed")
    challenge = await _create_challenge(
        db,
        purpose=PASSKEY_PURPOSE_REGISTER,
        challenge_b64url=challenge_b64url,
        user_id=user.id,
    )
    return challenge.id, public_key


async def complete_passkey_registration(
    db: AsyncSession,
    *,
    user: User,
    challenge_id: UUID,
    credential: dict[str, Any],
) -> PasskeyCredential:
    runtime = _load_webauthn_runtime()
    challenge = await _consume_challenge(
        db,
        challenge_id=challenge_id,
        purpose=PASSKEY_PURPOSE_REGISTER,
        user_id=user.id,
    )
    try:
        verification = runtime["verify_registration_response"](
            credential=credential,
            expected_challenge=_challenge_bytes(runtime, challenge.challenge_b64url),
            expected_rp_id=_effective_passkey_rp_id(),
            expected_origin=_effective_passkey_origin(),
            require_user_verification=True,
        )
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Passkey registration verification failed") from exc

    credential_id = _bytes_to_b64url(verification.credential_id)
    existing = (
        await db.execute(select(PasskeyCredential).where(PasskeyCredential.credential_id == credential_id))
    ).scalar_one_or_none()
    if existing is not None and existing.user_id != user.id:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Passkey is already registered")

    now = utc_now()
    if existing is None:
        existing = PasskeyCredential(
            user_id=user.id,
            credential_id=credential_id,
            public_key=_bytes_to_b64url(verification.credential_public_key),
            sign_count=int(getattr(verification, "sign_count", 0)),
            transports=(credential.get("response", {}) or {}).get("transports"),
        )
        db.add(existing)
    else:
        existing.public_key = _bytes_to_b64url(verification.credential_public_key)
        existing.sign_count = int(getattr(verification, "sign_count", existing.sign_count))
        existing.updated_at = now

    challenge.used_at = now
    await db.commit()
    await db.refresh(existing)
    return existing


async def begin_passkey_authentication(
    db: AsyncSession,
    *,
    email: str,
    user: User,
) -> tuple[UUID, dict[str, Any]]:
    runtime = _load_webauthn_runtime()
    credentials = await list_passkeys(db, user_id=user.id)
    if not credentials:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="No passkey registered for this account")
    options = runtime["generate_authentication_options"](
        rp_id=_effective_passkey_rp_id(),
        timeout=max(settings.passkey_timeout_ms, 1),
        user_verification=runtime["UserVerificationRequirement"].PREFERRED,
        allow_credentials=[_descriptor(runtime, item.credential_id) for item in credentials],
    )
    public_key = json.loads(runtime["options_to_json"](options))
    challenge_b64url = str(public_key.get("challenge") or "").strip()
    if not challenge_b64url:
        raise HTTPException(status_code=status.HTTP_500_INTERNAL_SERVER_ERROR, detail="Passkey challenge generation failed")
    challenge = await _create_challenge(
        db,
        purpose=PASSKEY_PURPOSE_AUTH,
        challenge_b64url=challenge_b64url,
        user_id=user.id,
        email=email,
    )
    return challenge.id, public_key


async def complete_passkey_authentication(
    db: AsyncSession,
    *,
    email: str,
    user: User,
    challenge_id: UUID,
    credential: dict[str, Any],
) -> None:
    runtime = _load_webauthn_runtime()
    challenge = await _consume_challenge(
        db,
        challenge_id=challenge_id,
        purpose=PASSKEY_PURPOSE_AUTH,
        user_id=user.id,
        email=email,
    )
    credential_id = str(credential.get("id") or "").strip()
    if not credential_id:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Passkey credential id is required")

    passkey = (
        await db.execute(
            select(PasskeyCredential)
            .where(PasskeyCredential.user_id == user.id)
            .where(PasskeyCredential.credential_id == credential_id)
        )
    ).scalar_one_or_none()
    if passkey is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Unknown passkey credential")

    try:
        verification = runtime["verify_authentication_response"](
            credential=credential,
            expected_challenge=_challenge_bytes(runtime, challenge.challenge_b64url),
            expected_rp_id=_effective_passkey_rp_id(),
            expected_origin=_effective_passkey_origin(),
            credential_public_key=_challenge_bytes(runtime, passkey.public_key),
            credential_current_sign_count=int(passkey.sign_count),
            require_user_verification=True,
        )
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Passkey authentication failed") from exc

    now = utc_now()
    passkey.sign_count = int(getattr(verification, "new_sign_count", passkey.sign_count))
    passkey.last_used_at = now
    challenge.used_at = now
    await db.commit()
