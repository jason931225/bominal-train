from __future__ import annotations

from datetime import date, datetime
from typing import Any
from uuid import UUID

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.config import get_settings
from app.core.crypto.service import get_envelope_crypto
from app.core.time import utc_now
from app.db.models import SystemPaymentSettings

settings = get_settings()

SYSTEM_PAYMENT_SETTINGS_ID = 1
SYSTEM_PAYMENT_CARD_AAD = "system_payment_settings:card"
SYSTEM_PAYMENT_SOURCE_OVERRIDE = "server_override"
SYSTEM_PAYMENT_SOURCE_ENV = "pay_env"
SYSTEM_PAYMENT_SOURCE_NONE = "none"


def _digits_only(value: str | None) -> str:
    return "".join(ch for ch in str(value or "") if ch.isdigit())


def mask_card_number(card_number: str | None) -> str | None:
    digits = _digits_only(card_number)
    if len(digits) < 4:
        return None
    return f"**** **** **** {digits[-4:]}"


def _build_execution_payment_card(
    *,
    card_number: str,
    pin2: str,
    birth_date: date,
    expiry_month: int,
    expiry_year_2digit: int,
) -> dict[str, int | str]:
    return {
        "card_number": card_number,
        "card_password": pin2,
        "validation_number": birth_date.strftime("%y%m%d"),
        "card_expire": f"{expiry_year_2digit % 100:02d}{expiry_month:02d}",
        "card_type": "J",
        "installment": 0,
    }


def _parse_card_parts(
    *,
    card_number_raw: str | None,
    expiry_mm_raw: str | None,
    expiry_yy_raw: str | None,
    dob_raw: str | None,
    pin2_raw: str | None,
) -> dict[str, Any] | None:
    card_number = _digits_only(card_number_raw)
    expiry_month_str = _digits_only(expiry_mm_raw)
    expiry_year_str = _digits_only(expiry_yy_raw)
    birth_date_str = _digits_only(dob_raw)
    pin2 = _digits_only(pin2_raw)

    if not card_number or not expiry_month_str or not expiry_year_str or not birth_date_str or not pin2:
        return None
    if not (13 <= len(card_number) <= 19):
        return None
    if len(expiry_month_str) != 2 or len(pin2) != 2 or len(birth_date_str) != 8:
        return None
    if len(expiry_year_str) not in {2, 4}:
        return None

    try:
        expiry_month = int(expiry_month_str)
        expiry_year = int(expiry_year_str)
        birth_date = datetime.strptime(birth_date_str, "%Y%m%d").date()
    except ValueError:
        return None

    if not (1 <= expiry_month <= 12):
        return None

    return {
        "card_number": card_number,
        "expiry_month": expiry_month,
        "expiry_year_2digit": expiry_year % 100,
        "birth_date": birth_date,
        "pin2": pin2,
    }


def _payload_to_execution_card(payload: dict[str, Any] | None) -> dict[str, int | str] | None:
    if not isinstance(payload, dict):
        return None

    card_number = _digits_only(str(payload.get("card_number") or ""))
    pin2 = _digits_only(str(payload.get("pin2") or ""))
    birth_date_iso = str(payload.get("birth_date") or "")
    expiry_month = payload.get("expiry_month")
    expiry_year_2digit = payload.get("expiry_year_2digit")

    if not card_number or not pin2 or not birth_date_iso:
        return None

    try:
        parsed_birth_date = datetime.fromisoformat(birth_date_iso).date()
        parsed_expiry_month = int(expiry_month)
        parsed_expiry_year = int(expiry_year_2digit)
    except Exception:
        return None

    if len(pin2) != 2:
        return None
    if not (13 <= len(card_number) <= 19):
        return None
    if not (1 <= parsed_expiry_month <= 12):
        return None

    return _build_execution_payment_card(
        card_number=card_number,
        pin2=pin2,
        birth_date=parsed_birth_date,
        expiry_month=parsed_expiry_month,
        expiry_year_2digit=parsed_expiry_year,
    )


def _backend_env_payment_card_for_execution() -> dict[str, int | str] | None:
    # Backend-only fallback loaded from production pay.env aliases.
    if settings.app_env.lower() != "production" or not settings.payment_enabled:
        return None

    parts = _parse_card_parts(
        card_number_raw=settings.backend_pay_cardnumber,
        expiry_mm_raw=settings.backend_pay_expirymm,
        expiry_yy_raw=settings.backend_pay_expiryyy,
        dob_raw=settings.backend_pay_dob,
        pin2_raw=settings.backend_pay_nn,
    )
    if parts is None:
        return None

    return _build_execution_payment_card(
        card_number=parts["card_number"],
        pin2=parts["pin2"],
        birth_date=parts["birth_date"],
        expiry_month=parts["expiry_month"],
        expiry_year_2digit=parts["expiry_year_2digit"],
    )


def _backend_env_card_masked() -> str | None:
    return mask_card_number(settings.backend_pay_cardnumber)


async def _load_settings_row(db: AsyncSession) -> SystemPaymentSettings | None:
    stmt = select(SystemPaymentSettings).where(SystemPaymentSettings.id == SYSTEM_PAYMENT_SETTINGS_ID)
    return (await db.execute(stmt)).scalar_one_or_none()


async def _ensure_settings_row(db: AsyncSession) -> SystemPaymentSettings:
    row = await _load_settings_row(db)
    if row is not None:
        return row

    row = SystemPaymentSettings(id=SYSTEM_PAYMENT_SETTINGS_ID, payment_enabled=True)
    db.add(row)
    await db.flush()
    return row


