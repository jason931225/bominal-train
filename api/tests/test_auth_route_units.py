from __future__ import annotations

from datetime import datetime, timedelta, timezone
from types import SimpleNamespace
from uuid import UUID, uuid4

import pytest
from fastapi import BackgroundTasks, HTTPException, Request, Response
from sqlalchemy import select
from sqlalchemy.exc import IntegrityError
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import joinedload

from app.core.security import hash_password, hash_token, verify_password
from app.db.models import PasswordResetToken, Role, Session, User, VerificationToken
from app.http.routes import auth as auth_routes
from app.schemas.auth import (
    AccountUpdateRequest,
    EmailChangeConfirmRequest,
    EmailVerificationConfirmRequest,
    EmailVerificationRequest,
    LoginRequest,
    PasswordResetConfirmRequest,
    PasswordResetRequest,
    PasswordVerifyRequest,
    PasskeyAuthenticationOptionsRequest,
    PasskeyAuthenticationVerifyRequest,
    PasskeyRegistrationVerifyRequest,
    PasskeyStepUpVerifyRequest,
    RegisterRequest,
)


def _request_for_login() -> Request:
    scope = {
        "type": "http",
        "http_version": "1.1",
        "method": "POST",
        "scheme": "http",
        "path": "/api/auth/login",
        "raw_path": b"/api/auth/login",
        "query_string": b"",
        "headers": [
            (b"user-agent", b"pytest"),
            (b"x-forwarded-for", b"198.51.100.1"),
        ],
        "client": ("203.0.113.1", 54321),
        "server": ("testserver", 80),
        "root_path": "",
    }
    return Request(scope)


async def _load_user_with_role(db_session: AsyncSession, *, email: str) -> User:
    return (
        await db_session.execute(
            select(User).options(joinedload(User.role)).where(User.email == email)
        )
    ).scalar_one()


def test_auth_helper_functions_and_template_payloads(monkeypatch):
    assert len(auth_routes._new_otp_code()) == 6
    assert auth_routes._integrity_conflict_detail(IntegrityError("x", {}, Exception("display_name"))) == (
        "Display name already registered"
    )
    assert auth_routes._integrity_conflict_detail(IntegrityError("x", {}, Exception("email"))) == (
        "Email already registered"
    )
    assert auth_routes._integrity_conflict_detail(IntegrityError("x", {}, Exception("other"))) == (
        "Account already exists"
    )

    monkeypatch.setattr(auth_routes.settings, "app_public_base_url", "https://app.example.com/")
    assert auth_routes._public_base_url() == "https://app.example.com"

    verify_payload = auth_routes._verification_template_payload(
        email="verify@example.com",
        display_name="Verifier",
        code="123456",
    )
    assert verify_payload.context["verify"]["url"].startswith("https://app.example.com/api/auth/verify-email")
    assert verify_payload.context["verify"]["ttl_minutes"] == auth_routes.EMAIL_OTP_TTL_MINUTES

    reset_payload = auth_routes._password_reset_template_payload(
        email="reset@example.com",
        code="654321",
    )
    assert reset_payload.context["reset"]["url"].startswith("https://app.example.com/reset-password")
    assert reset_payload.context["reset"]["ttl_minutes"] == auth_routes.PASSWORD_RESET_OTP_TTL_MINUTES


