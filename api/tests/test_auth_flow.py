from datetime import datetime, timedelta, timezone
from types import SimpleNamespace
from uuid import uuid4

import fakeredis.aioredis
import pytest
from sqlalchemy import select
from sqlalchemy.exc import IntegrityError

from app.core.security import hash_token
from app.db.models import PasswordResetToken, Secret, Session, Task, User, VerificationToken
from app.services.wallet import LEGACY_PAYMENT_CVV_REDIS_KEY_PREFIX, PAYMENT_CVV_REDIS_KEY_PREFIX
from tests.conftest import MockRedisContextManager


@pytest.fixture(autouse=True)
def _disable_access_approval_for_auth_flow(monkeypatch):
    monkeypatch.setattr("app.http.deps.settings.access_approval_required", False)


def _extract_otp_code(template_payload) -> str:
    for block in template_payload.blocks:
        if block.type == "otp":
            code_value = block.data.get("code")
            if isinstance(code_value, dict) and "$ref" in code_value:
                pointer = str(code_value["$ref"])
                current = template_payload.context
                for segment in pointer.split("."):
                    current = current[segment]
                return str(current)
            return str(code_value)
    raise AssertionError("OTP block not found")


def _as_utc(value: datetime) -> datetime:
    return value if value.tzinfo is not None else value.replace(tzinfo=timezone.utc)


@pytest.mark.asyncio
async def test_register_login_me_logout_flow(client):
    register_payload = {
        "email": "user@example.com",
        "password": "SuperSecret123",
        "display_name": "Bloom User",
    }
    register_res = await client.post("/api/auth/register", json=register_payload)
    assert register_res.status_code == 201
    assert register_res.json()["user"]["email"] == "user@example.com"
    assert register_res.json()["user"]["ui_locale"] == "en"

    login_res = await client.post(
        "/api/auth/login",
        json={"email": "user@example.com", "password": "SuperSecret123", "remember_me": True},
    )
    assert login_res.status_code == 200
    assert login_res.json()["user"]["ui_locale"] == "en"
    set_cookie = login_res.headers.get("set-cookie", "")
    assert "Max-Age=7776000" in set_cookie

    session_cookie = login_res.cookies.get("bominal_session")
    assert session_cookie

    me_res = await client.get("/api/auth/me", cookies={"bominal_session": session_cookie})
    assert me_res.status_code == 200
    me_json = me_res.json()
    assert me_json["user"]["email"] == "user@example.com"
    assert me_json["user"]["role"] == "user"
    assert me_json["user"]["ui_locale"] == "en"

    logout_res = await client.post("/api/auth/logout", cookies={"bominal_session": session_cookie})
    assert logout_res.status_code == 200

    me_after_logout = await client.get("/api/auth/me", cookies={"bominal_session": session_cookie})
    assert me_after_logout.status_code == 401


@pytest.mark.asyncio
async def test_session_activity_last_seen_writes_are_debounced(client, db_session, monkeypatch):
    email = f"debounce-{uuid4().hex[:8]}@example.com"
    password = "SuperSecret123"
    await client.post(
        "/api/auth/register",
        json={"email": email, "password": password, "display_name": "Debounce User"},
    )
    login_res = await client.post(
        "/api/auth/login",
        json={"email": email, "password": password, "remember_me": False},
    )
    session_cookie = login_res.cookies.get("bominal_session")
    assert session_cookie

    token_hash = hash_token(session_cookie)
    session_row = (await db_session.execute(select(Session).where(Session.token_hash == token_hash))).scalar_one()
    initial_last_seen = _as_utc(session_row.last_seen_at)

    monkeypatch.setattr("app.http.deps.settings.session_activity_debounce_seconds", 60)

    within_debounce_window = initial_last_seen + timedelta(seconds=10)
    monkeypatch.setattr(
        "app.http.deps.datetime",
        SimpleNamespace(now=lambda tz=None, value=within_debounce_window: value if tz else value.replace(tzinfo=None)),
    )
    first_me = await client.get("/api/auth/me", cookies={"bominal_session": session_cookie})
    assert first_me.status_code == 200

    db_session.expire_all()
    session_after_first_me = (await db_session.execute(select(Session).where(Session.token_hash == token_hash))).scalar_one()
    assert _as_utc(session_after_first_me.last_seen_at) == initial_last_seen

    after_debounce_window = initial_last_seen + timedelta(seconds=61)
    monkeypatch.setattr(
        "app.http.deps.datetime",
        SimpleNamespace(now=lambda tz=None, value=after_debounce_window: value if tz else value.replace(tzinfo=None)),
    )
    second_me = await client.get("/api/auth/me", cookies={"bominal_session": session_cookie})
    assert second_me.status_code == 200

    db_session.expire_all()
    session_after_second_me = (await db_session.execute(select(Session).where(Session.token_hash == token_hash))).scalar_one()
    assert _as_utc(session_after_second_me.last_seen_at) == after_debounce_window


@pytest.mark.asyncio
async def test_me_rejects_expired_session_when_activity_debounce_enabled(client, db_session, monkeypatch):
    email = f"expired-{uuid4().hex[:8]}@example.com"
    password = "SuperSecret123"
    await client.post(
        "/api/auth/register",
        json={"email": email, "password": password, "display_name": "Expired User"},
    )
    login_res = await client.post(
        "/api/auth/login",
        json={"email": email, "password": password, "remember_me": False},
    )
    session_cookie = login_res.cookies.get("bominal_session")
    assert session_cookie

    token_hash = hash_token(session_cookie)
    session_row = (await db_session.execute(select(Session).where(Session.token_hash == token_hash))).scalar_one()
    session_row.expires_at = datetime.now(timezone.utc) - timedelta(seconds=1)
    await db_session.commit()

    monkeypatch.setattr("app.http.deps.settings.session_activity_debounce_seconds", 3600)
    me_res = await client.get("/api/auth/me", cookies={"bominal_session": session_cookie})
    assert me_res.status_code == 401


@pytest.mark.asyncio
async def test_register_enqueues_onboarding_verification_email(client, monkeypatch):
    captured: dict[str, object] = {}

    async def _fake_enqueue(payload, defer_seconds: float = 0.0):
        captured["payload"] = payload
        captured["defer_seconds"] = defer_seconds
        return "job-onboarding-1"

    monkeypatch.setattr("app.http.routes.auth.enqueue_template_email", _fake_enqueue)
    monkeypatch.setattr("app.http.routes.auth._public_base_url", lambda: "https://mail.example.com")

    email = f"onboarding-{uuid4().hex[:8]}@example.com"
    register_res = await client.post(
        "/api/auth/register",
        json={"email": email, "password": "SuperSecret123", "display_name": "Onboarding User"},
    )
    assert register_res.status_code == 201
    payload = captured["payload"]
    assert payload.to_email == email
    assert "Verify your email" in payload.subject
    assert any(block.type == "cta" for block in payload.blocks)
    assert any(block.type == "otp" for block in payload.blocks)
    assert payload.context["verify"]["url"].startswith("https://mail.example.com/api/auth/verify-email")


