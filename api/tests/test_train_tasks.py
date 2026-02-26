from __future__ import annotations

import asyncio
import json
import random
from datetime import datetime, timedelta, timezone
from uuid import UUID

import fakeredis.aioredis
import pytest
from fastapi import HTTPException
from pydantic import ValidationError
from sqlalchemy import select

import app.modules.train.worker as train_worker
from app.db.models import Artifact, Task, TaskAttempt, User
from app.modules.train.providers.base import ProviderOutcome, ProviderSchedule
from app.modules.train.providers.ktx_client import parse_ktx_search_response
from app.modules.train.providers.srt_client import parse_srt_search_response
from app.modules.train.rate_limiter import RedisTokenBucketLimiter
from app.modules.train.schemas import TrainPassengers
from app.modules.train.service import get_task_detail, list_provider_reservations, list_tasks, retry_task_now
from app.modules.train.schemas import ProviderCredentialStatus
from app.modules.train.worker import run_train_task
from tests.conftest import make_fake_get_redis_client


@pytest.fixture(autouse=True)
def _enable_payment_for_train_task_tests(monkeypatch):
    monkeypatch.setattr(train_worker.settings, "payment_enabled", True)
    # Ensure poll-delay behavior tests exercise the jittered model unless a
    # specific test opts into forced max-rate mode.
    monkeypatch.setattr(train_worker.settings, "train_poll_force_max_rate", False)
    monkeypatch.setattr("app.modules.train.service.settings.payment_enabled", True)


def _utc_now() -> datetime:
    return datetime.now(timezone.utc)


def test_train_passengers_allows_children_only() -> None:
    passengers = TrainPassengers(adults=0, children=1)
    assert passengers.adults == 0
    assert passengers.children == 1


def test_train_passengers_requires_at_least_one_total() -> None:
    with pytest.raises(ValidationError):
        TrainPassengers(adults=0, children=0)


def test_poll_delay_seconds_mean_curve_hits_anchor_targets() -> None:
    max_interval = float(train_worker.settings.train_poll_max_seconds)
    assert train_worker._mean_poll_delay_seconds(24 * 60 * 60, max_interval) == pytest.approx(1.25, abs=1e-6)
    assert train_worker._mean_poll_delay_seconds(48 * 60 * 60, max_interval) == pytest.approx(1.5, abs=1e-6)
    assert train_worker._mean_poll_delay_seconds(72 * 60 * 60, max_interval) == pytest.approx(2.0, abs=1e-6)


def test_poll_delay_seconds_gamma_sampling_preserves_mean_for_unclamped_regime(monkeypatch) -> None:
    # Use a deterministic RNG stream for reproducibility.
    seeded_rng = random.Random(20260222)
    monkeypatch.setattr(train_worker.random, "gammavariate", seeded_rng.gammavariate)

    t = 48 * 60 * 60
    max_interval = float(train_worker.settings.train_poll_max_seconds)
    target_mean = train_worker._mean_poll_delay_seconds(t, max_interval)

    samples = [train_worker._poll_delay_seconds(1, seconds_until_departure=t) for _ in range(20_000)]
    sample_mean = sum(samples) / len(samples)

    # At 48h and current defaults, clamp interactions are minimal; mean should stay close.
    assert sample_mean == pytest.approx(target_mean, abs=0.03)


def test_poll_delay_seconds_is_clamped_to_valid_bounds(monkeypatch) -> None:
    seeded_rng = random.Random(7)
    monkeypatch.setattr(train_worker.random, "gammavariate", seeded_rng.gammavariate)

    max_interval = float(train_worker.settings.train_poll_max_seconds)
    min_interval = float(train_worker.POLL_DELAY_MIN_SECONDS)
    all_samples: list[float] = []

    for t in (0, 24 * 60 * 60, 48 * 60 * 60, 72 * 60 * 60, 14 * 24 * 60 * 60):
        all_samples.extend(train_worker._poll_delay_seconds(1, seconds_until_departure=t) for _ in range(2_000))

    assert min(all_samples) >= min_interval
    assert max(all_samples) <= max_interval


def test_poll_delay_seconds_force_max_rate_uses_constant_min_interval(monkeypatch) -> None:
    monkeypatch.setattr(train_worker.settings, "train_poll_force_max_rate", True)
    monkeypatch.setattr(train_worker.settings, "train_poll_min_seconds", 2.0)
    monkeypatch.setattr(train_worker.settings, "train_poll_max_seconds", 6.0)
    monkeypatch.setattr(train_worker.random, "gammavariate", lambda *_args, **_kwargs: 999.0)

    for t in (0, 60, 24 * 60 * 60, 48 * 60 * 60, 72 * 60 * 60):
        assert train_worker._poll_delay_seconds(1, seconds_until_departure=t) == pytest.approx(2.0)


async def _register_and_login(client, db_session, *, email: str = "train-user@example.com") -> str:
    register_res = await client.post(
        "/api/auth/register",
        json={"email": email, "password": "SuperSecret123", "display_name": "Train User"},
    )
    assert register_res.status_code == 201

    user = (await db_session.execute(select(User).where(User.email == email))).scalar_one()
    user.access_status = "approved"
    await db_session.commit()

    login_res = await client.post(
        "/api/auth/login",
        json={"email": email, "password": "SuperSecret123", "remember_me": True},
    )
    assert login_res.status_code == 200
    cookie = login_res.cookies.get("bominal_session")
    assert cookie
    return cookie