def _row_has_encrypted_card(row: SystemPaymentSettings | None) -> bool:
    if row is None:
        return False
    return all(
        [
            row.ciphertext,
            row.nonce,
            row.wrapped_dek,
            row.dek_nonce,
            row.aad,
            row.kek_version is not None,
        ]
    )


def _decrypt_row_card_payload(row: SystemPaymentSettings | None) -> dict[str, Any] | None:
    if not _row_has_encrypted_card(row):
        return None

    try:
        payload = get_envelope_crypto().decrypt_payload(
            ciphertext=str(row.ciphertext),
            nonce=str(row.nonce),
            wrapped_dek=str(row.wrapped_dek),
            dek_nonce=str(row.dek_nonce),
            aad=str(row.aad),
            kek_version=int(row.kek_version),
            enforce_kek_version=True,
        )
    except Exception:
        return None

    if not isinstance(payload, dict):
        return None
    return payload


async def is_payment_runtime_enabled(db: AsyncSession) -> bool:
    if not settings.payment_enabled:
        return False

    row = await _load_settings_row(db)
    if row is None:
        return True
    return bool(row.payment_enabled)


async def get_serverwide_payment_card_for_execution(db: AsyncSession) -> dict[str, int | str] | None:
    if not await is_payment_runtime_enabled(db):
        return None

    row = await _load_settings_row(db)
    override_card = _payload_to_execution_card(_decrypt_row_card_payload(row))
    if override_card is not None:
        return override_card

    return _backend_env_payment_card_for_execution()


async def get_system_payment_settings_status(db: AsyncSession) -> dict[str, Any]:
    row = await _load_settings_row(db)
    runtime_enabled = bool(settings.payment_enabled and (row.payment_enabled if row is not None else True))

    source = SYSTEM_PAYMENT_SOURCE_NONE
    card_masked: str | None = None
    override_payload = _decrypt_row_card_payload(row)
    override_card = _payload_to_execution_card(override_payload)

    if override_card is not None:
        source = SYSTEM_PAYMENT_SOURCE_OVERRIDE
        card_masked = mask_card_number(str(override_payload.get("card_number") if override_payload else ""))
    else:
        env_card = _backend_env_payment_card_for_execution()
        if env_card is not None:
            source = SYSTEM_PAYMENT_SOURCE_ENV
            card_masked = _backend_env_card_masked()

    return {
        "payment_enabled": runtime_enabled,
        "payment_enabled_env": bool(settings.payment_enabled),
        "payment_enabled_override": bool(row.payment_enabled) if row is not None else True,
        "configured": source != SYSTEM_PAYMENT_SOURCE_NONE,
        "source": source,
        "card_masked": card_masked,
        "updated_at": row.updated_at if row is not None else None,
        "updated_by_user_id": row.updated_by_user_id if row is not None else None,
    }


async def is_serverwide_payment_configured(db: AsyncSession) -> bool:
    status = await get_system_payment_settings_status(db)
    return bool(status["payment_enabled"] and status["configured"])


async def set_system_payment_enabled(
    db: AsyncSession,
    *,
    enabled: bool,
    updated_by_user_id: UUID | None,
) -> dict[str, Any]:
    row = await _ensure_settings_row(db)
    row.payment_enabled = bool(enabled)
    row.updated_at = utc_now()
    row.updated_by_user_id = updated_by_user_id
    await db.commit()
    return await get_system_payment_settings_status(db)


async def set_system_payment_card(
    db: AsyncSession,
    *,
    card_number: str,
    expiry_mm: str,
    expiry_yy: str,
    dob: str,
    pin2: str,
    updated_by_user_id: UUID | None,
) -> dict[str, Any] | None:
    parts = _parse_card_parts(
        card_number_raw=card_number,
        expiry_mm_raw=expiry_mm,
        expiry_yy_raw=expiry_yy,
        dob_raw=dob,
        pin2_raw=pin2,
    )
    if parts is None:
        return None

    payload = {
        "card_number": parts["card_number"],
        "expiry_month": parts["expiry_month"],
        "expiry_year_2digit": parts["expiry_year_2digit"],
        "birth_date": parts["birth_date"].isoformat(),
        "pin2": parts["pin2"],
        "updated_at": utc_now().isoformat(),
    }
    encrypted = get_envelope_crypto().encrypt_payload(payload=payload, aad_text=SYSTEM_PAYMENT_CARD_AAD)

    row = await _ensure_settings_row(db)
    row.ciphertext = encrypted.ciphertext
    row.nonce = encrypted.nonce
    row.wrapped_dek = encrypted.wrapped_dek
    row.dek_nonce = encrypted.dek_nonce
    row.aad = encrypted.aad
    row.kek_version = encrypted.kek_version
    row.updated_at = utc_now()
    row.updated_by_user_id = updated_by_user_id
    await db.commit()

    return await get_system_payment_settings_status(db)


async def clear_system_payment_card(
    db: AsyncSession,
    *,
    updated_by_user_id: UUID | None,
) -> dict[str, Any]:
    row = await _ensure_settings_row(db)
    row.ciphertext = None
    row.nonce = None
    row.wrapped_dek = None
    row.dek_nonce = None
    row.aad = None
    row.kek_version = None
    row.updated_at = utc_now()
    row.updated_by_user_id = updated_by_user_id
    await db.commit()

    return await get_system_payment_settings_status(db)
