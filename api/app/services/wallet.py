from __future__ import annotations

import json
from datetime import datetime, timedelta, timezone
from uuid import UUID

from fastapi import HTTPException, status
from redis.asyncio import Redis
from sqlalchemy import delete, select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.config import get_settings
from app.core.crypto.secrets_store import build_encrypted_secret, decrypt_secret
from app.core.crypto.service import get_envelope_crypto
from app.db.models import Secret, User
from app.schemas.wallet import PaymentCardSetRequest, PaymentCardStatusResponse

settings = get_settings()

SECRET_KIND_PAYMENT_CARD = "payment_card"
PAYMENT_CVV_REDIS_KEY_PREFIX = "wallet:payment:cvv"
LEGACY_PAYMENT_CVV_REDIS_KEY_PREFIX = "train:payment:cvv"


def _utc_now() -> datetime:
    return datetime.now(timezone.utc)


async def _latest_payment_secret_for_user(db: AsyncSession, *, user_id: UUID) -> Secret | None:
    stmt = (
        select(Secret)
        .where(Secret.user_id == user_id)
        .where(Secret.kind == SECRET_KIND_PAYMENT_CARD)
        .order_by(Secret.updated_at.desc())
        .limit(1)
    )
    return (await db.execute(stmt)).scalar_one_or_none()


def _mask_card_number(card_number: str) -> str:
    digits = "".join(ch for ch in card_number if ch.isdigit())
    if len(digits) >= 4:
        return f"**** **** **** {digits[-4:]}"
    return "****"


def _payment_cvv_redis_key(user_id: UUID) -> str:
    return f"{PAYMENT_CVV_REDIS_KEY_PREFIX}:{user_id}"


def _legacy_payment_cvv_redis_key(user_id: UUID) -> str:
    return f"{LEGACY_PAYMENT_CVV_REDIS_KEY_PREFIX}:{user_id}"


def _serialize_encrypted_payload(payload: dict) -> str:
    return json.dumps(payload, separators=(",", ":"), ensure_ascii=True)


def _deserialize_cached_cvv_payload(value: str) -> dict | None:
    try:
        encrypted_payload = json.loads(value)
        decrypted_payload = get_envelope_crypto().decrypt_payload(
            ciphertext=encrypted_payload["ciphertext"],
            nonce=encrypted_payload["nonce"],
            wrapped_dek=encrypted_payload["wrapped_dek"],
            dek_nonce=encrypted_payload["dek_nonce"],
            aad=encrypted_payload["aad"],
        )
        if not isinstance(decrypted_payload, dict):
            return None
        expires_at = decrypted_payload.get("expires_at")
        if not isinstance(expires_at, str):
            return None
        parsed_expires_at = datetime.fromisoformat(expires_at)
        return {
            "cvv": str(decrypted_payload.get("cvv") or ""),
            "expires_at": parsed_expires_at,
        }
    except Exception:
        return None


async def _load_cached_cvv_payload(*, user_id: UUID) -> dict | None:
    redis = Redis.from_url(settings.redis_url)
    try:
        for key in (_payment_cvv_redis_key(user_id), _legacy_payment_cvv_redis_key(user_id)):
            encrypted_blob = await redis.get(key)
            if not encrypted_blob:
                continue
            if isinstance(encrypted_blob, bytes):
                encrypted_blob = encrypted_blob.decode("utf-8")
            parsed = _deserialize_cached_cvv_payload(encrypted_blob)
            if parsed is not None:
                return parsed
        return None
    finally:
        await redis.aclose()


async def _load_cached_cvv_until(*, user_id: UUID) -> datetime | None:
    payload = await _load_cached_cvv_payload(user_id=user_id)
    if payload is None:
        return None
    return payload.get("expires_at")


async def _cache_cvv(*, user_id: UUID, cvv: str) -> datetime:
    ttl_seconds = max(60, settings.payment_cvv_ttl_seconds)
    expires_at = _utc_now() + timedelta(seconds=ttl_seconds)
    encrypted = get_envelope_crypto().encrypt_payload(
        payload={"cvv": cvv, "expires_at": expires_at.isoformat()},
        aad_text=f"payment_cvv:{user_id}",
    )

    redis = Redis.from_url(settings.redis_url)
    try:
        await redis.set(
            _payment_cvv_redis_key(user_id),
            _serialize_encrypted_payload(
                {
                    "ciphertext": encrypted.ciphertext,
                    "nonce": encrypted.nonce,
                    "wrapped_dek": encrypted.wrapped_dek,
                    "dek_nonce": encrypted.dek_nonce,
                    "aad": encrypted.aad,
                }
            ),
            ex=ttl_seconds,
        )
    finally:
        await redis.aclose()

    return expires_at


async def _clear_cached_cvv(*, user_id: UUID) -> None:
    redis = Redis.from_url(settings.redis_url)
    try:
        await redis.delete(_payment_cvv_redis_key(user_id), _legacy_payment_cvv_redis_key(user_id))
    finally:
        await redis.aclose()


async def clear_payment_card_cache(*, user_id: UUID) -> None:
    await _clear_cached_cvv(user_id=user_id)