@pytest.mark.asyncio
async def test_request_email_verification_and_verify_email_with_otp(client, monkeypatch, db_session):
    captured: dict[str, object] = {}

    async def _fake_enqueue(payload, defer_seconds: float = 0.0):
        captured["payload"] = payload
        captured["defer_seconds"] = defer_seconds
        return "job-verify-1"

    monkeypatch.setattr("app.http.routes.auth.enqueue_template_email", _fake_enqueue)

    email = f"verify-{uuid4().hex[:8]}@example.com"
    await client.post(
        "/api/auth/register",
        json={"email": email, "password": "SuperSecret123", "display_name": "Verify User"},
    )

    request_res = await client.post("/api/auth/request-email-verification", json={"email": email})
    assert request_res.status_code == 200
    otp = _extract_otp_code(captured["payload"])

    verify_res = await client.post("/api/auth/verify-email", json={"email": email, "code": otp})
    assert verify_res.status_code == 200
    assert "verified" in verify_res.json()["message"].lower()

    user = (await db_session.execute(select(User).where(User.email == email))).scalar_one()
    assert user.email_verified_at is not None


@pytest.mark.asyncio
async def test_request_password_reset_and_reset_password_with_otp(client, monkeypatch, db_session):
    captured: dict[str, object] = {}

    async def _fake_enqueue(payload, defer_seconds: float = 0.0):
        captured["payload"] = payload
        return "job-reset-1"

    monkeypatch.setattr("app.http.routes.auth.enqueue_template_email", _fake_enqueue)
    monkeypatch.setattr("app.http.routes.auth._public_base_url", lambda: "https://app.example.com")

    email = f"reset-{uuid4().hex[:8]}@example.com"
    await client.post(
        "/api/auth/register",
        json={"email": email, "password": "SuperSecret123", "display_name": "Reset User"},
    )

    request_res = await client.post("/api/auth/request-password-reset", json={"email": email})
    assert request_res.status_code == 200
    reset_payload = captured["payload"]
    assert reset_payload.context["reset"]["ttl_minutes"] == 15
    assert reset_payload.context["reset"]["url"].startswith("https://app.example.com/reset-password?email=")
    otp = _extract_otp_code(captured["payload"])

    user = (await db_session.execute(select(User).where(User.email == email))).scalar_one()
    reset_token = (
        await db_session.execute(
            select(PasswordResetToken)
            .where(PasswordResetToken.user_id == user.id)
            .where(PasswordResetToken.used_at.is_(None))
        )
    ).scalar_one()
    now = datetime.now(timezone.utc)
    expires_delta = _as_utc(reset_token.expires_at) - now
    assert 14 * 60 <= expires_delta.total_seconds() <= 15 * 60

    reset_res = await client.post(
        "/api/auth/reset-password",
        json={"email": email, "code": otp, "new_password": "NewPass12345"},
    )
    assert reset_res.status_code == 200
    assert reset_res.cookies.get("bominal_session")
    assert reset_res.cookies.get("bominal_passkey_setup_ctx")

    old_login = await client.post(
        "/api/auth/login",
        json={"email": email, "password": "SuperSecret123", "remember_me": False},
    )
    assert old_login.status_code == 401

    new_login = await client.post(
        "/api/auth/login",
        json={"email": email, "password": "NewPass12345", "remember_me": False},
    )
    assert new_login.status_code == 200


@pytest.mark.asyncio
async def test_login_allows_supabase_password_fallback_when_local_hash_mismatch(client, monkeypatch):
    email = f"supabase-fallback-{uuid4().hex[:8]}@example.com"
    await client.post(
        "/api/auth/register",
        json={"email": email, "password": "LocalOnly123", "display_name": "Supabase Fallback User"},
    )

    monkeypatch.setattr("app.http.routes.auth.settings.auth_mode", "supabase")
    monkeypatch.setattr("app.http.routes.auth.settings.supabase_auth_enabled", True)

    called = {"count": 0}

    async def _fake_verify_supabase_password(*, email: str, password: str):  # noqa: ARG001
        called["count"] += 1
        return type("Identity", (), {"user_id": "supabase-user-001", "email": email})()

    monkeypatch.setattr(
        "app.http.routes.auth.verify_supabase_password",
        _fake_verify_supabase_password,
        raising=False,
    )

    login_res = await client.post(
        "/api/auth/login",
        json={"email": email, "password": "SupabasePass123", "remember_me": False},
    )

    assert login_res.status_code == 200
    assert called["count"] == 1


@pytest.mark.asyncio
async def test_request_password_reset_uses_supabase_recovery_in_supabase_mode(client, monkeypatch):
    email = f"supabase-recovery-{uuid4().hex[:8]}@example.com"
    await client.post(
        "/api/auth/register",
        json={"email": email, "password": "SuperSecret123", "display_name": "Supabase Recovery User"},
    )

    monkeypatch.setattr("app.http.routes.auth.settings.auth_mode", "supabase")
    monkeypatch.setattr("app.http.routes.auth.settings.supabase_auth_enabled", True)
    monkeypatch.setattr("app.http.routes.auth._public_base_url", lambda: "https://www.bominal.com")

    captured: dict[str, str | None] = {"email": None, "redirect_to": None}
    enqueued = {"count": 0}
    issued_tokens = {"count": 0}

    async def _fake_send_supabase_password_recovery(*, email: str, redirect_to: str | None = None):
        captured["email"] = email
        captured["redirect_to"] = redirect_to
        return True

    async def _fake_enqueue(payload, defer_seconds: float = 0.0):  # noqa: ARG001
        enqueued["count"] += 1
        return "job-reset-123"

    async def _fake_issue_password_reset_token(*_args, **_kwargs):  # noqa: ANN002, ANN003
        issued_tokens["count"] += 1
        return ("123456", datetime.now(timezone.utc) + timedelta(minutes=15))

    monkeypatch.setattr(
        "app.http.routes.auth.send_supabase_password_recovery",
        _fake_send_supabase_password_recovery,
        raising=False,
    )
    monkeypatch.setattr("app.http.routes.auth.enqueue_template_email", _fake_enqueue)
    monkeypatch.setattr("app.http.routes.auth._issue_password_reset_token", _fake_issue_password_reset_token)

    response = await client.post("/api/auth/request-password-reset", json={"email": email})
    assert response.status_code == 200
    assert captured["email"] == email
    assert captured["redirect_to"] is not None
    assert captured["redirect_to"].endswith("/auth/verify?type=recovery")
    assert enqueued["count"] == 0
    assert issued_tokens["count"] == 0


@pytest.mark.asyncio
async def test_request_password_reset_uses_supabase_recovery_even_when_local_user_missing(client, monkeypatch):
    submitted_email = f"supabase-only-missing-{uuid4().hex[:8]}@example.com"

    monkeypatch.setattr("app.http.routes.auth.settings.auth_mode", "supabase")
    monkeypatch.setattr("app.http.routes.auth.settings.supabase_auth_enabled", True)
    monkeypatch.setattr("app.http.routes.auth._public_base_url", lambda: "https://www.bominal.com")

    captured: dict[str, str | None] = {"email": None, "redirect_to": None}
    enqueued = {"count": 0}
    delivered = {"count": 0}

    async def _fake_send_supabase_password_recovery(*, email: str, redirect_to: str | None = None):
        captured["email"] = email
        captured["redirect_to"] = redirect_to
        return True

    async def _fake_enqueue(*_args, **_kwargs):  # noqa: ANN002, ANN003
        enqueued["count"] += 1
        return "job-reset-123"

    async def _fake_deliver_email_job(*_args, **_kwargs):  # noqa: ANN002, ANN003
        delivered["count"] += 1
        return {"status": "sent"}

    monkeypatch.setattr(
        "app.http.routes.auth.send_supabase_password_recovery",
        _fake_send_supabase_password_recovery,
        raising=False,
    )
    monkeypatch.setattr("app.http.routes.auth.enqueue_template_email", _fake_enqueue)
    monkeypatch.setattr("app.http.routes.auth.deliver_email_job", _fake_deliver_email_job, raising=False)

    response = await client.post("/api/auth/request-password-reset", json={"email": submitted_email})
    assert response.status_code == 200
    assert captured["email"] == submitted_email
    assert captured["redirect_to"] is not None
    assert captured["redirect_to"].endswith("/auth/verify?type=recovery")
    assert enqueued["count"] == 0
    assert delivered["count"] == 0


