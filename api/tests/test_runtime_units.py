from __future__ import annotations

import asyncio
from datetime import datetime, timedelta, timezone
from types import SimpleNamespace
from uuid import uuid4

import pytest
from fastapi import HTTPException

import app.core.redis as redis_mod
import app.http.deps as deps
import app.services.wallet as wallet


class _FakeRedis:
    def __init__(self) -> None:
        self.closed = False
        self._store: dict[str, str | bytes] = {}
        self._scan_calls = 0
        self.deleted_keys: list[tuple[str, ...]] = []

    async def get(self, key: str):
        return self._store.get(key)

    async def set(self, key: str, value, ex: int | None = None):  # noqa: ANN001
        self._store[key] = value

    async def delete(self, *keys: str):
        deleted = 0
        for key in keys:
            if isinstance(key, bytes):
                key = key.decode("utf-8")
            if key in self._store:
                del self._store[key]
                deleted += 1
        self.deleted_keys.append(tuple(keys))
        return deleted

    async def scan(self, *, cursor: int, match: str, count: int):
        self._scan_calls += 1
        if self._scan_calls == 1:
            return (1, [b"wallet:payment:cvv:a", b"wallet:payment:cvv:b"])
        return (0, [b"wallet:payment:cvv:c"])

    async def aclose(self):
        self.closed = True


class _FakeRedisPool:
    def __init__(self, redis: _FakeRedis):
        self.redis = redis

    async def __aenter__(self):
        return self.redis

    async def __aexit__(self, *_args):
        return None


@pytest.mark.asyncio
async def test_redis_pool_lifecycle_and_helpers(monkeypatch):
    redis_mod._redis_pool_non_cde = None
    redis_mod._redis_pool_cde = None
    redis_mod._pool_lock_non_cde = None
    redis_mod._pool_lock_cde = None

    created: list[str] = []

    async def _fake_create_pool(*, purpose):  # noqa: ANN001
        created.append(purpose)
        return _FakeRedis()

    monkeypatch.setattr(redis_mod, "_create_pool", _fake_create_pool)

    non_cde_1 = await redis_mod.get_redis_client(purpose="non_cde")
    non_cde_2 = await redis_mod.get_redis_client(purpose="non_cde")
    cde_1 = await redis_mod.get_redis_client(purpose="cde")

    assert non_cde_1 is non_cde_2
    assert created.count("non_cde") == 1
    assert created.count("cde") == 1
    assert redis_mod._redis_url_for_purpose(purpose="non_cde")
    assert redis_mod._redis_url_for_purpose(purpose="cde")

    async with redis_mod.get_redis_pool(purpose="non_cde") as pooled:
        assert pooled is non_cde_1

    assert await redis_mod.get_cde_redis_client() is cde_1
    async with redis_mod.get_cde_redis_pool() as cde_pooled:
        assert cde_pooled is cde_1

    await redis_mod.close_redis_pool()
    assert non_cde_1.closed is True
    assert cde_1.closed is True
    await redis_mod.close_redis_pool()


@pytest.mark.asyncio
async def test_redis_create_pool_uses_expected_options(monkeypatch):
    captured: dict[str, object] = {}
    pool_marker = object()

    def _fake_pool_from_url(url: str, **kwargs):  # noqa: ANN001
        captured["pool_url"] = url
        captured["pool_kwargs"] = kwargs
        return pool_marker

    class _FakeRedisClient:
        def __init__(self, *, connection_pool=None):  # noqa: ANN001
            captured["redis_ctor_pool"] = connection_pool

        @classmethod
        def from_url(cls, url: str, **kwargs):  # noqa: ANN001
            captured["redis_from_url_called"] = True
            captured["redis_from_url_url"] = url
            captured["redis_from_url_kwargs"] = kwargs
            return cls(connection_pool=None)

    monkeypatch.setattr(
        redis_mod,
        "BlockingConnectionPool",
        SimpleNamespace(from_url=_fake_pool_from_url),
        raising=False,
    )
    monkeypatch.setattr(redis_mod, "Redis", _FakeRedisClient)
    await redis_mod._create_pool(purpose="non_cde")
    assert captured["pool_url"] == redis_mod.settings.resolved_redis_url_non_cde
    assert isinstance(captured["pool_kwargs"], dict)
    assert captured["pool_kwargs"]["decode_responses"] is False
    assert captured["redis_ctor_pool"] is pool_marker
    assert captured.get("redis_from_url_called") is not True


