from __future__ import annotations

import pytest

from app.services import supabase_auth


@pytest.mark.asyncio
async def test_verify_supabase_password_returns_none_when_disabled(monkeypatch):
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_enabled", False, raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_url", "https://project-ref.supabase.co", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_api_key", "anon-key", raising=False)

    assert await supabase_auth.verify_supabase_password(email="user@example.com", password="secret") is None


@pytest.mark.asyncio
async def test_verify_supabase_password_posts_to_password_grant(monkeypatch):
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_enabled", True, raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_url", "https://project-ref.supabase.co", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_api_key", "anon-key", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_service_role_key", None, raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_timeout_seconds", 5.0, raising=False)

    captured: dict[str, object] = {}

    class _FakeResponse:
        status_code = 200

        def json(self):  # noqa: ANN201
            return {"user": {"id": "supa-user-001", "email": "user@example.com"}}

    class _FakeClient:
        def __init__(self, *, timeout: float):
            captured["timeout"] = timeout

        async def __aenter__(self):  # noqa: ANN204
            return self

        async def __aexit__(self, exc_type, exc, tb):  # noqa: ANN001, ANN204
            return False

        async def post(self, url: str, *, params: dict[str, str], headers: dict[str, str], json: dict):  # noqa: ANN003
            captured["url"] = url
            captured["params"] = params
            captured["headers"] = headers
            captured["json"] = json
            return _FakeResponse()

    monkeypatch.setattr(supabase_auth.httpx, "AsyncClient", lambda timeout: _FakeClient(timeout=timeout))

    identity = await supabase_auth.verify_supabase_password(email="user@example.com", password="secret-123")

    assert identity is not None
    assert identity.user_id == "supa-user-001"
    assert identity.email == "user@example.com"
    assert captured["timeout"] == 5.0
    assert captured["url"] == "https://project-ref.supabase.co/auth/v1/token"
    assert captured["params"] == {"grant_type": "password"}
    assert captured["json"] == {"email": "user@example.com", "password": "secret-123"}


@pytest.mark.asyncio
async def test_send_supabase_password_recovery_posts_redirect(monkeypatch):
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_enabled", True, raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_url", "https://project-ref.supabase.co", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_api_key", "anon-key", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_service_role_key", None, raising=False)

    captured: dict[str, object] = {}

    class _FakeResponse:
        status_code = 200

    class _FakeClient:
        def __init__(self, *, timeout: float):  # noqa: ARG002
            pass

        async def __aenter__(self):  # noqa: ANN204
            return self

        async def __aexit__(self, exc_type, exc, tb):  # noqa: ANN001, ANN204
            return False

        async def post(self, url: str, *, headers: dict[str, str], json: dict):  # noqa: ANN003
            captured["url"] = url
            captured["headers"] = headers
            captured["json"] = json
            return _FakeResponse()

    monkeypatch.setattr(supabase_auth.httpx, "AsyncClient", lambda timeout: _FakeClient(timeout=timeout))

    ok = await supabase_auth.send_supabase_password_recovery(
        email="user@example.com",
        redirect_to="https://www.bominal.com/reset-password",
    )

    assert ok is True
    assert captured["url"] == "https://project-ref.supabase.co/auth/v1/recover"
    assert captured["json"] == {
        "email": "user@example.com",
        "redirect_to": "https://www.bominal.com/reset-password",
    }


@pytest.mark.asyncio
async def test_send_supabase_password_recovery_returns_false_on_http_error(monkeypatch):
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_enabled", True, raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_url", "https://project-ref.supabase.co", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_api_key", "anon-key", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_service_role_key", None, raising=False)

    class _FakeResponse:
        status_code = 500

    class _FakeClient:
        def __init__(self, *, timeout: float):  # noqa: ARG002
            pass

        async def __aenter__(self):  # noqa: ANN204
            return self

        async def __aexit__(self, exc_type, exc, tb):  # noqa: ANN001, ANN204
            return False

        async def post(self, url: str, *, headers: dict[str, str], json: dict):  # noqa: ANN003, ARG002
            return _FakeResponse()

    monkeypatch.setattr(supabase_auth.httpx, "AsyncClient", lambda timeout: _FakeClient(timeout=timeout))

    ok = await supabase_auth.send_supabase_password_recovery(email="user@example.com")
    assert ok is False