@pytest.mark.asyncio
async def test_request_password_reset_supabase_recover_failure_still_returns_generic_success(client, monkeypatch):
    email = f"supabase-recovery-failure-{uuid4().hex[:8]}@example.com"
    await client.post(
        "/api/auth/register",
        json={"email": email, "password": "SuperSecret123", "display_name": "Supabase Recovery Failure User"},
    )

    monkeypatch.setattr("app.http.routes.auth.settings.auth_mode", "supabase")
    monkeypatch.setattr("app.http.routes.auth.settings.supabase_auth_enabled", True)
    monkeypatch.setattr("app.http.routes.auth._public_base_url", lambda: "https://www.bominal.com")

    warnings: list[str] = []
    enqueued = {"count": 0}

    async def _fake_send_supabase_password_recovery(*, email: str, redirect_to: str | None = None):  # noqa: ARG001
        return False

    async def _fake_enqueue(*_args, **_kwargs):  # noqa: ANN002, ANN003
        enqueued["count"] += 1
        return "job-reset-123"

    def _capture_warning(message: str, *args, **kwargs):  # noqa: ANN002, ANN003
        formatted = message % args if args else message
        warnings.append(formatted)

    monkeypatch.setattr(
        "app.http.routes.auth.send_supabase_password_recovery",
        _fake_send_supabase_password_recovery,
        raising=False,
    )
    monkeypatch.setattr("app.http.routes.auth.enqueue_template_email", _fake_enqueue)
    monkeypatch.setattr("app.http.routes.auth.logger.warning", _capture_warning)

    response = await client.post("/api/auth/request-password-reset", json={"email": email})
    assert response.status_code == 200
    assert enqueued["count"] == 0
    assert any("Supabase password recovery request failed" in entry for entry in warnings)


@pytest.mark.asyncio
async def test_request_password_reset_logs_structured_outcome_fields(client, monkeypatch):
    submitted_email = f"supabase-log-outcome-{uuid4().hex[:8]}@example.com"

    monkeypatch.setattr("app.http.routes.auth.settings.auth_mode", "supabase")
    monkeypatch.setattr("app.http.routes.auth.settings.supabase_auth_enabled", True)
    monkeypatch.setattr("app.http.routes.auth._public_base_url", lambda: "https://www.bominal.com")

    infos: list[dict[str, object]] = []

    async def _fake_send_supabase_password_recovery(*, email: str, redirect_to: str | None = None):  # noqa: ARG001
        return True

    def _capture_info(message: str, *args, **kwargs):  # noqa: ANN002, ANN003
        extra = kwargs.get("extra")
        if isinstance(extra, dict):
            infos.append(extra)

    monkeypatch.setattr(
        "app.http.routes.auth.send_supabase_password_recovery",
        _fake_send_supabase_password_recovery,
        raising=False,
    )
    monkeypatch.setattr("app.http.routes.auth.logger.info", _capture_info)

    response = await client.post("/api/auth/request-password-reset", json={"email": submitted_email})
    assert response.status_code == 200
    assert infos, "expected structured info log payload"
    extra = infos[-1]
    assert extra.get("mode") == "supabase"
    assert extra.get("local_user_found") is False
    assert extra.get("supabase_recovery_requested") is True
    assert extra.get("supabase_recovery_ok") is True
    assert extra.get("local_reset_enqueued") is False
    assert extra.get("local_reset_direct_fallback") is False


@pytest.mark.asyncio
async def test_request_password_reset_falls_back_to_direct_delivery_when_enqueue_fails(client, monkeypatch):
    email = f"reset-direct-fallback-{uuid4().hex[:8]}@example.com"
    await client.post(
        "/api/auth/register",
        json={"email": email, "password": "SuperSecret123", "display_name": "Reset Direct Fallback User"},
    )

    async def _fail_enqueue(*_args, **_kwargs):  # noqa: ANN002, ANN003
        raise RuntimeError("queue unavailable")

    delivered = {"count": 0}

    async def _fake_deliver_email_job(*_args, **_kwargs):  # noqa: ANN002, ANN003
        delivered["count"] += 1
        return {"status": "sent"}

    monkeypatch.setattr("app.http.routes.auth.enqueue_template_email", _fail_enqueue)
    monkeypatch.setattr(
        "app.http.routes.auth.deliver_email_job",
        _fake_deliver_email_job,
        raising=False,
    )

    response = await client.post("/api/auth/request-password-reset", json={"email": email})
    assert response.status_code == 200
    assert delivered["count"] == 1


@pytest.mark.asyncio
async def test_auth_methods_exposes_capabilities(client, monkeypatch):
    monkeypatch.setattr("app.http.routes.auth.settings.passkey_enabled", True)
    monkeypatch.setattr("app.http.routes.auth.settings.auth_mode", "supabase")
    monkeypatch.setattr("app.http.routes.auth.settings.supabase_auth_enabled", True)
    monkeypatch.setattr("app.http.routes.auth.settings.supabase_signin_otp_enabled", True)

    response = await client.get("/api/auth/methods")
    assert response.status_code == 200
    assert response.json() == {
        "password": True,
        "passkey": True,
        "magic_link": True,
        "otp": True,
    }


@pytest.mark.asyncio
async def test_auth_methods_exposes_passkey_for_dev_demo_mode(client, monkeypatch):
    monkeypatch.setattr("app.http.routes.auth.settings.passkey_enabled", False)
    monkeypatch.setattr("app.http.routes.auth.settings.dev_demo_auth_enabled", True)

    response = await client.get("/api/auth/methods")
    assert response.status_code == 200
    assert response.json()["passkey"] is True


@pytest.mark.asyncio
async def test_dev_demo_login_authenticates_and_provisions_user(client, db_session, monkeypatch):
    monkeypatch.setattr("app.http.routes.auth.settings.dev_demo_auth_enabled", True)
    monkeypatch.setattr("app.http.routes.auth.settings.dev_demo_email", "demo@bominal.dev")
    monkeypatch.setattr("app.http.routes.auth.settings.dev_demo_password", "demo-passkey-123")
    monkeypatch.setattr("app.http.routes.auth.settings.dev_demo_role", "admin")
    monkeypatch.setattr("app.http.routes.auth.settings.auth_mode", "legacy")

    response = await client.post(
        "/api/auth/login",
        json={"email": "demo@bominal.dev", "password": "demo-passkey-123", "remember_me": False},
    )
    assert response.status_code == 200
    assert response.json()["user"]["email"] == "demo@bominal.dev"
    assert response.json()["user"]["role"] == "admin"
    session_cookie = response.cookies.get("bominal_session")
    assert session_cookie

    second = await client.post(
        "/api/auth/login",
        json={"email": "demo@bominal.dev", "password": "demo-passkey-123", "remember_me": False},
    )
    assert second.status_code == 200
    users = (await db_session.execute(select(User).where(User.email == "demo@bominal.dev"))).scalars().all()
    assert len(users) == 1
    assert users[0].access_status == "approved"
    assert users[0].role_id == 1


