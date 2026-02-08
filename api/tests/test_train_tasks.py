from __future__ import annotations

import asyncio
import json
from datetime import datetime, timedelta, timezone

import fakeredis.aioredis
import pytest
from fastapi import HTTPException
from sqlalchemy import select

from app.db.models import Artifact, Task, TaskAttempt, User
from app.modules.train.providers.base import ProviderOutcome, ProviderSchedule
from app.modules.train.providers.ktx_client import parse_ktx_search_response
from app.modules.train.providers.srt_client import parse_srt_search_response
from app.modules.train.rate_limiter import RedisTokenBucketLimiter
from app.modules.train.service import get_task_detail, retry_task_now
from app.modules.train.schemas import ProviderCredentialStatus
from app.modules.train.worker import run_train_task
from tests.conftest import make_fake_get_redis_client


def _utc_now() -> datetime:
    return datetime.now(timezone.utc)


async def _register_and_login(client, *, email: str = "train-user@example.com") -> str:
    await client.post(
        "/api/auth/register",
        json={"email": email, "password": "SuperSecret123", "display_name": "Train User"},
    )
    login_res = await client.post(
        "/api/auth/login",
        json={"email": email, "password": "SuperSecret123", "remember_me": True},
    )
    assert login_res.status_code == 200
    cookie = login_res.cookies.get("bominal_session")
    assert cookie
    return cookie


