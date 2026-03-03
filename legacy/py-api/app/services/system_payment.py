from __future__ import annotations

from typing import Any
from uuid import UUID

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.config import get_settings
from app.core.time import utc_now
from app.db.models import SystemPaymentSettings

settings = get_settings()

SYSTEM_PAYMENT_SETTINGS_ID = 1
SYSTEM_PAYMENT_SOURCE_NONE = "none"


async def _load_settings_row(db: AsyncSession) -> SystemPaymentSettings | None:
    stmt = select(SystemPaymentSettings).where(SystemPaymentSettings.id == SYSTEM_PAYMENT_SETTINGS_ID)
    result = await db.execute(stmt)
    row: Any | None = None

    scalar_one_or_none = getattr(result, "scalar_one_or_none", None)
    if callable(scalar_one_or_none):
        row = scalar_one_or_none()
    else:
        scalar_one = getattr(result, "scalar_one", None)
        if callable(scalar_one):
            try:
                row = scalar_one()
            except Exception:
                row = None
        else:
            scalar = getattr(result, "scalar", None)
            if callable(scalar):
                row = scalar()

    return row if isinstance(row, SystemPaymentSettings) else None


async def _ensure_settings_row(db: AsyncSession) -> SystemPaymentSettings:
    row = await _load_settings_row(db)
    if row is not None:
        return row

    row = SystemPaymentSettings(id=SYSTEM_PAYMENT_SETTINGS_ID, payment_enabled=True)
    db.add(row)
    await db.flush()
    return row


def _settings_status_payload(row: SystemPaymentSettings | None) -> dict[str, Any]:
    runtime_enabled = bool(settings.payment_enabled and (row.payment_enabled if row is not None else True))
    return {
        "payment_enabled": runtime_enabled,
        "payment_enabled_env": bool(settings.payment_enabled),
        "payment_enabled_override": bool(row.payment_enabled) if row is not None else True,
        "configured": False,
        "wallet_only": True,
        "source": SYSTEM_PAYMENT_SOURCE_NONE,
        "card_masked": None,
        "updated_at": row.updated_at if row is not None else None,
        "updated_by_user_id": row.updated_by_user_id if row is not None else None,
    }


async def is_payment_runtime_enabled(db: AsyncSession) -> bool:
    if not settings.payment_enabled:
        return False

    row = await _load_settings_row(db)
    if row is None:
        return True
    return bool(row.payment_enabled)


async def get_serverwide_payment_card_for_execution(db: AsyncSession) -> dict[str, int | str] | None:
    # Server-side payment-card custody is retired; execution is wallet-only.
    return None


async def get_system_payment_settings_status(db: AsyncSession) -> dict[str, Any]:
    row = await _load_settings_row(db)
    return _settings_status_payload(row)


async def is_serverwide_payment_configured(db: AsyncSession) -> bool:
    return False


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


async def clear_legacy_serverwide_card_material(
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