@pytest.mark.asyncio
async def test_send_supabase_magic_link_posts_otp_endpoint(monkeypatch):
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_enabled", True, raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_url", "https://project-ref.supabase.co", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_api_key", "anon-key", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_service_role_key", None, raising=False)

    captured: dict[str, object] = {}

    class _FakeResponse:
        status_code = 200

    class _FakeClient:
        def __init__(self, *, timeout: float):  # noqa: ARG002
            pass

        async def __aenter__(self):  # noqa: ANN204
            return self

        async def __aexit__(self, exc_type, exc, tb):  # noqa: ANN001, ANN204
            return False

        async def post(self, url: str, *, headers: dict[str, str], json: dict):  # noqa: ANN003
            captured["url"] = url
            captured["headers"] = headers
            captured["json"] = json
            return _FakeResponse()

    monkeypatch.setattr(supabase_auth.httpx, "AsyncClient", lambda timeout: _FakeClient(timeout=timeout))

    ok = await supabase_auth.send_supabase_magic_link(
        email="user@example.com",
        redirect_to="https://www.bominal.com/auth/verify?type=email",
    )
    assert ok is True
    assert captured["url"] == "https://project-ref.supabase.co/auth/v1/otp"
    assert captured["json"] == {
        "email": "user@example.com",
        "create_user": False,
        "should_create_user": False,
        "email_redirect_to": "https://www.bominal.com/auth/verify?type=email",
    }


@pytest.mark.asyncio
async def test_send_supabase_signin_otp_posts_otp_endpoint(monkeypatch):
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_enabled", True, raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_url", "https://project-ref.supabase.co", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_api_key", "anon-key", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_service_role_key", None, raising=False)

    captured: dict[str, object] = {}

    class _FakeResponse:
        status_code = 200

    class _FakeClient:
        def __init__(self, *, timeout: float):  # noqa: ARG002
            pass

        async def __aenter__(self):  # noqa: ANN204
            return self

        async def __aexit__(self, exc_type, exc, tb):  # noqa: ANN001, ANN204
            return False

        async def post(self, url: str, *, headers: dict[str, str], json: dict):  # noqa: ANN003
            captured["url"] = url
            captured["json"] = json
            return _FakeResponse()

    monkeypatch.setattr(supabase_auth.httpx, "AsyncClient", lambda timeout: _FakeClient(timeout=timeout))

    ok = await supabase_auth.send_supabase_signin_otp(email="user@example.com")
    assert ok is True
    assert captured["url"] == "https://project-ref.supabase.co/auth/v1/otp"
    assert captured["json"] == {
        "email": "user@example.com",
        "create_user": False,
        "should_create_user": False,
    }


@pytest.mark.asyncio
async def test_verify_supabase_signin_otp_returns_session(monkeypatch):
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_enabled", True, raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_url", "https://project-ref.supabase.co", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_api_key", "anon-key", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_service_role_key", None, raising=False)

    captured: dict[str, object] = {}

    class _FakeResponse:
        status_code = 200

        def json(self):  # noqa: ANN201
            return {
                "access_token": "access-token-123",
                "user": {"id": "supabase-user-001", "email": "user@example.com"},
            }

    class _FakeClient:
        def __init__(self, *, timeout: float):  # noqa: ARG002
            pass

        async def __aenter__(self):  # noqa: ANN204
            return self

        async def __aexit__(self, exc_type, exc, tb):  # noqa: ANN001, ANN204
            return False

        async def post(self, url: str, *, headers: dict[str, str], json: dict):  # noqa: ANN003
            captured["url"] = url
            captured["json"] = json
            return _FakeResponse()

    monkeypatch.setattr(supabase_auth.httpx, "AsyncClient", lambda timeout: _FakeClient(timeout=timeout))

    session = await supabase_auth.verify_supabase_signin_otp(email="user@example.com", code="123456")
    assert session is not None
    assert session.user_id == "supabase-user-001"
    assert session.email == "user@example.com"
    assert session.access_token == "access-token-123"
    assert captured["url"] == "https://project-ref.supabase.co/auth/v1/verify"
    assert captured["json"] == {"email": "user@example.com", "token": "123456", "type": "email"}