@pytest.mark.asyncio
async def test_issue_tokens_and_verification_reset_flows(db_session, monkeypatch):
    role_user = (await db_session.execute(select(Role).where(Role.name == "user"))).scalar_one()
    user = User(
        email=f"auth-flow-{uuid4().hex[:8]}@example.com",
        password_hash=hash_password("SuperSecret123"),
        display_name=f"AuthFlow-{uuid4().hex[:6]}",
        role_id=role_user.id,
        ui_locale="en",
    )
    db_session.add(user)
    await db_session.flush()

    now = datetime.now(timezone.utc)
    db_session.add(
        VerificationToken(
            user_id=user.id,
            token_hash=hash_token("111111"),
            expires_at=now + timedelta(minutes=5),
        )
    )
    db_session.add(
        PasswordResetToken(
            user_id=user.id,
            token_hash=hash_token("222222"),
            expires_at=now + timedelta(minutes=5),
        )
    )
    await db_session.commit()

    verify_code, verify_exp = await auth_routes._issue_verification_token(db_session, user_id=user.id)
    assert len(verify_code) == 6
    assert verify_exp > now
    active_verify = (
        await db_session.execute(
            select(VerificationToken)
            .where(VerificationToken.user_id == user.id)
            .where(VerificationToken.used_at.is_(None))
        )
    ).scalars().all()
    assert len(active_verify) == 1
    assert active_verify[0].token_hash == hash_token(verify_code)

    reset_code, reset_exp = await auth_routes._issue_password_reset_token(db_session, user_id=user.id)
    assert len(reset_code) == 6
    assert reset_exp > now
    active_reset = (
        await db_session.execute(
            select(PasswordResetToken)
            .where(PasswordResetToken.user_id == user.id)
            .where(PasswordResetToken.used_at.is_(None))
        )
    ).scalars().all()
    assert len(active_reset) == 1
    assert active_reset[0].token_hash == hash_token(reset_code)

    # request-email-verification nullable payload branch
    no_payload = await auth_routes.request_email_verification(payload=None, db=db_session)
    assert "eligible" in no_payload.message
    missing_user = await auth_routes.request_email_verification(
        payload=EmailVerificationRequest(email="missing@example.com"),
        db=db_session,
    )
    assert "eligible" in missing_user.message

    captured = {"verify": None, "reset": None}

    async def _capture_template(payload, defer_seconds: float = 0.0):  # noqa: ANN001
        if payload.metadata.get("kind") == "onboarding_verify":
            captured["verify"] = payload
        if payload.metadata.get("kind") == "password_reset":
            captured["reset"] = payload
        return "job-id"

    monkeypatch.setattr(auth_routes, "enqueue_template_email", _capture_template)

    verify_request = await auth_routes.request_email_verification(
        payload=EmailVerificationRequest(email=user.email),
        db=db_session,
    )
    assert "eligible" in verify_request.message
    verify_payload = captured["verify"]
    verify_code = str(verify_payload.context["verify"]["code"])

    with pytest.raises(HTTPException):
        await auth_routes.verify_email(
            payload=EmailVerificationConfirmRequest(email="missing@example.com", code=verify_code),
            db=db_session,
        )
    with pytest.raises(HTTPException):
        await auth_routes.verify_email(
            payload=EmailVerificationConfirmRequest(email=user.email, code="000000"),
            db=db_session,
        )
    verified = await auth_routes.verify_email(
        payload=EmailVerificationConfirmRequest(email=user.email, code=verify_code),
        db=db_session,
    )
    assert "verified" in verified.message.lower()

    # request-password-reset nullable payload branch
    no_payload_reset = await auth_routes.request_password_reset(payload=None, db=db_session)
    assert "eligible" in no_payload_reset.message
    missing_reset = await auth_routes.request_password_reset(
        payload=PasswordResetRequest(email="missing@example.com"),
        db=db_session,
    )
    assert "eligible" in missing_reset.message

    reset_request = await auth_routes.request_password_reset(
        payload=PasswordResetRequest(email=user.email),
        db=db_session,
    )
    assert "eligible" in reset_request.message
    reset_payload = captured["reset"]
    reset_code = str(reset_payload.context["reset"]["code"])

    with pytest.raises(HTTPException):
        await auth_routes.reset_password(
            payload=PasswordResetConfirmRequest(
                email="missing@example.com",
                code=reset_code,
                new_password="NewSecret123",
            ),
            db=db_session,
        )
    with pytest.raises(HTTPException):
        await auth_routes.reset_password(
            payload=PasswordResetConfirmRequest(
                email=user.email,
                code="999999",
                new_password="NewSecret123",
            ),
            db=db_session,
        )
    reset_done = await auth_routes.reset_password(
        payload=PasswordResetConfirmRequest(
            email=user.email,
            code=reset_code,
            new_password="NewSecret123",
        ),
        db=db_session,
    )
    assert "reset complete" in reset_done.message.lower()
    await db_session.refresh(user)
    assert verify_password("NewSecret123", user.password_hash)

    async def _raise_enqueue(_payload, defer_seconds: float = 0.0):  # noqa: ANN001
        raise RuntimeError("queue down")

    monkeypatch.setattr(auth_routes, "enqueue_template_email", _raise_enqueue)
    resilient_verify = await auth_routes.request_email_verification(
        payload=EmailVerificationRequest(email=user.email),
        db=db_session,
    )
    assert "eligible" in resilient_verify.message
    resilient_reset = await auth_routes.request_password_reset(
        payload=PasswordResetRequest(email=user.email),
        db=db_session,
    )
    assert "eligible" in resilient_reset.message