@pytest.mark.asyncio
async def test_redis_lock_and_double_check_branch(monkeypatch):
    redis_mod._redis_pool_non_cde = None
    redis_mod._pool_lock_non_cde = None
    redis_mod._pool_lock_cde = None

    # Cover lock reuse branches.
    existing_cde_lock = asyncio.Lock()
    redis_mod._pool_lock_cde = existing_cde_lock
    assert redis_mod._get_pool_lock(purpose="cde") is existing_cde_lock

    existing_non_cde_lock = asyncio.Lock()
    redis_mod._pool_lock_non_cde = existing_non_cde_lock
    assert redis_mod._get_pool_lock(purpose="non_cde") is existing_non_cde_lock

    # Cover double-check branch where pool appears after lock acquisition.
    redis_mod._redis_pool_non_cde = None

    class _LockCtx:
        async def __aenter__(self):  # noqa: ANN204
            redis_mod._redis_pool_non_cde = _FakeRedis()
            return self

        async def __aexit__(self, *_args):  # noqa: ANN204
            return None

    monkeypatch.setattr(redis_mod, "_get_pool_lock", lambda **_kwargs: _LockCtx())
    monkeypatch.setattr(redis_mod, "_create_pool", lambda **_kwargs: (_ for _ in ()).throw(AssertionError("should not create")))
    resolved = await redis_mod.get_redis_client(purpose="non_cde")
    assert isinstance(resolved, _FakeRedis)


def test_extract_bearer_token_parsing() -> None:
    assert deps._extract_bearer_token(None) is None
    assert deps._extract_bearer_token("Basic abc") is None
    assert deps._extract_bearer_token("Bearer") is None
    assert deps._extract_bearer_token("Bearer   token") == "token"


@pytest.mark.asyncio
async def test_auth_rate_limit_uses_client_ip(monkeypatch):
    seen: dict[str, object] = {}

    async def _fake_check(*, key: str, limit: int, window_seconds: int):
        seen["key"] = key
        seen["limit"] = limit
        seen["window"] = window_seconds

    monkeypatch.setattr(deps.rate_limiter, "check", _fake_check)
    monkeypatch.setattr(deps, "request_ip", lambda *_args, **_kwargs: "203.0.113.10")

    request = SimpleNamespace(
        client=SimpleNamespace(host="127.0.0.1"),
        headers={"x-forwarded-for": "10.0.0.1", "cf-connecting-ip": "10.0.0.2"},
        url=SimpleNamespace(path="/api/auth/login"),
    )

    await deps.auth_rate_limit(request)
    assert seen["key"] == "auth:203.0.113.10:/api/auth/login"


@pytest.mark.asyncio
async def test_internal_access_and_role_guards(monkeypatch):
    monkeypatch.setattr(deps.settings, "internal_api_key", None)
    with pytest.raises(HTTPException) as missing:
        await deps.require_internal_access("k")
    assert missing.value.status_code == 503

    monkeypatch.setattr(deps.settings, "internal_api_key", "expected")
    with pytest.raises(HTTPException) as denied:
        await deps.require_internal_access("wrong")
    assert denied.value.status_code == 403

    await deps.require_internal_access("expected")

    monkeypatch.setattr(deps.settings, "access_approval_required", True)
    pending_user = SimpleNamespace(access_status="pending", role=SimpleNamespace(name="user"))
    with pytest.raises(HTTPException) as pending:
        await deps.get_current_approved_user(pending_user)
    assert pending.value.status_code == 403

    approved_user = SimpleNamespace(access_status="approved", role=SimpleNamespace(name="user"))
    assert await deps.get_current_approved_user(approved_user) is approved_user

    with pytest.raises(HTTPException) as admin_required:
        await deps.get_current_admin(approved_user)
    assert admin_required.value.status_code == 403

    admin_user = SimpleNamespace(access_status="approved", role=SimpleNamespace(name="admin"))
    assert await deps.get_current_admin(admin_user) is admin_user

    dep = deps.require_role("admin")
    assert await dep(admin_user) is admin_user
    with pytest.raises(HTTPException):
        await dep(approved_user)


