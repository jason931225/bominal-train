from __future__ import annotations

from datetime import date, datetime, timedelta, timezone
from types import SimpleNamespace
from uuid import uuid4

import pytest
from fastapi import HTTPException

from app.db.models import Task
from app.modules.train import service as train_service
from app.modules.train.providers.base import ProviderOutcome, ProviderSchedule
from app.modules.train.schemas import RankedTrainSelection, TaskSummaryOut, TrainPassengers, TrainSearchRequest, TrainTaskCreateRequest
from app.modules.train.timezone import KST


class _Result:
    def __init__(self, *, scalar_value=None):  # noqa: ANN001
        self._scalar = scalar_value

    def scalar_one_or_none(self):  # noqa: ANN201
        return self._scalar

    def all(self):  # noqa: ANN201
        return []

    def scalars(self):  # noqa: ANN201
        return SimpleNamespace(all=lambda: [])


class _DB:
    def __init__(self, execute_scalars: list[object | None] | None = None) -> None:
        self._execute_scalars = list(execute_scalars or [])
        self.bind = None
        self.added: list[object] = []
        self.commits = 0
        self.refreshed: list[object] = []

    async def execute(self, _stmt):  # noqa: ANN001
        scalar_value = self._execute_scalars.pop(0) if self._execute_scalars else None
        return _Result(scalar_value=scalar_value)

    def add(self, obj: object) -> None:
        self.added.append(obj)

    async def commit(self) -> None:
        self.commits += 1

    async def refresh(self, obj: object) -> None:
        self.refreshed.append(obj)


def _user():
    return SimpleNamespace(id=uuid4())


def _ranked(provider: str = "SRT", schedule_id: str = "SRT-1", rank: int = 1):
    return [
        RankedTrainSelection(
            schedule_id=schedule_id,
            departure_at=datetime(2026, 2, 23, 9, 0, tzinfo=KST) + timedelta(minutes=rank),
            rank=rank,
            provider=provider,
        )
    ]


def _task_payload(
    *,
    dep: str = "수서",
    arr: str = "부산",
    provider: str = "SRT",
    retry_on_expiry: bool = False,
) -> TrainTaskCreateRequest:
    return TrainTaskCreateRequest(
        provider=provider,
        dep=dep,
        arr=arr,
        date=date(2026, 2, 23),
        selected_trains_ranked=_ranked(provider=provider),
        passengers=TrainPassengers(adults=1, children=0),
        seat_class="general",
        auto_pay=False,
        notify=False,
        retry_on_expiry=retry_on_expiry,
    )


def _summary() -> TaskSummaryOut:
    now = datetime(2026, 2, 22, 12, 0, tzinfo=timezone.utc)
    return TaskSummaryOut(
        id=uuid4(),
        module="train",
        state="QUEUED",
        deadline_at=now + timedelta(hours=1),
        created_at=now,
        updated_at=now,
        paused_at=None,
        cancelled_at=None,
        completed_at=None,
        failed_at=None,
        hidden_at=None,
        last_attempt_at=None,
        last_attempt_action=None,
        last_attempt_ok=None,
        last_attempt_error_code=None,
        last_attempt_error_message_safe=None,
        last_attempt_finished_at=None,
        next_run_at=None,
        retry_now_allowed=True,
        retry_now_reason=None,
        retry_now_available_at=None,
        spec_json={},
        ticket_status=None,
        ticket_paid=None,
        ticket_payment_deadline_at=None,
        ticket_reservation_id=None,
    )