@pytest.mark.asyncio
async def test_register_rejected_when_signup_disabled(db_session, monkeypatch):
    monkeypatch.setattr(auth_routes.settings, "auth_registration_enabled", False)
    with pytest.raises(HTTPException) as signup_disabled:
        await auth_routes.register(
            payload=RegisterRequest(
                email=f"disabled-{uuid4().hex[:8]}@example.com",
                password="SuperSecret123",
                display_name=f"Disabled-{uuid4().hex[:6]}",
            ),
            db=db_session,
        )
    assert signup_disabled.value.status_code == 403
    assert signup_disabled.value.detail == auth_routes.SIGNUP_DISABLED_DETAIL


@pytest.mark.asyncio
async def test_register_login_session_optional_logout_and_update_account(db_session, monkeypatch):
    captured_templates: list[object] = []

    async def _enqueue_ok(_payload, defer_seconds: float = 0.0):  # noqa: ANN001
        captured_templates.append(_payload)
        return "job-id"

    monkeypatch.setattr(auth_routes, "enqueue_template_email", _enqueue_ok)

    user_email = f"route-user-{uuid4().hex[:8]}@example.com"
    register = await auth_routes.register(
        payload=RegisterRequest(email=user_email, password="SuperSecret123", display_name=f"RouteUser-{uuid4().hex[:6]}"),
        db=db_session,
    )
    assert register.user.email == user_email

    with pytest.raises(HTTPException):
        await auth_routes.register(
            payload=RegisterRequest(email=user_email, password="SuperSecret123", display_name=f"Another-{uuid4().hex[:6]}"),
            db=db_session,
        )

    existing = await _load_user_with_role(db_session, email=user_email)

    with pytest.raises(HTTPException):
        await auth_routes.login(
            payload=LoginRequest(email="missing@example.com", password="SuperSecret123", remember_me=False),
            request=_request_for_login(),
            background_tasks=BackgroundTasks(),
            db=db_session,
        )
    with pytest.raises(HTTPException):
        await auth_routes.login(
            payload=LoginRequest(email=user_email, password="WrongPass123", remember_me=False),
            request=_request_for_login(),
            background_tasks=BackgroundTasks(),
            db=db_session,
        )

    login_response = await auth_routes.login(
        payload=LoginRequest(email=user_email, password="SuperSecret123", remember_me=True),
        request=_request_for_login(),
        background_tasks=BackgroundTasks(),
        db=db_session,
    )
    assert login_response.status_code == 200
    assert "set-cookie" in login_response.headers

    none_session = await auth_routes.get_current_session_optional(session_token=None, db=db_session)
    assert none_session is None
    invalid_session = await auth_routes.get_current_session_optional(session_token="invalid", db=db_session)
    assert invalid_session is None

    session_token = "plain-session-token"
    db_session.add(
        Session(
            user_id=existing.id,
            token_hash=hash_token(session_token),
            expires_at=datetime.now(timezone.utc) + timedelta(days=1),
            last_seen_at=datetime.now(timezone.utc),
        )
    )
    await db_session.commit()
    valid_session = await auth_routes.get_current_session_optional(session_token=session_token, db=db_session)
    assert valid_session is not None

    response = Response()
    logout_ok = await auth_routes.logout(response=response, db=db_session, auth_session=valid_session)
    assert "logged out" in logout_ok.message.lower()
    await db_session.refresh(valid_session)
    assert valid_session.revoked_at is not None

    logout_without_session = await auth_routes.logout(response=Response(), db=db_session, auth_session=None)
    assert "logged out" in logout_without_session.message.lower()

    me = await auth_routes.me(current_user=await _load_user_with_role(db_session, email=user_email))
    assert me.user.email == user_email

    # role missing branch for register
    role_user = (await db_session.execute(select(Role).where(Role.name == "user"))).scalar_one()
    role_user.name = "member"
    await db_session.commit()
    with pytest.raises(HTTPException) as missing_role:
        await auth_routes.register(
            payload=RegisterRequest(
                email=f"role-missing-{uuid4().hex[:8]}@example.com",
                password="SuperSecret123",
                display_name=f"RoleMissing-{uuid4().hex[:6]}",
            ),
            db=db_session,
        )
    assert missing_role.value.status_code == 500
    role_user.name = "user"
    await db_session.commit()

    # register IntegrityError branch
    real_commit = AsyncSession.commit

    async def _raise_integrity(self: AsyncSession):  # type: ignore[no-untyped-def]
        raise IntegrityError("insert", params={}, orig=Exception("users.email"))

    monkeypatch.setattr(AsyncSession, "commit", _raise_integrity)
    try:
        with pytest.raises(HTTPException) as integrity_conflict:
            await auth_routes.register(
                payload=RegisterRequest(
                    email=f"integrity-{uuid4().hex[:8]}@example.com",
                    password="SuperSecret123",
                    display_name=f"Integrity-{uuid4().hex[:6]}",
                ),
                db=db_session,
            )
        assert integrity_conflict.value.status_code == 409
    finally:
        monkeypatch.setattr(AsyncSession, "commit", real_commit)

    current_user = await _load_user_with_role(db_session, email=user_email)
    other_email = f"other-{uuid4().hex[:8]}@example.com"
    other_display = f"Other-{uuid4().hex[:6]}"
    await auth_routes.register(
        payload=RegisterRequest(email=other_email, password="SuperSecret123", display_name=other_display),
        db=db_session,
    )

    no_updates = await auth_routes.update_account(
        payload=AccountUpdateRequest(),
        current_user=current_user,
        db=db_session,
    )
    assert no_updates.user.email == current_user.email

    with pytest.raises(HTTPException):
        await auth_routes.update_account(
            payload=AccountUpdateRequest(email=None),
            current_user=current_user,
            db=db_session,
        )
    with pytest.raises(HTTPException):
        await auth_routes.update_account(
            payload=AccountUpdateRequest(display_name=other_display),
            current_user=current_user,
            db=db_session,
        )
    with pytest.raises(HTTPException):
        await auth_routes.update_account(
            payload=AccountUpdateRequest(ui_locale=None),
            current_user=current_user,
            db=db_session,
        )
    with pytest.raises(HTTPException):
        await auth_routes.update_account(
            payload=AccountUpdateRequest(new_password=None),
            current_user=current_user,
            db=db_session,
        )
    with pytest.raises(HTTPException):
        await auth_routes.update_account(
            payload=AccountUpdateRequest(email=other_email),
            current_user=current_user,
            db=db_session,
        )
    with pytest.raises(HTTPException):
        await auth_routes.update_account(
            payload=AccountUpdateRequest(email=f"new-{uuid4().hex[:8]}@example.com"),
            current_user=current_user,
            db=db_session,
        )
    with pytest.raises(HTTPException):
        await auth_routes.update_account(
            payload=AccountUpdateRequest(
                email=f"new-{uuid4().hex[:8]}@example.com",
                current_password="WrongPass123",
            ),
            current_user=current_user,
            db=db_session,
        )

    async def _consume_step_up(db, *, user_id, token):  # noqa: ANN001
        return token == "valid-step-up-token"

    monkeypatch.setattr(auth_routes, "consume_passkey_step_up_token", _consume_step_up)
    step_up_email = f"stepup-{uuid4().hex[:8]}@example.com"
    updated_sensitive_with_step_up = await auth_routes.update_account(
            payload=AccountUpdateRequest(
                email=step_up_email,
                passkey_step_up_token="valid-step-up-token",
            ),
        current_user=current_user,
        db=db_session,
    )
    assert updated_sensitive_with_step_up.user.email == current_user.email
    assert updated_sensitive_with_step_up.pending_email_change_to == step_up_email

    updated_non_sensitive = await auth_routes.update_account(
        payload=AccountUpdateRequest(
            phone_number="01099998888",
            billing_address="addr",
            billing_address_line1="line1",
            billing_address_line2="line2",
            billing_city="city",
            billing_state_province="state",
            billing_country="KR",
            billing_postal_code="12345",
            birthday=datetime(2000, 1, 1, tzinfo=timezone.utc).date(),
        ),
        current_user=current_user,
        db=db_session,
    )
    assert updated_non_sensitive.user.phone_number == "01099998888"

    previous_email = current_user.email
    target_email = f"changed-{uuid4().hex[:8]}@example.com"
    updated_sensitive = await auth_routes.update_account(
        payload=AccountUpdateRequest(
            email=target_email,
            current_password="SuperSecret123",
        ),
        current_user=current_user,
        db=db_session,
    )
    assert updated_sensitive.user.email == previous_email
    assert updated_sensitive.pending_email_change_to == target_email

    updated_password = await auth_routes.update_account(
        payload=AccountUpdateRequest(
            new_password="BrandNewSecret123",
            current_password="SuperSecret123",
        ),
        current_user=current_user,
        db=db_session,
    )
    assert updated_password.user.email == previous_email
    await db_session.refresh(current_user)
    assert verify_password("BrandNewSecret123", current_user.password_hash)
    assert captured_templates
    code = str(captured_templates[-1].context["verify"]["code"])  # type: ignore[index]

    confirmed = await auth_routes.confirm_email_change(
        payload=EmailChangeConfirmRequest(email=target_email, code=code),
        current_user=current_user,
        db=db_session,
    )
    assert confirmed.user.email == target_email


