from __future__ import annotations

from datetime import datetime, timedelta, timezone
from types import SimpleNamespace
from uuid import uuid4

import pytest
from fastapi import HTTPException

from app.db.models import Artifact, Task
from app.modules.train import service as train_service
from app.modules.train.providers.base import ProviderOutcome
from app.modules.train.schemas import KTXCredentialsSetRequest, TaskSummaryOut


@pytest.fixture(autouse=True)
def _enable_payment_for_service_remaining(monkeypatch):
    monkeypatch.setattr(train_service.settings, "payment_enabled", True)


class _ExecResult:
    def __init__(self, *, scalar=None, scalars=None, rows=None):  # noqa: ANN001
        self._scalar = scalar
        self._scalars = list(scalars or [])
        self._rows = list(rows or [])

    def scalar_one_or_none(self):  # noqa: ANN201
        return self._scalar

    def all(self):  # noqa: ANN201
        return list(self._rows)

    def scalars(self):  # noqa: ANN201
        return SimpleNamespace(all=lambda: list(self._scalars))


class _DB:
    def __init__(self, *, execute_results: list[_ExecResult] | None = None, bind_name: str | None = None) -> None:
        self._execute_results = list(execute_results or [])
        self.bind = (
            SimpleNamespace(dialect=SimpleNamespace(name=bind_name))
            if bind_name is not None
            else None
        )
        self.commits = 0
        self.added: list[object] = []
        self.refreshed: list[object] = []

    async def execute(self, _stmt):  # noqa: ANN001
        if not self._execute_results:
            return _ExecResult()
        return self._execute_results.pop(0)

    async def commit(self) -> None:
        self.commits += 1

    async def refresh(self, obj: object) -> None:
        self.refreshed.append(obj)

    def add(self, obj: object) -> None:
        self.added.append(obj)


def _user():
    return SimpleNamespace(id=uuid4())


def _summary() -> TaskSummaryOut:
    now = datetime.now(timezone.utc)
    return TaskSummaryOut(
        id=uuid4(),
        module="train",
        state="COMPLETED",
        deadline_at=now + timedelta(hours=1),
        created_at=now,
        updated_at=now,
        paused_at=None,
        cancelled_at=None,
        completed_at=now,
        failed_at=None,
        hidden_at=None,
        last_attempt_at=None,
        last_attempt_action=None,
        last_attempt_ok=None,
        last_attempt_error_code=None,
        last_attempt_error_message_safe=None,
        last_attempt_finished_at=None,
        next_run_at=None,
        retry_now_allowed=False,
        retry_now_reason=None,
        retry_now_available_at=None,
        spec_json={},
        ticket_status=None,
        ticket_paid=None,
        ticket_payment_deadline_at=None,
        ticket_reservation_id=None,
    )


def _task(*, state: str = "COMPLETED", completed_at: datetime | None = None) -> Task:
    now = datetime.now(timezone.utc)
    task = Task(
        user_id=uuid4(),
        module="train",
        state=state,
        deadline_at=now + timedelta(hours=2),
        spec_json={},
        idempotency_key=f"task-{uuid4().hex}",
    )
    task.id = uuid4()
    task.created_at = now
    task.updated_at = now
    task.completed_at = completed_at
    task.hidden_at = None
    task.paused_at = None
    task.cancelled_at = None
    task.failed_at = None
    task.artifacts = []
    task.attempts = []
    return task


def _ticket_artifact(data: dict) -> Artifact:
    return Artifact(
        task_id=uuid4(),
        module="train",
        kind="ticket",
        data_json_safe=data,
    )