@pytest.mark.asyncio
async def test_get_current_user_mode_selection(monkeypatch):
    fake_user = SimpleNamespace(id="u")

    async def _resolve_bearer(*, bearer_token: str, db):  # noqa: ANN001
        assert bearer_token == "token"
        return fake_user

    async def _resolve_cookie(*, session_token: str | None, db):  # noqa: ANN001
        if session_token == "cookie":
            return SimpleNamespace(user=fake_user)
        return None

    monkeypatch.setattr(deps, "_resolve_user_from_supabase_bearer", _resolve_bearer)
    monkeypatch.setattr(deps, "_resolve_session_from_cookie", _resolve_cookie)

    request = SimpleNamespace(headers={"authorization": "Bearer token"})

    monkeypatch.setattr(deps.settings, "auth_mode", "supabase")
    assert await deps.get_current_user(request=request, session_token=None, db=object()) is fake_user
    request_no_bearer = SimpleNamespace(headers={})
    assert await deps.get_current_user(request=request_no_bearer, session_token="cookie", db=object()) is fake_user

    monkeypatch.setattr(deps.settings, "auth_mode", "legacy")
    assert await deps.get_current_user(request=request_no_bearer, session_token="cookie", db=object()) is fake_user

    with pytest.raises(HTTPException):
        await deps.get_current_user(request=request_no_bearer, session_token=None, db=object())


@pytest.mark.asyncio
async def test_get_current_session_and_cookie_resolution(monkeypatch):
    class _Result:
        def __init__(self, value):  # noqa: ANN001
            self._value = value

        def scalar_one_or_none(self):  # noqa: ANN201
            return self._value

    class _DB:
        def __init__(self, value):  # noqa: ANN001
            self.value = value
            self.committed = 0

        async def execute(self, _stmt):  # noqa: ANN001
            return _Result(self.value)

        async def commit(self):
            self.committed += 1

    assert await deps._resolve_session_from_cookie(session_token=None, db=_DB(None)) is None

    db_none = _DB(None)
    monkeypatch.setattr(deps, "hash_token", lambda token: f"hash:{token}")
    assert await deps._resolve_session_from_cookie(session_token="cookie", db=db_none) is None

    session_obj = SimpleNamespace(
        user=SimpleNamespace(id="u1"),
        last_seen_at=datetime.now(timezone.utc) - timedelta(hours=1),
    )
    db_with_session = _DB(session_obj)
    monkeypatch.setattr(deps, "should_update_session_activity", lambda **_kwargs: True)
    resolved = await deps._resolve_session_from_cookie(session_token="cookie", db=db_with_session)
    assert resolved is session_obj
    assert db_with_session.committed == 1

    db_no_commit = _DB(session_obj)
    monkeypatch.setattr(deps, "should_update_session_activity", lambda **_kwargs: False)
    await deps._resolve_session_from_cookie(session_token="cookie", db=db_no_commit)
    assert db_no_commit.committed == 0

    with pytest.raises(HTTPException):
        await deps.get_current_session(session_token=None, db=db_none)

    resolved_session = await deps.get_current_session(session_token="cookie", db=db_with_session)
    assert resolved_session is session_obj


@pytest.mark.asyncio
async def test_resolve_user_from_supabase_bearer_value_error_branch(monkeypatch):
    monkeypatch.setattr(deps, "verify_supabase_jwt", lambda _token: {"sub": "u"})

    async def _raise_value_error(*_args, **_kwargs):
        raise ValueError("invalid local mapping")

    monkeypatch.setattr(deps, "get_or_create_local_user_from_supabase_claims", _raise_value_error)

    with pytest.raises(HTTPException):
        await deps._resolve_user_from_supabase_bearer(bearer_token="token", db=object())


@pytest.mark.asyncio
async def test_resolve_user_from_supabase_bearer_invalid_jwt_branch(monkeypatch):
    def _raise_jwt(_token: str):
        raise deps.SupabaseJWTError("bad token")

    monkeypatch.setattr(deps, "verify_supabase_jwt", _raise_jwt)

    with pytest.raises(HTTPException):
        await deps._resolve_user_from_supabase_bearer(bearer_token="token", db=object())


@pytest.mark.asyncio
async def test_get_current_user_supabase_rejects_when_missing_bearer_and_cookie(monkeypatch):
    monkeypatch.setattr(deps.settings, "auth_mode", "supabase")
    request = SimpleNamespace(headers={})
    with pytest.raises(HTTPException):
        await deps.get_current_user(request=request, session_token=None, db=object())


