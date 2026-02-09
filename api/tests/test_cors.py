import pytest


@pytest.mark.asyncio
async def test_cors_allows_common_local_dev_origins(client):
    response = await client.get("/health", headers={"Origin": "http://0.0.0.0:3000"})
    assert response.status_code == 200
    assert response.headers.get("access-control-allow-origin") == "http://0.0.0.0:3000"
    assert response.headers.get("access-control-allow-credentials") == "true"


@pytest.mark.asyncio
async def test_cors_preflight_allows_json_post_to_auth_routes(client):
    response = await client.options(
        "/api/auth/login",
        headers={
            "Origin": "http://0.0.0.0:3000",
            "Access-Control-Request-Method": "POST",
            "Access-Control-Request-Headers": "content-type",
        },
    )
    assert response.status_code in {200, 204}
    assert response.headers.get("access-control-allow-origin") == "http://0.0.0.0:3000"
    assert response.headers.get("access-control-allow-credentials") == "true"

