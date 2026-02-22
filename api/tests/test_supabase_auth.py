from sqlalchemy import select

from app.core.supabase_jwt import SupabaseJWTError
from app.db.models import Role, User


async def _register_and_login(client, *, email: str, display_name: str) -> str:
    register_response = await client.post(
        "/api/auth/register",
        json={"email": email, "password": "StrongPass123!", "display_name": display_name},
    )
    assert register_response.status_code == 201

    login_response = await client.post(
        "/api/auth/login",
        json={"email": email, "password": "StrongPass123!", "remember_me": False},
    )
    assert login_response.status_code == 200
    session_cookie = login_response.cookies.get("bominal_session")
    assert session_cookie
    return session_cookie


async def test_dual_mode_accepts_supabase_bearer_and_creates_local_user(client, db_session, monkeypatch):
    monkeypatch.setattr("app.http.deps.settings.auth_mode", "dual")
    monkeypatch.setattr(
        "app.http.deps.verify_supabase_jwt",
        lambda _token: {
            "sub": "supabase-user-001",
            "email": "supabase-user@example.com",
            "user_metadata": {"display_name": "Supabase User"},
        },
    )

    response = await client.get("/api/auth/me", headers={"Authorization": "Bearer test-token"})
    assert response.status_code == 200
    payload = response.json()["user"]
    assert payload["email"] == "supabase-user@example.com"
    assert payload["supabase_user_id"] == "supabase-user-001"
    assert payload["role"] == "user"

    user = (
        await db_session.execute(select(User).where(User.supabase_user_id == "supabase-user-001"))
    ).scalar_one_or_none()
    assert user is not None
    assert user.email == "supabase-user@example.com"


async def test_dual_mode_uses_local_role_not_jwt_role_claim(client, db_session, monkeypatch):
    email = "supabase-admin@example.com"
    await _register_and_login(client, email=email, display_name="SupabaseAdmin")

    user = (await db_session.execute(select(User).where(User.email == email))).scalar_one()
    admin_role = (await db_session.execute(select(Role).where(Role.name == "admin"))).scalar_one()
    user.role_id = admin_role.id
    user.supabase_user_id = "supabase-admin-001"
    user.access_status = "approved"
    await db_session.commit()

    monkeypatch.setattr("app.http.deps.settings.auth_mode", "dual")
    monkeypatch.setattr(
        "app.http.deps.verify_supabase_jwt",
        lambda _token: {"sub": "supabase-admin-001", "email": email, "role": "user"},
    )

    response = await client.get("/api/admin", headers={"Authorization": "Bearer test-admin-token"})
    assert response.status_code == 200


async def test_dual_mode_rejects_invalid_bearer_even_with_valid_cookie(client, monkeypatch):
    cookie = await _register_and_login(
        client,
        email="dual-cookie-fallback@example.com",
        display_name="DualCookieFallback",
    )

    monkeypatch.setattr("app.http.deps.settings.auth_mode", "dual")

    def _raise_invalid(_token: str):
        raise SupabaseJWTError("invalid token")

    monkeypatch.setattr("app.http.deps.verify_supabase_jwt", _raise_invalid)

    response = await client.get(
        "/api/auth/me",
        headers={"Authorization": "Bearer invalid"},
        cookies={"bominal_session": cookie},
    )
    assert response.status_code == 401


async def test_supabase_mode_rejects_cookie_only_auth(client, monkeypatch):
    cookie = await _register_and_login(
        client,
        email="supabase-only-cookie@example.com",
        display_name="SupabaseOnlyCookie",
    )
    monkeypatch.setattr("app.http.deps.settings.auth_mode", "supabase")

    response = await client.get("/api/auth/me", cookies={"bominal_session": cookie})
    assert response.status_code == 401


async def test_supabase_existing_user_does_not_mutate_email_from_claims(client, db_session, monkeypatch):
    email = "stable-email@example.com"
    await _register_and_login(client, email=email, display_name="StableEmail")
    user = (await db_session.execute(select(User).where(User.email == email))).scalar_one()
    user_id = user.id
    user.supabase_user_id = "supabase-stable-001"
    await db_session.commit()

    monkeypatch.setattr("app.http.deps.settings.auth_mode", "supabase")
    monkeypatch.setattr(
        "app.http.deps.verify_supabase_jwt",
        lambda _token: {"sub": "supabase-stable-001", "email": "new-email@example.com"},
    )

    response = await client.get("/api/auth/me", headers={"Authorization": "Bearer stable-token"})
    assert response.status_code == 200

    db_session.expire_all()
    refreshed = (await db_session.execute(select(User).where(User.id == user_id))).scalar_one()
    assert refreshed.email == email


async def test_supabase_auto_provision_avoids_display_name_collision(client, db_session, monkeypatch):
    await _register_and_login(client, email="display-taken@example.com", display_name="DisplayTaken")

    monkeypatch.setattr("app.http.deps.settings.auth_mode", "supabase")
    monkeypatch.setattr(
        "app.http.deps.verify_supabase_jwt",
        lambda _token: {
            "sub": "supabase-display-001",
            "email": "new-display-user@example.com",
            "user_metadata": {"display_name": "DisplayTaken"},
        },
    )

    response = await client.get("/api/auth/me", headers={"Authorization": "Bearer display-token"})
    assert response.status_code == 200
    assert response.json()["user"]["display_name"] is None