@pytest.mark.asyncio
async def test_login_rehashes_password_when_policy_changes(db_session, monkeypatch):
    role_user = (await db_session.execute(select(Role).where(Role.name == "user"))).scalar_one()
    email = f"rehash-{uuid4().hex[:8]}@example.com"
    user = User(
        email=email,
        password_hash=hash_password("SuperSecret123"),
        display_name=f"Rehash-{uuid4().hex[:6]}",
        role_id=role_user.id,
        ui_locale="en",
    )
    db_session.add(user)
    await db_session.commit()

    old_hash = user.password_hash
    monkeypatch.setattr(auth_routes, "password_needs_rehash", lambda _hash: True)

    response = await auth_routes.login(
        payload=LoginRequest(email=email, password="SuperSecret123", remember_me=False),
        request=_request_for_login(),
        background_tasks=BackgroundTasks(),
        db=db_session,
    )
    assert response.status_code == 200

    await db_session.refresh(user)
    assert user.password_hash != old_hash
    assert verify_password("SuperSecret123", user.password_hash)


@pytest.mark.asyncio
async def test_login_refreshes_train_reservations_after_sign_in(db_session, monkeypatch):
    role_user = (await db_session.execute(select(Role).where(Role.name == "user"))).scalar_one()
    email = f"signin-refresh-{uuid4().hex[:8]}@example.com"
    user = User(
        email=email,
        password_hash=hash_password("SuperSecret123"),
        display_name=f"SigninRefresh-{uuid4().hex[:6]}",
        role_id=role_user.id,
        ui_locale="en",
    )
    db_session.add(user)
    await db_session.commit()

    refresh_calls: list[UUID] = []

    async def _refresh_background(*, user_id: UUID) -> None:
        refresh_calls.append(user_id)

    monkeypatch.setattr(auth_routes, "_refresh_train_reservations_after_sign_in_background", _refresh_background)

    response = await auth_routes.login(
        payload=LoginRequest(email=email, password="SuperSecret123", remember_me=False),
        request=_request_for_login(),
        background_tasks=BackgroundTasks(),
        db=db_session,
    )
    assert response.status_code == 200
    assert refresh_calls == []
    if response.background is not None:
        await response.background()
    assert refresh_calls == [user.id]