@pytest.mark.asyncio
async def test_dev_demo_passkey_endpoint_authenticates_without_webauthn(client, monkeypatch):
    monkeypatch.setattr("app.http.routes.auth.settings.dev_demo_auth_enabled", True)
    monkeypatch.setattr("app.http.routes.auth.settings.dev_demo_email", "demo@bominal.dev")
    monkeypatch.setattr("app.http.routes.auth.settings.dev_demo_password", "demo-passkey-123")
    monkeypatch.setattr("app.http.routes.auth.settings.dev_demo_role", "admin")

    response = await client.post(
        "/api/auth/passkeys/auth/dev-demo",
        json={"email": "demo@bominal.dev", "remember_me": True},
    )
    assert response.status_code == 200
    assert response.json()["user"]["email"] == "demo@bominal.dev"
    assert response.cookies.get("bominal_session")

    me_res = await client.get("/api/auth/me", cookies={"bominal_session": response.cookies["bominal_session"]})
    assert me_res.status_code == 200
    assert me_res.json()["user"]["email"] == "demo@bominal.dev"


@pytest.mark.asyncio
async def test_request_magic_link_legacy_enqueues_for_local_user(client, monkeypatch, db_session):
    email = f"magic-link-{uuid4().hex[:8]}@example.com"
    await client.post(
        "/api/auth/register",
        json={"email": email, "password": "SuperSecret123", "display_name": "Magic User"},
    )

    captured: dict[str, object] = {}

    async def _fake_enqueue(payload, defer_seconds: float = 0.0):  # noqa: ARG001
        captured["payload"] = payload
        return "job-magic-1"

    monkeypatch.setattr("app.http.routes.auth.enqueue_template_email", _fake_enqueue)
    monkeypatch.setattr("app.http.routes.auth._public_base_url", lambda: "https://www.bominal.com")

    response = await client.post("/api/auth/request-magic-link", json={"email": email})
    assert response.status_code == 200
    assert response.json()["message"] == "If eligible, a sign-in link has been sent"
    assert "payload" in captured
    code = _extract_otp_code(captured["payload"])
    assert code

    user = (await db_session.execute(select(User).where(User.email == email))).scalar_one()
    token = (
        await db_session.execute(
            select(VerificationToken)
            .where(VerificationToken.user_id == user.id)
            .where(VerificationToken.purpose == "magic_login")
            .where(VerificationToken.used_at.is_(None))
        )
    ).scalar_one_or_none()
    assert token is not None


@pytest.mark.asyncio
async def test_magic_link_confirm_legacy_sets_session_cookie(client, monkeypatch):
    email = f"magic-confirm-{uuid4().hex[:8]}@example.com"
    await client.post(
        "/api/auth/register",
        json={"email": email, "password": "SuperSecret123", "display_name": "Magic Confirm"},
    )

    captured: dict[str, object] = {}

    async def _fake_enqueue(payload, defer_seconds: float = 0.0):  # noqa: ARG001
        captured["payload"] = payload
        return "job-magic-2"

    monkeypatch.setattr("app.http.routes.auth.enqueue_template_email", _fake_enqueue)
    monkeypatch.setattr("app.http.routes.auth._public_base_url", lambda: "https://www.bominal.com")

    request_res = await client.post("/api/auth/request-magic-link", json={"email": email})
    assert request_res.status_code == 200
    code = _extract_otp_code(captured["payload"])

    confirm_res = await client.post(
        "/api/auth/magic-link/confirm",
        json={"email": email, "code": code},
    )
    assert confirm_res.status_code == 200
    assert confirm_res.cookies.get("bominal_session")
    assert confirm_res.cookies.get("bominal_passkey_setup_ctx")


@pytest.mark.asyncio
async def test_request_magic_link_supabase_calls_provider(client, monkeypatch):
    monkeypatch.setattr("app.http.routes.auth.settings.auth_mode", "supabase")
    monkeypatch.setattr("app.http.routes.auth.settings.supabase_auth_enabled", True)
    monkeypatch.setattr("app.http.routes.auth._public_base_url", lambda: "https://www.bominal.com")

    captured: dict[str, str | None] = {"email": None, "redirect_to": None}

    async def _fake_send(*, email: str, redirect_to: str | None = None):
        captured["email"] = email
        captured["redirect_to"] = redirect_to
        return True

    monkeypatch.setattr("app.http.routes.auth.send_supabase_magic_link", _fake_send, raising=False)

    response = await client.post("/api/auth/request-magic-link", json={"email": "supabase-magic@example.com"})
    assert response.status_code == 200
    assert captured["email"] == "supabase-magic@example.com"
    assert captured["redirect_to"] is not None
    assert captured["redirect_to"].endswith("/auth/verify?type=email")


@pytest.mark.asyncio
async def test_request_signin_otp_calls_provider_only_when_enabled(client, monkeypatch):
    captured: dict[str, str | None] = {"email": None}

    async def _fake_send(*, email: str):
        captured["email"] = email
        return True

    monkeypatch.setattr("app.http.routes.auth.send_supabase_signin_otp", _fake_send, raising=False)
    monkeypatch.setattr("app.http.routes.auth.settings.auth_mode", "supabase")
    monkeypatch.setattr("app.http.routes.auth.settings.supabase_auth_enabled", True)
    monkeypatch.setattr("app.http.routes.auth.settings.supabase_signin_otp_enabled", False)

    disabled_response = await client.post("/api/auth/request-signin-otp", json={"email": "otp-disabled@example.com"})
    assert disabled_response.status_code == 200
    assert disabled_response.json()["message"] == "If eligible, a sign-in code has been sent"
    assert captured["email"] is None

    monkeypatch.setattr("app.http.routes.auth.settings.supabase_signin_otp_enabled", True)
    enabled_response = await client.post("/api/auth/request-signin-otp", json={"email": "otp-enabled@example.com"})
    assert enabled_response.status_code == 200
    assert enabled_response.json()["message"] == "If eligible, a sign-in code has been sent"
    assert captured["email"] == "otp-enabled@example.com"


@pytest.mark.asyncio
async def test_verify_signin_otp_sets_session_when_enabled(client, monkeypatch):
    monkeypatch.setattr("app.http.routes.auth.settings.auth_mode", "supabase")
    monkeypatch.setattr("app.http.routes.auth.settings.supabase_auth_enabled", True)
    monkeypatch.setattr("app.http.routes.auth.settings.supabase_signin_otp_enabled", True)

    async def _fake_verify(*, email: str, code: str):  # noqa: ARG001
        return SimpleNamespace(user_id="supabase-user-otp", email=email, access_token="access-otp")

    monkeypatch.setattr("app.http.routes.auth.verify_supabase_signin_otp", _fake_verify, raising=False)

    response = await client.post(
        "/api/auth/verify-signin-otp",
        json={"email": "otp-signin@example.com", "code": "123456", "remember_me": False},
    )
    assert response.status_code == 200
    assert response.cookies.get("bominal_session")