def test_is_access_approved_disabled_flag(monkeypatch):
    monkeypatch.setattr(deps.settings, "access_approval_required", False)
    assert deps._is_access_approved(SimpleNamespace(access_status="pending")) is True


def test_wallet_masking_and_cache_alias() -> None:
    assert wallet._mask_card_number("4111 1111 1111 1234") == "**** **** **** 1234"
    assert wallet._mask_card_number("12") == "****"
    assert wallet.get_redis_pool() is not None


def test_wallet_deserialize_payload_branches(monkeypatch) -> None:
    class _FakeCrypto:
        def decrypt_payload(self, **_kwargs):
            return {"cvv": "123", "expires_at": datetime.now(timezone.utc).isoformat()}

    monkeypatch.setattr(wallet, "get_envelope_crypto", lambda: _FakeCrypto())
    monkeypatch.setattr(wallet.settings, "payment_require_cvv_kek_version", False)

    payload = {
        "ciphertext": "c",
        "nonce": "n",
        "wrapped_dek": "w",
        "dek_nonce": "d",
        "aad": "a",
        "kek_version": 1,
    }

    parsed = wallet._deserialize_cached_cvv_payload(wallet._serialize_encrypted_payload(payload))
    assert parsed is not None

    monkeypatch.setattr(wallet.settings, "payment_require_cvv_kek_version", True)
    missing_version = payload.copy()
    missing_version.pop("kek_version")
    assert wallet._deserialize_cached_cvv_payload(wallet._serialize_encrypted_payload(missing_version)) is None

    monkeypatch.setattr(wallet, "get_envelope_crypto", lambda: SimpleNamespace(decrypt_payload=lambda **_kwargs: "bad"))
    assert wallet._deserialize_cached_cvv_payload(wallet._serialize_encrypted_payload(payload)) is None

    monkeypatch.setattr(
        wallet,
        "get_envelope_crypto",
        lambda: SimpleNamespace(decrypt_payload=lambda **_kwargs: {"cvv": "123", "expires_at": 123}),
    )
    assert wallet._deserialize_cached_cvv_payload(wallet._serialize_encrypted_payload(payload)) is None
    assert wallet._deserialize_cached_cvv_payload("not-json") is None


@pytest.mark.asyncio
async def test_wallet_load_and_delete_helpers(monkeypatch):
    user_id = uuid4()
    redis = _FakeRedis()

    # First key returns bytes blob.
    valid_blob = wallet._serialize_encrypted_payload(
        {
            "ciphertext": "c",
            "nonce": "n",
            "wrapped_dek": "w",
            "dek_nonce": "d",
            "aad": "a",
            "kek_version": 1,
        }
    )
    redis._store[wallet._payment_cvv_redis_key(user_id)] = valid_blob.encode("utf-8")

    monkeypatch.setattr(wallet, "get_redis_pool", lambda: _FakeRedisPool(redis))
    monkeypatch.setattr(
        wallet,
        "_deserialize_cached_cvv_payload",
        lambda _blob: {"cvv": "123", "expires_at": datetime.now(timezone.utc)},
    )

    loaded = await wallet._load_cached_cvv_payload(user_id=user_id)
    assert loaded is not None

    expires_at = await wallet._load_cached_cvv_until(user_id=user_id)
    assert isinstance(expires_at, datetime)

    await wallet._clear_cached_cvv(user_id=user_id)
    assert redis.deleted_keys

    redis._store["wallet:payment:cvv:a"] = "1"
    redis._store["wallet:payment:cvv:b"] = "2"
    redis._store["wallet:payment:cvv:c"] = "3"
    deleted = await wallet._delete_redis_keys_matching(pattern="wallet:payment:cvv:*")
    assert deleted == 3

    redis._store.clear()
    assert await wallet._load_cached_cvv_payload(user_id=user_id) is None
    assert await wallet._load_cached_cvv_until(user_id=user_id) is None

    # Cover loop continue path (first parse returns None, second key returns payload).
    redis._store[wallet._payment_cvv_redis_key(user_id)] = "bad"
    redis._store[wallet._legacy_payment_cvv_redis_key(user_id)] = "good"
    calls: list[str] = []

    def _parse(blob: str):  # noqa: ANN001
        calls.append(blob)
        if blob == "bad":
            return None
        return {"cvv": "321", "expires_at": datetime.now(timezone.utc)}

    monkeypatch.setattr(wallet, "_deserialize_cached_cvv_payload", _parse)
    loaded_from_legacy = await wallet._load_cached_cvv_payload(user_id=user_id)
    assert loaded_from_legacy is not None
    assert calls == ["bad", "good"]