@pytest.mark.asyncio
async def test_login_keeps_sign_in_successful_when_reservation_refresh_fails(db_session, monkeypatch):
    role_user = (await db_session.execute(select(Role).where(Role.name == "user"))).scalar_one()
    email = f"signin-refresh-fail-{uuid4().hex[:8]}@example.com"
    user = User(
        email=email,
        password_hash=hash_password("SuperSecret123"),
        display_name=f"SigninRefreshFail-{uuid4().hex[:6]}",
        role_id=role_user.id,
        ui_locale="en",
    )
    db_session.add(user)
    await db_session.commit()

    class _SessionContext:
        def __init__(self, session: AsyncSession):
            self._session = session

        async def __aenter__(self) -> AsyncSession:
            return self._session

        async def __aexit__(self, exc_type, exc, tb) -> None:  # noqa: ANN001
            return None

    async def _refresh_train_reservations_after_sign_in(db, *, user):  # noqa: ANN001
        raise RuntimeError("provider unavailable")

    monkeypatch.setattr(auth_routes, "SessionLocal", lambda: _SessionContext(db_session))
    monkeypatch.setattr(auth_routes, "refresh_train_reservations_after_sign_in", _refresh_train_reservations_after_sign_in)

    response = await auth_routes.login(
        payload=LoginRequest(email=email, password="SuperSecret123", remember_me=False),
        request=_request_for_login(),
        background_tasks=BackgroundTasks(),
        db=db_session,
    )
    assert response.status_code == 200
    if response.background is not None:
        await response.background()