async def get_payment_card_status(
    db: AsyncSession,
    *,
    user: User,
) -> PaymentCardStatusResponse:
    secret = await _latest_payment_secret_for_user(db, user_id=user.id)
    if secret is None:
        return PaymentCardStatusResponse(configured=False, detail="No payment card saved")

    try:
        payload = decrypt_secret(secret)
    except Exception:
        return PaymentCardStatusResponse(configured=False, detail="Payment card could not be loaded")

    card_number = str(payload.get("card_number") or "")
    expiry_month = payload.get("expiry_month")
    expiry_year = payload.get("expiry_year")
    try:
        parsed_expiry_month = int(expiry_month)
        parsed_expiry_year = int(expiry_year)
    except (TypeError, ValueError):
        return PaymentCardStatusResponse(configured=False, detail="Saved payment card data is invalid")

    cvv_cached_until = await _load_cached_cvv_until(user_id=user.id)
    return PaymentCardStatusResponse(
        configured=True,
        card_masked=_mask_card_number(card_number),
        expiry_month=parsed_expiry_month,
        expiry_year=parsed_expiry_year,
        updated_at=secret.updated_at,
        cvv_cached_until=cvv_cached_until,
    )


async def get_payment_card_for_execution(
    db: AsyncSession,
    *,
    user_id: UUID,
) -> dict | None:
    secret = await _latest_payment_secret_for_user(db, user_id=user_id)
    if secret is None:
        return None

    try:
        payload = decrypt_secret(secret)
    except Exception:
        return None

    card_number = str(payload.get("card_number") or "").strip()
    pin2 = str(payload.get("pin2") or "").strip()
    birth_date_iso = str(payload.get("birth_date") or "").strip()
    expiry_month = payload.get("expiry_month")
    expiry_year = payload.get("expiry_year")

    if not card_number or not pin2 or not birth_date_iso:
        return None

    try:
        parsed_expiry_month = int(expiry_month)
        parsed_expiry_year = int(expiry_year)
        parsed_birth_date = datetime.fromisoformat(birth_date_iso).date()
    except Exception:
        return None

    cached_cvv_payload = await _load_cached_cvv_payload(user_id=user_id)
    cvv = ""
    cvv_cached_until = None
    if cached_cvv_payload is not None:
        cvv = str(cached_cvv_payload.get("cvv") or "")
        cvv_cached_until = cached_cvv_payload.get("expires_at")

    return {
        "card_number": card_number,
        "card_password": pin2,
        "validation_number": parsed_birth_date.strftime("%y%m%d"),
        "card_expire": f"{parsed_expiry_year % 100:02d}{parsed_expiry_month:02d}",
        "card_type": "J",
        "installment": 0,
        "cvv": cvv,
        "cvv_cached_until": cvv_cached_until.isoformat() if cvv_cached_until else None,
    }


async def set_payment_card(
    db: AsyncSession,
    *,
    user: User,
    payload: PaymentCardSetRequest,
) -> PaymentCardStatusResponse:
    now = _utc_now()
    now_utc = now.astimezone(timezone.utc)
    if payload.expiry_year < now_utc.year or (
        payload.expiry_year == now_utc.year and payload.expiry_month < now_utc.month
    ):
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Card expiry date is in the past")

    secret_payload = {
        "card_number": payload.card_number,
        "expiry_month": payload.expiry_month,
        "expiry_year": payload.expiry_year,
        "birth_date": payload.birth_date.isoformat(),
        "pin2": payload.pin2,
        "updated_at": now.isoformat(),
    }
    encrypted_secret = build_encrypted_secret(
        user_id=user.id,
        kind=SECRET_KIND_PAYMENT_CARD,
        payload=secret_payload,
    )
    existing_secret = await _latest_payment_secret_for_user(db, user_id=user.id)

    if existing_secret is None:
        db.add(encrypted_secret)
    else:
        existing_secret.ciphertext = encrypted_secret.ciphertext
        existing_secret.nonce = encrypted_secret.nonce
        existing_secret.wrapped_dek = encrypted_secret.wrapped_dek
        existing_secret.dek_nonce = encrypted_secret.dek_nonce
        existing_secret.aad = encrypted_secret.aad
        existing_secret.kek_version = encrypted_secret.kek_version
        existing_secret.updated_at = now

    await db.commit()
    cvv_cached_until = await _cache_cvv(user_id=user.id, cvv=payload.cvv)

    return PaymentCardStatusResponse(
        configured=True,
        card_masked=_mask_card_number(payload.card_number),
        expiry_month=payload.expiry_month,
        expiry_year=payload.expiry_year,
        updated_at=now,
        cvv_cached_until=cvv_cached_until,
    )


async def clear_payment_card(
    db: AsyncSession,
    *,
    user: User,
) -> PaymentCardStatusResponse:
    await db.execute(
        delete(Secret)
        .where(Secret.user_id == user.id)
        .where(Secret.kind == SECRET_KIND_PAYMENT_CARD)
    )
    await db.commit()
    await _clear_cached_cvv(user_id=user.id)

    return PaymentCardStatusResponse(
        configured=False,
        detail="Payment settings removed",
    )