@pytest.mark.asyncio
async def test_search_schedules_station_and_provider_error_paths(monkeypatch):
    db = _DB()
    user = _user()

    with pytest.raises(HTTPException) as unknown_station:
        await train_service.search_schedules(
            db,
            payload=TrainSearchRequest(
                providers=["SRT"],
                dep="UNKNOWN",
                arr="부산",
                date=date(2026, 2, 23),
                time_window={"start": "06:00", "end": "12:00"},
            ),
            user=user,
        )
    assert unknown_station.value.status_code == 400

    async def _redis():  # noqa: ANN202
        return object()

    monkeypatch.setattr(train_service, "get_redis_client", _redis)

    class _Limiter:
        async def acquire_provider_call(self, **_kwargs):  # noqa: ANN003
            return SimpleNamespace(waited_ms=0, rounds=1)

    monkeypatch.setattr(train_service, "RedisTokenBucketLimiter", lambda _redis: _Limiter())

    async def _missing_creds(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return None

    monkeypatch.setattr(train_service, "_load_provider_credentials", _missing_creds)
    with pytest.raises(HTTPException) as all_failed:
        await train_service.search_schedules(
            db,
            payload=TrainSearchRequest(
                providers=["SRT"],
                dep="수서",
                arr="부산",
                date=date(2026, 2, 23),
                time_window={"start": "06:00", "end": "12:00"},
            ),
            user=user,
        )
    assert all_failed.value.status_code == 502
    assert "srt_credentials_missing" in str(all_failed.value.detail)

    async def _creds(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {"username": "u", "password": "p", "verified_at": ""}

    monkeypatch.setattr(train_service, "_load_provider_credentials", _creds)

    class _LoginRaises:
        async def login(self, **_kwargs):  # noqa: ANN003
            raise RuntimeError("transport")

    monkeypatch.setattr(train_service, "get_provider_client", lambda _provider: _LoginRaises())
    with pytest.raises(HTTPException) as login_failed:
        await train_service.search_schedules(
            db,
            payload=TrainSearchRequest(
                providers=["SRT"],
                dep="수서",
                arr="부산",
                date=date(2026, 2, 23),
                time_window={"start": "06:00", "end": "12:00"},
            ),
            user=user,
        )
    assert "provider_login_transport_error" in str(login_failed.value.detail)

    class _LoginRejected:
        async def login(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=False, error_code="bad_login", error_message_safe="invalid credentials")

    monkeypatch.setattr(train_service, "get_provider_client", lambda _provider: _LoginRejected())
    with pytest.raises(HTTPException) as login_rejected:
        await train_service.search_schedules(
            db,
            payload=TrainSearchRequest(
                providers=["SRT"],
                dep="수서",
                arr="부산",
                date=date(2026, 2, 23),
                time_window={"start": "06:00", "end": "12:00"},
            ),
            user=user,
        )
    assert "bad_login" in str(login_rejected.value.detail)

    class _SearchRaises:
        async def login(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=True)

        async def search(self, **_kwargs):  # noqa: ANN003
            raise RuntimeError("search transport")

    monkeypatch.setattr(train_service, "get_provider_client", lambda _provider: _SearchRaises())
    with pytest.raises(HTTPException) as search_raises:
        await train_service.search_schedules(
            db,
            payload=TrainSearchRequest(
                providers=["SRT"],
                dep="수서",
                arr="부산",
                date=date(2026, 2, 23),
                time_window={"start": "06:00", "end": "12:00"},
            ),
            user=user,
        )
    assert "provider_transport_error" in str(search_raises.value.detail)


@pytest.mark.asyncio
async def test_search_schedules_success_sorts_provider_schedules_and_filters_invalid_rows(monkeypatch):
    db = _DB()
    user = _user()

    async def _redis():  # noqa: ANN202
        return object()

    monkeypatch.setattr(train_service, "get_redis_client", _redis)

    class _Limiter:
        async def acquire_provider_call(self, **_kwargs):  # noqa: ANN003
            return SimpleNamespace(waited_ms=0, rounds=1)

    monkeypatch.setattr(train_service, "RedisTokenBucketLimiter", lambda _redis: _Limiter())

    async def _creds(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {"username": "u", "password": "p", "verified_at": ""}

    monkeypatch.setattr(train_service, "_load_provider_credentials", _creds)

    class _Provider:
        async def login(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=True)

        async def search(self, **_kwargs):  # noqa: ANN003
            early = ProviderSchedule(
                schedule_id="SRT-1",
                provider="SRT",
                dep="수서",
                arr="부산",
                departure_at=datetime(2026, 2, 23, 8, 0, tzinfo=KST),
                arrival_at=datetime(2026, 2, 23, 10, 0, tzinfo=KST),
                train_no="101",
                availability={"general": True, "special": False},
                metadata={},
            )
            late = ProviderSchedule(
                schedule_id="SRT-2",
                provider="SRT",
                dep="수서",
                arr="부산",
                departure_at=datetime(2026, 2, 23, 9, 0, tzinfo=KST),
                arrival_at=datetime(2026, 2, 23, 11, 0, tzinfo=KST),
                train_no="102",
                availability={"general": True, "special": False},
                metadata={},
            )
            return ProviderOutcome(ok=True, data={"schedules": [late, {"bad": "row"}, early]})

    monkeypatch.setattr(train_service, "get_provider_client", lambda _provider: _Provider())
    result = await train_service.search_schedules(
        db,
        payload=TrainSearchRequest(
            providers=["SRT"],
            dep="수서",
            arr="부산",
            date=date(2026, 2, 23),
            time_window={"start": "06:00", "end": "12:00"},
        ),
        user=user,
    )
    assert [s.schedule_id for s in result.schedules] == ["SRT-1", "SRT-2"]
    assert result.provider_errors == {}


@pytest.mark.asyncio
async def test_create_task_branch_coverage_for_station_credential_dedupe_and_create(monkeypatch):
    user = _user()

    with pytest.raises(HTTPException) as unknown_station:
        await train_service.create_task(_DB(), user=user, payload=_task_payload(dep="UNKNOWN", arr="부산"))
    assert unknown_station.value.status_code == 400

    # KTX-only station with SRT provider should fail station compatibility.
    with pytest.raises(HTTPException) as unsupported_srt_station:
        await train_service.create_task(_DB(), user=user, payload=_task_payload(dep="서울", arr="부산", provider="SRT"))
    assert unsupported_srt_station.value.status_code == 400

    async def _status_not_configured(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return train_service.ProviderCredentialStatus(
            configured=False,
            verified=False,
            detail="missing",
        )

    monkeypatch.setattr(train_service, "_verify_provider_credentials", _status_not_configured)
    with pytest.raises(HTTPException) as missing_credentials:
        await train_service.create_task(_DB(), user=user, payload=_task_payload())
    assert missing_credentials.value.status_code == 400

    async def _status_not_verified(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return train_service.ProviderCredentialStatus(
            configured=True,
            verified=False,
            detail="bad login",
        )

    monkeypatch.setattr(train_service, "_verify_provider_credentials", _status_not_verified)
    with pytest.raises(HTTPException) as bad_credentials:
        await train_service.create_task(_DB(), user=user, payload=_task_payload())
    assert bad_credentials.value.status_code == 400

    async def _status_ok(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return train_service.ProviderCredentialStatus(
            configured=True,
            verified=True,
            detail=None,
        )

    monkeypatch.setattr(train_service, "_verify_provider_credentials", _status_ok)
    monkeypatch.setattr(train_service, "task_to_summary", lambda _task, **_kwargs: _summary())  # noqa: ANN003

    existing = Task(
        user_id=user.id,
        module="train",
        state="QUEUED",
        deadline_at=datetime.now(timezone.utc) + timedelta(hours=1),
        spec_json={},
        idempotency_key="existing-idempotency",
    )
    dedupe_db = _DB(execute_scalars=[existing])
    deduped = await train_service.create_task(dedupe_db, user=user, payload=_task_payload())
    assert deduped.deduplicated is True
    assert deduped.queued is False

    created_task_ids: list[str] = []

    async def _enqueue(task_id: str):  # noqa: ANN001
        created_task_ids.append(task_id)

    monkeypatch.setattr(train_service, "enqueue_train_task", _enqueue)
    create_db = _DB(execute_scalars=[None])
    created = await train_service.create_task(create_db, user=user, payload=_task_payload())
    assert created.deduplicated is False
    assert created.queued is True
    assert create_db.added
    created_task = create_db.added[0]
    assert isinstance(created_task, Task)
    assert created_task.spec_json.get("retry_on_expiry") is False
    assert create_db.commits == 1
    assert create_db.refreshed

    create_retry_db = _DB(execute_scalars=[None])
    created_retry = await train_service.create_task(
        create_retry_db,
        user=user,
        payload=_task_payload(retry_on_expiry=True),
    )
    assert created_retry.deduplicated is False
    assert created_retry.queued is True
    assert create_retry_db.added
    created_retry_task = create_retry_db.added[0]
    assert isinstance(created_retry_task, Task)
    assert created_retry_task.spec_json.get("retry_on_expiry") is True