@pytest.mark.asyncio
async def test_wallet_execution_and_status_branches(monkeypatch):
    user_id = uuid4()
    secret = SimpleNamespace(updated_at=datetime.now(timezone.utc), id=1)

    async def _latest_secret(*_args, **_kwargs):
        return secret

    monkeypatch.setattr(wallet, "_latest_payment_secret_for_user", _latest_secret)

    async def _latest_none(*_args, **_kwargs):
        return None

    monkeypatch.setattr(wallet, "_latest_payment_secret_for_user", _latest_none)
    status_none = await wallet.get_payment_card_status(db=object(), user=SimpleNamespace(id=user_id))
    assert status_none.configured is False
    assert await wallet.get_payment_card_for_execution(db=object(), user_id=user_id) is None

    monkeypatch.setattr(wallet, "_latest_payment_secret_for_user", _latest_secret)

    # Status decrypt failure.
    monkeypatch.setattr(wallet, "decrypt_secret", lambda _secret: (_ for _ in ()).throw(ValueError("bad")))
    status = await wallet.get_payment_card_status(db=object(), user=SimpleNamespace(id=user_id))
    assert status.configured is False

    # Status invalid expiry payload.
    monkeypatch.setattr(wallet, "decrypt_secret", lambda _secret: {"card_number": "4111", "expiry_month": "x", "expiry_year": "y"})
    status = await wallet.get_payment_card_status(db=object(), user=SimpleNamespace(id=user_id))
    assert status.configured is False
    monkeypatch.setattr(wallet, "decrypt_secret", lambda _secret: {"card_number": "4111", "expiry_month": 12, "expiry_year": "bad"})
    status = await wallet.get_payment_card_status(db=object(), user=SimpleNamespace(id=user_id))
    assert status.configured is False

    # Execution decrypt failure and validation failures.
    monkeypatch.setattr(wallet, "decrypt_secret", lambda _secret: (_ for _ in ()).throw(ValueError("bad")))
    assert await wallet.get_payment_card_for_execution(db=object(), user_id=user_id) is None

    monkeypatch.setattr(wallet, "decrypt_secret", lambda _secret: {"card_number": "", "pin2": "12", "birth_date": "1990-01-01", "expiry_month": 1, "expiry_year": 2099})
    assert await wallet.get_payment_card_for_execution(db=object(), user_id=user_id) is None

    monkeypatch.setattr(wallet, "decrypt_secret", lambda _secret: {"card_number": "4111", "pin2": "12", "birth_date": "bad", "expiry_month": 1, "expiry_year": 2099})
    assert await wallet.get_payment_card_for_execution(db=object(), user_id=user_id) is None

    # Happy execution path returns only non-CVV payment fields.
    monkeypatch.setattr(
        wallet,
        "decrypt_secret",
        lambda _secret: {
            "card_number": "4111111111111111",
            "pin2": "12",
            "birth_date": "1990-01-01",
            "expiry_month": 12,
            "expiry_year": 2099,
        },
    )
    execution = await wallet.get_payment_card_for_execution(db=object(), user_id=user_id)
    assert execution is not None
    assert execution["card_expire"] == "9912"
    assert "cvv" not in execution
    assert "cvv_cached_until" not in execution

    monkeypatch.setattr(
        wallet,
        "decrypt_secret",
        lambda _secret: {
            "card_number": "4111111111111111",
            "pin2": "12",
            "birth_date": "1990-01-01",
            "expiry_month": 12,
            "expiry_year": 2099,
        },
    )
    status_ok = await wallet.get_payment_card_status(db=object(), user=SimpleNamespace(id=user_id))
    assert status_ok.configured is True


@pytest.mark.asyncio
async def test_wallet_set_card_rejects_expired_card(monkeypatch):
    payload = SimpleNamespace(
        card_number="4111 1111 1111 1111",
        expiry_month=1,
        expiry_year=2000,
        birth_date=datetime(1990, 1, 1).date(),
        pin2="12",
        cvv="123",
    )

    monkeypatch.setattr(wallet, "utc_now", lambda: datetime(2026, 1, 1, tzinfo=timezone.utc))

    with pytest.raises(HTTPException) as exc:
        await wallet.set_payment_card(db=object(), user=SimpleNamespace(id=uuid4()), payload=payload)
    assert exc.value.status_code == 400