@pytest.mark.asyncio
async def test_register_duplicate_display_name_and_update_integrity_delete_paths(db_session, monkeypatch):
    async def _enqueue_ok(_payload, defer_seconds: float = 0.0):  # noqa: ANN001
        return "job-id"

    monkeypatch.setattr(auth_routes, "enqueue_template_email", _enqueue_ok)

    first_email = f"display-a-{uuid4().hex[:8]}@example.com"
    second_email = f"display-b-{uuid4().hex[:8]}@example.com"
    display_name = f"Display-{uuid4().hex[:6]}"

    await auth_routes.register(
        payload=RegisterRequest(email=first_email, password="SuperSecret123", display_name=display_name),
        db=db_session,
    )

    with pytest.raises(HTTPException) as duplicate_display:
        await auth_routes.register(
            payload=RegisterRequest(email=second_email, password="SuperSecret123", display_name=display_name.lower()),
            db=db_session,
        )
    assert duplicate_display.value.status_code == 400
    assert "display name already registered" in str(duplicate_display.value.detail).lower()

    current_user = await _load_user_with_role(db_session, email=first_email)
    updated = await auth_routes.update_account(
        payload=AccountUpdateRequest(ui_locale="ko"),
        current_user=current_user,
        db=db_session,
    )
    assert updated.user.ui_locale == "ko"
    updated_display = await auth_routes.update_account(
        payload=AccountUpdateRequest(display_name=f"Renamed-{uuid4().hex[:6]}"),
        current_user=current_user,
        db=db_session,
    )
    assert updated_display.user.display_name.startswith("Renamed-")

    real_commit = AsyncSession.commit

    async def _raise_integrity(self: AsyncSession):  # type: ignore[no-untyped-def]
        raise IntegrityError("update", params={}, orig=Exception("users.email"))

    monkeypatch.setattr(AsyncSession, "commit", _raise_integrity)
    try:
        with pytest.raises(HTTPException) as update_integrity:
            await auth_routes.update_account(
                payload=AccountUpdateRequest(
                    display_name=f"renamed-{uuid4().hex[:6]}",
                ),
                current_user=current_user,
                db=db_session,
            )
        assert update_integrity.value.status_code == 409
    finally:
        monkeypatch.setattr(AsyncSession, "commit", real_commit)

    calls: list[str] = []

    async def _delete_account_data(db, *, user):  # noqa: ANN001
        calls.append("called")

    monkeypatch.setattr(auth_routes, "delete_account_data", _delete_account_data)
    response = Response()
    deleted = await auth_routes.delete_account(response=response, current_user=current_user, db=db_session)
    assert "account deleted" in deleted.message.lower()
    assert calls == ["called"]


