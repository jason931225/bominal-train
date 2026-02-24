from __future__ import annotations

import asyncio
from datetime import datetime, timezone
from types import SimpleNamespace
from uuid import uuid4

import pytest
from fastapi import HTTPException

from app.modules.train import service as train_service
from app.modules.train.providers.base import ProviderOutcome
from app.modules.train.schemas import KTXCredentialsSetRequest, SRTCredentialsSetRequest


class _DummyDB:
    def __init__(self) -> None:
        self.added: list[object] = []
        self.commits = 0
        self.executed = 0

    def add(self, obj: object) -> None:
        self.added.append(obj)

    async def commit(self) -> None:
        self.commits += 1

    async def execute(self, _stmt):  # noqa: ANN001
        self.executed += 1
        return SimpleNamespace(scalar_one_or_none=lambda: None)


def _user():
    return SimpleNamespace(id=uuid4())


@pytest.mark.asyncio
async def test_load_provider_credentials_handles_missing_secret_decrypt_errors_and_success(monkeypatch):
    user_id = uuid4()

    async def _latest_none(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return None

    monkeypatch.setattr(train_service, "_latest_secret_for_user", _latest_none)
    assert await train_service._load_provider_credentials(_DummyDB(), user_id=user_id, provider="SRT") is None

    async def _latest_secret(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return object()

    monkeypatch.setattr(train_service, "_latest_secret_for_user", _latest_secret)

    def _raise_decrypt(_secret):  # noqa: ANN001
        raise RuntimeError("bad decrypt")

    monkeypatch.setattr(train_service, "decrypt_secret", _raise_decrypt)
    assert await train_service._load_provider_credentials(_DummyDB(), user_id=user_id, provider="SRT") is None

    monkeypatch.setattr(train_service, "decrypt_secret", lambda _secret: {"username": "", "password": "pw"})
    assert await train_service._load_provider_credentials(_DummyDB(), user_id=user_id, provider="SRT") is None

    monkeypatch.setattr(
        train_service,
        "decrypt_secret",
        lambda _secret: {"username": "  tester  ", "password": "pw", "verified_at": "2026-02-22T12:00:00+00:00"},
    )
    loaded = await train_service._load_provider_credentials(_DummyDB(), user_id=user_id, provider="SRT")
    assert loaded == {
        "username": "tester",
        "password": "pw",
        "verified_at": "2026-02-22T12:00:00+00:00",
    }


@pytest.mark.asyncio
async def test_save_provider_credentials_inserts_and_updates_existing_secret(monkeypatch):
    db = _DummyDB()
    now = datetime(2026, 2, 22, 12, 0, tzinfo=timezone.utc)
    monkeypatch.setattr(train_service, "utc_now", lambda: now)

    encrypted = SimpleNamespace(
        ciphertext="cipher",
        nonce="nonce",
        wrapped_dek="wrapped",
        dek_nonce="dek_nonce",
        aad="aad",
        kek_version=2,
    )
    monkeypatch.setattr(train_service, "build_encrypted_secret", lambda **_kwargs: encrypted)

    async def _latest_none(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return None

    monkeypatch.setattr(train_service, "_latest_secret_for_user", _latest_none)
    await train_service._save_provider_credentials(
        db,
        user_id=uuid4(),
        provider="SRT",
        payload={"username": "u", "password": "p"},
    )
    assert db.added and db.added[0] is encrypted

    existing = SimpleNamespace(
        ciphertext="old",
        nonce="old",
        wrapped_dek="old",
        dek_nonce="old",
        aad="old",
        kek_version=1,
        updated_at=None,
    )

    async def _latest_existing(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return existing

    monkeypatch.setattr(train_service, "_latest_secret_for_user", _latest_existing)
    await train_service._save_provider_credentials(
        db,
        user_id=uuid4(),
        provider="KTX",
        payload={"username": "u", "password": "p"},
    )
    assert existing.ciphertext == "cipher"
    assert existing.nonce == "nonce"
    assert existing.wrapped_dek == "wrapped"
    assert existing.dek_nonce == "dek_nonce"
    assert existing.aad == "aad"
    assert existing.kek_version == 2
    assert existing.updated_at == now


@pytest.mark.asyncio
async def test_verify_provider_credentials_handles_missing_cached_timeout_errors_and_success(monkeypatch):
    db = _DummyDB()
    user = _user()

    async def _load_none(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return None

    monkeypatch.setattr(train_service, "_load_provider_credentials", _load_none)
    missing = await train_service._verify_provider_credentials(db, user=user, provider="SRT")
    assert missing.configured is False
    assert missing.verified is False

    verified_at_text = "2026-02-22T12:00:00+00:00"

    async def _load_cached(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {"username": "u", "password": "p", "verified_at": verified_at_text}

    monkeypatch.setattr(train_service, "_load_provider_credentials", _load_cached)
    monkeypatch.setattr(train_service, "_is_recent_verification", lambda _v: True)
    cached = await train_service._verify_provider_credentials(db, user=user, provider="SRT")
    assert cached.verified is True
    assert cached.detail is None

    monkeypatch.setattr(train_service, "_is_recent_verification", lambda _v: False)

    class _Client:
        async def login(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=True, data={"membership_number": "1", "membership_name": "Name"})

    monkeypatch.setattr(train_service, "get_provider_client", lambda _provider: _Client())

    saved_payloads: list[dict] = []

    async def _save(_db, *, user_id, provider, payload):  # noqa: ANN001
        saved_payloads.append(payload)

    monkeypatch.setattr(train_service, "_save_provider_credentials", _save)
    monkeypatch.setattr(train_service, "utc_now", lambda: datetime(2026, 2, 22, 13, 0, tzinfo=timezone.utc))
    success = await train_service._verify_provider_credentials(db, user=user, provider="SRT")
    assert success.verified is True
    assert saved_payloads
    assert db.commits == 1

    async def _raise_timeout(_coro, timeout: float):  # noqa: ANN001
        _coro.close()
        raise asyncio.TimeoutError

    monkeypatch.setattr(train_service.asyncio, "wait_for", _raise_timeout)
    timeout_result = await train_service._verify_provider_credentials(db, user=user, provider="SRT")
    assert timeout_result.verified is True

    async def _load_unverified(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {"username": "u", "password": "p", "verified_at": ""}

    monkeypatch.setattr(train_service, "_load_provider_credentials", _load_unverified)
    timeout_unverified = await train_service._verify_provider_credentials(db, user=user, provider="SRT")
    assert timeout_unverified.verified is False
    assert "timed out" in str(timeout_unverified.detail).lower()


@pytest.mark.asyncio
async def test_verify_provider_credentials_non_ok_and_guarded_status_wrappers(monkeypatch):
    db = _DummyDB()
    user = _user()

    async def _load(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {"username": "u", "password": "p", "verified_at": "2026-02-22T12:00:00+00:00"}

    monkeypatch.setattr(train_service, "_load_provider_credentials", _load)
    monkeypatch.setattr(train_service, "_is_recent_verification", lambda _v: False)

    class _ClientRetry:
        async def login(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=False, retryable=True, error_message_safe="temporary")

    monkeypatch.setattr(train_service, "get_provider_client", lambda _provider: _ClientRetry())
    retry_status = await train_service._verify_provider_credentials(db, user=user, provider="KTX")
    assert retry_status.verified is True

    class _ClientFail:
        async def login(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=False, retryable=False, error_message_safe="bad creds")

    monkeypatch.setattr(train_service, "get_provider_client", lambda _provider: _ClientFail())
    fail_status = await train_service._verify_provider_credentials(db, user=user, provider="KTX")
    assert fail_status.verified is False
    assert "bad creds" in str(fail_status.detail)

    async def _verify_raise(*_args, **_kwargs):  # noqa: ANN002, ANN003
        raise RuntimeError("boom")

    async def _status_fallback(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return train_service.ProviderCredentialStatus(
            configured=True,
            verified=False,
            detail="fallback",
        )

    monkeypatch.setattr(train_service, "_verify_provider_credentials", _verify_raise)
    monkeypatch.setattr(train_service, "_status_from_saved_credentials", _status_fallback)
    guarded = await train_service._verify_provider_credentials_guarded(db, user=user, provider="SRT")
    assert guarded.detail == "fallback"

    monkeypatch.setattr(train_service, "_verify_provider_credentials_guarded", _status_fallback)
    both = await train_service.get_train_credentials_status(db, user=user)
    assert both.ktx.detail == "fallback"
    assert both.srt.detail == "fallback"

    srt_status = await train_service.get_srt_credential_status(db, user=user)
    ktx_status = await train_service.get_ktx_credential_status(db, user=user)
    assert srt_status.detail == "fallback"
    assert ktx_status.detail == "fallback"


@pytest.mark.asyncio
async def test_set_and_clear_credentials_and_get_logged_in_provider_client(monkeypatch):
    db = _DummyDB()
    user = _user()
    now = datetime(2026, 2, 22, 14, 0, tzinfo=timezone.utc)
    monkeypatch.setattr(train_service, "utc_now", lambda: now)

    captured: list[tuple[str, dict]] = []

    async def _save(_db, *, user_id, provider, payload):  # noqa: ANN001
        captured.append((provider, payload))

    monkeypatch.setattr(train_service, "_save_provider_credentials", _save)

    class _Client:
        def __init__(self, outcome: ProviderOutcome):
            self._outcome = outcome

        async def login(self, **_kwargs):  # noqa: ANN003
            return self._outcome

    monkeypatch.setattr(
        train_service,
        "get_provider_client",
        lambda provider: _Client(ProviderOutcome(ok=True, data={"membership_number": "1", "membership_name": provider})),
    )

    srt_ok = await train_service.set_srt_credentials(
        db,
        user=user,
        payload=SRTCredentialsSetRequest(username="  srt-user  ", password="pass1234"),
    )
    assert srt_ok.verified is True
    assert captured[-1][0] == "SRT"

    ktx_ok = await train_service.set_ktx_credentials(
        db,
        user=user,
        payload=KTXCredentialsSetRequest(username="  ktx-user  ", password="pass1234"),
    )
    assert ktx_ok.verified is True
    assert captured[-1][0] == "KTX"

    monkeypatch.setattr(
        train_service,
        "get_provider_client",
        lambda _provider: _Client(ProviderOutcome(ok=False, retryable=True, error_message_safe="retry")),
    )
    with pytest.raises(HTTPException) as srt_retry:
        await train_service.set_srt_credentials(
            db,
            user=user,
            payload=SRTCredentialsSetRequest(username="srt-user", password="pass1234"),
        )
    assert srt_retry.value.status_code == 502

    monkeypatch.setattr(
        train_service,
        "get_provider_client",
        lambda _provider: _Client(ProviderOutcome(ok=False, retryable=False, error_message_safe="invalid")),
    )
    with pytest.raises(HTTPException) as ktx_fail:
        await train_service.set_ktx_credentials(
            db,
            user=user,
            payload=KTXCredentialsSetRequest(username="ktx-user", password="pass1234"),
        )
    assert ktx_fail.value.status_code == 400

    status_srt = await train_service.clear_srt_credentials(db, user=user)
    status_ktx = await train_service.clear_ktx_credentials(db, user=user)
    assert status_srt.configured is False
    assert status_ktx.configured is False
    assert db.executed >= 2

    async def _load_none(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return None

    monkeypatch.setattr(train_service, "_load_provider_credentials", _load_none)
    with pytest.raises(HTTPException) as missing:
        await train_service._get_logged_in_provider_client(db, user=user, provider="SRT")
    assert missing.value.status_code == 400

    async def _load_creds(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {"username": "user", "password": "pass", "verified_at": ""}

    monkeypatch.setattr(train_service, "_load_provider_credentials", _load_creds)
    monkeypatch.setattr(
        train_service,
        "get_provider_client",
        lambda _provider: _Client(ProviderOutcome(ok=False, retryable=True, error_message_safe="down")),
    )
    with pytest.raises(HTTPException) as login_fail:
        await train_service._get_logged_in_provider_client(db, user=user, provider="SRT")
    assert login_fail.value.status_code == 502

    client_ok = _Client(ProviderOutcome(ok=True))
    monkeypatch.setattr(train_service, "get_provider_client", lambda _provider: client_ok)
    assert await train_service._get_logged_in_provider_client(db, user=user, provider="SRT") is client_ok


@pytest.mark.asyncio
async def test_provider_reservation_and_ticket_info_services_map_provider_outcomes(monkeypatch):
    db = _DummyDB()
    user = _user()

    class _ProviderClient:
        def __init__(self, reservation_outcome: ProviderOutcome, ticket_outcome: ProviderOutcome):
            self._reservation_outcome = reservation_outcome
            self._ticket_outcome = ticket_outcome

        async def get_reservations(self, **_kwargs):  # noqa: ANN003
            return self._reservation_outcome

        async def ticket_info(self, **_kwargs):  # noqa: ANN003
            return self._ticket_outcome

    ok_client = _ProviderClient(
        ProviderOutcome(ok=True, data={"reservations": [{"reservation_id": "PNR-1", "provider": "SRT", "paid": False}]}),
        ProviderOutcome(ok=True, data={"tickets": [{"car_no": "1"}], "wct_no": "W1"}),
    )
    async def _client_ok(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return ok_client

    monkeypatch.setattr(train_service, "_get_logged_in_provider_client", _client_ok)
    reservations = await train_service.list_provider_reservations(db, user=user, provider="SRT", paid_only=False)
    assert reservations.reservations
    info = await train_service.get_provider_ticket_info(db, user=user, provider="SRT", reservation_id="PNR-1")
    assert info.wct_no == "W1"

    retry_client = _ProviderClient(
        ProviderOutcome(ok=False, retryable=True, error_message_safe="provider down"),
        ProviderOutcome(ok=False, retryable=False, error_message_safe="bad reservation"),
    )
    async def _client_retry(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return retry_client

    monkeypatch.setattr(train_service, "_get_logged_in_provider_client", _client_retry)
    with pytest.raises(HTTPException) as reservations_fail:
        await train_service.list_provider_reservations(db, user=user, provider="SRT", paid_only=False)
    assert reservations_fail.value.status_code == 502

    with pytest.raises(HTTPException) as ticket_fail:
        await train_service.get_provider_ticket_info(db, user=user, provider="SRT", reservation_id="PNR-2")
    assert ticket_fail.value.status_code == 400