@pytest.mark.asyncio
async def test_clear_payment_card_cache_alias(monkeypatch):
    called: list[object] = []

    async def _clear(*, user_id):  # noqa: ANN001
        called.append(user_id)

    monkeypatch.setattr(wallet, "_clear_cached_cvv", _clear)
    user_id = uuid4()
    await wallet.clear_payment_card_cache(user_id=user_id)
    assert called == [user_id]


@pytest.mark.asyncio
async def test_wallet_set_payment_card_creates_new_secret(monkeypatch):
    now = datetime(2026, 2, 22, tzinfo=timezone.utc)
    user = SimpleNamespace(id=uuid4())

    payload = SimpleNamespace(
        card_number="4111 1111 1111 1111",
        expiry_month=12,
        expiry_year=2099,
        birth_date=datetime(1990, 1, 1).date(),
        pin2="12",
        cvv="123",
    )

    encrypted_secret = SimpleNamespace(
        ciphertext="new-c",
        nonce="new-n",
        wrapped_dek="new-w",
        dek_nonce="new-d",
        aad="new-a",
        kek_version=9,
        updated_at=now,
    )

    class _DB:
        def __init__(self) -> None:
            self.added: list[object] = []
            self.commits = 0

        def add(self, value):  # noqa: ANN001
            self.added.append(value)

        async def commit(self):
            self.commits += 1

    db = _DB()

    async def _latest_secret_none(*_args, **_kwargs):
        return None

    monkeypatch.setattr(wallet, "utc_now", lambda: now)
    monkeypatch.setattr(wallet, "build_encrypted_secret", lambda **_kwargs: encrypted_secret)
    monkeypatch.setattr(wallet, "_latest_payment_secret_for_user", _latest_secret_none)

    status = await wallet.set_payment_card(db=db, user=user, payload=payload)
    assert db.added == [encrypted_secret]
    assert db.commits == 1
    assert status.configured is True


@pytest.mark.asyncio
async def test_wallet_set_payment_card_updates_existing_secret(monkeypatch):
    now = datetime(2026, 2, 22, tzinfo=timezone.utc)
    user = SimpleNamespace(id=uuid4())
    existing_secret = SimpleNamespace(
        ciphertext="old",
        nonce="old",
        wrapped_dek="old",
        dek_nonce="old",
        aad="old",
        kek_version=1,
        updated_at=now - timedelta(days=1),
    )

    payload = SimpleNamespace(
        card_number="4111 1111 1111 1111",
        expiry_month=12,
        expiry_year=2099,
        birth_date=datetime(1990, 1, 1).date(),
        pin2="12",
        cvv="123",
    )

    encrypted_secret = SimpleNamespace(
        ciphertext="new-c",
        nonce="new-n",
        wrapped_dek="new-w",
        dek_nonce="new-d",
        aad="new-a",
        kek_version=9,
        updated_at=now,
    )

    class _DB:
        def __init__(self) -> None:
            self.added: list[object] = []
            self.commits = 0

        def add(self, value):  # noqa: ANN001
            self.added.append(value)

        async def commit(self):
            self.commits += 1

    db = _DB()

    async def _latest_secret(*_args, **_kwargs):
        return existing_secret

    monkeypatch.setattr(wallet, "utc_now", lambda: now)
    monkeypatch.setattr(wallet, "build_encrypted_secret", lambda **_kwargs: encrypted_secret)
    monkeypatch.setattr(wallet, "_latest_payment_secret_for_user", _latest_secret)

    status = await wallet.set_payment_card(db=db, user=user, payload=payload)

    assert db.added == []
    assert db.commits == 1
    assert existing_secret.ciphertext == "new-c"
    assert existing_secret.nonce == "new-n"
    assert existing_secret.wrapped_dek == "new-w"
    assert existing_secret.dek_nonce == "new-d"
    assert existing_secret.aad == "new-a"
    assert existing_secret.kek_version == 9
    assert status.configured is True


