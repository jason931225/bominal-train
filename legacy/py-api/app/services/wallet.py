from __future__ import annotations

import json
from datetime import datetime, timedelta
from typing import Any
from uuid import UUID

from sqlalchemy import delete, func, select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.config import get_settings
from app.core.crypto.secrets_store import build_encrypted_secret, decrypt_secret
from app.core.crypto.service import get_envelope_crypto
from app.core.redis import get_cde_redis_pool
from app.core.time import utc_now
from app.db.models import Secret, User
from app.schemas.wallet import PaymentCardConfiguredResponse, PaymentCardSetRequest, PaymentCardStatusResponse

settings = get_settings()

SECRET_KIND_PAYMENT_CARD = "payment_card"
PAYMENT_CVV_REDIS_KEY_PREFIX = "wallet:payment:cvv"
LEGACY_PAYMENT_CVV_REDIS_KEY_PREFIX = "train:payment:cvv"


def get_redis_pool():
    """Compatibility shim for legacy call sites/tests.

    Payment CVV cache is CDE-scoped, so the compatibility alias intentionally
    returns the CDE Redis pool.
    """
    return get_cde_redis_pool()


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


def _digits_only(value: str | None) -> str:
    return "".join(ch for ch in str(value or "") if ch.isdigit())


def _build_evervault_execution_payment_card(
    *,
    encrypted_card_number: str,
    encrypted_pin2: str,
    encrypted_birth_date: str,
    encrypted_expiry: str,
    last4: str,
    brand: str | None,
) -> dict[str, Any]:
    encrypted_fields = {
        "encrypted_card_number": encrypted_card_number,
        "encrypted_pin2": encrypted_pin2,
        "encrypted_birth_date": encrypted_birth_date,
        "encrypted_expiry": encrypted_expiry,
    }
    return {
        "source": "evervault",
        # Provider clients currently consume these keys for payment payload composition.
        "card_number": encrypted_card_number,
        "card_password": encrypted_pin2,
        "validation_number": encrypted_birth_date,
        "card_expire": encrypted_expiry,
        "card_type": "J",
        "installment": 0,
        "encrypted_fields": encrypted_fields,
        "safe_meta": {"last4": last4, "brand": brand},
    }


def _execution_payment_card_from_secret_payload(payload: dict[str, Any] | None) -> dict[str, Any] | None:
    if not isinstance(payload, dict):
        return None

    source = str(payload.get("source") or "").strip().lower()
    if source != "evervault":
        return None

    encrypted_card_number = str(payload.get("encrypted_card_number") or "").strip()
    encrypted_pin2 = str(payload.get("encrypted_pin2") or "").strip()
    encrypted_birth_date = str(payload.get("encrypted_birth_date") or "").strip()
    encrypted_expiry = str(payload.get("encrypted_expiry") or "").strip()
    last4 = _digits_only(str(payload.get("last4") or ""))
    brand_raw = str(payload.get("brand") or "").strip()
    brand = brand_raw or None
    if not all((encrypted_card_number, encrypted_pin2, encrypted_birth_date, encrypted_expiry)):
        return None
    if len(last4) != 4:
        return None
    return _build_evervault_execution_payment_card(
        encrypted_card_number=encrypted_card_number,
        encrypted_pin2=encrypted_pin2,
        encrypted_birth_date=encrypted_birth_date,
        encrypted_expiry=encrypted_expiry,
        last4=last4,
        brand=brand,
    )


def _payment_cvv_redis_key(user_id: UUID) -> str:
    return f"{PAYMENT_CVV_REDIS_KEY_PREFIX}:{user_id}"


def _legacy_payment_cvv_redis_key(user_id: UUID) -> str:
    return f"{LEGACY_PAYMENT_CVV_REDIS_KEY_PREFIX}:{user_id}"


def _serialize_encrypted_payload(payload: dict) -> str:
    return json.dumps(payload, separators=(",", ":"), ensure_ascii=True)


