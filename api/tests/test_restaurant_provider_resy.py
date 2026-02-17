from __future__ import annotations

import json

import pytest

from app.modules.restaurant.providers.resy_adapter import ResyProviderClient
from app.modules.train.providers.transport import TransportResponse


class _QueueTransport:
    def __init__(self, responses: list[TransportResponse]) -> None:
        self._responses = list(responses)
        self.requests: list[dict] = []

    async def request(
        self,
        *,
        method: str,
        url: str,
        headers: dict[str, str] | None = None,
        json_body: dict | None = None,
        data: dict | None = None,
        params: dict | None = None,
        timeout: float = 20.0,
    ) -> TransportResponse:
        self.requests.append(
            {
                "method": method,
                "url": url,
                "headers": headers or {},
                "json_body": json_body,
                "data": data,
                "params": params,
                "timeout": timeout,
            }
        )
        if not self._responses:
            raise AssertionError("No queued response available")
        return self._responses.pop(0)


def _response(payload: dict, *, status_code: int = 200) -> TransportResponse:
    return TransportResponse(status_code=status_code, text=json.dumps(payload), headers={})


@pytest.mark.asyncio
async def test_resy_auth_start_requires_password():
    client = ResyProviderClient(transport=_QueueTransport([]), auth_api_key="key-1")

    start_result = await client.authenticate_start(account_identifier="user@example.com", password=None)

    assert start_result.ok is False
    assert start_result.error_code == "auth_start_password_required"


@pytest.mark.asyncio
async def test_resy_auth_start_posts_password_contract_and_completes_login():
    transport = _QueueTransport([_response({"user": {"id": "user-1"}})])
    client = ResyProviderClient(
        transport=transport,
        base_url="https://api.resy.com",
        auth_password_path="/4/auth/password",
        auth_api_key="key-1",
        x_origin="https://resy.com",
    )

    start_result = await client.authenticate_start(account_identifier="user@example.com", password="secret")

    assert start_result.ok is True
    assert start_result.data["requires_otp"] is False
    assert start_result.data["password_flow_complete"] is True
    assert start_result.data["provider_account_ref"] == "user-1"
    challenge_payload = json.loads(start_result.data["challenge_token"])
    assert challenge_payload == {
        "password_flow_complete": True,
        "provider_account_ref": "user-1",
    }
    assert len(transport.requests) == 1
    request = transport.requests[0]
    assert request["method"] == "POST"
    assert request["url"].endswith("/4/auth/password")
    assert request["headers"]["Authorization"] == 'ResyAPI api_key="key-1"'
    assert request["headers"]["X-Origin"] == "https://resy.com"
    assert request["headers"]["Content-Type"] == "application/x-www-form-urlencoded"
    assert request["data"] == {
        "email": "user@example.com",
        "password": "secret",
    }


@pytest.mark.asyncio
async def test_resy_auth_start_handles_http_error_codes():
    transport = _QueueTransport([_response({"message": "Unauthorized"}, status_code=401)])
    client = ResyProviderClient(transport=transport, auth_api_key="key-1")

    start_result = await client.authenticate_start(account_identifier="user@example.com", password="secret")

    assert start_result.ok is False
    assert start_result.error_code == "auth_start_auth_required"
    assert start_result.retryable is False


@pytest.mark.asyncio
async def test_resy_auth_start_rejects_body_level_failure_on_http_200():
    transport = _QueueTransport([_response({"success": False, "code": "INVALID_CREDENTIALS"})])
    client = ResyProviderClient(transport=transport, auth_api_key="key-1")

    start_result = await client.authenticate_start(account_identifier="user@example.com", password="secret")

    assert start_result.ok is False
    assert start_result.error_code == "auth_start_failed"
    assert start_result.data["provider_error_code"] == "INVALID_CREDENTIALS"


@pytest.mark.asyncio
async def test_resy_auth_complete_uses_start_challenge_payload_without_network_call():
    transport = _QueueTransport([])
    client = ResyProviderClient(transport=transport, auth_api_key="key-1")

    complete_result = await client.authenticate_complete(
        account_identifier="user@example.com",
        challenge_token=json.dumps({"password_flow_complete": True, "provider_account_ref": "user-1"}),
        otp_code=None,
    )

    assert complete_result.ok is True
    assert complete_result.data["authenticated"] is True
    assert complete_result.data["provider_account_ref"] == "user-1"
    assert transport.requests == []


@pytest.mark.asyncio
async def test_resy_auth_complete_requires_password_flow_challenge_token():
    client = ResyProviderClient(transport=_QueueTransport([]), auth_api_key="key-1")

    complete_result = await client.authenticate_complete(
        account_identifier="user@example.com",
        challenge_token=None,
        otp_code=None,
    )

    assert complete_result.ok is False
    assert complete_result.error_code == "auth_complete_challenge_missing"