@pytest.mark.asyncio
async def test_latest_payment_secret_for_user_query_helper() -> None:
    sentinel = object()

    class _Result:
        def scalar_one_or_none(self):  # noqa: ANN201
            return sentinel

    class _DB:
        async def execute(self, _stmt):  # noqa: ANN001, ANN201
            return _Result()

    resolved = await wallet._latest_payment_secret_for_user(_DB(), user_id=uuid4())
    assert resolved is sentinel


@pytest.mark.asyncio
async def test_wallet_cache_cvv_writes_clamped_ttl(monkeypatch):
    user_id = uuid4()
    redis = _FakeRedis()
    seen: dict[str, object] = {}

    class _Encrypted:
        ciphertext = "cipher"
        nonce = "nonce"
        wrapped_dek = "wrapped"
        dek_nonce = "dek"
        aad = "aad"
        kek_version = 7

    class _Crypto:
        def encrypt_payload(self, *, payload, aad_text):  # noqa: ANN001
            seen["payload"] = payload
            seen["aad_text"] = aad_text
            return _Encrypted()

    async def _capturing_set(key: str, value, ex: int | None = None):  # noqa: ANN001
        seen["set_key"] = key
        seen["set_value"] = value
        seen["set_ex"] = ex
        await _FakeRedis.set(redis, key, value, ex=ex)

    monkeypatch.setattr(redis, "set", _capturing_set)
    monkeypatch.setattr(wallet, "get_redis_pool", lambda: _FakeRedisPool(redis))
    monkeypatch.setattr(wallet, "get_envelope_crypto", lambda: _Crypto())
    monkeypatch.setattr(wallet.settings, "payment_cvv_ttl_min_seconds", 60)
    monkeypatch.setattr(wallet.settings, "payment_cvv_ttl_seconds", 1200)
    monkeypatch.setattr(wallet.settings, "payment_cvv_ttl_max_seconds", 900)

    expires_at = await wallet._cache_cvv(user_id=user_id, cvv="321")
    assert isinstance(expires_at, datetime)
    assert seen["set_key"] == wallet._payment_cvv_redis_key(user_id)
    assert seen["set_ex"] == 900
    assert seen["aad_text"] == f"payment_cvv:{user_id}"


@pytest.mark.asyncio
async def test_wallet_delete_redis_keys_matching_no_key_branch(monkeypatch):
    class _NoKeysRedis(_FakeRedis):
        async def scan(self, *, cursor: int, match: str, count: int):
            return (0, [])

    redis = _NoKeysRedis()
    monkeypatch.setattr(wallet, "get_redis_pool", lambda: _FakeRedisPool(redis))
    deleted = await wallet._delete_redis_keys_matching(pattern="wallet:payment:cvv:*")
    assert deleted == 0
    assert redis.deleted_keys == []


@pytest.mark.asyncio
async def test_purge_all_saved_payment_data_and_clear_payment_card(monkeypatch):
    class _ScalarResult:
        def __init__(self, value):  # noqa: ANN001
            self._value = value

        def scalar_one(self):  # noqa: ANN201
            return self._value

    class _DB:
        def __init__(self) -> None:
            self.execute_calls = 0
            self.commits = 0

        async def execute(self, _stmt):  # noqa: ANN001
            self.execute_calls += 1
            if self.execute_calls == 1:
                return _ScalarResult(5)
            return _ScalarResult(None)

        async def commit(self):
            self.commits += 1

    db = _DB()
    async def _purge_cached_cvv() -> dict[str, int]:
        return {
            "redis_cvv_keys_deleted_current": 2,
            "redis_cvv_keys_deleted_legacy": 3,
            "redis_cvv_keys_deleted_total": 5,
        }

    cleared: list[object] = []

    async def _clear_cached(*, user_id):  # noqa: ANN001
        cleared.append(user_id)

    monkeypatch.setattr(wallet, "purge_cached_payment_cvv_data", _purge_cached_cvv)
    monkeypatch.setattr(wallet, "_clear_cached_cvv", _clear_cached)

    result = await wallet.purge_all_saved_payment_data(db)
    assert result == {
        "db_payment_card_secrets_deleted": 5,
        "redis_cvv_keys_deleted_current": 2,
        "redis_cvv_keys_deleted_legacy": 3,
        "redis_cvv_keys_deleted_total": 5,
    }

    user = SimpleNamespace(id=uuid4())
    cleared_status = await wallet.clear_payment_card(db=db, user=user)
    assert db.commits >= 2
    assert cleared == [user.id]
    assert cleared_status.configured is False