@pytest.mark.asyncio
async def test_service_remaining_credential_and_status_branches(monkeypatch):
    db = _DB()
    user = _user()

    async def _load_verified(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {"username": "u", "password": "p", "verified_at": "2026-02-22T12:00:00+00:00"}

    monkeypatch.setattr(train_service, "_load_provider_credentials", _load_verified)
    monkeypatch.setattr(train_service, "_is_recent_verification", lambda _v: False)
    monkeypatch.setattr(train_service, "get_provider_client", lambda _provider: SimpleNamespace())

    async def _raise_wait_for(_coro, timeout: float):  # noqa: ANN001
        _ = timeout
        raise RuntimeError("transport")

    monkeypatch.setattr(train_service.asyncio, "wait_for", _raise_wait_for)
    verified_exception = await train_service._verify_provider_credentials(db, user=user, provider="KTX")
    assert verified_exception.verified is True

    async def _load_unverified(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {"username": "u", "password": "p", "verified_at": ""}

    monkeypatch.setattr(train_service, "_load_provider_credentials", _load_unverified)
    unverified_exception = await train_service._verify_provider_credentials(db, user=user, provider="KTX")
    assert unverified_exception.verified is False
    assert "login check failed" in str(unverified_exception.detail).lower()

    async def _load_none(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return None

    monkeypatch.setattr(train_service, "_load_provider_credentials", _load_none)
    status_missing = await train_service._status_from_saved_credentials(db, user=user, provider="SRT")
    assert status_missing.configured is False

    async def _load_no_verified(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {"username": "u", "password": "p", "verified_at": ""}

    monkeypatch.setattr(train_service, "_load_provider_credentials", _load_no_verified)
    status_unverified = await train_service._status_from_saved_credentials(
        db,
        user=user,
        provider="SRT",
        fallback_detail="fallback",
    )
    assert status_unverified.configured is True
    assert status_unverified.verified is False
    assert status_unverified.detail == "fallback"

    class _RetryLoginClient:
        async def login(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=False, retryable=True, error_message_safe="temporary")

    monkeypatch.setattr(train_service, "get_provider_client", lambda _provider: _RetryLoginClient())
    with pytest.raises(HTTPException) as ktx_retry:
        await train_service.set_ktx_credentials(
            db,
            user=user,
            payload=KTXCredentialsSetRequest(username="user", password="pass1234"),
        )
    assert ktx_retry.value.status_code == 502


def test_service_remaining_statement_and_visibility_helpers():
    user = _user()
    stmt_all = train_service._task_list_stmt(user, "all", limit=5)  # noqa: SLF001
    assert stmt_all is not None
    assert train_service._is_terminal_task_expired_for_visibility(_task(state="RUNNING")) is False  # noqa: SLF001

    terminal_no_completed_at = _task(state="FAILED", completed_at=None)
    terminal_no_completed_at.failed_at = None
    terminal_no_completed_at.cancelled_at = None
    assert train_service._is_terminal_task_expired_for_visibility(terminal_no_completed_at) is False  # noqa: SLF001

    terminal_no_timestamps = _task(state="FAILED", completed_at=None)
    terminal_no_timestamps.failed_at = None
    terminal_no_timestamps.cancelled_at = None
    terminal_no_timestamps.updated_at = None
    assert train_service._is_terminal_task_expired_for_visibility(terminal_no_timestamps) is False  # noqa: SLF001


@pytest.mark.asyncio
async def test_service_remaining_map_helpers_and_lookup_branches(monkeypatch):
    task_id = uuid4()
    attempt = SimpleNamespace(task_id=task_id)
    artifact = SimpleNamespace(task_id=task_id)

    db = _DB(
        execute_results=[
            _ExecResult(rows=[(task_id, datetime.now(timezone.utc))]),
            _ExecResult(scalars=[attempt]),
            _ExecResult(scalars=[artifact]),
            _ExecResult(scalar=None),
        ],
        bind_name="postgresql",
    )

    empty_last = await train_service._last_attempt_map(db, [])  # noqa: SLF001
    assert empty_last == {}

    last_map = await train_service._last_attempt_map(db, [task_id])  # noqa: SLF001
    assert task_id in last_map

    empty_latest_attempt = await train_service._latest_attempt_map(db, [])  # noqa: SLF001
    assert empty_latest_attempt == {}
    latest_attempt = await train_service._latest_attempt_map(db, [task_id])  # noqa: SLF001
    assert latest_attempt[task_id] is attempt

    empty_latest_artifact = await train_service._latest_ticket_artifact_map(db, [])  # noqa: SLF001
    assert empty_latest_artifact == {}
    latest_artifact = await train_service._latest_ticket_artifact_map(db, [task_id])  # noqa: SLF001
    assert latest_artifact[task_id] is artifact

    with pytest.raises(HTTPException) as missing_task:
        await train_service.get_task_for_user(db, task_id=uuid4(), user=_user())
    assert missing_task.value.status_code == 404


@pytest.mark.asyncio
async def test_service_remaining_list_refresh_and_refresh_ticket_branches(monkeypatch):
    user = _user()
    now = datetime.now(timezone.utc)

    task_pending = _task(state="COMPLETED", completed_at=now)
    task_pending.id = uuid4()
    task_terminal = _task(state="FAILED", completed_at=now)
    task_terminal.id = uuid4()
    no_artifact_task = _task(state="RUNNING", completed_at=now)
    no_artifact_task.id = uuid4()

    class _ListResult:
        def __init__(self, tasks):  # noqa: ANN001
            self._tasks = tasks

        def scalars(self):  # noqa: ANN201
            return SimpleNamespace(all=lambda: list(self._tasks))

    db = _DB()
    async def _execute(_stmt):  # noqa: ANN001
        return _ListResult([task_pending, task_terminal, no_artifact_task])

    db.execute = _execute  # type: ignore[method-assign]

    pending_artifact = _ticket_artifact({"provider": "SRT", "reservation_id": "PNR-1", "status": "awaiting_payment"})
    terminal_artifact = _ticket_artifact({"provider": "SRT", "reservation_id": "PNR-2", "status": "paid"})

    monkeypatch.setattr(train_service, "_task_list_stmt", lambda *_args, **_kwargs: object())
    async def _latest_attempts(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {}

    monkeypatch.setattr(train_service, "_latest_attempt_map", _latest_attempts)
    async def _latest_artifacts(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {task_pending.id: pending_artifact, task_terminal.id: terminal_artifact}

    monkeypatch.setattr(
        train_service,
        "_latest_ticket_artifact_map",
        _latest_artifacts,
    )
    async def _redis_client():
        return object()

    monkeypatch.setattr(train_service, "get_redis_client", _redis_client)
    monkeypatch.setattr(train_service, "RedisTokenBucketLimiter", lambda _redis: object())
    monkeypatch.setattr(
        train_service,
        "_ticket_summary_from_artifact",
        lambda artifact: {"status": "awaiting_payment"} if artifact is pending_artifact else {"status": "paid"},
    )
    monkeypatch.setattr(train_service, "_is_manual_payment_pending", lambda summary: summary.get("status") == "awaiting_payment")
    monkeypatch.setattr(train_service, "task_to_summary", lambda *_args, **_kwargs: _summary())
    original_refresh = train_service._refresh_ticket_artifact_status

    refresh_calls: list[Artifact] = []

    async def _refresh_stub(*_args, **kwargs):  # noqa: ANN002, ANN003
        refresh_calls.append(kwargs["artifact"])
        return True

    monkeypatch.setattr(train_service, "_refresh_ticket_artifact_status", _refresh_stub)
    response = await train_service.list_tasks(
        db,
        user=user,
        status_filter="all",
        refresh_completed=True,
    )
    assert len(response.tasks) == 3
    assert len(refresh_calls) == 2
    assert db.commits == 1

    # _refresh_ticket_artifact_status branch matrix.
    monkeypatch.setattr(train_service, "_refresh_ticket_artifact_status", original_refresh)
    non_ticket = Artifact(task_id=uuid4(), module="train", kind="receipt", data_json_safe={"provider": "SRT"})
    assert (
        await train_service._refresh_ticket_artifact_status(  # noqa: SLF001
            db,
            user=user,
            artifact=non_ticket,
            limiter=object(),
        )
    ) is False

    invalid_ticket = _ticket_artifact({"provider": "OTHER"})
    assert (
        await train_service._refresh_ticket_artifact_status(  # noqa: SLF001
            db,
            user=user,
            artifact=invalid_ticket,
            limiter=object(),
        )
    ) is False

    should_refresh_ticket = _ticket_artifact({"provider": "SRT", "reservation_id": "PNR-3"})
    monkeypatch.setattr(train_service, "_should_refresh_ticket_artifact", lambda *_args, **_kwargs: True)
    async def _client_login_error(*_args, **_kwargs):  # noqa: ANN002, ANN003
        raise HTTPException(status_code=400, detail="login required")

    monkeypatch.setattr(train_service, "_get_logged_in_provider_client", _client_login_error)
    monkeypatch.setattr(train_service, "validate_safe_metadata", lambda payload: payload)
    refreshed_on_login_error = await train_service._refresh_ticket_artifact_status(  # noqa: SLF001
        db,
        user=user,
        artifact=should_refresh_ticket,
        limiter=object(),
    )
    assert refreshed_on_login_error is True

    fetch_error_ticket = _ticket_artifact({"provider": "SRT", "reservation_id": "PNR-4"})
    async def _client_ok(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return object()

    async def _sync_error(*_args, **_kwargs):  # noqa: ANN002, ANN003
        raise RuntimeError("boom")

    monkeypatch.setattr(train_service, "_get_logged_in_provider_client", _client_ok)
    monkeypatch.setattr(train_service, "fetch_ticket_sync_snapshot", _sync_error)
    refreshed_on_fetch_error = await train_service._refresh_ticket_artifact_status(  # noqa: SLF001
        db,
        user=user,
        artifact=fetch_error_ticket,
        limiter=object(),
    )
    assert refreshed_on_fetch_error is True

    fixed_sync_at = datetime(2026, 2, 22, 12, 0, tzinfo=timezone.utc)
    monkeypatch.setattr(train_service, "utc_now", lambda: fixed_sync_at)
    unchanged_ticket = _ticket_artifact(
        {
            "provider": "SRT",
            "reservation_id": "PNR-5",
            "status": "paid",
            "paid": None,
            "waiting": None,
            "expired": None,
            "payment_deadline_at": None,
            "seat_count": None,
            "tickets": None,
            "reservation_snapshot": None,
            "provider_sync": None,
            "last_provider_sync_at": fixed_sync_at.isoformat(),
        }
    )
    async def _sync_unchanged(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {"status": "paid", "synced_at": fixed_sync_at.isoformat()}

    monkeypatch.setattr(train_service, "fetch_ticket_sync_snapshot", _sync_unchanged)
    refreshed_unchanged = await train_service._refresh_ticket_artifact_status(  # noqa: SLF001
        db,
        user=user,
        artifact=unchanged_ticket,
        limiter=object(),
    )
    assert refreshed_unchanged is False


@pytest.mark.asyncio
async def test_service_remaining_pause_resume_retry_cancel_and_pay_branches(monkeypatch):
    user = _user()
    db = _DB()
    now = datetime.now(timezone.utc)

    terminal_task = _task(state="FAILED")
    async def _terminal_task(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return terminal_task

    monkeypatch.setattr(train_service, "get_task_for_user", _terminal_task)
    with pytest.raises(HTTPException) as pause_terminal:
        await train_service.pause_task(db, task_id=uuid4(), user=user)
    assert pause_terminal.value.status_code == 400

    queued_task = _task(state="QUEUED")
    async def _queued_task(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return queued_task

    monkeypatch.setattr(train_service, "get_task_for_user", _queued_task)
    monkeypatch.setattr(train_service, "task_to_summary", lambda *_args, **_kwargs: _summary())
    paused = await train_service.pause_task(db, task_id=uuid4(), user=user)
    assert queued_task.state == "PAUSED"

    paused_task = _task(state="PAUSED")
    async def _paused_task(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return paused_task

    async def _enqueue(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return None

    monkeypatch.setattr(train_service, "get_task_for_user", _paused_task)
    monkeypatch.setattr(train_service, "enqueue_train_task", _enqueue)
    resumed = await train_service.resume_task(db, task_id=uuid4(), user=user)
    assert paused_task.state == "QUEUED"
    assert paused_task.paused_at is None

    paused_flag_task = _task(state="RUNNING")
    paused_flag_task.paused_at = now

    async def _paused_flag_task(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return paused_flag_task

    monkeypatch.setattr(train_service, "get_task_for_user", _paused_flag_task)
    await train_service.resume_task(db, task_id=uuid4(), user=user)
    assert paused_flag_task.state == "QUEUED"
    assert paused_flag_task.paused_at is None

    terminal_paused_flag_task = _task(state="COMPLETED")
    terminal_paused_flag_task.paused_at = now

    async def _terminal_paused_flag_task(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return terminal_paused_flag_task

    monkeypatch.setattr(train_service, "get_task_for_user", _terminal_paused_flag_task)
    with pytest.raises(HTTPException) as terminal_paused_resume:
        await train_service.resume_task(db, task_id=uuid4(), user=user)
    assert terminal_paused_resume.value.status_code == 400

    not_paused_task = _task(state="RUNNING")
    async def _not_paused_task(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return not_paused_task

    monkeypatch.setattr(train_service, "get_task_for_user", _not_paused_task)
    with pytest.raises(HTTPException) as not_paused:
        await train_service.resume_task(db, task_id=uuid4(), user=user)
    assert not_paused.value.status_code == 400

    retry_task = _task(state="RUNNING")
    async def _retry_task(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return retry_task

    monkeypatch.setattr(train_service, "get_task_for_user", _retry_task)

    monkeypatch.setattr(train_service, "_compute_retry_now_status", lambda *_args, **_kwargs: (False, "terminal_state", None))
    with pytest.raises(HTTPException) as retry_terminal:
        await train_service.retry_task_now(db, task_id=uuid4(), user=user)
    assert retry_terminal.value.status_code == 409

    monkeypatch.setattr(train_service, "_compute_retry_now_status", lambda *_args, **_kwargs: (False, "unknown", None))
    with pytest.raises(HTTPException) as retry_unknown:
        await train_service.retry_task_now(db, task_id=uuid4(), user=user)
    assert retry_unknown.value.status_code == 409

    terminal_cancel_task = _task(state="COMPLETED")
    async def _terminal_cancel_task(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return terminal_cancel_task

    monkeypatch.setattr(train_service, "get_task_for_user", _terminal_cancel_task)
    cancel_terminal = await train_service.cancel_task(db, task_id=uuid4(), user=user)
    assert cancel_terminal.task.state == "COMPLETED"

    active_cancel_task = _task(state="RUNNING")
    async def _active_cancel_task(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return active_cancel_task

    monkeypatch.setattr(train_service, "get_task_for_user", _active_cancel_task)
    cancel_active = await train_service.cancel_task(db, task_id=uuid4(), user=user)
    assert active_cancel_task.state == "CANCELLED"

    async def _redis_obj():
        return object()

    monkeypatch.setattr(train_service, "get_redis_client", _redis_obj)
    monkeypatch.setattr(train_service, "RedisTokenBucketLimiter", lambda _redis: object())
    monkeypatch.setattr(train_service, "validate_safe_metadata", lambda payload: payload)
    monkeypatch.setattr(train_service, "task_to_summary", lambda *_args, **_kwargs: _summary())
    async def _last_attempts(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {}

    monkeypatch.setattr(train_service, "_last_attempt_map", _last_attempts)
    monkeypatch.setattr(train_service, "_ticket_summary_from_artifact", lambda _artifact: {})

    def _pay_task_with_artifact(data: dict) -> Task:
        task = _task(state="COMPLETED")
        task.id = uuid4()
        task.artifacts = [_ticket_artifact(data)]
        return task

    provider_missing_task = _pay_task_with_artifact({"provider": "SRT", "status": "awaiting_payment"})
    async def _provider_missing_task(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return provider_missing_task

    async def _refresh_true(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return True

    monkeypatch.setattr(train_service, "get_task_for_user", _provider_missing_task)
    monkeypatch.setattr(train_service, "_refresh_ticket_artifact_status", _refresh_true)
    with pytest.raises(HTTPException) as provider_missing:
        await train_service.pay_task(db, task_id=uuid4(), user=user)
    assert provider_missing.value.status_code == 400

    cancelled_ticket_task = _pay_task_with_artifact(
        {"provider": "SRT", "reservation_id": "PNR-1", "status": "cancelled", "cancelled": True}
    )
    async def _cancelled_ticket_task(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return cancelled_ticket_task

    monkeypatch.setattr(train_service, "get_task_for_user", _cancelled_ticket_task)
    with pytest.raises(HTTPException) as cancelled_ticket:
        await train_service.pay_task(db, task_id=uuid4(), user=user)
    assert cancelled_ticket.value.status_code == 409

    paid_ticket_task = _pay_task_with_artifact(
        {"provider": "SRT", "reservation_id": "PNR-2", "status": "awaiting_payment", "paid": True}
    )
    async def _paid_ticket_task(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return paid_ticket_task

    monkeypatch.setattr(train_service, "get_task_for_user", _paid_ticket_task)
    paid_ticket = await train_service.pay_task(db, task_id=uuid4(), user=user)
    assert isinstance(paid_ticket.task, TaskSummaryOut)

    expired_status_task = _pay_task_with_artifact(
        {"provider": "SRT", "reservation_id": "PNR-3", "status": "expired", "paid": False}
    )
    async def _expired_status_task(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return expired_status_task

    monkeypatch.setattr(train_service, "get_task_for_user", _expired_status_task)
    with pytest.raises(HTTPException) as expired_status:
        await train_service.pay_task(db, task_id=uuid4(), user=user)
    assert expired_status.value.status_code == 409

    invalid_status_task = _pay_task_with_artifact(
        {"provider": "SRT", "reservation_id": "PNR-4", "status": "unknown", "paid": False}
    )
    async def _invalid_status_task(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return invalid_status_task

    monkeypatch.setattr(train_service, "get_task_for_user", _invalid_status_task)
    with pytest.raises(HTTPException) as invalid_status:
        await train_service.pay_task(db, task_id=uuid4(), user=user)
    assert invalid_status.value.status_code == 409

    deadline_passed_task = _pay_task_with_artifact(
        {
            "provider": "SRT",
            "reservation_id": "PNR-5",
            "status": "awaiting_payment",
            "paid": False,
            "payment_deadline_at": (now - timedelta(minutes=1)).isoformat(),
        }
    )
    async def _deadline_passed_task(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return deadline_passed_task

    monkeypatch.setattr(train_service, "get_task_for_user", _deadline_passed_task)
    with pytest.raises(HTTPException) as deadline_passed:
        await train_service.pay_task(db, task_id=uuid4(), user=user)
    assert deadline_passed.value.status_code == 409

    no_card_task = _pay_task_with_artifact(
        {
            "provider": "SRT",
            "reservation_id": "PNR-6",
            "status": "awaiting_payment",
            "paid": False,
            "payment_deadline_at": (now + timedelta(hours=1)).isoformat(),
        }
    )
    async def _no_card_task(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return no_card_task

    async def _no_card(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return None

    monkeypatch.setattr(train_service, "get_task_for_user", _no_card_task)
    monkeypatch.setattr(train_service, "get_payment_card_for_execution", _no_card)
    with pytest.raises(HTTPException) as no_card:
        await train_service.pay_task(db, task_id=uuid4(), user=user)
    assert no_card.value.status_code == 400