@pytest.mark.asyncio
async def test_train_task_creation_idempotency(client, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()

    async def _noop_enqueue(task_id: str, defer_seconds: float = 0.0) -> None:
        return None

    monkeypatch.setattr("app.modules.train.service.enqueue_train_task", _noop_enqueue)
    monkeypatch.setattr("app.modules.train.service.get_redis_client", make_fake_get_redis_client(fake_redis))

    cookie = await _register_and_login(client, db_session)

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
async def test_train_task_creation_accepts_mixed_provider_selection(client, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()

    async def _noop_enqueue(task_id: str, defer_seconds: float = 0.0) -> None:
        return None

    async def _always_verified(_db, *, user, provider, force_live=False):
        return ProviderCredentialStatus(
            configured=True,
            verified=True,
            detail=None,
        )

    monkeypatch.setattr("app.modules.train.service.enqueue_train_task", _noop_enqueue)
    monkeypatch.setattr("app.modules.train.service.get_redis_client", make_fake_get_redis_client(fake_redis))
    monkeypatch.setattr("app.modules.train.service._verify_provider_credentials", _always_verified)

    cookie = await _register_and_login(client, db_session, email="mixed-task@example.com")
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
async def test_train_duplicate_check_classifies_waiting_reserved_polling_and_skips_cancelled_failed(
    client,
    db_session,
    monkeypatch,
):
    async def _always_verified(_db, *, user, provider, force_live=False):
        return ProviderCredentialStatus(
            configured=True,
            verified=True,
            detail=None,
        )

    monkeypatch.setattr("app.modules.train.service._verify_provider_credentials", _always_verified)
    cookie = await _register_and_login(client, db_session, email="duplicate-check@example.com")
    user = (await db_session.execute(select(User).where(User.email == "duplicate-check@example.com"))).scalar_one()

    departure = (_utc_now() + timedelta(hours=2)).isoformat()
    date_value = _utc_now().date().isoformat()

    def _spec(*, adults: int = 1) -> dict:
        return {
            "provider": "SRT",
            "providers": ["SRT"],
            "dep": "수서",
            "arr": "부산",
            "date": date_value,
            "selected_trains_ranked": [
                {
                    "schedule_id": "srt-dup-1",
                    "departure_at": departure,
                    "rank": 1,
                    "provider": "SRT",
                }
            ],
            "passengers": {"adults": adults, "children": 0},
            "seat_class": "general",
            "auto_pay": False,
            "notify": False,
        }

    waiting_task = Task(
        user_id=user.id,
        module="train",
        state="COMPLETED",
        deadline_at=_utc_now() + timedelta(hours=3),
        spec_json=_spec(adults=1),
        idempotency_key="dup-check-waiting",
    )
    reserved_task = Task(
        user_id=user.id,
        module="train",
        state="COMPLETED",
        deadline_at=_utc_now() + timedelta(hours=3),
        spec_json=_spec(adults=1),
        idempotency_key="dup-check-reserved",
    )
    polling_task = Task(
        user_id=user.id,
        module="train",
        state="POLLING",
        deadline_at=_utc_now() + timedelta(hours=3),
        spec_json=_spec(adults=1),
        idempotency_key="dup-check-polling",
    )
    cancelled_task = Task(
        user_id=user.id,
        module="train",
        state="CANCELLED",
        deadline_at=_utc_now() + timedelta(hours=3),
        spec_json=_spec(adults=1),
        idempotency_key="dup-check-cancelled",
        cancelled_at=_utc_now(),
    )
    failed_task = Task(
        user_id=user.id,
        module="train",
        state="FAILED",
        deadline_at=_utc_now() + timedelta(hours=3),
        spec_json=_spec(adults=1),
        idempotency_key="dup-check-failed",
        failed_at=_utc_now(),
    )
    db_session.add_all([waiting_task, reserved_task, polling_task, cancelled_task, failed_task])
    await db_session.flush()

    db_session.add(
        Artifact(
            task_id=waiting_task.id,
            module="train",
            kind="ticket",
            data_json_safe={
                "provider": "SRT",
                "reservation_id": "WAIT-1",
                "status": "waiting",
                "waiting": True,
                "paid": False,
            },
        )
    )
    db_session.add(
        Artifact(
            task_id=reserved_task.id,
            module="train",
            kind="ticket",
            data_json_safe={
                "provider": "SRT",
                "reservation_id": "RES-1",
                "status": "awaiting_payment",
                "waiting": False,
                "paid": False,
            },
        )
    )
    await db_session.commit()

    response = await client.post(
        "/api/train/tasks/duplicate-check",
        cookies={"bominal_session": cookie},
        json={
            "provider": "SRT",
            "dep": "수서",
            "arr": "부산",
            "date": date_value,
            "selected_trains_ranked": [{"schedule_id": "candidate-1", "departure_at": departure, "rank": 1}],
            "passengers": {"adults": 3, "children": 0},
            "seat_class": "general",
            "auto_pay": False,
            "notify": False,
        },
    )
    assert response.status_code == 200
    body = response.json()
    assert body["has_duplicate"] is True
    assert body["summary"]["already_reserved"] == 1
    assert body["summary"]["waiting"] == 1
    assert body["summary"]["polling"] == 1
    categories = {row["category"] for row in body["matches"]}
    assert categories == {"already_reserved", "waiting", "polling"}
    matched_ids = {row["task_id"] for row in body["matches"]}
    assert str(waiting_task.id) in matched_ids
    assert str(reserved_task.id) in matched_ids
    assert str(polling_task.id) in matched_ids
    assert str(cancelled_task.id) not in matched_ids
    assert str(failed_task.id) not in matched_ids


@pytest.mark.asyncio
async def test_train_task_create_requires_confirm_duplicate_to_create_new_same_time_task(
    client,
    db_session,
    monkeypatch,
):
    async def _always_verified(_db, *, user, provider, force_live=False):
        return ProviderCredentialStatus(
            configured=True,
            verified=True,
            detail=None,
        )

    async def _noop_enqueue(task_id: str, defer_seconds: float = 0.0) -> None:
        return None

    monkeypatch.setattr("app.modules.train.service._verify_provider_credentials", _always_verified)
    monkeypatch.setattr("app.modules.train.service.enqueue_train_task", _noop_enqueue)

    cookie = await _register_and_login(client, db_session, email="duplicate-confirm@example.com")
    user = (await db_session.execute(select(User).where(User.email == "duplicate-confirm@example.com"))).scalar_one()

    departure = (_utc_now() + timedelta(hours=2)).isoformat()
    date_value = _utc_now().date().isoformat()
    existing = Task(
        user_id=user.id,
        module="train",
        state="POLLING",
        deadline_at=_utc_now() + timedelta(hours=3),
        spec_json={
            "provider": "SRT",
            "providers": ["SRT"],
            "dep": "수서",
            "arr": "부산",
            "date": date_value,
            "selected_trains_ranked": [
                {
                    "schedule_id": "existing-1",
                    "departure_at": departure,
                    "rank": 1,
                    "provider": "SRT",
                }
            ],
            "passengers": {"adults": 1, "children": 0},
            "seat_class": "general",
            "auto_pay": False,
            "notify": False,
        },
        idempotency_key="duplicate-confirm-existing",
    )
    db_session.add(existing)
    await db_session.commit()

    payload = {
        "provider": "SRT",
        "dep": "수서",
        "arr": "부산",
        "date": date_value,
        "selected_trains_ranked": [
            {
                "schedule_id": "new-1",
                "departure_at": departure,
                "rank": 1,
            }
        ],
        "passengers": {"adults": 2, "children": 0},
        "seat_class": "general",
        "auto_pay": False,
        "notify": False,
    }

    dedup = await client.post("/api/train/tasks", cookies={"bominal_session": cookie}, json=payload)
    assert dedup.status_code == 200
    dedup_body = dedup.json()
    assert dedup_body["deduplicated"] is True
    assert dedup_body["queued"] is False
    assert dedup_body["task"]["id"] == str(existing.id)

    created = await client.post(
        "/api/train/tasks",
        cookies={"bominal_session": cookie},
        json={**payload, "confirm_duplicate": True},
    )
    assert created.status_code == 200
    created_body = created.json()
    assert created_body["deduplicated"] is False
    assert created_body["queued"] is True
    assert created_body["task"]["id"] != str(existing.id)

@pytest.mark.asyncio
async def test_train_search_returns_provider_errors_when_all_fail(client, db_session, monkeypatch):
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

    cookie = await _register_and_login(client, db_session, email="search-fail@example.com")
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
async def test_train_search_returns_partial_provider_errors_with_200(client, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()

    class MixedProvider:
        def __init__(self, provider_name: str):
            self.provider_name = provider_name

        async def login(self, **kwargs):
            return ProviderOutcome(ok=True)

        async def search(self, **kwargs):
            if self.provider_name == "SRT":
                return ProviderOutcome(
                    ok=False,
                    retryable=True,
                    error_code="provider_unreachable",
                    error_message_safe="temporary provider error",
                )
            return ProviderOutcome(ok=True, data={"schedules": []})

        async def reserve(self, **kwargs):
            return ProviderOutcome(ok=False)

        async def pay(self, **kwargs):
            return ProviderOutcome(ok=False)

        async def cancel(self, **kwargs):
            return ProviderOutcome(ok=False)

    monkeypatch.setattr("app.modules.train.service.get_redis_client", make_fake_get_redis_client(fake_redis))
    monkeypatch.setattr(
        "app.modules.train.service.get_provider_client",
        lambda provider: MixedProvider(provider),
    )

    cookie = await _register_and_login(client, db_session, email="search-partial-fail@example.com")
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

    assert response.status_code == 200
    body = response.json()
    assert body["schedules"] == []
    assert body["provider_errors"]["SRT"]["error_code"] == "provider_unreachable"
    assert body["provider_errors"]["SRT"]["error_message"] == "temporary provider error"
    assert "KTX" not in body["provider_errors"]


@pytest.mark.asyncio
async def test_srt_credentials_required_for_srt_search(client, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    monkeypatch.setattr("app.modules.train.service.get_redis_client", make_fake_get_redis_client(fake_redis))

    cookie = await _register_and_login(client, db_session, email="credential-check@example.com")

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
async def test_provider_credentials_status_checks_both(client, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    monkeypatch.setattr("app.modules.train.service.get_redis_client", make_fake_get_redis_client(fake_redis))

    cookie = await _register_and_login(client, db_session, email="status-check@example.com")

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
async def test_worker_resumes_polling_when_reserve_fails_due_to_non_payment_expiry(
    db_session_factory, db_session, monkeypatch
):
    fake_redis = fakeredis.aioredis.FakeRedis()
    enqueued: list[tuple[str, float]] = []

    class _NoLimitResult:
        waited_ms = 0
        rounds = 1

    async def _no_limit(self, **kwargs):
        return _NoLimitResult()

    async def _capture_enqueue(task_id: str, defer_seconds: float = 0.0) -> None:
        enqueued.append((task_id, defer_seconds))

    async def _fake_credentials(db, *, user_id, provider):
        return {"username": f"{provider.lower()}-user", "password": "secret"}

    class FakeProvider:
        provider_name = "SRT"

        async def login(self, **kwargs):
            return ProviderOutcome(ok=True)

        async def search(self, **kwargs):
            departure = _utc_now() + timedelta(minutes=35)
            return ProviderOutcome(
                ok=True,
                data={
                    "schedules": [
                        ProviderSchedule(
                            schedule_id="srt-non-pay-expiry",
                            provider="SRT",
                            dep=kwargs["dep"],
                            arr=kwargs["arr"],
                            departure_at=departure,
                            arrival_at=departure + timedelta(minutes=100),
                            train_no="S777",
                            availability={"general": True, "special": False},
                        )
                    ]
                },
            )

        async def reserve(self, **kwargs):
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="srt_reserve_fail",
                error_message_safe="Ticket not found: check reservation status",
            )

        async def pay(self, **kwargs):
            return ProviderOutcome(ok=False)

        async def cancel(self, **kwargs):
            return ProviderOutcome(ok=True)

    monkeypatch.setattr("app.modules.train.worker.enqueue_train_task", _capture_enqueue)
    monkeypatch.setattr("app.modules.train.worker.RedisTokenBucketLimiter.acquire_provider_call", _no_limit)
    monkeypatch.setattr("app.modules.train.worker._load_provider_credentials", _fake_credentials)
    monkeypatch.setattr("app.modules.train.worker.get_provider_client", lambda _provider: FakeProvider())

    user = User(
        email="reserve-non-payment-expiry@example.com",
        password_hash="x",
        display_name="Reserve Non Payment Expiry User",
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
            "arr": "부산",
            "date": _utc_now().date().isoformat(),
            "selected_trains_ranked": [
                {
                    "schedule_id": "srt-non-pay-expiry",
                    "departure_at": (_utc_now() + timedelta(hours=2)).isoformat(),
                    "rank": 1,
                    "provider": "SRT",
                }
            ],
            "passengers": {"adults": 1, "children": 0},
            "seat_class": "general",
            "auto_pay": False,
        },
        idempotency_key="reserve-non-payment-expiry",
    )
    db_session.add(task)
    await db_session.commit()

    await run_train_task({"db_factory": db_session_factory, "redis": fake_redis}, str(task.id))

    async with db_session_factory() as verify_session:
        refreshed = await verify_session.get(Task, task.id)
        assert refreshed is not None
        assert refreshed.state == "POLLING"
        assert refreshed.spec_json.get("next_run_at")

        reserve_attempt = (
            await verify_session.execute(
                select(TaskAttempt)
                .where(TaskAttempt.task_id == task.id)
                .where(TaskAttempt.action == "RESERVE")
                .order_by(TaskAttempt.started_at.desc())
                .limit(1)
            )
        ).scalar_one()

    assert reserve_attempt.retryable is True
    assert reserve_attempt.meta_json_safe is not None
    assert reserve_attempt.meta_json_safe.get("non_payment_expiry_retry") is True
    assert len(enqueued) == 1
    assert enqueued[0][0] == str(task.id)
    assert enqueued[0][1] > 0


@pytest.mark.asyncio
async def test_worker_retries_polling_on_reserve_sold_out(db_session_factory, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    enqueued: list[tuple[str, float]] = []

    class _NoLimitResult:
        waited_ms = 0
        rounds = 1

    async def _no_limit(self, **kwargs):
        return _NoLimitResult()

    async def _capture_enqueue(task_id: str, defer_seconds: float = 0.0) -> None:
        enqueued.append((task_id, defer_seconds))

    async def _fake_credentials(*args, **kwargs):
        return {"username": "soldout-user", "password": "soldout-password"}

    class FakeProvider:
        provider_name = "SRT"

        async def login(self, **kwargs):
            return ProviderOutcome(ok=True, data={"membership_number": "M-123"})

        async def search(self, **kwargs):
            departure = _utc_now() + timedelta(hours=2)
            return ProviderOutcome(
                ok=True,
                data={
                    "schedules": [
                        ProviderSchedule(
                            schedule_id="srt-sold-out-schedule",
                            provider="SRT",
                            dep="수서",
                            arr="부산",
                            departure_at=departure,
                            arrival_at=departure + timedelta(minutes=100),
                            train_no="S555",
                            availability={"general": True, "special": False},
                        )
                    ]
                },
            )

        async def reserve(self, **kwargs):
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="sold_out",
                error_message_safe="No reservable seats are available for this schedule.",
            )

        async def pay(self, **kwargs):
            return ProviderOutcome(ok=False)

        async def cancel(self, **kwargs):
            return ProviderOutcome(ok=True)

    monkeypatch.setattr("app.modules.train.worker.enqueue_train_task", _capture_enqueue)
    monkeypatch.setattr("app.modules.train.worker.RedisTokenBucketLimiter.acquire_provider_call", _no_limit)
    monkeypatch.setattr("app.modules.train.worker._load_provider_credentials", _fake_credentials)
    monkeypatch.setattr("app.modules.train.worker.get_provider_client", lambda _provider: FakeProvider())

    user = User(
        email="reserve-sold-out-retry@example.com",
        password_hash="x",
        display_name="Reserve Sold Out Retry User",
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
            "arr": "부산",
            "date": _utc_now().date().isoformat(),
            "selected_trains_ranked": [
                {
                    "schedule_id": "srt-sold-out-schedule",
                    "departure_at": (_utc_now() + timedelta(hours=2)).isoformat(),
                    "rank": 1,
                    "provider": "SRT",
                }
            ],
            "passengers": {"adults": 1, "children": 0},
            "seat_class": "general",
            "auto_pay": True,
        },
        idempotency_key="reserve-sold-out-retry",
    )
    db_session.add(task)
    await db_session.commit()

    await run_train_task({"db_factory": db_session_factory, "redis": fake_redis}, str(task.id))

    async with db_session_factory() as verify_session:
        refreshed = await verify_session.get(Task, task.id)
        assert refreshed is not None
        assert refreshed.state == "POLLING"
        assert refreshed.spec_json.get("next_run_at")

        reserve_attempt = (
            await verify_session.execute(
                select(TaskAttempt)
                .where(TaskAttempt.task_id == task.id)
                .where(TaskAttempt.action == "RESERVE")
                .order_by(TaskAttempt.started_at.desc())
                .limit(1)
            )
        ).scalar_one()

    assert reserve_attempt.retryable is True
    assert reserve_attempt.error_code == "sold_out"
    assert reserve_attempt.meta_json_safe is not None
    assert reserve_attempt.meta_json_safe.get("sold_out_retry") is True
    assert len(enqueued) == 1
    assert enqueued[0][0] == str(task.id)
    assert enqueued[0][1] > 0


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
                                "paid": False,
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
async def test_compact_and_prune_task_attempts_dedupes_and_applies_retention(db_session, monkeypatch):
    now = _utc_now()
    monkeypatch.setattr(train_worker.settings, "train_sync_keep_latest_only", True)
    monkeypatch.setattr(train_worker.settings, "train_compact_repetitive_attempts", True)
    monkeypatch.setattr(train_worker.settings, "train_attempt_retention_days", 30)

    user = User(
        email="attempt-cleanup@example.com",
        password_hash="x",
        display_name="Attempt Cleanup User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    active_task = Task(
        user_id=user.id,
        module="train",
        state="POLLING",
        deadline_at=now + timedelta(hours=2),
        spec_json={"provider": "SRT"},
        idempotency_key="attempt-cleanup-active",
    )
    old_terminal_task = Task(
        user_id=user.id,
        module="train",
        state="FAILED",
        deadline_at=now - timedelta(days=40),
        spec_json={"provider": "SRT"},
        idempotency_key="attempt-cleanup-old-terminal",
        failed_at=now - timedelta(days=40),
        updated_at=now - timedelta(days=40),
    )
    recent_terminal_task = Task(
        user_id=user.id,
        module="train",
        state="FAILED",
        deadline_at=now - timedelta(days=1),
        spec_json={"provider": "SRT"},
        idempotency_key="attempt-cleanup-recent-terminal",
        failed_at=now - timedelta(days=1),
        updated_at=now - timedelta(days=1),
    )
    db_session.add_all([active_task, old_terminal_task, recent_terminal_task])
    await db_session.flush()

    def _attempt(
        *,
        task_id,
        action: str,
        started_at: datetime,
        finished_at: datetime,
        ok: bool,
        retryable: bool,
        error_code: str | None,
        error_message_safe: str | None,
    ) -> TaskAttempt:
        return TaskAttempt(
            task_id=task_id,
            action=action,
            provider="SRT",
            ok=ok,
            retryable=retryable,
            error_code=error_code,
            error_message_safe=error_message_safe,
            duration_ms=50,
            meta_json_safe={},
            started_at=started_at,
            finished_at=finished_at,
        )

    sync1 = _attempt(
        task_id=active_task.id,
        action="SYNC",
        started_at=now - timedelta(minutes=8),
        finished_at=now - timedelta(minutes=8),
        ok=True,
        retryable=True,
        error_code=None,
        error_message_safe=None,
    )
    sync2 = _attempt(
        task_id=active_task.id,
        action="SYNC",
        started_at=now - timedelta(minutes=2),
        finished_at=now - timedelta(minutes=2),
        ok=True,
        retryable=True,
        error_code=None,
        error_message_safe=None,
    )
    search1 = _attempt(
        task_id=active_task.id,
        action="SEARCH",
        started_at=now - timedelta(minutes=7),
        finished_at=now - timedelta(minutes=7),
        ok=False,
        retryable=True,
        error_code="seat_unavailable",
        error_message_safe="No reservable seats are available for this schedule.",
    )
    search2 = _attempt(
        task_id=active_task.id,
        action="SEARCH",
        started_at=now - timedelta(minutes=6),
        finished_at=now - timedelta(minutes=6),
        ok=False,
        retryable=True,
        error_code="seat_unavailable",
        error_message_safe="No reservable seats are available for this schedule.",
    )
    search3 = _attempt(
        task_id=active_task.id,
        action="SEARCH",
        started_at=now - timedelta(minutes=5),
        finished_at=now - timedelta(minutes=5),
        ok=False,
        retryable=False,
        error_code="provider_unreachable",
        error_message_safe="provider outage",
    )
    search4 = _attempt(
        task_id=active_task.id,
        action="SEARCH",
        started_at=now - timedelta(minutes=4),
        finished_at=now - timedelta(minutes=4),
        ok=False,
        retryable=False,
        error_code="provider_unreachable",
        error_message_safe="provider outage",
    )
    old_terminal_attempt = _attempt(
        task_id=old_terminal_task.id,
        action="SEARCH",
        started_at=now - timedelta(days=40),
        finished_at=now - timedelta(days=40),
        ok=False,
        retryable=True,
        error_code="seat_unavailable",
        error_message_safe="No reservable seats are available for this schedule.",
    )
    recent_terminal_attempt = _attempt(
        task_id=recent_terminal_task.id,
        action="SEARCH",
        started_at=now - timedelta(hours=12),
        finished_at=now - timedelta(hours=12),
        ok=False,
        retryable=True,
        error_code="seat_unavailable",
        error_message_safe="No reservable seats are available for this schedule.",
    )
    db_session.add_all([sync1, sync2, search1, search2, search3, search4, old_terminal_attempt, recent_terminal_attempt])
    await db_session.commit()

    stats = await train_worker.compact_and_prune_task_attempts(db_session)
    assert stats["deleted_sync_rows"] >= 1
    assert stats["deleted_repetitive_rows"] >= 1
    assert stats["deleted_retention_rows"] >= 1

    active_attempts = (
        (
            await db_session.execute(
                select(TaskAttempt)
                .where(TaskAttempt.task_id == active_task.id)
                .order_by(TaskAttempt.action.asc(), TaskAttempt.started_at.asc(), TaskAttempt.id.asc())
            )
        )
        .scalars()
        .all()
    )
    active_sync_attempts = [attempt for attempt in active_attempts if attempt.action == "SYNC"]
    assert len(active_sync_attempts) == 1
    assert active_sync_attempts[0].id == sync2.id

    active_search_attempts = [attempt for attempt in active_attempts if attempt.action == "SEARCH"]
    assert len(active_search_attempts) == 3
    assert {attempt.id for attempt in active_search_attempts} == {search1.id, search3.id, search4.id}

    old_terminal_attempts = (
        (
            await db_session.execute(select(TaskAttempt).where(TaskAttempt.task_id == old_terminal_task.id))
        )
        .scalars()
        .all()
    )
    assert old_terminal_attempts == []

    recent_terminal_attempts = (
        (
            await db_session.execute(select(TaskAttempt).where(TaskAttempt.task_id == recent_terminal_task.id))
        )
        .scalars()
        .all()
    )
    assert len(recent_terminal_attempts) == 1


@pytest.mark.asyncio
async def test_task_detail_returns_cached_artifact_without_provider_refresh(db_session, monkeypatch):
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

    refresh_calls: list[str] = []

    async def _refresh_stub(*_args, **kwargs):  # noqa: ANN002, ANN003
        refresh_calls.append(str(kwargs["artifact"].id))
        return True

    monkeypatch.setattr("app.modules.train.service._refresh_ticket_artifact_status", _refresh_stub)

    detail = await get_task_detail(db_session, task_id=task.id, user=user)
    assert detail.artifacts
    ticket = detail.artifacts[0].data_json_safe
    assert ticket.get("status") == "reserved"
    assert refresh_calls == []


@pytest.mark.asyncio
async def test_task_detail_does_not_refresh_waitlisted_polling_ticket_on_page_load(db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    monkeypatch.setattr("app.modules.train.service.get_redis_client", make_fake_get_redis_client(fake_redis))

    user = User(
        email="detail-waitlisted@example.com",
        password_hash="x",
        display_name="Detail Waitlisted User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    task = Task(
        user_id=user.id,
        module="train",
        state="POLLING",
        deadline_at=_utc_now() + timedelta(hours=4),
        spec_json={"module": "train"},
        idempotency_key="detail-waitlisted",
    )
    db_session.add(task)
    await db_session.flush()

    artifact = Artifact(
        task_id=task.id,
        module="train",
        kind="ticket",
        data_json_safe={
            "provider": "SRT",
            "reservation_id": "PNR-WAIT-1",
            "paid": False,
            "status": "waiting",
            "waiting": True,
        },
    )
    db_session.add(artifact)
    await db_session.commit()

    refresh_calls: list[str] = []

    async def _refresh_stub(*_args, **kwargs):  # noqa: ANN002, ANN003
        refresh_calls.append(str(kwargs["artifact"].id))
        return True

    monkeypatch.setattr("app.modules.train.service._refresh_ticket_artifact_status", _refresh_stub)

    detail = await get_task_detail(db_session, task_id=task.id, user=user)

    assert detail.task.state == "POLLING"
    assert detail.artifacts[0].data_json_safe.get("status") == "waiting"
    assert refresh_calls == []


@pytest.mark.asyncio
async def test_list_tasks_does_not_refresh_completed_ticket_status_on_page_load(client, db_session, monkeypatch):
    email = "list-refresh@example.com"
    cookie = await _register_and_login(client, db_session, email=email)

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

    refresh_calls: list[str] = []

    async def _refresh_stub(*_args, **kwargs):  # noqa: ANN002, ANN003
        refresh_calls.append(str(kwargs["artifact"].id))
        return True

    monkeypatch.setattr("app.modules.train.service._refresh_ticket_artifact_status", _refresh_stub)

    completed_response = await client.get(
        "/api/train/tasks?status=completed&refresh_completed=true",
        cookies={"bominal_session": cookie},
    )
    assert completed_response.status_code == 200
    completed_rows = completed_response.json()["tasks"]
    assert any(row["id"] == str(task.id) for row in completed_rows)
    assert refresh_calls == []

    active_response = await client.get(
        "/api/train/tasks?status=active",
        cookies={"bominal_session": cookie},
    )
    assert active_response.status_code == 200
    active_rows = active_response.json()["tasks"]
    assert all(row["id"] != str(task.id) for row in active_rows)


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

    cookie = await _register_and_login(client, db_session, email="manual-pay-expired@example.com")
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
async def test_manual_pay_rejects_expired_ticket_status(client, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    monkeypatch.setattr("app.modules.train.service.get_redis_client", make_fake_get_redis_client(fake_redis))

    cookie = await _register_and_login(client, db_session, email="manual-pay-expired-status@example.com")
    user = (
        await db_session.execute(select(User).where(User.email == "manual-pay-expired-status@example.com"))
    ).scalar_one()

    task = Task(
        user_id=user.id,
        module="train",
        state="COMPLETED",
        deadline_at=_utc_now() + timedelta(hours=1),
        spec_json={"module": "train"},
        idempotency_key="manual-pay-expired-status",
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
                "reservation_id": "PNR-EXPIRED-STATUS-1",
                "status": "expired",
                "paid": False,
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

    cookie = await _register_and_login(client, db_session, email="manual-pay-success@example.com")
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

    cookie = await _register_and_login(client, db_session, email="manual-cancel-attempt@example.com")
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
async def test_ktx_wait_reserve_path_runs_when_selected_train_is_waitlist_only(
    db_session_factory,
    db_session,
    monkeypatch,
):
    fake_redis = fakeredis.aioredis.FakeRedis()
    utc_now_naive = lambda: datetime.now(timezone.utc).replace(tzinfo=None)

    class FakeKTXProvider:
        provider_name = "KTX"

        def __init__(self):
            self.reserve_calls: list[str] = []

        async def login(self, **kwargs):
            return ProviderOutcome(ok=True)

        async def search(self, **kwargs):
            dep = kwargs["dep"]
            arr = kwargs["arr"]
            departure = _utc_now() + timedelta(minutes=20)
            schedule = ProviderSchedule(
                schedule_id="ktx-waitlist-only",
                provider="KTX",
                dep=dep,
                arr=arr,
                departure_at=departure,
                arrival_at=departure + timedelta(minutes=75),
                train_no="K310",
                availability={"general": False, "special": False},
                metadata={"wait_reserve_flag": "9"},
            )
            return ProviderOutcome(ok=True, data={"schedules": [schedule]})

        async def reserve(self, **kwargs):
            self.reserve_calls.append(kwargs["seat_class"])
            return ProviderOutcome(ok=True, data={"reservation_id": "ktx-rsv-waitlist"})

        async def pay(self, **kwargs):
            return ProviderOutcome(ok=False, retryable=False, error_code="unexpected", error_message_safe="unexpected")

        async def cancel(self, **kwargs):
            return ProviderOutcome(ok=False, retryable=False, error_code="not_supported", error_message_safe="not supported")

    fake_provider = FakeKTXProvider()

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
        email="ktx-waitlist-worker@example.com",
        password_hash="x",
        display_name="KTX Waitlist User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    deadline = utc_now_naive() + timedelta(minutes=45)
    task = Task(
        user_id=user.id,
        module="train",
        state="QUEUED",
        deadline_at=deadline,
        spec_json={
            "provider": "KTX",
            "dep": "서울",
            "arr": "부산",
            "date": utc_now_naive().date().isoformat(),
            "selected_trains_ranked": [{"schedule_id": "ktx-waitlist-only", "departure_at": deadline.isoformat(), "rank": 1}],
            "passengers": {"adults": 1, "children": 0},
            "seat_class": "general",
            "auto_pay": False,
        },
        idempotency_key="ktx-waitlist-worker",
    )
    db_session.add(task)
    await db_session.flush()

    # Skip the worker's first-cycle visual delay so we can validate reserve behavior in one run.
    db_session.add(
        TaskAttempt(
            task_id=task.id,
            action="SEARCH",
            provider="KTX",
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

    assert fake_provider.reserve_calls == ["general"]
    assert len(artifacts) == 1
    assert artifacts[0].data_json_safe.get("provider") == "KTX"
    assert artifacts[0].data_json_safe.get("reservation_id") == "ktx-rsv-waitlist"


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
async def test_retry_task_now_falls_back_to_short_deferred_enqueue_on_collision(db_session, monkeypatch):
    enqueue_calls: list[tuple[str, float]] = []

    async def _enqueue(task_id: str, defer_seconds: float = 0.0) -> bool:
        enqueue_calls.append((task_id, float(defer_seconds)))
        if defer_seconds == 0.0:
            return False
        return True

    monkeypatch.setattr("app.modules.train.service.enqueue_train_task", _enqueue)

    user = User(
        email="retry-fallback@example.com",
        password_hash="x",
        display_name="Retry Fallback User",
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
        idempotency_key="retry-fallback",
    )
    db_session.add(task)
    await db_session.commit()

    response = await retry_task_now(db_session, task_id=task.id, user=user)

    assert response.task.state == "QUEUED"
    assert enqueue_calls == [
        (str(task.id), 0.0),
        (str(task.id), 0.01),
    ]


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
async def test_retry_task_now_allows_expired_terminal_task(db_session, monkeypatch):
    enqueue_calls: list[tuple[str, float]] = []

    async def _enqueue(task_id: str, defer_seconds: float = 0.0) -> bool:
        enqueue_calls.append((task_id, float(defer_seconds)))
        return True

    monkeypatch.setattr("app.modules.train.service.enqueue_train_task", _enqueue)

    user = User(
        email="retry-expired-terminal@example.com",
        password_hash="x",
        display_name="Retry Expired Terminal",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    now = _utc_now()
    task = Task(
        user_id=user.id,
        module="train",
        state="EXPIRED",
        deadline_at=now + timedelta(minutes=10),
        completed_at=now - timedelta(minutes=1),
        spec_json={"provider": "SRT", "manual_retry_last_at": (now - timedelta(minutes=5)).isoformat()},
        idempotency_key="retry-expired-terminal",
    )
    db_session.add(task)
    await db_session.commit()

    response = await retry_task_now(db_session, task_id=task.id, user=user)

    assert response.task.state == "QUEUED"
    assert enqueue_calls == [(str(task.id), 0.0)]

    await db_session.refresh(task)
    assert task.state == "QUEUED"
    assert task.completed_at is None
    assert task.failed_at is None
    assert task.cancelled_at is None
    assert task.spec_json.get("manual_retry_last_at")


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


@pytest.mark.asyncio
async def test_task_summary_includes_next_run_at_for_polling(db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    monkeypatch.setattr("app.modules.train.service.get_redis_client", make_fake_get_redis_client(fake_redis))

    user = User(
        email="next-run@example.com",
        password_hash="x",
        display_name="Next Run User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    task = Task(
        user_id=user.id,
        module="train",
        state="POLLING",
        deadline_at=_utc_now() + timedelta(minutes=10),
        spec_json={"provider": "SRT", "next_run_at": (_utc_now() + timedelta(seconds=45)).isoformat()},
        idempotency_key="next-run",
    )
    db_session.add(task)
    await db_session.commit()

    detail = await get_task_detail(db_session, task_id=task.id, user=user)
    assert detail.task.next_run_at is not None


@pytest.mark.asyncio
async def test_task_list_includes_last_attempt_summary(db_session):
    user = User(
        email="attempt-summary@example.com",
        password_hash="x",
        display_name="Attempt Summary User",
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
        idempotency_key="attempt-summary",
    )
    db_session.add(task)
    await db_session.flush()

    started_1 = _utc_now() - timedelta(seconds=20)
    finished_1 = _utc_now() - timedelta(seconds=18)
    started_2 = _utc_now() - timedelta(seconds=10)
    finished_2 = _utc_now() - timedelta(seconds=8)

    db_session.add_all(
        [
            TaskAttempt(
                task_id=task.id,
                action="SEARCH",
                provider="SRT",
                ok=False,
                retryable=True,
                error_code="search_failed",
                error_message_safe="search failed",
                duration_ms=123,
                meta_json_safe={},
                started_at=started_1,
                finished_at=finished_1,
            ),
            TaskAttempt(
                task_id=task.id,
                action="RESERVE",
                provider="SRT",
                ok=False,
                retryable=True,
                error_code="reserve_failed",
                error_message_safe="reserve failed",
                duration_ms=456,
                meta_json_safe={},
                started_at=started_2,
                finished_at=finished_2,
            ),
        ]
    )
    await db_session.commit()

    response = await list_tasks(db_session, user=user, status_filter="active")
    assert response.tasks
    summary = next(item for item in response.tasks if item.id == task.id)

    assert summary.last_attempt_action == "RESERVE"
    assert summary.last_attempt_ok is False
    assert summary.last_attempt_error_code == "reserve_failed"
    assert summary.last_attempt_error_message_safe == "reserve failed"
    assert summary.last_attempt_finished_at is not None
    assert summary.last_attempt_at == summary.last_attempt_finished_at


@pytest.mark.asyncio
async def test_task_list_latest_attempt_tie_breaks_with_descending_id(db_session):
    user = User(
        email="attempt-tie@example.com",
        password_hash="x",
        display_name="Attempt Tie User",
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
        idempotency_key="attempt-tie-break",
    )
    db_session.add(task)
    await db_session.flush()

    tied_finished_at = _utc_now() - timedelta(seconds=5)
    db_session.add_all(
        [
            TaskAttempt(
                id=UUID("00000000-0000-0000-0000-000000000001"),
                task_id=task.id,
                action="SEARCH",
                provider="SRT",
                ok=False,
                retryable=True,
                error_code="first_attempt",
                error_message_safe="first",
                duration_ms=100,
                meta_json_safe={},
                started_at=tied_finished_at - timedelta(seconds=1),
                finished_at=tied_finished_at,
            ),
            TaskAttempt(
                id=UUID("00000000-0000-0000-0000-000000000002"),
                task_id=task.id,
                action="RESERVE",
                provider="SRT",
                ok=True,
                retryable=False,
                error_code=None,
                error_message_safe=None,
                duration_ms=120,
                meta_json_safe={},
                started_at=tied_finished_at - timedelta(seconds=2),
                finished_at=tied_finished_at,
            ),
        ]
    )
    await db_session.commit()

    response = await list_tasks(db_session, user=user, status_filter="active")
    summary = next(item for item in response.tasks if item.id == task.id)

    assert summary.last_attempt_action == "RESERVE"
    assert summary.last_attempt_ok is True
    assert summary.last_attempt_error_code is None
    assert summary.last_attempt_error_message_safe is None


@pytest.mark.asyncio
async def test_task_list_latest_ticket_tie_breaks_with_descending_id(db_session):
    user = User(
        email="ticket-tie@example.com",
        password_hash="x",
        display_name="Ticket Tie User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    task = Task(
        user_id=user.id,
        module="train",
        state="COMPLETED",
        deadline_at=_utc_now() + timedelta(minutes=10),
        completed_at=_utc_now(),
        spec_json={"provider": "SRT"},
        idempotency_key="ticket-tie-break",
    )
    db_session.add(task)
    await db_session.flush()

    tied_created_at = _utc_now() - timedelta(minutes=1)
    db_session.add_all(
        [
            Artifact(
                id=UUID("10000000-0000-0000-0000-000000000001"),
                task_id=task.id,
                module="train",
                kind="ticket",
                data_json_safe={
                    "status": "reserved",
                    "paid": False,
                    "reservation_id": "RES-1",
                },
                created_at=tied_created_at,
            ),
            Artifact(
                id=UUID("10000000-0000-0000-0000-000000000002"),
                task_id=task.id,
                module="train",
                kind="ticket",
                data_json_safe={
                    "status": "awaiting_payment",
                    "paid": True,
                    "reservation_id": "RES-2",
                },
                created_at=tied_created_at,
            ),
        ]
    )
    await db_session.commit()

    response = await list_tasks(db_session, user=user, status_filter="completed")
    summary = next(item for item in response.tasks if item.id == task.id)

    assert summary.ticket_status == "awaiting_payment"
    assert summary.ticket_paid is True
    assert summary.ticket_reservation_id == "RES-2"


@pytest.mark.asyncio
async def test_task_list_classifies_completed_awaiting_payment_as_active(db_session):
    user = User(
        email="awaiting-payment-active@example.com",
        password_hash="x",
        display_name="Awaiting Payment Active User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    pending_task = Task(
        user_id=user.id,
        module="train",
        state="COMPLETED",
        deadline_at=_utc_now() + timedelta(minutes=20),
        completed_at=_utc_now(),
        spec_json={"provider": "SRT"},
        idempotency_key="awaiting-payment-active",
    )
    paid_task = Task(
        user_id=user.id,
        module="train",
        state="COMPLETED",
        deadline_at=_utc_now() + timedelta(minutes=20),
        completed_at=_utc_now(),
        spec_json={"provider": "SRT"},
        idempotency_key="awaiting-payment-paid",
    )
    waiting_task = Task(
        user_id=user.id,
        module="train",
        state="COMPLETED",
        deadline_at=_utc_now() + timedelta(minutes=20),
        completed_at=_utc_now(),
        spec_json={"provider": "SRT"},
        idempotency_key="waiting-active",
    )
    db_session.add_all([pending_task, paid_task, waiting_task])
    await db_session.flush()

    db_session.add_all(
        [
            Artifact(
                task_id=pending_task.id,
                module="train",
                kind="ticket",
                data_json_safe={
                    "status": "awaiting_payment",
                    "paid": False,
                    "reservation_id": "PENDING-RES-1",
                },
            ),
            Artifact(
                task_id=paid_task.id,
                module="train",
                kind="ticket",
                data_json_safe={
                    "status": "paid",
                    "paid": True,
                    "reservation_id": "PAID-RES-1",
                },
            ),
            Artifact(
                task_id=waiting_task.id,
                module="train",
                kind="ticket",
                data_json_safe={
                    "status": "waiting",
                    "paid": False,
                    "waiting": True,
                    "reservation_id": "WAITING-RES-1",
                },
            ),
        ]
    )
    await db_session.commit()

    active_response = await list_tasks(db_session, user=user, status_filter="active")
    active_ids = {item.id for item in active_response.tasks}
    assert pending_task.id in active_ids
    assert waiting_task.id in active_ids
    assert paid_task.id not in active_ids

    completed_response = await list_tasks(db_session, user=user, status_filter="completed")
    completed_ids = {item.id for item in completed_response.tasks}
    assert pending_task.id not in completed_ids
    assert waiting_task.id not in completed_ids
    assert paid_task.id in completed_ids


@pytest.mark.asyncio
async def test_provider_reservation_discovery_populates_active_and_completed_lists(db_session, monkeypatch):
    user = User(
        email="provider-discovery@example.com",
        password_hash="x",
        display_name="Provider Discovery User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    now = _utc_now()
    waiting_deadline = (now + timedelta(minutes=20)).isoformat()
    waiting_departure = (now + timedelta(hours=3)).isoformat()
    paid_departure = (now + timedelta(hours=5)).isoformat()

    class _ProviderClient:
        async def get_reservations(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(
                ok=True,
                data={
                    "reservations": [
                        {
                            "reservation_id": "DISCOVERED-WAIT-1",
                            "provider": "KTX",
                            "paid": False,
                            "waiting": True,
                            "dep": "서울",
                            "arr": "부산",
                            "departure_at": waiting_departure,
                            "payment_deadline_at": waiting_deadline,
                            "tickets": [],
                        },
                        {
                            "reservation_id": "DISCOVERED-PAID-1",
                            "provider": "KTX",
                            "paid": True,
                            "waiting": False,
                            "dep": "서울",
                            "arr": "동대구",
                            "departure_at": paid_departure,
                            "tickets": [{"car_no": "5", "seat_no": "10A"}],
                        },
                    ]
                },
            )

    async def _provider_client(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return _ProviderClient()

    monkeypatch.setattr("app.modules.train.service._get_logged_in_provider_client", _provider_client)

    first = await list_provider_reservations(db_session, user=user, provider="KTX", paid_only=False)
    assert len(first.reservations) == 2

    active = await list_tasks(db_session, user=user, status_filter="active")
    completed = await list_tasks(db_session, user=user, status_filter="completed")

    active_row = next((row for row in active.tasks if row.ticket_reservation_id == "DISCOVERED-WAIT-1"), None)
    assert active_row is not None
    assert active_row.ticket_status == "waiting"
    assert active_row.ticket_paid is False

    completed_row = next((row for row in completed.tasks if row.ticket_reservation_id == "DISCOVERED-PAID-1"), None)
    assert completed_row is not None
    assert completed_row.ticket_status == "paid"
    assert completed_row.ticket_paid is True

    second = await list_provider_reservations(db_session, user=user, provider="KTX", paid_only=False)
    assert len(second.reservations) == 2
    task_count = len((await db_session.execute(select(Task).where(Task.user_id == user.id))).scalars().all())
    assert task_count == 2


@pytest.mark.asyncio
async def test_provider_reservation_discovery_preserves_existing_active_waiting_task_state(db_session, monkeypatch):
    user = User(
        email="provider-discovery-existing@example.com",
        password_hash="x",
        display_name="Provider Discovery Existing User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    task = Task(
        user_id=user.id,
        module="train",
        state="POLLING",
        deadline_at=_utc_now() + timedelta(hours=6),
        spec_json={"provider": "KTX"},
        idempotency_key="provider-discovery-existing",
    )
    db_session.add(task)
    await db_session.flush()
    db_session.add(
        Artifact(
            task_id=task.id,
            module="train",
            kind="ticket",
            data_json_safe={
                "provider": "KTX",
                "reservation_id": "DISCOVERED-EXISTING-WAIT-1",
                "status": "waiting",
                "paid": False,
            },
        )
    )
    await db_session.commit()

    class _ProviderClient:
        async def get_reservations(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(
                ok=True,
                data={
                    "reservations": [
                        {
                            "reservation_id": "DISCOVERED-EXISTING-WAIT-1",
                            "provider": "KTX",
                            "paid": False,
                            "waiting": True,
                            "dep": "서울",
                            "arr": "부산",
                            "departure_at": (_utc_now() + timedelta(hours=2)).isoformat(),
                        }
                    ]
                },
            )

    async def _provider_client(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return _ProviderClient()

    monkeypatch.setattr("app.modules.train.service._get_logged_in_provider_client", _provider_client)
    await list_provider_reservations(db_session, user=user, provider="KTX", paid_only=False)
    await db_session.refresh(task)
    assert task.state == "POLLING"


@pytest.mark.asyncio
async def test_active_task_list_does_not_refresh_awaiting_payment_ticket_status(db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    monkeypatch.setattr("app.modules.train.service.get_redis_client", make_fake_get_redis_client(fake_redis))

    user = User(
        email="awaiting-payment-refresh@example.com",
        password_hash="x",
        display_name="Awaiting Payment Refresh User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    task = Task(
        user_id=user.id,
        module="train",
        state="COMPLETED",
        deadline_at=_utc_now() + timedelta(minutes=30),
        completed_at=_utc_now(),
        spec_json={"provider": "SRT"},
        idempotency_key="awaiting-payment-refresh",
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
                "reservation_id": "REFRESH-RES-1",
                "status": "awaiting_payment",
                "paid": False,
            },
        )
    )
    await db_session.commit()

    refresh_calls = 0

    async def _fake_refresh(db, *, user, artifact, limiter, force=False, client_cache=None):
        nonlocal refresh_calls
        refresh_calls += 1
        return False

    monkeypatch.setattr("app.modules.train.service._refresh_ticket_artifact_status", _fake_refresh)

    response = await list_tasks(db_session, user=user, status_filter="active")
    assert any(item.id == task.id for item in response.tasks)
    assert refresh_calls == 0


@pytest.mark.asyncio
async def test_active_task_list_does_not_refresh_waitlisted_ticket_status(db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()
    monkeypatch.setattr("app.modules.train.service.get_redis_client", make_fake_get_redis_client(fake_redis))

    user = User(
        email="waitlisted-no-refresh@example.com",
        password_hash="x",
        display_name="Waitlisted No Refresh User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    task = Task(
        user_id=user.id,
        module="train",
        state="COMPLETED",
        deadline_at=_utc_now() + timedelta(minutes=30),
        completed_at=_utc_now(),
        spec_json={"provider": "SRT"},
        idempotency_key="waitlisted-no-refresh",
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
                "reservation_id": "WAIT-NO-REFRESH-1",
                "status": "waiting",
                "waiting": True,
                "paid": False,
            },
        )
    )
    await db_session.commit()

    refresh_calls = 0

    async def _fake_refresh(db, *, user, artifact, limiter, force=False, client_cache=None):
        nonlocal refresh_calls
        refresh_calls += 1
        return False

    monkeypatch.setattr("app.modules.train.service._refresh_ticket_artifact_status", _fake_refresh)

    response = await list_tasks(db_session, user=user, status_filter="active")
    assert any(item.id == task.id for item in response.tasks)
    assert refresh_calls == 0


@pytest.mark.asyncio
async def test_task_list_limit_bounds_results(db_session):
    user = User(
        email="task-limit@example.com",
        password_hash="x",
        display_name="Task Limit User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    for idx in range(3):
        db_session.add(
            Task(
                user_id=user.id,
                module="train",
                state="QUEUED",
                deadline_at=_utc_now() + timedelta(minutes=10 + idx),
                spec_json={"provider": "SRT", "index": idx},
                idempotency_key=f"task-limit-{idx}",
            )
        )

    await db_session.commit()

    response = await list_tasks(db_session, user=user, status_filter="active", limit=2)
    assert len(response.tasks) == 2


@pytest.mark.asyncio
async def test_train_tasks_endpoint_rejects_invalid_limit_query(client, db_session):
    cookie = await _register_and_login(client, db_session, email="task-limit-query@example.com")

    response = await client.get(
        "/api/train/tasks?status=active&limit=0",
        cookies={"bominal_session": cookie},
    )
    assert response.status_code == 422


@pytest.mark.asyncio
async def test_task_list_includes_retry_now_status_fields(db_session):
    now = _utc_now()
    deadline = now + timedelta(minutes=10)

    user = User(
        email="retry-status@example.com",
        password_hash="x",
        display_name="Retry Status User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    allowed_task = Task(
        user_id=user.id,
        module="train",
        state="QUEUED",
        deadline_at=deadline,
        spec_json={"provider": "SRT"},
        idempotency_key="retry-status-allowed",
    )
    paused_task = Task(
        user_id=user.id,
        module="train",
        state="PAUSED",
        paused_at=now,
        deadline_at=deadline,
        spec_json={"provider": "SRT"},
        idempotency_key="retry-status-paused",
    )
    running_task = Task(
        user_id=user.id,
        module="train",
        state="RUNNING",
        deadline_at=deadline,
        spec_json={"provider": "SRT"},
        idempotency_key="retry-status-running",
    )
    deadline_passed_task = Task(
        user_id=user.id,
        module="train",
        state="QUEUED",
        deadline_at=now - timedelta(seconds=1),
        spec_json={"provider": "SRT"},
        idempotency_key="retry-status-deadline",
    )
    cooldown_task = Task(
        user_id=user.id,
        module="train",
        state="POLLING",
        deadline_at=deadline,
        spec_json={"provider": "SRT", "manual_retry_last_at": now.isoformat()},
        idempotency_key="retry-status-cooldown",
    )

    db_session.add_all([allowed_task, paused_task, running_task, deadline_passed_task, cooldown_task])
    await db_session.commit()

    response = await list_tasks(db_session, user=user, status_filter="active")

    allowed_summary = next(item for item in response.tasks if item.id == allowed_task.id)
    assert allowed_summary.retry_now_allowed is True
    assert allowed_summary.retry_now_reason is None
    assert allowed_summary.retry_now_available_at is None

    paused_summary = next(item for item in response.tasks if item.id == paused_task.id)
    assert paused_summary.retry_now_allowed is False
    assert paused_summary.retry_now_reason == "paused_use_resume"

    running_summary = next(item for item in response.tasks if item.id == running_task.id)
    assert running_summary.retry_now_allowed is False
    assert running_summary.retry_now_reason == "task_running"

    deadline_summary = next(item for item in response.tasks if item.id == deadline_passed_task.id)
    assert deadline_summary.retry_now_allowed is False
    assert deadline_summary.retry_now_reason == "deadline_passed"

    cooldown_summary = next(item for item in response.tasks if item.id == cooldown_task.id)
    assert cooldown_summary.retry_now_allowed is False
    assert cooldown_summary.retry_now_reason == "cooldown_active"
    assert cooldown_summary.retry_now_available_at is not None


@pytest.mark.asyncio
async def test_worker_schedule_retry_sets_next_run_at(db_session_factory, db_session, monkeypatch):
    fake_redis = fakeredis.aioredis.FakeRedis()

    class _NoLimitResult:
        waited_ms = 0
        rounds = 1

    async def _no_limit(self, **kwargs):
        return _NoLimitResult()

    async def _noop_enqueue(task_id: str, defer_seconds: float = 0.0) -> None:
        return None

    async def _fake_credentials(db, *, user_id, provider):
        return {"username": "mock-user", "password": "mock-password"}

    class SeatUnavailableProvider:
        provider_name = "SRT"

        async def login(self, **kwargs):
            return ProviderOutcome(ok=True)

        async def search(self, **kwargs):
            departure = _utc_now() + timedelta(minutes=30)
            schedule = ProviderSchedule(
                schedule_id="srt-unavailable",
                provider="SRT",
                dep=kwargs["dep"],
                arr=kwargs["arr"],
                departure_at=departure,
                arrival_at=departure + timedelta(minutes=120),
                train_no="S500",
                availability={"general": False, "special": False},
            )
            return ProviderOutcome(ok=True, data={"schedules": [schedule]})

        async def reserve(self, **kwargs):
            return ProviderOutcome(ok=False)

        async def pay(self, **kwargs):
            return ProviderOutcome(ok=False)

        async def cancel(self, **kwargs):
            return ProviderOutcome(ok=False)

    monkeypatch.setattr("app.modules.train.worker.enqueue_train_task", _noop_enqueue)
    monkeypatch.setattr("app.modules.train.worker.RedisTokenBucketLimiter.acquire_provider_call", _no_limit)
    monkeypatch.setattr("app.modules.train.worker._load_provider_credentials", _fake_credentials)
    monkeypatch.setattr("app.modules.train.worker.get_provider_client", lambda provider: SeatUnavailableProvider())

    user = User(
        email="poll-next-run@example.com",
        password_hash="x",
        display_name="Polling User",
        role_id=2,
    )
    db_session.add(user)
    await db_session.flush()

    deadline = _utc_now() + timedelta(minutes=60)
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
            "selected_trains_ranked": [
                {"schedule_id": "srt-unavailable", "departure_at": (_utc_now() + timedelta(minutes=30)).isoformat(), "rank": 1}
            ],
            "passengers": {"adults": 1, "children": 0},
            "seat_class": "general",
            "auto_pay": False,
        },
        idempotency_key="poll-next-run",
    )
    db_session.add(task)
    await db_session.commit()

    await run_train_task({"db_factory": db_session_factory, "redis": fake_redis}, str(task.id))

    async with db_session_factory() as verify_session:
        refreshed = await verify_session.get(Task, task.id)
        assert refreshed is not None
        assert refreshed.state == "POLLING"
        assert refreshed.spec_json.get("next_run_at")


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


def test_parse_srt_search_response_accepts_numeric_train_code():
    raw = {
        "outDataSets": {
            "dsOutput0": [{"strResult": "SUCC", "msgTxt": "ok"}],
            "dsOutput1": [
                {
                    "stlbTrnClsfCd": 17,
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