@pytest.mark.asyncio
async def test_supabase_confirm_returns_recovery_mode_payload(client, monkeypatch):
    async def _fake_exchange_detailed(*, token_hash: str, token_type: str):  # noqa: ARG001
        return SimpleNamespace(
            session=SimpleNamespace(
                user_id="supabase-user-001",
                email="user@example.com",
                access_token="access-token-123",
            ),
            failure=None,
        )

    monkeypatch.setattr(
        "app.http.routes.auth.exchange_supabase_token_hash_detailed",
        _fake_exchange_detailed,
        raising=False,
    )

    response = await client.post(
        "/api/auth/supabase/confirm",
        json={"token_hash": "hash-abc", "type": "recovery"},
    )
    assert response.status_code == 200
    payload = response.json()
    assert payload["mode"] == "recovery"
    assert payload.get("access_token") in {None, ""}
    assert payload["redirect_to"].endswith("/reset-password")
    assert response.cookies.get("bominal_supabase_recovery_mode") == "1"
    assert response.cookies.get("bominal_supabase_recovery_access") == "access-token-123"


@pytest.mark.asyncio
async def test_supabase_recovery_confirm_sets_cookie_used_by_reset(client, monkeypatch):
    email = f"supabase-cookie-reset-{uuid4().hex[:8]}@example.com"
    await client.post(
        "/api/auth/register",
        json={"email": email, "password": "OldPassword123", "display_name": "Supabase Cookie Reset User"},
    )

    async def _fake_exchange_detailed(*, token_hash: str, token_type: str):  # noqa: ARG001
        return SimpleNamespace(
            session=SimpleNamespace(
                user_id="supabase-user-cookie-reset",
                email=email,
                access_token="access-token-cookie-123",
                refresh_token="refresh-token-cookie-123",
            ),
            failure=None,
        )

    async def _fake_update_supabase_password(
        *,
        access_token: str,
        new_password: str,
        refresh_token: str | None = None,  # noqa: ARG001
    ):
        assert access_token == "access-token-cookie-123"
        assert refresh_token == "refresh-token-cookie-123"
        return type("Identity", (), {"user_id": "supabase-user-cookie-reset", "email": email})()

    monkeypatch.setattr(
        "app.http.routes.auth.exchange_supabase_token_hash_detailed",
        _fake_exchange_detailed,
        raising=False,
    )
    monkeypatch.setattr("app.http.routes.auth.update_supabase_password", _fake_update_supabase_password, raising=False)

    confirm_response = await client.post(
        "/api/auth/supabase/confirm",
        json={"token_hash": "hash-cookie-reset", "type": "recovery"},
    )
    assert confirm_response.status_code == 200
    assert confirm_response.cookies.get("bominal_supabase_recovery_mode") == "1"

    reset_response = await client.post(
        "/api/auth/reset-password/supabase",
        json={"new_password": "NewPassword123"},
    )
    assert reset_response.status_code == 200


@pytest.mark.asyncio
async def test_supabase_confirm_magiclink_sets_cookie(client, monkeypatch):
    async def _fake_exchange_detailed(*, token_hash: str, token_type: str):  # noqa: ARG001
        return SimpleNamespace(
            session=SimpleNamespace(
                user_id="supabase-user-001",
                email="magiclink@example.com",
                access_token="access-token-123",
            ),
            failure=None,
        )

    monkeypatch.setattr(
        "app.http.routes.auth.exchange_supabase_token_hash_detailed",
        _fake_exchange_detailed,
        raising=False,
    )

    response = await client.post(
        "/api/auth/supabase/confirm",
        json={"token_hash": "hash-abc", "type": "magiclink"},
    )
    assert response.status_code == 200
    assert response.json()["mode"] == "magiclink"
    assert response.json()["redirect_to"].startswith("/auth/passkey-setup")
    assert response.cookies.get("bominal_session")
    assert response.cookies.get("bominal_passkey_setup_ctx")


@pytest.mark.asyncio
async def test_supabase_confirm_email_type_sets_cookie(client, monkeypatch):
    async def _fake_exchange_detailed(*, token_hash: str, token_type: str):  # noqa: ARG001
        assert token_type == "email"
        return SimpleNamespace(
            session=SimpleNamespace(
                user_id="supabase-user-002",
                email="email-type@example.com",
                access_token="access-token-email-123",
            ),
            failure=None,
        )

    monkeypatch.setattr(
        "app.http.routes.auth.exchange_supabase_token_hash_detailed",
        _fake_exchange_detailed,
        raising=False,
    )

    response = await client.post(
        "/api/auth/supabase/confirm",
        json={"token_hash": "hash-email", "type": "email"},
    )
    assert response.status_code == 200
    payload = response.json()
    assert payload["mode"] == "magiclink"
    assert payload["redirect_to"] == "/auth/passkey-setup?source=magiclink&next=/modules/train"
    assert response.cookies.get("bominal_session")
    assert response.cookies.get("bominal_passkey_setup_ctx")


@pytest.mark.asyncio
async def test_supabase_confirm_rejects_access_token_contract(client):
    response = await client.post(
        "/api/auth/supabase/confirm",
        json={"access_token": "jwt-token-abc-def-ghijkl", "type": "magiclink"},
    )
    assert response.status_code == 422


@pytest.mark.asyncio
async def test_supabase_confirm_rejects_invalid_or_expired_token_hash(client, monkeypatch):
    async def _fake_exchange_detailed(*, token_hash: str, token_type: str):  # noqa: ARG001
        return SimpleNamespace(session=None, failure=SimpleNamespace(category="invalid", status_code=400, error_code=None))

    monkeypatch.setattr(
        "app.http.routes.auth.exchange_supabase_token_hash_detailed",
        _fake_exchange_detailed,
        raising=False,
    )

    response = await client.post(
        "/api/auth/supabase/confirm",
        json={"token_hash": "hash-expired-abc", "type": "magiclink"},
    )
    assert response.status_code == 400
    assert "fresh link" in response.json()["detail"].lower()


@pytest.mark.asyncio
async def test_supabase_confirm_logs_failure_classification(client, monkeypatch):
    warnings: list[dict[str, object]] = []

    def _capture_warning(message: str, *args, **kwargs):  # noqa: ANN002, ANN003
        warnings.append({"message": message, "args": args, "kwargs": kwargs})

    async def _fake_exchange_detailed(*, token_hash: str, token_type: str):  # noqa: ARG001
        return SimpleNamespace(
            session=None,
            failure=SimpleNamespace(category="expired", status_code=400, error_code="otp_expired"),
        )

    monkeypatch.setattr("app.http.routes.auth.logger.warning", _capture_warning)
    monkeypatch.setattr(
        "app.http.routes.auth.exchange_supabase_token_hash_detailed",
        _fake_exchange_detailed,
        raising=False,
    )

    response = await client.post(
        "/api/auth/supabase/confirm",
        json={"token_hash": "hash-expired-abc", "type": "magiclink"},
    )

    assert response.status_code == 400
    assert warnings, "expected a structured warning log for supabase confirm failure"
    extra = warnings[0]["kwargs"].get("extra")
    assert isinstance(extra, dict)
    assert extra.get("failure_category") == "expired"
    assert extra.get("failure_status_code") == 400
    assert extra.get("failure_error_code") == "otp_expired"
    assert extra.get("confirm_type") == "magiclink"
    assert isinstance(extra.get("confirm_correlation_id"), str)


