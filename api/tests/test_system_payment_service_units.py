from __future__ import annotations

from uuid import uuid4

import pytest
from sqlalchemy import select

from app.db.models import Role, User
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
async def test_system_payment_uses_pay_env_fallback_in_production(monkeypatch):
    monkeypatch.setattr(system_payment.settings, "app_env", "production")
    monkeypatch.setattr(system_payment.settings, "payment_enabled", True)
    monkeypatch.setattr(system_payment.settings, "backend_pay_cardnumber", "4111-1111-1111-1111")
    monkeypatch.setattr(system_payment.settings, "backend_pay_expirymm", "12")
    monkeypatch.setattr(system_payment.settings, "backend_pay_expiryyy", "99")
    monkeypatch.setattr(system_payment.settings, "backend_pay_dob", "19900101")
    monkeypatch.setattr(system_payment.settings, "backend_pay_nn", "12")

    async def _row_none(_db):  # noqa: ANN001
        return None

    monkeypatch.setattr(system_payment, "_load_settings_row", _row_none)

    execution = await system_payment.get_serverwide_payment_card_for_execution(db=object())
    assert execution is not None
    assert execution["card_number"] == "4111111111111111"

    status_payload = await system_payment.get_system_payment_settings_status(db=object())
    assert status_payload["configured"] is True
    assert status_payload["source"] == "pay_env"


@pytest.mark.asyncio
async def test_system_payment_override_and_runtime_toggle(db_session, monkeypatch):
    monkeypatch.setattr(system_payment.settings, "app_env", "development")
    monkeypatch.setattr(system_payment.settings, "payment_enabled", True)
    monkeypatch.setattr(system_payment.settings, "backend_pay_cardnumber", None)
    monkeypatch.setattr(system_payment.settings, "backend_pay_expirymm", None)
    monkeypatch.setattr(system_payment.settings, "backend_pay_expiryyy", None)
    monkeypatch.setattr(system_payment.settings, "backend_pay_dob", None)
    monkeypatch.setattr(system_payment.settings, "backend_pay_nn", None)

    admin_user = await _make_admin_user(db_session)

    updated = await system_payment.set_system_payment_card(
        db_session,
        card_number="5555 5555 5555 4444",
        expiry_mm="01",
        expiry_yy="31",
        dob="19900101",
        pin2="34",
        updated_by_user_id=admin_user.id,
    )
    assert updated is not None
    assert updated["source"] == "server_override"
    assert str(updated["card_masked"] or "").endswith("4444")

    card = await system_payment.get_serverwide_payment_card_for_execution(db_session)
    assert card is not None
    assert card["card_number"] == "5555555555554444"

    disabled = await system_payment.set_system_payment_enabled(
        db_session,
        enabled=False,
        updated_by_user_id=admin_user.id,
    )
    assert disabled["payment_enabled"] is False
    assert await system_payment.is_payment_runtime_enabled(db_session) is False
    assert await system_payment.get_serverwide_payment_card_for_execution(db_session) is None

    enabled = await system_payment.set_system_payment_enabled(
        db_session,
        enabled=True,
        updated_by_user_id=admin_user.id,
    )
    assert enabled["payment_enabled"] is True

    cleared = await system_payment.clear_system_payment_card(
        db_session,
        updated_by_user_id=admin_user.id,
    )
    assert cleared["source"] == "none"
    assert cleared["configured"] is False


@pytest.mark.asyncio
async def test_system_payment_card_rejects_invalid_payload(db_session):
    result = await system_payment.set_system_payment_card(
        db_session,
        card_number="1234",
        expiry_mm="99",
        expiry_yy="1",
        dob="bad",
        pin2="x",
        updated_by_user_id=None,
    )
    assert result is None
