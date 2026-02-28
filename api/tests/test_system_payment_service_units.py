from __future__ import annotations

from uuid import uuid4

import pytest
from sqlalchemy import select

from app.db.models import Role, SystemPaymentSettings, User
from app.services import system_payment


async def _make_admin_user(db_session) -> User:
    role_id = (await db_session.execute(select(Role.id).where(Role.name == "admin"))).scalar_one()
    user = User(
        email=f"system-payment-admin-{uuid4().hex[:8]}@example.com",
        password_hash=f"hash-{uuid4().hex}",
        display_name=f"admin-{uuid4().hex[:8]}",
        role_id=role_id,
        access_status="approved",
    )
    db_session.add(user)
    await db_session.commit()
    await db_session.refresh(user)
    return user


@pytest.mark.asyncio
async def test_system_payment_status_is_wallet_only_without_server_card_fallback(monkeypatch):
    monkeypatch.setattr(system_payment.settings, "app_env", "production")
    monkeypatch.setattr(system_payment.settings, "payment_enabled", True)

    async def _row_none(_db):  # noqa: ANN001
        return None

    monkeypatch.setattr(system_payment, "_load_settings_row", _row_none)

    execution = await system_payment.get_serverwide_payment_card_for_execution(db=object())
    assert execution is None

    status_payload = await system_payment.get_system_payment_settings_status(db=object())
    assert status_payload["configured"] is False
    assert status_payload["wallet_only"] is True
    assert status_payload["source"] == "none"


@pytest.mark.asyncio
async def test_system_payment_toggle_and_legacy_material_cleanup(db_session, monkeypatch):
    monkeypatch.setattr(system_payment.settings, "app_env", "development")
    monkeypatch.setattr(system_payment.settings, "payment_enabled", True)

    admin_user = await _make_admin_user(db_session)

    disabled = await system_payment.set_system_payment_enabled(
        db_session,
        enabled=False,
        updated_by_user_id=admin_user.id,
    )
    assert disabled["payment_enabled"] is False
    assert disabled["wallet_only"] is True
    assert await system_payment.is_payment_runtime_enabled(db_session) is False

    enabled = await system_payment.set_system_payment_enabled(
        db_session,
        enabled=True,
        updated_by_user_id=admin_user.id,
    )
    assert enabled["payment_enabled"] is True
    assert enabled["configured"] is False

    row = await system_payment._ensure_settings_row(db_session)
    row.ciphertext = "legacy-c"
    row.nonce = "legacy-n"
    row.wrapped_dek = "legacy-w"
    row.dek_nonce = "legacy-d"
    row.aad = "legacy-a"
    row.kek_version = 7
    await db_session.commit()

    cleared = await system_payment.clear_legacy_serverwide_card_material(
        db_session,
        updated_by_user_id=admin_user.id,
    )
    assert cleared["wallet_only"] is True
    assert cleared["configured"] is False

    persisted = (
        await db_session.execute(
            select(SystemPaymentSettings).where(SystemPaymentSettings.id == system_payment.SYSTEM_PAYMENT_SETTINGS_ID)
        )
    ).scalar_one()
    assert persisted.ciphertext is None
    assert persisted.nonce is None
    assert persisted.wrapped_dek is None
    assert persisted.dek_nonce is None
    assert persisted.aad is None
    assert persisted.kek_version is None