@pytest.mark.asyncio
async def test_reset_password_supabase_updates_local_password_hash(client, monkeypatch):
    email = f"supabase-reset-{uuid4().hex[:8]}@example.com"
    await client.post(
        "/api/auth/register",
        json={"email": email, "password": "OldPassword123", "display_name": "Supabase Reset User"},
    )

    async def _fake_update_supabase_password(
        *,
        access_token: str,  # noqa: ARG001
        new_password: str,  # noqa: ARG001
        refresh_token: str | None = None,  # noqa: ARG001
    ):
        return type("Identity", (), {"user_id": "supabase-user-001", "email": email})()

    monkeypatch.setattr("app.http.routes.auth.update_supabase_password", _fake_update_supabase_password, raising=False)

    reset_response = await client.post(
        "/api/auth/reset-password/supabase",
        headers={"Authorization": "Bearer recovery-access-token"},
        json={"new_password": "NewPassword123"},
    )
    assert reset_response.status_code == 200
    assert reset_response.cookies.get("bominal_session")
    assert reset_response.cookies.get("bominal_passkey_setup_ctx")

    old_login = await client.post(
        "/api/auth/login",
        json={"email": email, "password": "OldPassword123", "remember_me": False},
    )
    assert old_login.status_code == 401

    new_login = await client.post(
        "/api/auth/login",
        json={"email": email, "password": "NewPassword123", "remember_me": False},
    )
    assert new_login.status_code == 200


@pytest.mark.asyncio
async def test_reset_password_supabase_requires_bearer_token(client):
    response = await client.post(
        "/api/auth/reset-password/supabase",
        json={"new_password": "NewPassword123"},
    )
    assert response.status_code == 401


@pytest.mark.asyncio
async def test_login_returns_generic_error_for_unknown_email(client):
    login_res = await client.post(
        "/api/auth/login",
        json={"email": "notfound@example.com", "password": "WrongPass123", "remember_me": False},
    )

    assert login_res.status_code == 401
    assert login_res.json()["detail"] == "Invalid email or password"


@pytest.mark.asyncio
async def test_register_requires_display_name(client):
    missing_name = await client.post(
        "/api/auth/register",
        json={"email": "missing-name@example.com", "password": "SuperSecret123"},
    )
    assert missing_name.status_code == 422

    blank_name = await client.post(
        "/api/auth/register",
        json={"email": "blank-name@example.com", "password": "SuperSecret123", "display_name": "   "},
    )
    assert blank_name.status_code == 422


@pytest.mark.asyncio
async def test_register_rejected_when_signup_disabled(client, monkeypatch):
    monkeypatch.setattr("app.http.routes.auth.settings.auth_registration_enabled", False)
    disabled = await client.post(
        "/api/auth/register",
        json={"email": "signup-off@example.com", "password": "SuperSecret123", "display_name": "Signup Off"},
    )
    assert disabled.status_code == 403
    assert disabled.json()["detail"] == "Sign up is currently disabled"


@pytest.mark.asyncio
async def test_register_rejects_duplicate_email_and_display_name(client):
    first = await client.post(
        "/api/auth/register",
        json={"email": "duplicate@example.com", "password": "SuperSecret123", "display_name": "Duplicate Name"},
    )
    assert first.status_code == 201

    duplicate_email = await client.post(
        "/api/auth/register",
        json={"email": "DUPLICATE@example.com", "password": "SuperSecret123", "display_name": "Another Name"},
    )
    assert duplicate_email.status_code == 400
    assert duplicate_email.json()["detail"] == "Email already registered"

    duplicate_display_name = await client.post(
        "/api/auth/register",
        json={"email": "another@example.com", "password": "SuperSecret123", "display_name": "duplicate name"},
    )
    assert duplicate_display_name.status_code == 400
    assert duplicate_display_name.json()["detail"] == "Display name already registered"


@pytest.mark.asyncio
async def test_register_is_idempotent_for_same_credentials(client):
    payload = {
        "email": "idempotent@example.com",
        "password": "SuperSecret123",
        "display_name": "Idempotent User",
    }
    first = await client.post("/api/auth/register", json=payload)
    assert first.status_code == 201

    retry = await client.post(
        "/api/auth/register",
        json={
            "email": "IDEMPOTENT@example.com",
            "password": "SuperSecret123",
            "display_name": "idempotent user",
        },
    )
    assert retry.status_code == 201
    assert retry.json()["user"]["email"] == payload["email"]
    assert retry.json()["notice"] == "Account already exists. Continuing with existing account."


@pytest.mark.asyncio
async def test_register_maps_integrity_error_to_conflict(client, monkeypatch):
    from sqlalchemy.ext.asyncio import AsyncSession

    real_commit = AsyncSession.commit

    async def _boom(self: AsyncSession):  # type: ignore[no-untyped-def]
        raise IntegrityError("insert", params={}, orig=Exception("unique constraint failed: users.email"))

    monkeypatch.setattr(AsyncSession, "commit", _boom)
    try:
        res = await client.post(
            "/api/auth/register",
            json={"email": "race@example.com", "password": "SuperSecret123", "display_name": "Race User"},
        )
    finally:
        monkeypatch.setattr(AsyncSession, "commit", real_commit)

    assert res.status_code == 409
    assert res.json()["detail"] == "Email already registered"


@pytest.mark.asyncio
async def test_account_update_maps_integrity_error_to_conflict(client, monkeypatch):
    from sqlalchemy.ext.asyncio import AsyncSession

    await client.post(
        "/api/auth/register",
        json={"email": "race-update@example.com", "password": "SuperSecret123", "display_name": "Race Update"},
    )
    login_res = await client.post(
        "/api/auth/login",
        json={"email": "race-update@example.com", "password": "SuperSecret123", "remember_me": False},
    )
    cookie = login_res.cookies.get("bominal_session")
    assert cookie

    real_commit = AsyncSession.commit

    call_count = {"n": 0}

    async def _boom(self: AsyncSession):  # type: ignore[no-untyped-def]
        call_count["n"] += 1
        if call_count["n"] >= 1:
            raise IntegrityError("update", params={}, orig=Exception("unique constraint failed: users.display_name"))
        return await real_commit(self)

    monkeypatch.setattr(AsyncSession, "commit", _boom)
    try:
        res = await client.patch(
            "/api/auth/account",
            cookies={"bominal_session": cookie},
            json={"display_name": "New Name", "current_password": "SuperSecret123"},
        )
    finally:
        monkeypatch.setattr(AsyncSession, "commit", real_commit)

    assert res.status_code == 409
    assert res.json()["detail"] == "Display name already registered"


@pytest.mark.asyncio
async def test_account_update_allows_passwordless_update_for_non_sensitive_fields(client):
    await client.post(
        "/api/auth/register",
        json={"email": "account-user@example.com", "password": "SuperSecret123", "display_name": "User"},
    )
    login_res = await client.post(
        "/api/auth/login",
        json={"email": "account-user@example.com", "password": "SuperSecret123", "remember_me": False},
    )
    session_cookie = login_res.cookies.get("bominal_session")
    assert session_cookie

    update_res = await client.patch(
        "/api/auth/account",
        cookies={"bominal_session": session_cookie},
        json={"display_name": "Updated User", "ui_locale": "ko"},
    )
    assert update_res.status_code == 200
    updated_user = update_res.json()["user"]
    assert updated_user["display_name"] == "Updated User"
    assert updated_user["ui_locale"] == "ko"


@pytest.mark.asyncio
async def test_account_update_requires_current_password_for_email_change(client):
    await client.post(
        "/api/auth/register",
        json={"email": "account-email-update@example.com", "password": "SuperSecret123", "display_name": "Email User"},
    )
    login_res = await client.post(
        "/api/auth/login",
        json={"email": "account-email-update@example.com", "password": "SuperSecret123", "remember_me": False},
    )
    session_cookie = login_res.cookies.get("bominal_session")
    assert session_cookie

    update_res = await client.patch(
        "/api/auth/account",
        cookies={"bominal_session": session_cookie},
        json={"email": "account-email-update-new@example.com"},
    )
    assert update_res.status_code == 401
    assert "Current password is required" in update_res.json()["detail"]