@pytest.mark.asyncio
async def test_passkey_route_units_with_mocked_service(db_session, monkeypatch):
    role_user = (await db_session.execute(select(Role).where(Role.name == "user"))).scalar_one()
    email = f"passkey-{uuid4().hex[:8]}@example.com"
    user = User(
        email=email,
        password_hash=hash_password("SuperSecret123"),
        display_name=f"Passkey-{uuid4().hex[:6]}",
        role_id=role_user.id,
        ui_locale="en",
    )
    db_session.add(user)
    await db_session.commit()
    user = await _load_user_with_role(db_session, email=email)

    monkeypatch.setattr(auth_routes, "ensure_passkeys_enabled", lambda: None)

    created_at = datetime.now(timezone.utc)
    passkey_id = uuid4()

    async def _list_passkeys(db, *, user_id):  # noqa: ANN001
        assert user_id == user.id
        return [SimpleNamespace(id=passkey_id, created_at=created_at, last_used_at=None)]

    async def _begin_registration(db, *, user):  # noqa: ANN001
        return uuid4(), {"challenge": "abc", "user": {"id": "AQID"}}

    async def _complete_registration(db, *, user, challenge_id, credential):  # noqa: ANN001
        assert credential == {"id": "cred-1"}
        return SimpleNamespace(id=passkey_id, created_at=created_at, last_used_at=None)

    async def _delete_passkey(db, *, user_id, passkey_id):  # noqa: ANN001
        return str(passkey_id) == "00000000-0000-0000-0000-000000000001"

    async def _begin_authentication(db, *, email, user):  # noqa: ANN001
        return uuid4(), {"challenge": "xyz", "allowCredentials": []}

    async def _complete_authentication(db, *, email, user, challenge_id, credential):  # noqa: ANN001
        assert credential == {"id": "cred-1"}
        return None

    refresh_calls: list[UUID] = []

    async def _refresh_background(*, user_id: UUID) -> None:
        refresh_calls.append(user_id)

    monkeypatch.setattr(auth_routes, "list_passkeys", _list_passkeys)
    monkeypatch.setattr(auth_routes, "begin_passkey_registration", _begin_registration)
    monkeypatch.setattr(auth_routes, "complete_passkey_registration", _complete_registration)
    monkeypatch.setattr(auth_routes, "delete_passkey", _delete_passkey)
    monkeypatch.setattr(auth_routes, "begin_passkey_authentication", _begin_authentication)
    monkeypatch.setattr(auth_routes, "complete_passkey_authentication", _complete_authentication)
    monkeypatch.setattr(auth_routes, "_refresh_train_reservations_after_sign_in_background", _refresh_background)

    async def _issue_step_up_token(db, *, user_id):  # noqa: ANN001
        return "step-up-token"

    monkeypatch.setattr(auth_routes, "issue_passkey_step_up_token", _issue_step_up_token)

    listed = await auth_routes.get_passkeys(current_user=user, db=db_session)
    assert len(listed.credentials) == 1
    assert listed.credentials[0].id == passkey_id

    options = await auth_routes.passkey_register_options(current_user=user, db=db_session)
    assert "challenge" in options.public_key

    verified = await auth_routes.passkey_register_verify(
        payload=PasskeyRegistrationVerifyRequest(challenge_id=options.challenge_id, credential={"id": "cred-1"}),
        current_user=user,
        db=db_session,
    )
    assert verified.id == passkey_id

    with pytest.raises(HTTPException) as missing:
        await auth_routes.remove_passkey(
            passkey_id=uuid4(),
            current_user=user,
            db=db_session,
        )
    assert missing.value.status_code == 404

    removed = await auth_routes.remove_passkey(
        passkey_id=UUID("00000000-0000-0000-0000-000000000001"),
        current_user=user,
        db=db_session,
    )
    assert "removed" in removed.message.lower()

    auth_options = await auth_routes.passkey_auth_options(
        payload=PasskeyAuthenticationOptionsRequest(email=email),
        db=db_session,
    )
    assert "challenge" in auth_options.public_key

    step_up_options = await auth_routes.passkey_step_up_options(
        current_user=user,
        db=db_session,
    )
    assert "challenge" in step_up_options.public_key

    step_up_verified = await auth_routes.passkey_step_up_verify(
        payload=PasskeyStepUpVerifyRequest(
            challenge_id=step_up_options.challenge_id,
            credential={"id": "cred-1"},
        ),
        current_user=user,
        db=db_session,
    )
    assert step_up_verified.step_up_token == "step-up-token"

    auth_response = await auth_routes.passkey_auth_verify(
        payload=PasskeyAuthenticationVerifyRequest(
            email=email,
            challenge_id=auth_options.challenge_id,
            credential={"id": "cred-1"},
            remember_me=False,
        ),
        request=_request_for_login(),
        background_tasks=BackgroundTasks(),
        db=db_session,
    )
    assert auth_response.status_code == 200
    assert "set-cookie" in auth_response.headers
    assert refresh_calls == []
    if auth_response.background is not None:
        await auth_response.background()
    assert refresh_calls == [user.id]

    verified_password = await auth_routes.verify_current_password(
        payload=PasswordVerifyRequest(current_password="SuperSecret123"),
        current_user=user,
    )
    assert "verified" in verified_password.message.lower()

    with pytest.raises(HTTPException) as invalid_password:
        await auth_routes.verify_current_password(
            payload=PasswordVerifyRequest(current_password="WrongPass123"),
            current_user=user,
        )
    assert invalid_password.value.status_code == 401