@pytest.mark.asyncio
async def test_train_task_creation_idempotency(client, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()

    async def _noop_enqueue(task_id: str, defer_seconds: float = 0.0) -> None:
        return None

    monkeypatch.setattr("app.modules.train.service.enqueue_train_task", _noop_enqueue)
    monkeypatch.setattr("app.modules.train.service.get_redis_client", make_fake_get_redis_client(fake_redis))

    cookie = await _register_and_login(client)

    stations_res = await client.get("/api/train/stations", cookies={"bominal_session": cookie})
    assert stations_res.status_code == 200
    stations = stations_res.json()["stations"]
    assert any(station["name"] == "수서" and station["srt_code"] == "0551" for station in stations)

    credentials_res = await client.post(
        "/api/train/credentials/srt",
        cookies={"bominal_session": cookie},
        json={"username": "mock-user", "password": "mock-password"},
    )
    assert credentials_res.status_code == 200
    assert credentials_res.json()["configured"] is True

    search_res = await client.post(
        "/api/train/search",
        cookies={"bominal_session": cookie},
        json={
            "providers": ["SRT"],
            "dep": "수서",
            "arr": "부산",
            "date": _utc_now().date().isoformat(),
            "time_window": {"start": "06:00", "end": "12:00"},
        },
    )
    assert search_res.status_code == 200
    schedules = search_res.json()["schedules"]
    assert schedules

    selected = [
        {
            "schedule_id": schedules[0]["schedule_id"],
            "departure_at": schedules[0]["departure_at"],
            "rank": 1,
        }
    ]

    payload = {
        "provider": "SRT",
        "dep": "수서",
        "arr": "부산",
        "date": _utc_now().date().isoformat(),
        "selected_trains_ranked": selected,
        "passengers": {"adults": 1, "children": 0},
        "seat_class": "general",
        "auto_pay": True,
    }

    first = await client.post("/api/train/tasks", cookies={"bominal_session": cookie}, json=payload)
    second = await client.post("/api/train/tasks", cookies={"bominal_session": cookie}, json=payload)

    assert first.status_code == 200
    assert second.status_code == 200
    assert first.json()["task"]["id"] == second.json()["task"]["id"]
    assert second.json()["deduplicated"] is True


@pytest.mark.asyncio
async def test_train_task_creation_accepts_mixed_provider_selection(client, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()

    async def _noop_enqueue(task_id: str, defer_seconds: float = 0.0) -> None:
        return None

    async def _always_verified(_db, *, user, provider, force_live=False):
        return ProviderCredentialStatus(
            configured=True,
            verified=True,
            username=f"{provider.lower()}-user",
            verified_at=None,
            detail=None,
        )

    monkeypatch.setattr("app.modules.train.service.enqueue_train_task", _noop_enqueue)
    monkeypatch.setattr("app.modules.train.service.get_redis_client", make_fake_get_redis_client(fake_redis))
    monkeypatch.setattr("app.modules.train.service._verify_provider_credentials", _always_verified)

    cookie = await _register_and_login(client, email="mixed-task@example.com")
    departure = (_utc_now() + timedelta(hours=2)).isoformat()
    departure_later = (_utc_now() + timedelta(hours=3)).isoformat()

    response = await client.post(
        "/api/train/tasks",
        cookies={"bominal_session": cookie},
        json={
            "dep": "수서",
            "arr": "마산",
            "date": _utc_now().date().isoformat(),
            "selected_trains_ranked": [
                {"schedule_id": "srt-1", "departure_at": departure, "rank": 1, "provider": "SRT"},
                {"schedule_id": "ktx-1", "departure_at": departure_later, "rank": 2, "provider": "KTX"},
            ],
            "passengers": {"adults": 1, "children": 0},
            "seat_class": "general_preferred",
            "auto_pay": False,
        },
    )

    assert response.status_code == 200
    body = response.json()
    assert body["deduplicated"] is False
    assert body["task"]["spec_json"]["providers"] == ["KTX", "SRT"]


@pytest.mark.asyncio
async def test_train_search_returns_provider_errors_when_all_fail(client, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()

    class FailingProvider:
        provider_name = "SRT"

        async def login(self, **kwargs):
            return ProviderOutcome(ok=True)

        async def search(self, **kwargs):
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="provider_unreachable",
                error_message_safe="temporary provider error",
            )

        async def reserve(self, **kwargs):
            return ProviderOutcome(ok=False)

        async def pay(self, **kwargs):
            return ProviderOutcome(ok=False)

        async def cancel(self, **kwargs):
            return ProviderOutcome(ok=False)

    monkeypatch.setattr("app.modules.train.service.get_redis_client", make_fake_get_redis_client(fake_redis))
    monkeypatch.setattr("app.modules.train.service.get_provider_client", lambda provider: FailingProvider())

    cookie = await _register_and_login(client, email="search-fail@example.com")
    srt_credentials_res = await client.post(
        "/api/train/credentials/srt",
        cookies={"bominal_session": cookie},
        json={"username": "srt-user", "password": "srt-pass"},
    )
    assert srt_credentials_res.status_code == 200

    ktx_credentials_res = await client.post(
        "/api/train/credentials/ktx",
        cookies={"bominal_session": cookie},
        json={"username": "ktx-user", "password": "ktx-pass"},
    )
    assert ktx_credentials_res.status_code == 200

    response = await client.post(
        "/api/train/search",
        cookies={"bominal_session": cookie},
        json={
            "providers": ["SRT", "KTX"],
            "dep": "수서",
            "arr": "마산",
            "date": _utc_now().date().isoformat(),
            "time_window": {"start": "06:00", "end": "12:00"},
        },
    )
    assert response.status_code == 502
    assert "All provider searches failed" in response.json()["detail"]


@pytest.mark.asyncio
async def test_srt_credentials_required_for_srt_search(client, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    monkeypatch.setattr("app.modules.train.service.get_redis_client", make_fake_get_redis_client(fake_redis))

    cookie = await _register_and_login(client, email="credential-check@example.com")

    missing_res = await client.post(
        "/api/train/search",
        cookies={"bominal_session": cookie},
        json={
            "providers": ["SRT"],
            "dep": "수서",
            "arr": "마산",
            "date": _utc_now().date().isoformat(),
            "time_window": {"start": "06:00", "end": "12:00"},
        },
    )
    assert missing_res.status_code == 502
    assert "srt_credentials_missing" in missing_res.json()["detail"]

    credential_res = await client.post(
        "/api/train/credentials/srt",
        cookies={"bominal_session": cookie},
        json={"username": "mock-user", "password": "mock-password"},
    )
    assert credential_res.status_code == 200
    assert credential_res.json()["configured"] is True

    ok_res = await client.post(
        "/api/train/search",
        cookies={"bominal_session": cookie},
        json={
            "providers": ["SRT"],
            "dep": "수서",
            "arr": "마산",
            "date": _utc_now().date().isoformat(),
            "time_window": {"start": "06:00", "end": "12:00"},
        },
    )
    assert ok_res.status_code == 200
    assert len(ok_res.json()["schedules"]) > 0


@pytest.mark.asyncio
async def test_provider_credentials_status_checks_both(client, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    monkeypatch.setattr("app.modules.train.service.get_redis_client", make_fake_get_redis_client(fake_redis))

    cookie = await _register_and_login(client, email="status-check@example.com")

    initial_status = await client.get("/api/train/credentials/status", cookies={"bominal_session": cookie})
    assert initial_status.status_code == 200
    body = initial_status.json()
    assert body["ktx"]["configured"] is False
    assert body["srt"]["configured"] is False

    srt_credentials_res = await client.post(
        "/api/train/credentials/srt",
        cookies={"bominal_session": cookie},
        json={"username": "srt-user", "password": "srt-pass"},
    )
    assert srt_credentials_res.status_code == 200

    ktx_credentials_res = await client.post(
        "/api/train/credentials/ktx",
        cookies={"bominal_session": cookie},
        json={"username": "ktx-user", "password": "ktx-pass"},
    )
    assert ktx_credentials_res.status_code == 200

    verified_status = await client.get("/api/train/credentials/status", cookies={"bominal_session": cookie})
    assert verified_status.status_code == 200
    body = verified_status.json()
    assert body["ktx"]["verified"] is True
    assert body["srt"]["verified"] is True


@pytest.mark.asyncio
async def test_task_expires_when_deadline_passed(db_session_factory, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()

    async def _noop_enqueue(task_id: str, defer_seconds: float = 0.0) -> None:
        return None

    monkeypatch.setattr("app.modules.train.worker.enqueue_train_task", _noop_enqueue)

    user = User(
        email="expire@example.com",
        password_hash="x",
        display_name="Expire User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    task = Task(
        user_id=user.id,
        module="train",
        state="QUEUED",
        deadline_at=_utc_now() - timedelta(minutes=1),
        spec_json={
            "provider": "SRT",
            "dep": "수서",
            "arr": "부산",
            "date": _utc_now().date().isoformat(),
            "selected_trains_ranked": [
                {
                    "schedule_id": "expired-schedule",
                    "departure_at": (_utc_now() - timedelta(minutes=1)).isoformat(),
                    "rank": 1,
                }
            ],
            "passengers": {"adults": 1, "children": 0},
            "seat_class": "general",
            "auto_pay": True,
        },
        idempotency_key="expired-test",
    )
    db_session.add(task)
    await db_session.commit()

    await run_train_task({"db_factory": db_session_factory, "redis": fake_redis}, str(task.id))

    await db_session.refresh(task)
    refreshed = await db_session.get(Task, task.id)
    assert refreshed is not None
    assert refreshed.state == "EXPIRED"


@pytest.mark.asyncio
async def test_rate_limiter_under_concurrency(monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    limiter = RedisTokenBucketLimiter(fake_redis)

    monkeypatch.setattr(
        "app.modules.train.rate_limiter.DEFAULT_BUCKET_CONFIG",
        {
            "global": {"capacity": 1.0, "refill_per_second": 5.0},
            "provider": {"capacity": 1.0, "refill_per_second": 5.0},
            "credential": {"capacity": 1.0, "refill_per_second": 5.0},
        },
    )

    async def _acquire_once():
        return await limiter.acquire_provider_call(
            provider="SRT",
            user_bucket_key="u1",
            host_bucket_key="host1",
        )

    results = await asyncio.gather(*[_acquire_once() for _ in range(5)])
    waited = [result.waited_ms for result in results]
    assert any(wait > 0 for wait in waited)


@pytest.mark.asyncio
async def test_worker_mixed_provider_selection_cancels_loser_and_pays_winner(db_session_factory, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()

    class _NoLimitResult:
        waited_ms = 0
        rounds = 1

    async def _no_limit(self, **kwargs):
        return _NoLimitResult()

    async def _noop_enqueue(task_id: str, defer_seconds: float = 0.0) -> None:
        return None

    async def _fake_credentials(db, *, user_id, provider):
        return {"username": f"{provider.lower()}-user", "password": "secret"}

    class FakeProvider:
        def __init__(self, provider: str, schedule_id: str, reservation_id: str, reserve_delay: float = 0.0) -> None:
            self.provider_name = provider
            self.schedule_id = schedule_id
            self.reservation_id = reservation_id
            self.reserve_delay = reserve_delay
            self.pay_calls = 0
            self.cancel_calls = 0

        async def login(self, **kwargs):
            return ProviderOutcome(ok=True)

        async def search(self, **kwargs):
            departure = _utc_now() + timedelta(minutes=30)
            schedule = ProviderSchedule(
                schedule_id=self.schedule_id,
                provider=self.provider_name,
                dep=kwargs["dep"],
                arr=kwargs["arr"],
                departure_at=departure,
                arrival_at=departure + timedelta(minutes=120),
                train_no=f"{self.provider_name[:1]}500",
                availability={"general": True, "special": True},
            )
            return ProviderOutcome(ok=True, data={"schedules": [schedule]})

        async def reserve(self, **kwargs):
            if self.reserve_delay:
                await asyncio.sleep(self.reserve_delay)
            return ProviderOutcome(ok=True, data={"reservation_id": self.reservation_id})

        async def pay(self, **kwargs):
            self.pay_calls += 1
            return ProviderOutcome(
                ok=True,
                data={"payment_id": f"{self.provider_name}-pay-1", "ticket_no": f"{self.provider_name}-ticket-1"},
            )

        async def cancel(self, **kwargs):
            self.cancel_calls += 1
            return ProviderOutcome(ok=True, data={"cancelled": True})

    clients = {
        "SRT": FakeProvider("SRT", schedule_id="srt-rank1", reservation_id="srt-rsv-1", reserve_delay=0.12),
        "KTX": FakeProvider("KTX", schedule_id="ktx-rank2", reservation_id="ktx-rsv-1", reserve_delay=0.01),
    }

    monkeypatch.setattr("app.modules.train.worker.enqueue_train_task", _noop_enqueue)
    monkeypatch.setattr("app.modules.train.worker.RedisTokenBucketLimiter.acquire_provider_call", _no_limit)
    monkeypatch.setattr("app.modules.train.worker._load_provider_credentials", _fake_credentials)
    monkeypatch.setattr("app.modules.train.worker.get_provider_client", lambda provider: clients[provider])

    user = User(
        email="mixed-worker@example.com",
        password_hash="x",
        display_name="Mixed Worker User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    deadline = _utc_now() + timedelta(hours=1)
    task = Task(
        user_id=user.id,
        module="train",
        state="QUEUED",
        deadline_at=deadline,
        spec_json={
            "providers": ["SRT", "KTX"],
            "dep": "수서",
            "arr": "마산",
            "date": _utc_now().date().isoformat(),
            "selected_trains_ranked": [
                {
                    "schedule_id": "srt-rank1",
                    "departure_at": (_utc_now() + timedelta(hours=2)).isoformat(),
                    "rank": 1,
                    "provider": "SRT",
                },
                {
                    "schedule_id": "ktx-rank2",
                    "departure_at": (_utc_now() + timedelta(hours=2, minutes=10)).isoformat(),
                    "rank": 2,
                    "provider": "KTX",
                },
            ],
            "passengers": {"adults": 1, "children": 0},
            "seat_class": "general_preferred",
            "auto_pay": True,
        },
        idempotency_key="mixed-worker-idempotency",
    )
    db_session.add(task)
    await db_session.commit()

    await run_train_task({"db_factory": db_session_factory, "redis": fake_redis}, str(task.id))

    async with db_session_factory() as verify_session:
        refreshed = await verify_session.get(Task, task.id)
        assert refreshed is not None
        assert refreshed.state == "COMPLETED"

        artifacts = (
            await verify_session.execute(select(Artifact).where(Artifact.task_id == task.id).where(Artifact.kind == "ticket"))
        ).scalars().all()
        assert len(artifacts) == 1
        assert artifacts[0].data_json_safe["provider"] == "SRT"
        assert artifacts[0].data_json_safe["paid"] is True

        attempts = (await verify_session.execute(select(TaskAttempt).where(TaskAttempt.task_id == task.id))).scalars().all()
    assert any(attempt.action == "CANCEL" and attempt.provider == "KTX" and attempt.ok for attempt in attempts)
    assert any(attempt.action == "PAY" and attempt.provider == "SRT" and attempt.ok for attempt in attempts)

    assert clients["SRT"].pay_calls == 1
    assert clients["KTX"].pay_calls == 0
    assert clients["KTX"].cancel_calls == 1


@pytest.mark.asyncio
async def test_worker_relogs_and_retries_reserve_on_provider_auth_error(db_session_factory, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()

    class _NoLimitResult:
        waited_ms = 0
        rounds = 1

    async def _no_limit(self, **kwargs):
        return _NoLimitResult()

    async def _noop_enqueue(task_id: str, defer_seconds: float = 0.0) -> None:
        return None

    async def _fake_credentials(db, *, user_id, provider):
        return {"username": f"{provider.lower()}-user", "password": "secret"}

    class FakeProvider:
        provider_name = "SRT"

        def __init__(self) -> None:
            self.login_calls = 0
            self.reserve_calls = 0
            self.pay_calls = 0

        async def login(self, **kwargs):
            self.login_calls += 1
            return ProviderOutcome(ok=True)

        async def search(self, **kwargs):
            departure = _utc_now() + timedelta(minutes=40)
            return ProviderOutcome(
                ok=True,
                data={
                    "schedules": [
                        ProviderSchedule(
                            schedule_id="srt-relogin-schedule",
                            provider="SRT",
                            dep=kwargs["dep"],
                            arr=kwargs["arr"],
                            departure_at=departure,
                            arrival_at=departure + timedelta(minutes=90),
                            train_no="S900",
                            availability={"general": True, "special": True},
                        )
                    ]
                },
            )

        async def reserve(self, **kwargs):
            self.reserve_calls += 1
            if self.reserve_calls == 1:
                return ProviderOutcome(
                    ok=False,
                    retryable=False,
                    error_code="srt_reserve_fail_auth",
                    error_message_safe="로그인 후 사용하십시요.",
                )
            return ProviderOutcome(
                ok=True,
                data={
                    "reservation_id": "rsv-relogin-1",
                    "http_trace": {"endpoint": "reserve", "status_code": 200},
                },
            )

        async def pay(self, **kwargs):
            self.pay_calls += 1
            return ProviderOutcome(
                ok=True,
                data={
                    "payment_id": "pay-relogin-1",
                    "ticket_no": "ticket-relogin-1",
                    "http_trace": {"endpoint": "pay", "status_code": 200},
                },
            )

        async def get_reservations(self, **kwargs):
            paid = self.pay_calls > 0
            return ProviderOutcome(
                ok=True,
                data={
                    "reservations": [
                        {
                            "reservation_id": "rsv-relogin-1",
                            "provider": "SRT",
                            "paid": paid,
                            "waiting": False,
                            "payment_deadline_at": (_utc_now() + timedelta(hours=2)).isoformat(),
                            "seat_count": 1,
                            "tickets": [{"car_no": "2", "seat_no": "4A"}],
                        }
                    ],
                    "http_trace": {"endpoint": "get_reservations", "status_code": 200},
                },
            )

        async def ticket_info(self, **kwargs):
            return ProviderOutcome(
                ok=True,
                data={
                    "reservation_id": "rsv-relogin-1",
                    "tickets": [{"car_no": "2", "seat_no": "4A"}],
                    "http_trace": {"endpoint": "ticket_info", "status_code": 200},
                },
            )

        async def cancel(self, **kwargs):
            return ProviderOutcome(ok=True, data={"cancelled": True})

    provider = FakeProvider()

    monkeypatch.setattr("app.modules.train.worker.enqueue_train_task", _noop_enqueue)
    monkeypatch.setattr("app.modules.train.worker.RedisTokenBucketLimiter.acquire_provider_call", _no_limit)
    monkeypatch.setattr("app.modules.train.worker._load_provider_credentials", _fake_credentials)
    monkeypatch.setattr("app.modules.train.worker.get_provider_client", lambda _provider: provider)

    user = User(
        email="reserve-relogin@example.com",
        password_hash="x",
        display_name="Reserve Relogin User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    task = Task(
        user_id=user.id,
        module="train",
        state="QUEUED",
        deadline_at=_utc_now() + timedelta(hours=1),
        spec_json={
            "provider": "SRT",
            "providers": ["SRT"],
            "dep": "수서",
            "arr": "마산",
            "date": _utc_now().date().isoformat(),
            "selected_trains_ranked": [
                {
                    "schedule_id": "srt-relogin-schedule",
                    "departure_at": (_utc_now() + timedelta(hours=2)).isoformat(),
                    "rank": 1,
                    "provider": "SRT",
                }
            ],
            "passengers": {"adults": 1, "children": 0},
            "seat_class": "general",
            "auto_pay": True,
        },
        idempotency_key="reserve-relogin",
    )
    db_session.add(task)
    await db_session.commit()

    await run_train_task({"db_factory": db_session_factory, "redis": fake_redis}, str(task.id))

    async with db_session_factory() as verify_session:
        refreshed = await verify_session.get(Task, task.id)
        assert refreshed is not None
        assert refreshed.state == "COMPLETED"

        reserve_attempt = (
            await verify_session.execute(
                select(TaskAttempt)
                .where(TaskAttempt.task_id == task.id)
                .where(TaskAttempt.action == "RESERVE")
                .order_by(TaskAttempt.started_at.desc())
                .limit(1)
            )
        ).scalar_one()

    assert reserve_attempt.ok is True
    assert reserve_attempt.meta_json_safe is not None
    assert reserve_attempt.meta_json_safe.get("auth_relogin_retry") is True
    assert provider.login_calls >= 2
    assert provider.reserve_calls == 2


@pytest.mark.asyncio
async def test_worker_relogs_and_retries_pay_on_provider_auth_error(db_session_factory, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()

    class _NoLimitResult:
        waited_ms = 0
        rounds = 1

    async def _no_limit(self, **kwargs):
        return _NoLimitResult()

    async def _noop_enqueue(task_id: str, defer_seconds: float = 0.0) -> None:
        return None

    async def _fake_credentials(db, *, user_id, provider):
        return {"username": f"{provider.lower()}-user", "password": "secret"}

    class FakeProvider:
        provider_name = "SRT"

        def __init__(self) -> None:
            self.login_calls = 0
            self.pay_calls = 0

        async def login(self, **kwargs):
            self.login_calls += 1
            return ProviderOutcome(ok=True)

        async def pay(self, **kwargs):
            self.pay_calls += 1
            if self.pay_calls == 1:
                return ProviderOutcome(
                    ok=False,
                    retryable=False,
                    error_code="srt_pay_fail_auth",
                    error_message_safe="로그인 후 사용하십시요.",
                )
            return ProviderOutcome(
                ok=True,
                data={
                    "payment_id": "pay-relogin-2",
                    "ticket_no": "ticket-relogin-2",
                    "http_trace": {"endpoint": "payment", "status_code": 200},
                },
            )

        async def get_reservations(self, **kwargs):
            return ProviderOutcome(
                ok=True,
                data={
                    "reservations": [
                        {
                            "reservation_id": "rsv-pay-relogin-1",
                            "provider": "SRT",
                            "paid": True,
                            "waiting": False,
                            "payment_deadline_at": (_utc_now() + timedelta(hours=2)).isoformat(),
                            "seat_count": 1,
                            "tickets": [{"car_no": "2", "seat_no": "4A"}],
                        }
                    ],
                    "http_trace": {"endpoint": "get_reservations", "status_code": 200},
                },
            )

        async def ticket_info(self, **kwargs):
            return ProviderOutcome(
                ok=True,
                data={
                    "reservation_id": "rsv-pay-relogin-1",
                    "tickets": [{"car_no": "2", "seat_no": "4A"}],
                    "http_trace": {"endpoint": "ticket_info", "status_code": 200},
                },
            )

        async def cancel(self, **kwargs):
            return ProviderOutcome(ok=True, data={"cancelled": True})

    provider = FakeProvider()

    monkeypatch.setattr("app.modules.train.worker.enqueue_train_task", _noop_enqueue)
    monkeypatch.setattr("app.modules.train.worker.RedisTokenBucketLimiter.acquire_provider_call", _no_limit)
    monkeypatch.setattr("app.modules.train.worker._load_provider_credentials", _fake_credentials)
    monkeypatch.setattr("app.modules.train.worker.get_provider_client", lambda _provider: provider)

    user = User(
        email="pay-relogin@example.com",
        password_hash="x",
        display_name="Pay Relogin User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    task = Task(
        user_id=user.id,
        module="train",
        state="QUEUED",
        deadline_at=_utc_now() + timedelta(hours=1),
        spec_json={
            "provider": "SRT",
            "providers": ["SRT"],
            "dep": "수서",
            "arr": "마산",
            "date": _utc_now().date().isoformat(),
            "selected_trains_ranked": [
                {
                    "schedule_id": "srt-pay-relogin-schedule",
                    "departure_at": (_utc_now() + timedelta(hours=2)).isoformat(),
                    "rank": 1,
                    "provider": "SRT",
                }
            ],
            "passengers": {"adults": 1, "children": 0},
            "seat_class": "general",
            "auto_pay": True,
        },
        idempotency_key="pay-relogin",
    )
    db_session.add(task)
    await db_session.flush()

    db_session.add(
        Artifact(
            task_id=task.id,
            module="train",
            kind="ticket",
            data_json_safe={
                "provider": "SRT",
                "reservation_id": "rsv-pay-relogin-1",
                "paid": False,
                "status": "reserved",
                "train_no": "381",
                "schedule_id": "srt-pay-relogin-schedule",
                "departure_at": (_utc_now() + timedelta(hours=2)).isoformat(),
                "arrival_at": (_utc_now() + timedelta(hours=4)).isoformat(),
                "provider_http": {"reserve": {"endpoint": "reserve", "status_code": 200}},
            },
        )
    )
    await db_session.commit()

    await run_train_task({"db_factory": db_session_factory, "redis": fake_redis}, str(task.id))

    async with db_session_factory() as verify_session:
        refreshed = await verify_session.get(Task, task.id)
        assert refreshed is not None
        assert refreshed.state == "COMPLETED"

        pay_attempt = (
            await verify_session.execute(
                select(TaskAttempt)
                .where(TaskAttempt.task_id == task.id)
                .where(TaskAttempt.action == "PAY")
                .order_by(TaskAttempt.started_at.desc())
                .limit(1)
            )
        ).scalar_one()
        artifact = (
            await verify_session.execute(
                select(Artifact)
                .where(Artifact.task_id == task.id)
                .where(Artifact.kind == "ticket")
                .limit(1)
            )
        ).scalar_one()

    assert pay_attempt.ok is True
    assert pay_attempt.meta_json_safe is not None
    assert pay_attempt.meta_json_safe.get("auth_relogin_retry") is True
    assert provider.login_calls >= 2
    assert provider.pay_calls == 2
    assert artifact.data_json_safe.get("paid") is True
    assert artifact.data_json_safe.get("payment_id") == "pay-relogin-2"


@pytest.mark.asyncio
async def test_task_detail_refreshes_ticket_artifact_status_from_provider(db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()

    class _NoLimitResult:
        waited_ms = 0
        rounds = 1

    async def _no_limit(self, **kwargs):
        return _NoLimitResult()

    class FakeClient:
        async def get_reservations(self, **kwargs):
            return ProviderOutcome(
                ok=True,
                data={
                    "reservations": [
                        {
                            "reservation_id": "PNR-9001",
                            "provider": "SRT",
                            "paid": False,
                            "waiting": False,
                            "payment_deadline_at": (_utc_now() + timedelta(hours=2)).isoformat(),
                            "seat_count": 1,
                            "tickets": [{"car_no": "3", "seat_no": "7A"}],
                        }
                    ],
                    "http_trace": {"endpoint": "get_reservations", "status_code": 200},
                },
            )

        async def ticket_info(self, **kwargs):
            return ProviderOutcome(
                ok=True,
                data={
                    "reservation_id": "PNR-9001",
                    "tickets": [{"car_no": "3", "seat_no": "7A"}],
                    "http_trace": {"endpoint": "ticket_info", "status_code": 200},
                },
            )

    async def _fake_client(db, *, user, provider):
        return FakeClient()

    monkeypatch.setattr("app.modules.train.service.get_redis_client", make_fake_get_redis_client(fake_redis))
    monkeypatch.setattr("app.modules.train.service.RedisTokenBucketLimiter.acquire_provider_call", _no_limit)
    monkeypatch.setattr("app.modules.train.service._get_logged_in_provider_client", _fake_client)

    user = User(
        email="detail-refresh@example.com",
        password_hash="x",
        display_name="Detail Refresh User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    deadline = _utc_now() + timedelta(hours=4)
    task = Task(
        user_id=user.id,
        module="train",
        state="COMPLETED",
        deadline_at=deadline,
        spec_json={"module": "train"},
        idempotency_key="detail-refresh",
    )
    db_session.add(task)
    await db_session.flush()

    artifact = Artifact(
        task_id=task.id,
        module="train",
        kind="ticket",
        data_json_safe={
            "provider": "SRT",
            "reservation_id": "PNR-9001",
            "paid": False,
            "status": "reserved",
        },
    )
    db_session.add(artifact)
    await db_session.commit()

    detail = await get_task_detail(db_session, task_id=task.id, user=user)
    assert detail.artifacts
    ticket = detail.artifacts[0].data_json_safe
    assert ticket.get("status") == "awaiting_payment"
    assert ticket.get("seat_count") == 1
    assert ticket.get("tickets", [{}])[0].get("seat_no") == "7A"
    assert ticket.get("provider_http", {}).get("get_reservations", {}).get("endpoint") == "get_reservations"


@pytest.mark.asyncio
async def test_list_tasks_refreshes_completed_ticket_status_on_page_load(client, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()

    class _NoLimitResult:
        waited_ms = 0
        rounds = 1

    async def _no_limit(self, **kwargs):
        return _NoLimitResult()

    class FakeClient:
        async def get_reservations(self, **kwargs):
            return ProviderOutcome(
                ok=True,
                data={
                    "reservations": [
                        {
                            "reservation_id": "PNR-LIST-1",
                            "provider": "SRT",
                            "paid": False,
                            "waiting": False,
                            "payment_deadline_at": (_utc_now() + timedelta(hours=3)).isoformat(),
                            "seat_count": 1,
                            "tickets": [{"car_no": "4", "seat_no": "8A"}],
                        }
                    ],
                    "http_trace": {"endpoint": "get_reservations", "status_code": 200},
                },
            )

        async def ticket_info(self, **kwargs):
            return ProviderOutcome(
                ok=True,
                data={
                    "reservation_id": "PNR-LIST-1",
                    "tickets": [{"car_no": "4", "seat_no": "8A"}],
                    "http_trace": {"endpoint": "ticket_info", "status_code": 200},
                },
            )

    async def _fake_client(db, *, user, provider):
        return FakeClient()

    monkeypatch.setattr("app.modules.train.service.get_redis_client", make_fake_get_redis_client(fake_redis))
    monkeypatch.setattr("app.modules.train.service.RedisTokenBucketLimiter.acquire_provider_call", _no_limit)
    monkeypatch.setattr("app.modules.train.service._get_logged_in_provider_client", _fake_client)

    email = "list-refresh@example.com"
    cookie = await _register_and_login(client, email=email)

    user = (await db_session.execute(select(User).where(User.email == email))).scalar_one()
    task = Task(
        user_id=user.id,
        module="train",
        state="COMPLETED",
        deadline_at=_utc_now() + timedelta(hours=2),
        spec_json={"module": "train"},
        idempotency_key="list-refresh-task",
        completed_at=_utc_now(),
    )
    db_session.add(task)
    await db_session.flush()

    db_session.add(
        Artifact(
            task_id=task.id,
            module="train",
            kind="ticket",
            data_json_safe={
                "provider": "SRT",
                "reservation_id": "PNR-LIST-1",
                "paid": False,
                "status": "reserved",
            },
        )
    )
    await db_session.commit()

    response = await client.get(
        "/api/train/tasks?status=completed&refresh_completed=true",
        cookies={"bominal_session": cookie},
    )
    assert response.status_code == 200
    rows = response.json()["tasks"]
    task_row = next(row for row in rows if row["id"] == str(task.id))
    assert task_row["ticket_status"] == "awaiting_payment"
    assert task_row["ticket_paid"] is False
    assert task_row["ticket_reservation_id"] == "PNR-LIST-1"


@pytest.mark.asyncio
async def test_payment_idempotency_does_not_double_pay(db_session_factory, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()

    class FakeProvider:
        provider_name = "SRT"

        def __init__(self):
            self.pay_calls = 0

        async def search(self, **kwargs):
            dep = kwargs["dep"]
            arr = kwargs["arr"]
            departure = _utc_now() + timedelta(minutes=15)
            schedule = ProviderSchedule(
                schedule_id="schedule-1",
                provider="SRT",
                dep=dep,
                arr=arr,
                departure_at=departure,
                arrival_at=departure + timedelta(minutes=70),
                train_no="S200",
                availability={"general": True, "special": True},
            )
            return ProviderOutcome(ok=True, data={"schedules": [schedule]})

        async def reserve(self, **kwargs):
            return ProviderOutcome(ok=True, data={"reservation_id": "rsv-123"})

        async def pay(self, **kwargs):
            self.pay_calls += 1
            return ProviderOutcome(ok=True, data={"payment_id": "pay-123", "ticket_no": "T-123"})

        async def cancel(self, **kwargs):
            return ProviderOutcome(ok=False, retryable=False, error_code="not_supported", error_message_safe="not supported")

    fake_provider = FakeProvider()

    async def _noop_enqueue(task_id: str, defer_seconds: float = 0.0) -> None:
        return None

    monkeypatch.setattr("app.modules.train.worker.enqueue_train_task", _noop_enqueue)
    monkeypatch.setattr("app.modules.train.worker.get_provider_client", lambda provider: fake_provider)

    user = User(
        email="pay@example.com",
        password_hash="x",
        display_name="Pay User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    deadline = _utc_now() + timedelta(minutes=20)
    task = Task(
        user_id=user.id,
        module="train",
        state="QUEUED",
        deadline_at=deadline,
        spec_json={
            "provider": "SRT",
            "dep": "수서",
            "arr": "부산",
            "date": _utc_now().date().isoformat(),
            "selected_trains_ranked": [{"schedule_id": "schedule-1", "departure_at": deadline.isoformat(), "rank": 1}],
            "passengers": {"adults": 1, "children": 0},
            "seat_class": "general",
            "auto_pay": True,
        },
        idempotency_key="pay-idempotency",
    )
    db_session.add(task)
    await db_session.flush()

    db_session.add(
        TaskAttempt(
            task_id=task.id,
            action="SEARCH",
            provider="SRT",
            ok=False,
            retryable=True,
            error_code="seat_unavailable",
            error_message_safe="seed",
            duration_ms=100,
            meta_json_safe={},
            started_at=_utc_now(),
            finished_at=_utc_now(),
        )
    )

    db_session.add(
        Artifact(
            task_id=task.id,
            module="train",
            kind="ticket",
            data_json_safe={
                "provider": "SRT",
                "reservation_id": "rsv-previous",
                "payment_id": "pay-previous",
                "ticket_no": "T-OLD",
            },
        )
    )
    await db_session.commit()

    await run_train_task({"db_factory": db_session_factory, "redis": fake_redis}, str(task.id))

    async with db_session_factory() as verify_session:
        refreshed = await verify_session.get(Task, task.id)
        assert refreshed is not None
        assert refreshed.state == "COMPLETED"
    assert fake_provider.pay_calls == 0


@pytest.mark.asyncio
async def test_manual_pay_rejects_expired_payment_window(client, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    monkeypatch.setattr("app.modules.train.service.get_redis_client", make_fake_get_redis_client(fake_redis))

    cookie = await _register_and_login(client, email="manual-pay-expired@example.com")
    user = (await db_session.execute(select(User).where(User.email == "manual-pay-expired@example.com"))).scalar_one()

    task = Task(
        user_id=user.id,
        module="train",
        state="COMPLETED",
        deadline_at=_utc_now() + timedelta(hours=1),
        spec_json={"module": "train"},
        idempotency_key="manual-pay-expired",
    )
    db_session.add(task)
    await db_session.flush()

    db_session.add(
        Artifact(
            task_id=task.id,
            module="train",
            kind="ticket",
            data_json_safe={
                "provider": "SRT",
                "reservation_id": "PNR-EXPIRED-1",
                "status": "awaiting_payment",
                "paid": False,
                "payment_deadline_at": (_utc_now() - timedelta(minutes=1)).isoformat(),
            },
        )
    )
    await db_session.commit()

    response = await client.post(
        f"/api/train/tasks/{task.id}/pay",
        cookies={"bominal_session": cookie},
    )
    assert response.status_code == 409
    assert "payment window has expired" in response.json()["detail"]


@pytest.mark.asyncio
async def test_manual_pay_marks_ticket_paid(client, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    monkeypatch.setattr("app.modules.train.service.get_redis_client", make_fake_get_redis_client(fake_redis))
    class _NoLimitResult:
        waited_ms = 0
        rounds = 1

    async def _no_limit(self, **kwargs):
        return _NoLimitResult()

    monkeypatch.setattr("app.modules.train.service.RedisTokenBucketLimiter.acquire_provider_call", _no_limit)

    async def _fake_payment_card(_db, *, user_id):
        return {
            "card_number": "1234567890123456",
            "card_password": "12",
            "validation_number": "900101",
            "card_expire": "2911",
            "card_type": "J",
            "installment": 0,
        }

    monkeypatch.setattr("app.modules.train.service.get_payment_card_for_execution", _fake_payment_card)

    class FakeProviderClient:
        def __init__(self) -> None:
            self.paid = False

        async def pay(self, **kwargs):
            self.paid = True
            return ProviderOutcome(
                ok=True,
                data={
                    "payment_id": "manual-pay-1",
                    "ticket_no": "T-MANUAL-1",
                    "http_trace": {"endpoint": "payment", "status_code": 200},
                },
            )

        async def get_reservations(self, **kwargs):
            return ProviderOutcome(
                ok=True,
                data={
                    "reservations": [
                        {
                            "reservation_id": "PNR-MANUAL-1",
                            "provider": "SRT",
                            "paid": self.paid,
                            "waiting": False,
                            "payment_deadline_at": (_utc_now() + timedelta(hours=2)).isoformat(),
                            "seat_count": 1,
                            "tickets": [{"car_no": "3", "seat_no": "7A"}],
                        }
                    ],
                    "http_trace": {"endpoint": "get_reservations", "status_code": 200},
                },
            )

        async def ticket_info(self, **kwargs):
            return ProviderOutcome(
                ok=True,
                data={
                    "reservation_id": "PNR-MANUAL-1",
                    "tickets": [{"car_no": "3", "seat_no": "7A"}],
                    "http_trace": {"endpoint": "ticket_info", "status_code": 200},
                },
            )

    fake_client = FakeProviderClient()

    async def _fake_logged_in_provider_client(db, *, user, provider):
        return fake_client

    monkeypatch.setattr("app.modules.train.service._get_logged_in_provider_client", _fake_logged_in_provider_client)

    cookie = await _register_and_login(client, email="manual-pay-success@example.com")
    user = (await db_session.execute(select(User).where(User.email == "manual-pay-success@example.com"))).scalar_one()

    task = Task(
        user_id=user.id,
        module="train",
        state="COMPLETED",
        deadline_at=_utc_now() + timedelta(hours=1),
        spec_json={"module": "train"},
        idempotency_key="manual-pay-success",
    )
    db_session.add(task)
    await db_session.flush()

    db_session.add(
        Artifact(
            task_id=task.id,
            module="train",
            kind="ticket",
            data_json_safe={
                "provider": "SRT",
                "reservation_id": "PNR-MANUAL-1",
                "status": "awaiting_payment",
                "paid": False,
                "payment_deadline_at": (_utc_now() + timedelta(minutes=30)).isoformat(),
            },
        )
    )
    await db_session.commit()

    response = await client.post(
        f"/api/train/tasks/{task.id}/pay",
        cookies={"bominal_session": cookie},
    )
    assert response.status_code == 200
    assert response.json()["task"]["ticket_paid"] is True
    assert response.json()["task"]["ticket_status"] == "paid"

    pay_attempt = (
        await db_session.execute(
            select(TaskAttempt)
            .where(TaskAttempt.task_id == task.id)
            .where(TaskAttempt.action == "PAY")
            .order_by(TaskAttempt.started_at.desc())
            .limit(1)
        )
    ).scalar_one()
    assert pay_attempt.ok is True
    assert pay_attempt.meta_json_safe is not None
    assert pay_attempt.meta_json_safe.get("manual_trigger") is True

    artifact = (
        await db_session.execute(
            select(Artifact)
            .where(Artifact.task_id == task.id)
            .where(Artifact.kind == "ticket")
            .limit(1)
        )
    ).scalar_one()
    assert artifact.data_json_safe.get("paid") is True
    assert artifact.data_json_safe.get("status") == "paid"


@pytest.mark.asyncio
async def test_manual_cancel_records_cancel_attempt(client, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    monkeypatch.setattr("app.modules.train.service.get_redis_client", make_fake_get_redis_client(fake_redis))

    class _NoLimitResult:
        waited_ms = 0
        rounds = 1

    async def _no_limit(self, **kwargs):
        return _NoLimitResult()

    monkeypatch.setattr("app.modules.train.service.RedisTokenBucketLimiter.acquire_provider_call", _no_limit)

    class FakeProviderClient:
        async def cancel(self, **kwargs):
            reservation_id = str(kwargs.get("artifact_data", {}).get("reservation_id") or "PNR-CANCEL-1")
            return ProviderOutcome(
                ok=True,
                data={
                    "cancelled": True,
                    "http_trace": {"endpoint": "cancel", "reservation_id": reservation_id},
                },
            )

        async def get_reservations(self, **kwargs):
            reservation_id = str(kwargs.get("reservation_id") or "PNR-CANCEL-1")
            return ProviderOutcome(
                ok=True,
                data={
                    "reservations": [
                        {
                            "reservation_id": reservation_id,
                            "provider": "SRT",
                            "paid": False,
                            "waiting": False,
                            "payment_deadline_at": (_utc_now() + timedelta(hours=2)).isoformat(),
                            "seat_count": 1,
                            "tickets": [{"car_no": "3", "seat_no": "7A"}],
                        }
                    ],
                },
            )

        async def ticket_info(self, **kwargs):
            reservation_id = str(kwargs.get("reservation_id") or "PNR-CANCEL-1")
            return ProviderOutcome(
                ok=True,
                data={
                    "reservation_id": reservation_id,
                    "tickets": [{"car_no": "3", "seat_no": "7A"}],
                },
            )

    fake_client = FakeProviderClient()

    async def _fake_logged_in_provider_client(db, *, user, provider):
        return fake_client

    monkeypatch.setattr("app.modules.train.service._get_logged_in_provider_client", _fake_logged_in_provider_client)

    cookie = await _register_and_login(client, email="manual-cancel-attempt@example.com")
    user = (await db_session.execute(select(User).where(User.email == "manual-cancel-attempt@example.com"))).scalar_one()

    task = Task(
        user_id=user.id,
        module="train",
        state="COMPLETED",
        deadline_at=_utc_now() + timedelta(hours=1),
        spec_json={"module": "train"},
        idempotency_key="manual-cancel-attempt",
    )
    db_session.add(task)
    await db_session.flush()

    artifact = Artifact(
        task_id=task.id,
        module="train",
        kind="ticket",
        data_json_safe={
            "provider": "SRT",
            "reservation_id": "PNR-CANCEL-1",
            "status": "awaiting_payment",
            "paid": False,
            "payment_deadline_at": (_utc_now() + timedelta(minutes=30)).isoformat(),
        },
    )
    db_session.add(artifact)
    await db_session.commit()

    cancel_response = await client.post(
        f"/api/train/tickets/{artifact.id}/cancel",
        cookies={"bominal_session": cookie},
    )
    assert cancel_response.status_code == 200
    assert cancel_response.json()["status"] == "cancelled"

    detail_response = await client.get(
        f"/api/train/tasks/{task.id}",
        cookies={"bominal_session": cookie},
    )
    assert detail_response.status_code == 200
    attempts = detail_response.json()["attempts"]
    assert any(attempt["action"] == "CANCEL" and attempt["provider"] == "SRT" and attempt["ok"] for attempt in attempts)

    cancel_attempt = (
        await db_session.execute(
            select(TaskAttempt)
            .where(TaskAttempt.task_id == task.id)
            .where(TaskAttempt.action == "CANCEL")
            .order_by(TaskAttempt.started_at.desc())
            .limit(1)
        )
    ).scalar_one()
    assert cancel_attempt.meta_json_safe is not None
    assert cancel_attempt.meta_json_safe.get("manual_trigger") is True


@pytest.mark.asyncio
@pytest.mark.parametrize(
    ("seat_class", "availability", "expected_reserved"),
    [
        ("general_preferred", {"general": False, "special": True}, "special"),
        ("special_preferred", {"general": True, "special": False}, "general"),
    ],
)
async def test_seat_preference_fallback_reserves_available_class(
    db_session_factory,
    db_session,
    monkeypatch,
    seat_class,
    availability,
    expected_reserved,
):
    fake_redis = fakeredis.aioredis.FakeRedis()
    utc_now_naive = lambda: datetime.now(timezone.utc).replace(tzinfo=None)

    class FakeProvider:
        provider_name = "SRT"

        def __init__(self):
            self.reserve_calls: list[str] = []

        async def login(self, **kwargs):
            return ProviderOutcome(ok=True)

        async def search(self, **kwargs):
            dep = kwargs["dep"]
            arr = kwargs["arr"]
            departure = _utc_now() + timedelta(minutes=20)
            schedule = ProviderSchedule(
                schedule_id="schedule-pref",
                provider="SRT",
                dep=dep,
                arr=arr,
                departure_at=departure,
                arrival_at=departure + timedelta(minutes=65),
                train_no="S210",
                availability=availability,
            )
            return ProviderOutcome(ok=True, data={"schedules": [schedule]})

        async def reserve(self, **kwargs):
            self.reserve_calls.append(kwargs["seat_class"])
            return ProviderOutcome(ok=True, data={"reservation_id": "rsv-pref"})

        async def pay(self, **kwargs):
            return ProviderOutcome(ok=True, data={"payment_id": "pay-pref", "ticket_no": "T-PREF"})

        async def cancel(self, **kwargs):
            return ProviderOutcome(ok=False, retryable=False, error_code="not_supported", error_message_safe="not supported")

    fake_provider = FakeProvider()

    async def _noop_enqueue(task_id: str, defer_seconds: float = 0.0) -> None:
        return None

    async def _fake_credentials(db, *, user_id, provider):
        return {"username": "mock-user", "password": "mock-password"}

    class _NoLimitResult:
        waited_ms = 0
        rounds = 1

    async def _acquire_without_wait(self, **kwargs):
        return _NoLimitResult()

    monkeypatch.setattr("app.modules.train.worker.enqueue_train_task", _noop_enqueue)
    monkeypatch.setattr("app.modules.train.worker.get_provider_client", lambda provider: fake_provider)
    monkeypatch.setattr("app.modules.train.worker._load_provider_credentials", _fake_credentials)
    monkeypatch.setattr("app.core.time.utc_now", utc_now_naive)
    monkeypatch.setattr("app.modules.train.worker.RedisTokenBucketLimiter.acquire_provider_call", _acquire_without_wait)

    user = User(
        email=f"seat-pref-{seat_class}@example.com",
        password_hash="x",
        display_name="Seat Preference User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    deadline = utc_now_naive() + timedelta(minutes=30)
    task = Task(
        user_id=user.id,
        module="train",
        state="QUEUED",
        deadline_at=deadline,
        spec_json={
            "provider": "SRT",
            "dep": "수서",
            "arr": "부산",
            "date": utc_now_naive().date().isoformat(),
            "selected_trains_ranked": [{"schedule_id": "schedule-pref", "departure_at": deadline.isoformat(), "rank": 1}],
            "passengers": {"adults": 1, "children": 0},
            "seat_class": seat_class,
            "auto_pay": False,
        },
        idempotency_key=f"seat-pref-{seat_class}",
    )
    db_session.add(task)
    await db_session.flush()

    # Skip the worker's first-cycle visual delay so we can validate reserve behavior in one run.
    db_session.add(
        TaskAttempt(
            task_id=task.id,
            action="SEARCH",
            provider="SRT",
            ok=False,
            retryable=True,
            error_code="seed",
            error_message_safe="seed",
            duration_ms=1,
            meta_json_safe={},
            started_at=utc_now_naive(),
            finished_at=utc_now_naive(),
        )
    )
    await db_session.commit()

    await run_train_task({"db_factory": db_session_factory, "redis": fake_redis}, str(task.id))

    async with db_session_factory() as verify_session:
        refreshed = await verify_session.get(Task, task.id)
        assert refreshed is not None
        assert refreshed.state == "COMPLETED"

        artifacts = (
            await verify_session.execute(
                select(Artifact).where(Artifact.task_id == task.id).where(Artifact.kind == "ticket")
            )
        ).scalars().all()

    assert fake_provider.reserve_calls == [expected_reserved]
    assert len(artifacts) == 1
    assert artifacts[0].data_json_safe.get("seat_class_requested") == seat_class
    assert artifacts[0].data_json_safe.get("seat_class_reserved") == expected_reserved


@pytest.mark.asyncio
async def test_retry_task_now_allows_queued(db_session, monkeypatch):
    calls: list[str] = []

    async def _record_enqueue(task_id: str, defer_seconds: float = 0.0) -> None:
        calls.append(task_id)

    monkeypatch.setattr("app.modules.train.service.enqueue_train_task", _record_enqueue)

    user = User(
        email="retry-queued@example.com",
        password_hash="x",
        display_name="Retry User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    task = Task(
        user_id=user.id,
        module="train",
        state="QUEUED",
        deadline_at=_utc_now() + timedelta(minutes=10),
        spec_json={"provider": "SRT"},
        idempotency_key="retry-queued",
    )
    db_session.add(task)
    await db_session.commit()

    response = await retry_task_now(db_session, task_id=task.id, user=user)

    assert response.task.id == task.id
    assert response.task.state == "QUEUED"
    assert calls == [str(task.id)]

    await db_session.refresh(task)
    assert task.spec_json.get("manual_retry_last_at")


@pytest.mark.asyncio
async def test_retry_task_now_allows_polling_and_clears_next_run_at(db_session, monkeypatch):
    async def _noop_enqueue(task_id: str, defer_seconds: float = 0.0) -> None:
        return None

    monkeypatch.setattr("app.modules.train.service.enqueue_train_task", _noop_enqueue)

    user = User(
        email="retry-polling@example.com",
        password_hash="x",
        display_name="Retry Polling User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    task = Task(
        user_id=user.id,
        module="train",
        state="POLLING",
        deadline_at=_utc_now() + timedelta(minutes=10),
        spec_json={"provider": "SRT", "next_run_at": (_utc_now() + timedelta(minutes=5)).isoformat()},
        idempotency_key="retry-polling",
    )
    db_session.add(task)
    await db_session.commit()

    response = await retry_task_now(db_session, task_id=task.id, user=user)

    assert response.task.state == "QUEUED"

    await db_session.refresh(task)
    assert task.state == "QUEUED"
    assert "next_run_at" not in task.spec_json


@pytest.mark.asyncio
async def test_retry_task_now_rejects_paused(db_session, monkeypatch):
    async def _noop_enqueue(task_id: str, defer_seconds: float = 0.0) -> None:
        return None

    monkeypatch.setattr("app.modules.train.service.enqueue_train_task", _noop_enqueue)

    user = User(
        email="retry-paused@example.com",
        password_hash="x",
        display_name="Retry Paused User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    task = Task(
        user_id=user.id,
        module="train",
        state="PAUSED",
        paused_at=_utc_now(),
        deadline_at=_utc_now() + timedelta(minutes=10),
        spec_json={"provider": "SRT"},
        idempotency_key="retry-paused",
    )
    db_session.add(task)
    await db_session.commit()

    with pytest.raises(HTTPException) as excinfo:
        await retry_task_now(db_session, task_id=task.id, user=user)

    assert excinfo.value.status_code == 409


@pytest.mark.asyncio
async def test_retry_task_now_rejects_processing_states(db_session, monkeypatch):
    async def _noop_enqueue(task_id: str, defer_seconds: float = 0.0) -> None:
        return None

    monkeypatch.setattr("app.modules.train.service.enqueue_train_task", _noop_enqueue)

    user = User(
        email="retry-processing@example.com",
        password_hash="x",
        display_name="Retry Processing User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    for state in ("RUNNING", "RESERVING", "PAYING"):
        task = Task(
            user_id=user.id,
            module="train",
            state=state,
            deadline_at=_utc_now() + timedelta(minutes=10),
            spec_json={"provider": "SRT"},
            idempotency_key=f"retry-processing-{state}",
        )
        db_session.add(task)

    await db_session.commit()

    tasks = (await db_session.execute(select(Task).where(Task.user_id == user.id))).scalars().all()
    processing = [task for task in tasks if task.idempotency_key.startswith("retry-processing-")]
    assert len(processing) == 3

    for task in processing:
        with pytest.raises(HTTPException) as excinfo:
            await retry_task_now(db_session, task_id=task.id, user=user)
        assert excinfo.value.status_code == 409


@pytest.mark.asyncio
async def test_retry_task_now_marks_expired_when_deadline_passed(db_session, monkeypatch):
    async def _noop_enqueue(task_id: str, defer_seconds: float = 0.0) -> None:
        return None

    monkeypatch.setattr("app.modules.train.service.enqueue_train_task", _noop_enqueue)

    user = User(
        email="retry-expired@example.com",
        password_hash="x",
        display_name="Retry Expired User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    task = Task(
        user_id=user.id,
        module="train",
        state="QUEUED",
        deadline_at=_utc_now() - timedelta(seconds=1),
        spec_json={"provider": "SRT"},
        idempotency_key="retry-expired",
    )
    db_session.add(task)
    await db_session.commit()

    with pytest.raises(HTTPException) as excinfo:
        await retry_task_now(db_session, task_id=task.id, user=user)

    assert excinfo.value.status_code == 410

    await db_session.refresh(task)
    assert task.state == "EXPIRED"


@pytest.mark.asyncio
async def test_retry_task_now_enforces_cooldown(db_session, monkeypatch):
    calls: list[str] = []

    async def _record_enqueue(task_id: str, defer_seconds: float = 0.0) -> None:
        calls.append(task_id)

    monkeypatch.setattr("app.modules.train.service.enqueue_train_task", _record_enqueue)

    user = User(
        email="retry-cooldown@example.com",
        password_hash="x",
        display_name="Retry Cooldown User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    task = Task(
        user_id=user.id,
        module="train",
        state="POLLING",
        deadline_at=_utc_now() + timedelta(minutes=10),
        spec_json={"provider": "SRT"},
        idempotency_key="retry-cooldown",
    )
    db_session.add(task)
    await db_session.commit()

    first = await retry_task_now(db_session, task_id=task.id, user=user)
    assert first.task.state == "QUEUED"
    assert calls == [str(task.id)]

    with pytest.raises(HTTPException) as excinfo:
        await retry_task_now(db_session, task_id=task.id, user=user)

    assert excinfo.value.status_code == 429
    assert calls == [str(task.id)]


def test_parse_srt_search_response_from_srtgo_shape():
    raw = {
        "outDataSets": {
            "dsOutput0": [{"strResult": "SUCC", "msgTxt": "ok"}],
            "dsOutput1": [
                {
                    "stlbTrnClsfCd": "17",
                    "trnNo": "301",
                    "dptDt": "20260203",
                    "dptTm": "103000",
                    "arvDt": "20260203",
                    "arvTm": "123500",
                    "gnrmRsvPsbStr": "예약가능",
                    "sprmRsvPsbStr": "매진",
                    "rsvWaitPsbCd": "9",
                    "rsvWaitPsbCdNm": "가능",
                }
            ],
        }
    }

    outcome = parse_srt_search_response(json.dumps(raw), dep="수서", arr="부산")
    assert outcome.ok is True
    schedules = outcome.data["schedules"]
    assert len(schedules) == 1
    assert schedules[0].provider == "SRT"
    assert schedules[0].availability["general"] is True
    assert schedules[0].departure_at.utcoffset() == timedelta(hours=9)


def test_parse_ktx_search_response_from_srtgo_shape():
    raw = {
        "strResult": "SUCC",
        "trn_infos": {
            "trn_info": [
                {
                    "h_trn_no": "123",
                    "h_dpt_dt": "20260203",
                    "h_dpt_tm": "080000",
                    "h_arv_dt": "20260203",
                    "h_arv_tm": "101500",
                    "h_gen_rsv_cd": "11",
                    "h_spe_rsv_cd": "00",
                    "h_rsv_psb_nm": "예약가능",
                    "h_wait_rsv_flg": "9",
                    "h_trn_clsf_nm": "KTX",
                }
            ]
        },
    }

    outcome = parse_ktx_search_response(json.dumps(raw), dep="수서", arr="부산")
    assert outcome.ok is True
    schedules = outcome.data["schedules"]
    assert len(schedules) == 1
    assert schedules[0].provider == "KTX"
    assert schedules[0].availability["general"] is True
    assert schedules[0].departure_at.utcoffset() == timedelta(hours=9)
