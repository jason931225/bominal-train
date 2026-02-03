import pytest


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

    login_res = await client.post(
        "/api/auth/login",
        json={"email": "user@example.com", "password": "SuperSecret123", "remember_me": True},
    )
    assert login_res.status_code == 200
    set_cookie = login_res.headers.get("set-cookie", "")
    assert "Max-Age=7776000" in set_cookie

    session_cookie = login_res.cookies.get("bominal_session")
    assert session_cookie

    me_res = await client.get("/api/auth/me", cookies={"bominal_session": session_cookie})
    assert me_res.status_code == 200
    me_json = me_res.json()
    assert me_json["user"]["email"] == "user@example.com"
    assert me_json["user"]["role"] == "user"

    logout_res = await client.post("/api/auth/logout", cookies={"bominal_session": session_cookie})
    assert logout_res.status_code == 200

    me_after_logout = await client.get("/api/auth/me", cookies={"bominal_session": session_cookie})
    assert me_after_logout.status_code == 401


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
async def test_account_update_requires_current_password_for_changes(client):
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
        json={"display_name": "Updated User"},
    )
    assert update_res.status_code == 401
    assert "Current password is required" in update_res.json()["detail"]


@pytest.mark.asyncio
async def test_account_update_updates_optional_fields_and_password(client):
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
            "new_password": "EvenMoreSecret123",
            "current_password": "SuperSecret123",
        },
    )
    assert update_res.status_code == 200
    updated_user = update_res.json()["user"]
    assert updated_user["email"] == "account-update-new@example.com"
    assert updated_user["display_name"] == "New Name"
    assert updated_user["phone_number"] == "010-1234-5678"
    assert updated_user["billing_address_line1"] == "123 Blossom St"
    assert updated_user["billing_address_line2"] == "Apt 402"
    assert updated_user["billing_city"] == "Seoul"
    assert updated_user["billing_state_province"] == "Seoul"
    assert updated_user["billing_country"] == "KR"
    assert updated_user["billing_postal_code"] == "04524"
    assert updated_user["birthday"] == "1990-01-01"

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