@pytest.mark.asyncio
async def test_exchange_supabase_token_hash_returns_recovery_session(monkeypatch):
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_enabled", True, raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_url", "https://project-ref.supabase.co", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_api_key", "anon-key", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_service_role_key", None, raising=False)

    captured: dict[str, object] = {}

    class _FakeResponse:
        status_code = 200

        def json(self):  # noqa: ANN201
            return {
                "access_token": "access-token-123",
                "user": {"id": "supabase-user-001", "email": "user@example.com"},
            }

    class _FakeClient:
        def __init__(self, *, timeout: float):  # noqa: ARG002
            pass

        async def __aenter__(self):  # noqa: ANN204
            return self

        async def __aexit__(self, exc_type, exc, tb):  # noqa: ANN001, ANN204
            return False

        async def post(self, url: str, *, headers: dict[str, str], json: dict):  # noqa: ANN003
            captured["url"] = url
            captured["headers"] = headers
            captured["json"] = json
            return _FakeResponse()

    monkeypatch.setattr(supabase_auth.httpx, "AsyncClient", lambda timeout: _FakeClient(timeout=timeout))

    result = await supabase_auth.exchange_supabase_token_hash(token_hash="hash-abc", token_type="recovery")

    assert result is not None
    assert result.user_id == "supabase-user-001"
    assert result.email == "user@example.com"
    assert result.access_token == "access-token-123"
    assert captured["url"] == "https://project-ref.supabase.co/auth/v1/verify"
    assert captured["json"] == {"type": "recovery", "token_hash": "hash-abc"}


@pytest.mark.asyncio
async def test_exchange_supabase_token_hash_detailed_classifies_expired_otp(monkeypatch):
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_enabled", True, raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_url", "https://project-ref.supabase.co", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_api_key", "anon-key", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_service_role_key", None, raising=False)

    class _FakeResponse:
        status_code = 400

        def json(self):  # noqa: ANN201
            return {"code": "otp_expired", "message": "Token has expired"}

    class _FakeClient:
        def __init__(self, *, timeout: float):  # noqa: ARG002
            pass

        async def __aenter__(self):  # noqa: ANN204
            return self

        async def __aexit__(self, exc_type, exc, tb):  # noqa: ANN001, ANN204
            return False

        async def post(self, url: str, *, headers: dict[str, str], json: dict):  # noqa: ANN003, ARG002
            return _FakeResponse()

    monkeypatch.setattr(supabase_auth.httpx, "AsyncClient", lambda timeout: _FakeClient(timeout=timeout))

    result = await supabase_auth.exchange_supabase_token_hash_detailed(token_hash="hash-abc", token_type="magiclink")

    assert result.session is None
    assert result.failure is not None
    assert result.failure.category == "expired"
    assert result.failure.status_code == 400
    assert result.failure.error_code == "otp_expired"


@pytest.mark.asyncio
async def test_exchange_supabase_token_hash_detailed_classifies_transport_error(monkeypatch):
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_enabled", True, raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_url", "https://project-ref.supabase.co", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_api_key", "anon-key", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_service_role_key", None, raising=False)

    class _FakeClient:
        def __init__(self, *, timeout: float):  # noqa: ARG002
            pass

        async def __aenter__(self):  # noqa: ANN204
            return self

        async def __aexit__(self, exc_type, exc, tb):  # noqa: ANN001, ANN204
            return False

        async def post(self, url: str, *, headers: dict[str, str], json: dict):  # noqa: ANN003, ARG002
            raise RuntimeError("network-down")

    monkeypatch.setattr(supabase_auth.httpx, "AsyncClient", lambda timeout: _FakeClient(timeout=timeout))

    result = await supabase_auth.exchange_supabase_token_hash_detailed(token_hash="hash-abc", token_type="magiclink")

    assert result.session is None
    assert result.failure is not None
    assert result.failure.category == "transport"
    assert result.failure.status_code is None