@pytest.mark.asyncio
async def test_account_update_requires_current_password_for_password_change(client):
    await client.post(
        "/api/auth/register",
        json={"email": "account-password-update@example.com", "password": "SuperSecret123", "display_name": "Pass User"},
    )
    login_res = await client.post(
        "/api/auth/login",
        json={"email": "account-password-update@example.com", "password": "SuperSecret123", "remember_me": False},
    )
    session_cookie = login_res.cookies.get("bominal_session")
    assert session_cookie

    update_res = await client.patch(
        "/api/auth/account",
        cookies={"bominal_session": session_cookie},
        json={"new_password": "EvenMoreSecret123"},
    )
    assert update_res.status_code == 401
    assert "Current password is required" in update_res.json()["detail"]


@pytest.mark.asyncio
async def test_account_update_rejects_invalid_ui_locale(client):
    await client.post(
        "/api/auth/register",
        json={"email": "account-locale@example.com", "password": "SuperSecret123", "display_name": "Locale User"},
    )
    login_res = await client.post(
        "/api/auth/login",
        json={"email": "account-locale@example.com", "password": "SuperSecret123", "remember_me": False},
    )
    session_cookie = login_res.cookies.get("bominal_session")
    assert session_cookie

    update_res = await client.patch(
        "/api/auth/account",
        cookies={"bominal_session": session_cookie},
        json={"ui_locale": "jp"},
    )
    assert update_res.status_code == 422


@pytest.mark.asyncio
async def test_account_update_updates_optional_fields_and_password(client, monkeypatch):
    captured: dict[str, object] = {}

    async def _capture_enqueue(payload, defer_seconds: float = 0.0):
        captured["payload"] = payload
        return "job-email-change-1"

    monkeypatch.setattr("app.http.routes.auth.enqueue_template_email", _capture_enqueue)

    await client.post(
        "/api/auth/register",
        json={"email": "account-update@example.com", "password": "SuperSecret123", "display_name": "Old Name"},
    )
    login_res = await client.post(
        "/api/auth/login",
        json={"email": "account-update@example.com", "password": "SuperSecret123", "remember_me": False},
    )
    session_cookie = login_res.cookies.get("bominal_session")
    assert session_cookie

    update_res = await client.patch(
        "/api/auth/account",
        cookies={"bominal_session": session_cookie},
        json={
            "email": "account-update-new@example.com",
            "display_name": "New Name",
            "phone_number": "010-1234-5678",
            "billing_address_line1": "123 Blossom St",
            "billing_address_line2": "Apt 402",
            "billing_city": "Seoul",
            "billing_state_province": "Seoul",
            "billing_country": "KR",
            "billing_postal_code": "04524",
            "birthday": "1990-01-01",
            "ui_locale": "en",
            "new_password": "EvenMoreSecret123",
            "current_password": "SuperSecret123",
        },
    )
    assert update_res.status_code == 200
    update_payload = update_res.json()
    updated_user = update_payload["user"]
    assert updated_user["email"] == "account-update@example.com"
    assert updated_user["display_name"] == "New Name"
    assert updated_user["phone_number"] == "010-1234-5678"
    assert updated_user["billing_address_line1"] == "123 Blossom St"
    assert updated_user["billing_address_line2"] == "Apt 402"
    assert updated_user["billing_city"] == "Seoul"
    assert updated_user["billing_state_province"] == "Seoul"
    assert updated_user["billing_country"] == "KR"
    assert updated_user["billing_postal_code"] == "04524"
    assert updated_user["birthday"] == "1990-01-01"
    assert updated_user["ui_locale"] == "en"
    assert update_payload["pending_email_change_to"] == "account-update-new@example.com"
    code = _extract_otp_code(captured["payload"])

    confirm_res = await client.post(
        "/api/auth/account/email-change/confirm",
        cookies={"bominal_session": session_cookie},
        json={"email": "account-update-new@example.com", "code": code},
    )
    assert confirm_res.status_code == 200
    assert confirm_res.json()["user"]["email"] == "account-update-new@example.com"

    old_password_login = await client.post(
        "/api/auth/login",
        json={"email": "account-update-new@example.com", "password": "SuperSecret123", "remember_me": False},
    )
    assert old_password_login.status_code == 401

    new_password_login = await client.post(
        "/api/auth/login",
        json={"email": "account-update-new@example.com", "password": "EvenMoreSecret123", "remember_me": False},
    )
    assert new_password_login.status_code == 200


@pytest.mark.asyncio
async def test_account_update_rejects_duplicate_email_and_display_name(client):
    await client.post(
        "/api/auth/register",
        json={"email": "primary@example.com", "password": "SuperSecret123", "display_name": "Primary User"},
    )
    await client.post(
        "/api/auth/register",
        json={"email": "secondary@example.com", "password": "SuperSecret123", "display_name": "Secondary User"},
    )

    login_res = await client.post(
        "/api/auth/login",
        json={"email": "secondary@example.com", "password": "SuperSecret123", "remember_me": False},
    )
    session_cookie = login_res.cookies.get("bominal_session")
    assert session_cookie

    duplicate_email = await client.patch(
        "/api/auth/account",
        cookies={"bominal_session": session_cookie},
        json={"email": "PRIMARY@example.com", "current_password": "SuperSecret123"},
    )
    assert duplicate_email.status_code == 400
    assert duplicate_email.json()["detail"] == "Email already registered"

    duplicate_display_name = await client.patch(
        "/api/auth/account",
        cookies={"bominal_session": session_cookie},
        json={"display_name": "primary user", "current_password": "SuperSecret123"},
    )
    assert duplicate_display_name.status_code == 400
    assert duplicate_display_name.json()["detail"] == "Display name already registered"