def _deserialize_cached_cvv_payload(value: str) -> dict | None:
    try:
        encrypted_payload = json.loads(value)
        raw_kek_version = encrypted_payload.get("kek_version")
        parsed_kek_version: int | None = None
        if raw_kek_version is not None:
            parsed_kek_version = int(raw_kek_version)
        if settings.payment_require_cvv_kek_version and parsed_kek_version is None:
            return None
        decrypted_payload = get_envelope_crypto().decrypt_payload(
            ciphertext=encrypted_payload["ciphertext"],
            nonce=encrypted_payload["nonce"],
            wrapped_dek=encrypted_payload["wrapped_dek"],
            dek_nonce=encrypted_payload["dek_nonce"],
            aad=encrypted_payload["aad"],
            kek_version=parsed_kek_version,
            enforce_kek_version=settings.payment_require_cvv_kek_version or parsed_kek_version is not None,
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
    async with get_redis_pool() as redis:
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


async def _load_cached_cvv_until(*, user_id: UUID) -> datetime | None:
    payload = await _load_cached_cvv_payload(user_id=user_id)
    if payload is None:
        return None
    return payload.get("expires_at")


async def _cache_cvv(*, user_id: UUID, cvv: str) -> datetime:
    ttl_seconds = max(
        settings.payment_cvv_ttl_min_seconds,
        min(settings.payment_cvv_ttl_seconds, settings.payment_cvv_ttl_max_seconds),
    )
    expires_at = utc_now() + timedelta(seconds=ttl_seconds)
    encrypted = get_envelope_crypto().encrypt_payload(
        payload={"cvv": cvv, "expires_at": expires_at.isoformat()},
        aad_text=f"payment_cvv:{user_id}",
    )

    async with get_redis_pool() as redis:
        await redis.set(
            _payment_cvv_redis_key(user_id),
            _serialize_encrypted_payload(
                {
                    "ciphertext": encrypted.ciphertext,
                    "nonce": encrypted.nonce,
                    "wrapped_dek": encrypted.wrapped_dek,
                    "dek_nonce": encrypted.dek_nonce,
                    "aad": encrypted.aad,
                    "kek_version": encrypted.kek_version,
                }
            ),
            ex=ttl_seconds,
        )

    return expires_at


async def _clear_cached_cvv(*, user_id: UUID) -> None:
    async with get_redis_pool() as redis:
        await redis.delete(_payment_cvv_redis_key(user_id), _legacy_payment_cvv_redis_key(user_id))


async def clear_payment_card_cache(*, user_id: UUID) -> None:
    await _clear_cached_cvv(user_id=user_id)


async def _delete_redis_keys_matching(*, pattern: str) -> int:
    deleted_total = 0
    async with get_redis_pool() as redis:
        cursor: int = 0
        while True:
            cursor, keys = await redis.scan(cursor=cursor, match=pattern, count=500)
            if keys:
                deleted_total += int(await redis.delete(*keys))
            if cursor == 0:
                break
    return deleted_total


async def purge_cached_payment_cvv_data() -> dict[str, int]:
    redis_deleted_current = await _delete_redis_keys_matching(pattern=f"{PAYMENT_CVV_REDIS_KEY_PREFIX}:*")
    redis_deleted_legacy = await _delete_redis_keys_matching(pattern=f"{LEGACY_PAYMENT_CVV_REDIS_KEY_PREFIX}:*")
    return {
        "redis_cvv_keys_deleted_current": redis_deleted_current,
        "redis_cvv_keys_deleted_legacy": redis_deleted_legacy,
        "redis_cvv_keys_deleted_total": redis_deleted_current + redis_deleted_legacy,
    }


async def purge_all_saved_payment_data(db: AsyncSession) -> dict[str, int]:
    secret_count_stmt = select(func.count(Secret.id)).where(Secret.kind == SECRET_KIND_PAYMENT_CARD)
    secret_count = int((await db.execute(secret_count_stmt)).scalar_one() or 0)

    await db.execute(delete(Secret).where(Secret.kind == SECRET_KIND_PAYMENT_CARD))
    await db.commit()

    redis_summary = await purge_cached_payment_cvv_data()

    return {
        "db_payment_card_secrets_deleted": secret_count,
        **redis_summary,
    }


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

    source = str(payload.get("source") or "").strip().lower()
    if source != "evervault":
        return PaymentCardStatusResponse(configured=False, detail="Saved payment card data is invalid")

    last4 = _digits_only(str(payload.get("last4") or ""))
    encrypted_card_number = str(payload.get("encrypted_card_number") or "").strip()
    encrypted_pin2 = str(payload.get("encrypted_pin2") or "").strip()
    encrypted_birth_date = str(payload.get("encrypted_birth_date") or "").strip()
    encrypted_expiry = str(payload.get("encrypted_expiry") or "").strip()
    if len(last4) != 4 or not all((encrypted_card_number, encrypted_pin2, encrypted_birth_date, encrypted_expiry)):
        return PaymentCardStatusResponse(configured=False, detail="Saved payment card data is invalid")
    brand_raw = str(payload.get("brand") or "").strip()
    return PaymentCardStatusResponse(
        configured=True,
        card_masked=f"**** **** **** {last4}",
        source="evervault",
        brand=brand_raw or None,
        updated_at=secret.updated_at,
    )


async def get_payment_card_configured(
    db: AsyncSession,
    *,
    user: User,
) -> PaymentCardConfiguredResponse:
    secret = await _latest_payment_secret_for_user(db, user_id=user.id)
    if secret is None:
        return PaymentCardConfiguredResponse(configured=False)
    try:
        payload = decrypt_secret(secret)
    except Exception:
        return PaymentCardConfiguredResponse(configured=False)
    return PaymentCardConfiguredResponse(configured=_execution_payment_card_from_secret_payload(payload) is not None)


async def get_payment_card_for_execution(
    db: AsyncSession,
    *,
    user_id: UUID,
) -> dict | None:
    secret = await _latest_payment_secret_for_user(db, user_id=user_id)
    if secret is not None:
        try:
            payload = decrypt_secret(secret)
        except Exception:
            payload = None
        execution_card = _execution_payment_card_from_secret_payload(payload)
        if execution_card is not None:
            return execution_card

    return None


async def set_payment_card(
    db: AsyncSession,
    *,
    user: User,
    payload: PaymentCardSetRequest,
) -> PaymentCardStatusResponse:
    now = utc_now()
    secret_payload = {
        "source": "evervault",
        "encrypted_card_number": payload.encrypted_card_number,
        "encrypted_pin2": payload.encrypted_pin2,
        "encrypted_birth_date": payload.encrypted_birth_date,
        "encrypted_expiry": payload.encrypted_expiry,
        "last4": payload.last4,
        "brand": payload.brand,
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
    return PaymentCardStatusResponse(
        configured=True,
        card_masked=f"**** **** **** {payload.last4}",
        source="evervault",
        brand=payload.brand,
        updated_at=now,
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