@pytest.mark.asyncio
async def test_update_supabase_password_uses_bearer_access_token(monkeypatch):
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_enabled", True, raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_url", "https://project-ref.supabase.co", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_api_key", "anon-key", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_service_role_key", None, raising=False)

    captured: dict[str, object] = {}

    class _FakeResponse:
        status_code = 200

        def json(self):  # noqa: ANN201
            return {"id": "supabase-user-001", "email": "user@example.com"}

    class _FakeClient:
        def __init__(self, *, timeout: float):  # noqa: ARG002
            pass

        async def __aenter__(self):  # noqa: ANN204
            return self

        async def __aexit__(self, exc_type, exc, tb):  # noqa: ANN001, ANN204
            return False

        async def put(self, url: str, *, headers: dict[str, str], json: dict):  # noqa: ANN003
            captured["url"] = url
            captured["headers"] = headers
            captured["json"] = json
            return _FakeResponse()

    monkeypatch.setattr(supabase_auth.httpx, "AsyncClient", lambda timeout: _FakeClient(timeout=timeout))

    identity = await supabase_auth.update_supabase_password(
        access_token="access-token-123",
        new_password="NewPassword123!",
    )

    assert identity is not None
    assert identity.user_id == "supabase-user-001"
    assert identity.email == "user@example.com"
    assert captured["url"] == "https://project-ref.supabase.co/auth/v1/user"
    assert captured["headers"]["Authorization"] == "Bearer access-token-123"
    assert captured["json"] == {"password": "NewPassword123!"}


@pytest.mark.asyncio
async def test_update_supabase_password_refreshes_and_retries_after_initial_reject(monkeypatch):
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_enabled", True, raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_url", "https://project-ref.supabase.co", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_auth_api_key", "anon-key", raising=False)
    monkeypatch.setattr(supabase_auth.settings, "supabase_service_role_key", None, raising=False)

    captured_put_headers: list[dict[str, str]] = []
    captured_refresh: dict[str, object] = {}

    class _FakeResponse:
        def __init__(self, *, status_code: int, payload: dict[str, object]):
            self.status_code = status_code
            self._payload = payload

        def json(self):  # noqa: ANN201
            return self._payload

    class _FakeClient:
        def __init__(self, *, timeout: float):  # noqa: ARG002
            self._put_call_count = 0

        async def __aenter__(self):  # noqa: ANN204
            return self

        async def __aexit__(self, exc_type, exc, tb):  # noqa: ANN001, ANN204
            return False

        async def put(self, url: str, *, headers: dict[str, str], json: dict):  # noqa: ANN003
            captured_put_headers.append(headers)
            self._put_call_count += 1
            if self._put_call_count == 1:
                return _FakeResponse(status_code=401, payload={"message": "expired"})
            return _FakeResponse(status_code=200, payload={"id": "supabase-user-001", "email": "user@example.com"})

        async def post(self, url: str, *, params: dict[str, str], headers: dict[str, str], json: dict):  # noqa: ANN003
            captured_refresh["url"] = url
            captured_refresh["params"] = params
            captured_refresh["headers"] = headers
            captured_refresh["json"] = json
            return _FakeResponse(status_code=200, payload={"access_token": "fresh-access-token"})

    monkeypatch.setattr(supabase_auth.httpx, "AsyncClient", lambda timeout: _FakeClient(timeout=timeout))

    identity = await supabase_auth.update_supabase_password(
        access_token="stale-access-token",
        refresh_token="refresh-token-123",
        new_password="NewPassword123!",
    )

    assert identity is not None
    assert identity.user_id == "supabase-user-001"
    assert identity.email == "user@example.com"
    assert len(captured_put_headers) == 2
    assert captured_put_headers[0]["Authorization"] == "Bearer stale-access-token"
    assert captured_put_headers[1]["Authorization"] == "Bearer fresh-access-token"
    assert captured_refresh["url"] == "https://project-ref.supabase.co/auth/v1/token"
    assert captured_refresh["params"] == {"grant_type": "refresh_token"}
    assert captured_refresh["json"] == {"refresh_token": "refresh-token-123"}