@pytest.mark.asyncio
async def test_update_account_email_change_enqueue_failure_returns_500(db_session, monkeypatch):
    role_user = (await db_session.execute(select(Role).where(Role.name == "user"))).scalar_one()
    email = f"email-change-fail-{uuid4().hex[:8]}@example.com"
    user = User(
        email=email,
        password_hash=hash_password("SuperSecret123"),
        display_name=f"EmailFail-{uuid4().hex[:6]}",
        role_id=role_user.id,
        ui_locale="en",
    )
    db_session.add(user)
    await db_session.commit()
    user = await _load_user_with_role(db_session, email=email)

    async def _enqueue_fail(_payload, defer_seconds: float = 0.0):  # noqa: ANN001
        raise RuntimeError("email queue down")

    monkeypatch.setattr(auth_routes, "enqueue_template_email", _enqueue_fail)

    with pytest.raises(HTTPException) as enqueue_error:
        await auth_routes.update_account(
            payload=AccountUpdateRequest(
                email=f"new-{uuid4().hex[:8]}@example.com",
                current_password="SuperSecret123",
            ),
            current_user=user,
            db=db_session,
        )
    assert enqueue_error.value.status_code == 500
    assert "Could not send email verification for address change" in str(enqueue_error.value.detail)


@pytest.mark.asyncio
async def test_confirm_email_change_rejects_invalid_and_duplicate_target_email(db_session):
    role_user = (await db_session.execute(select(Role).where(Role.name == "user"))).scalar_one()
    user = User(
        email=f"confirm-src-{uuid4().hex[:8]}@example.com",
        password_hash=hash_password("SuperSecret123"),
        display_name=f"ConfirmSrc-{uuid4().hex[:6]}",
        role_id=role_user.id,
        ui_locale="en",
    )
    other_user = User(
        email=f"confirm-dst-{uuid4().hex[:8]}@example.com",
        password_hash=hash_password("SuperSecret123"),
        display_name=f"ConfirmDst-{uuid4().hex[:6]}",
        role_id=role_user.id,
        ui_locale="en",
    )
    db_session.add_all([user, other_user])
    await db_session.commit()
    user = await _load_user_with_role(db_session, email=user.email)

    with pytest.raises(HTTPException) as invalid_code:
        await auth_routes.confirm_email_change(
            payload=EmailChangeConfirmRequest(email=f"new-{uuid4().hex[:8]}@example.com", code="000000"),
            current_user=user,
            db=db_session,
        )
    assert invalid_code.value.status_code == 400
    assert "Invalid or expired verification code" in str(invalid_code.value.detail)

    duplicate_target = other_user.email
    code, _ = await auth_routes._issue_verification_token(  # noqa: SLF001
        db_session,
        user_id=user.id,
        purpose=auth_routes.VERIFICATION_PURPOSE_EMAIL_CHANGE,
        target_email=duplicate_target,
    )
    with pytest.raises(HTTPException) as duplicate_email:
        await auth_routes.confirm_email_change(
            payload=EmailChangeConfirmRequest(email=duplicate_target, code=code),
            current_user=user,
            db=db_session,
        )
    assert duplicate_email.value.status_code == 400
    assert "Email already registered" in str(duplicate_email.value.detail)


@pytest.mark.asyncio
async def test_passkey_auth_routes_reject_unknown_email(db_session, monkeypatch):
    monkeypatch.setattr(auth_routes, "ensure_passkeys_enabled", lambda: None)

    with pytest.raises(HTTPException) as options_error:
        await auth_routes.passkey_auth_options(
            payload=PasskeyAuthenticationOptionsRequest(email=f"missing-{uuid4().hex[:8]}@example.com"),
            db=db_session,
        )
    assert options_error.value.status_code == 400
    assert "No passkey registered for this account" in str(options_error.value.detail)

    with pytest.raises(HTTPException) as verify_error:
        await auth_routes.passkey_auth_verify(
            payload=PasskeyAuthenticationVerifyRequest(
                email=f"missing-{uuid4().hex[:8]}@example.com",
                challenge_id=uuid4(),
                credential={"id": "cred-1"},
                remember_me=False,
            ),
            request=_request_for_login(),
            background_tasks=BackgroundTasks(),
            db=db_session,
        )
    assert verify_error.value.status_code == 400
    assert "Passkey authentication failed" in str(verify_error.value.detail)