@pytest.mark.asyncio
async def test_passkey_http_routes_with_mocked_service(client, monkeypatch):
    await client.post(
        "/api/auth/register",
        json={"email": "passkey-http@example.com", "password": "SuperSecret123", "display_name": "Passkey HTTP"},
    )
    login_res = await client.post(
        "/api/auth/login",
        json={"email": "passkey-http@example.com", "password": "SuperSecret123", "remember_me": False},
    )
    session_cookie = login_res.cookies.get("bominal_session")
    assert session_cookie

    created_at = datetime.now(timezone.utc)
    stored_passkey_id = str(uuid4())
    challenge_id = str(uuid4())
    auth_challenge_id = str(uuid4())

    async def _list_passkeys(_db, *, user_id):  # noqa: ANN001
        return [SimpleNamespace(id=stored_passkey_id, created_at=created_at, last_used_at=None)]

    async def _begin_register(_db, *, user):  # noqa: ANN001
        return challenge_id, {"challenge": "reg", "user": {"id": "AQID"}}

    async def _finish_register(_db, *, user, challenge_id, credential):  # noqa: ANN001
        assert credential["id"] == "cred-id"
        return SimpleNamespace(id=stored_passkey_id, created_at=created_at, last_used_at=None)

    async def _delete_passkey(_db, *, user_id, passkey_id):  # noqa: ANN001
        return str(passkey_id) == stored_passkey_id

    async def _begin_auth(_db, *, email, user):  # noqa: ANN001
        return auth_challenge_id, {"challenge": "auth", "allowCredentials": []}

    async def _finish_auth(_db, *, email, user, challenge_id, credential):  # noqa: ANN001
        assert credential["id"] == "cred-id"
        return None

    monkeypatch.setattr("app.http.routes.auth.ensure_passkeys_enabled", lambda: None)
    monkeypatch.setattr("app.http.routes.auth.list_passkeys", _list_passkeys)
    monkeypatch.setattr("app.http.routes.auth.begin_passkey_registration", _begin_register)
    monkeypatch.setattr("app.http.routes.auth.complete_passkey_registration", _finish_register)
    monkeypatch.setattr("app.http.routes.auth.delete_passkey", _delete_passkey)
    monkeypatch.setattr("app.http.routes.auth.begin_passkey_authentication", _begin_auth)
    monkeypatch.setattr("app.http.routes.auth.complete_passkey_authentication", _finish_auth)

    listed = await client.get("/api/auth/passkeys", cookies={"bominal_session": session_cookie})
    assert listed.status_code == 200
    assert listed.json()["credentials"][0]["id"] == stored_passkey_id

    register_options = await client.post(
        "/api/auth/passkeys/register/options",
        cookies={"bominal_session": session_cookie},
    )
    assert register_options.status_code == 200
    assert register_options.json()["challenge_id"] == challenge_id

    register_verify = await client.post(
        "/api/auth/passkeys/register/verify",
        cookies={"bominal_session": session_cookie},
        json={"challenge_id": challenge_id, "credential": {"id": "cred-id"}},
    )
    assert register_verify.status_code == 200
    assert register_verify.json()["id"] == stored_passkey_id

    remove_res = await client.delete(
        f"/api/auth/passkeys/{stored_passkey_id}",
        cookies={"bominal_session": session_cookie},
    )
    assert remove_res.status_code == 200

    auth_options = await client.post("/api/auth/passkeys/auth/options", json={"email": "passkey-http@example.com"})
    assert auth_options.status_code == 200
    assert auth_options.json()["challenge_id"] == auth_challenge_id

    auth_verify = await client.post(
        "/api/auth/passkeys/auth/verify",
        json={
            "email": "passkey-http@example.com",
            "challenge_id": auth_challenge_id,
            "credential": {"id": "cred-id"},
            "remember_me": False,
        },
    )
    assert auth_verify.status_code == 200
    assert "set-cookie" in auth_verify.headers


@pytest.mark.asyncio
async def test_delete_account_blocks_when_outstanding_worker_instances_exist(client, db_session):
    email = "account-delete-blocked@example.com"
    await client.post(
        "/api/auth/register",
        json={"email": email, "password": "SuperSecret123", "display_name": "Delete Blocked"},
    )
    login_res = await client.post(
        "/api/auth/login",
        json={"email": email, "password": "SuperSecret123", "remember_me": False},
    )
    session_cookie = login_res.cookies.get("bominal_session")
    assert session_cookie

    user = (await db_session.execute(select(User).where(User.email == email))).scalar_one()
    db_session.add(
        Task(
            user_id=user.id,
            module="train",
            state="RUNNING",
            deadline_at=datetime.now(timezone.utc) + timedelta(hours=1),
            spec_json={"dep": "수서", "arr": "마산"},
            idempotency_key="active-delete-block",
        )
    )
    await db_session.commit()

    delete_res = await client.delete("/api/auth/account", cookies={"bominal_session": session_cookie})
    assert delete_res.status_code == 409
    assert "outstanding worker instances" in delete_res.json()["detail"].lower()

    user_after = (await db_session.execute(select(User).where(User.id == user.id))).scalar_one_or_none()
    assert user_after is not None
    assert user_after.email == email


@pytest.mark.asyncio
async def test_delete_account_scrubs_user_and_marks_tasks_for_removal(client, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    monkeypatch.setattr("app.services.wallet.get_cde_redis_pool", lambda: MockRedisContextManager(fake_redis))

    email = "account-delete-success@example.com"
    password = "SuperSecret123"
    await client.post(
        "/api/auth/register",
        json={"email": email, "password": password, "display_name": "Delete Success"},
    )
    login_res = await client.post(
        "/api/auth/login",
        json={"email": email, "password": password, "remember_me": False},
    )
    session_cookie = login_res.cookies.get("bominal_session")
    assert session_cookie

    user = (await db_session.execute(select(User).where(User.email == email))).scalar_one()
    user_id = user.id
    now = datetime.now(timezone.utc)
    db_session.add(
        Task(
            user_id=user_id,
            module="train",
            state="COMPLETED",
            deadline_at=now + timedelta(hours=2),
            spec_json={"dep": "수서", "arr": "부산"},
            idempotency_key="completed-delete-success",
            completed_at=now - timedelta(hours=1),
        )
    )
    db_session.add(
        Secret(
            user_id=user_id,
            kind="payment_card",
            ciphertext="ciphertext",
            nonce="nonce",
            wrapped_dek="wrapped",
            dek_nonce="dek-nonce",
            aad="aad",
            kek_version=1,
        )
    )
    await db_session.commit()
    await fake_redis.set(f"{PAYMENT_CVV_REDIS_KEY_PREFIX}:{user_id}", "encrypted-cvv")
    await fake_redis.set(f"{LEGACY_PAYMENT_CVV_REDIS_KEY_PREFIX}:{user_id}", "legacy-cvv")

    delete_res = await client.delete("/api/auth/account", cookies={"bominal_session": session_cookie})
    assert delete_res.status_code == 200
    assert delete_res.json()["message"] == "Account deleted"
    assert "bominal_session=" in (delete_res.headers.get("set-cookie") or "")

    me_res = await client.get("/api/auth/me", cookies={"bominal_session": session_cookie})
    assert me_res.status_code == 401

    db_session.expire_all()
    deleted_user = (await db_session.execute(select(User).where(User.id == user_id))).scalar_one()
    assert deleted_user.email.startswith(f"deleted-{user_id}")
    assert deleted_user.email.endswith("@deleted.bominal.local")
    assert deleted_user.display_name is None
    assert deleted_user.phone_number is None
    assert deleted_user.billing_address_line1 is None
    assert deleted_user.billing_address_line2 is None
    assert deleted_user.billing_city is None
    assert deleted_user.billing_state_province is None
    assert deleted_user.billing_country is None
    assert deleted_user.billing_postal_code is None
    assert deleted_user.birthday is None
    assert deleted_user.email_verified_at is None
    assert deleted_user.ui_locale == "en"

    sessions = (await db_session.execute(select(Session).where(Session.user_id == user_id))).scalars().all()
    assert sessions == []

    secrets = (await db_session.execute(select(Secret).where(Secret.user_id == user_id))).scalars().all()
    assert secrets == []
    assert await fake_redis.get(f"{PAYMENT_CVV_REDIS_KEY_PREFIX}:{user_id}") is None
    assert await fake_redis.get(f"{LEGACY_PAYMENT_CVV_REDIS_KEY_PREFIX}:{user_id}") is None

    tasks = (await db_session.execute(select(Task).where(Task.user_id == user_id))).scalars().all()
    assert len(tasks) == 1
    assert tasks[0].hidden_at is not None
    removal_safe = tasks[0].spec_json.get("account_removal_safe")
    assert isinstance(removal_safe, dict)
    assert removal_safe.get("reason") == "account_deleted"
    marked_for_removal_at = datetime.fromisoformat(str(removal_safe["marked_for_removal_at"]))
    remove_after_at = datetime.fromisoformat(str(removal_safe["remove_after_at"]))
    assert remove_after_at > marked_for_removal_at
    assert (remove_after_at - marked_for_removal_at) >= timedelta(days=365)
