from __future__ import annotations

from datetime import datetime, timedelta, timezone
from types import SimpleNamespace
from uuid import uuid4

import pytest
from fastapi import HTTPException

from app.db.models import Artifact, Task
from app.modules.train import service as train_service
from app.modules.train.providers.base import ProviderOutcome
from app.modules.train.schemas import TaskSummaryOut


class _Result:
    def __init__(self, scalar_value):  # noqa: ANN001
        self._scalar = scalar_value

    def scalar_one_or_none(self):  # noqa: ANN201
        return self._scalar


class _DB:
    def __init__(self, artifact: Artifact | None = None) -> None:
        self.artifact = artifact
        self.added: list[object] = []
        self.commits = 0

    async def execute(self, _stmt):  # noqa: ANN001
        return _Result(self.artifact)

    def add(self, obj: object) -> None:
        self.added.append(obj)

    async def commit(self) -> None:
        self.commits += 1

    async def refresh(self, _obj: object) -> None:
        return None


class _Limiter:
    async def acquire_provider_call(self, **_kwargs):  # noqa: ANN003
        return SimpleNamespace(waited_ms=0, rounds=1)


def _summary(*, ticket_status: str | None = None, ticket_paid: bool | None = None) -> TaskSummaryOut:
    now = datetime(2026, 2, 22, 12, 0, tzinfo=timezone.utc)
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
        ticket_status=ticket_status,
        ticket_paid=ticket_paid,
        ticket_payment_deadline_at=None,
        ticket_reservation_id=None,
    )


def _task(*, state: str = "COMPLETED") -> Task:
    return Task(
        user_id=uuid4(),
        module="train",
        state=state,
        deadline_at=datetime.now(timezone.utc) + timedelta(hours=1),
        spec_json={},
        idempotency_key=f"task-{uuid4().hex}",
    )


@pytest.mark.asyncio
async def test_pay_task_early_validation_branches(monkeypatch):
    db = _DB()
    user = SimpleNamespace(id=uuid4())

    expired_task = _task(state="EXPIRED")
    async def _task_expired(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return expired_task

    monkeypatch.setattr(train_service, "get_task_for_user", _task_expired)
    with pytest.raises(HTTPException) as expired:
        await train_service.pay_task(db, task_id=uuid4(), user=user)
    assert expired.value.status_code == 400

    no_artifact_task = _task(state="COMPLETED")
    no_artifact_task.artifacts = []
    async def _task_no_artifact(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return no_artifact_task

    monkeypatch.setattr(train_service, "get_task_for_user", _task_no_artifact)
    with pytest.raises(HTTPException) as missing_artifact:
        await train_service.pay_task(db, task_id=uuid4(), user=user)
    assert missing_artifact.value.status_code == 400

    invalid_artifact_task = _task(state="COMPLETED")
    invalid_artifact_task.artifacts = [
        Artifact(task_id=uuid4(), module="train", kind="ticket", data_json_safe={"provider": "SRT", "status": "awaiting_payment"})
    ]
    async def _task_invalid(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return invalid_artifact_task

    async def _redis_obj():
        return object()

    async def _refresh_false(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return False

    monkeypatch.setattr(train_service, "get_task_for_user", _task_invalid)
    monkeypatch.setattr(train_service, "get_redis_client", _redis_obj)
    monkeypatch.setattr(train_service, "RedisTokenBucketLimiter", lambda _redis: _Limiter())
    monkeypatch.setattr(train_service, "_refresh_ticket_artifact_status", _refresh_false)
    with pytest.raises(HTTPException) as invalid_provider_data:
        await train_service.pay_task(db, task_id=uuid4(), user=user)
    assert invalid_provider_data.value.status_code == 400

    cancelled_artifact_task = _task(state="COMPLETED")
    cancelled_artifact_task.artifacts = [
        Artifact(
            task_id=uuid4(),
            module="train",
            kind="ticket",
            data_json_safe={
                "provider": "SRT",
                "reservation_id": "PNR-1",
                "status": "cancelled",
                "cancelled": True,
            },
        )
    ]
    async def _task_cancelled(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return cancelled_artifact_task

    monkeypatch.setattr(train_service, "get_task_for_user", _task_cancelled)
    with pytest.raises(HTTPException) as cancelled:
        await train_service.pay_task(db, task_id=uuid4(), user=user)
    assert cancelled.value.status_code == 409


@pytest.mark.asyncio
async def test_pay_task_paid_missing_card_and_payment_failure_paths(monkeypatch):
    user = SimpleNamespace(id=uuid4())
    db = _DB()
    task = _task(state="COMPLETED")
    artifact = Artifact(
        task_id=uuid4(),
        module="train",
        kind="ticket",
        data_json_safe={
            "provider": "SRT",
            "reservation_id": "PNR-2",
            "status": "awaiting_payment",
            "paid": True,
        },
    )
    task.artifacts = [artifact]

    async def _task_lookup(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return task

    async def _last_attempts(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {task.id: datetime.now(timezone.utc)}

    async def _redis_obj():
        return object()

    async def _refresh_false(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return False

    monkeypatch.setattr(train_service, "get_task_for_user", _task_lookup)
    monkeypatch.setattr(train_service, "_last_attempt_map", _last_attempts)
    monkeypatch.setattr(train_service, "_ticket_summary_from_artifact", lambda _artifact: {"ticket_status": "paid", "ticket_paid": True})
    monkeypatch.setattr(train_service, "task_to_summary", lambda *_args, **_kwargs: _summary(ticket_status="paid", ticket_paid=True))
    monkeypatch.setattr(train_service, "get_redis_client", _redis_obj)
    monkeypatch.setattr(train_service, "RedisTokenBucketLimiter", lambda _redis: _Limiter())
    monkeypatch.setattr(train_service, "_refresh_ticket_artifact_status", _refresh_false)
    paid_result = await train_service.pay_task(db, task_id=uuid4(), user=user)
    assert paid_result.task.ticket_paid is True

    artifact.data_json_safe = {
        "provider": "SRT",
        "reservation_id": "PNR-2",
        "status": "awaiting_payment",
        "paid": False,
        "payment_deadline_at": (datetime.now(timezone.utc) + timedelta(minutes=30)).isoformat(),
    }
    async def _no_card(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return None

    monkeypatch.setattr(train_service, "get_payment_card_for_execution", _no_card)
    with pytest.raises(HTTPException) as no_card:
        await train_service.pay_task(db, task_id=uuid4(), user=user)
    assert no_card.value.status_code == 400

    class _Client:
        async def pay(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=False, retryable=True, error_message_safe="gateway timeout")

    async def _card_ok(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {"card_number": "4111"}

    async def _client_ok(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return _Client()

    monkeypatch.setattr(train_service, "get_payment_card_for_execution", _card_ok)
    monkeypatch.setattr(train_service, "_get_logged_in_provider_client", _client_ok)
    monkeypatch.setattr(train_service, "validate_safe_metadata", lambda payload: payload)
    with pytest.raises(HTTPException) as pay_fail:
        await train_service.pay_task(db, task_id=uuid4(), user=user)
    assert pay_fail.value.status_code == 502
    assert db.added
    assert db.commits >= 1


@pytest.mark.asyncio
async def test_cancel_ticket_branch_matrix(monkeypatch):
    user = SimpleNamespace(id=uuid4())

    db_none = _DB(artifact=None)
    not_found = await train_service.cancel_ticket(db_none, artifact_id=uuid4(), user=user)
    assert not_found.status == "not_found"

    cancelled_artifact = Artifact(
        task_id=uuid4(),
        module="train",
        kind="ticket",
        data_json_safe={"provider": "SRT", "reservation_id": "PNR-3", "cancelled": True},
    )
    db_cancelled = _DB(artifact=cancelled_artifact)
    already = await train_service.cancel_ticket(db_cancelled, artifact_id=uuid4(), user=user)
    assert already.status == "already_cancelled"

    unsupported_artifact = Artifact(
        task_id=uuid4(),
        module="train",
        kind="ticket",
        data_json_safe={"provider": "OTHER", "reservation_id": "PNR-4"},
    )
    db_unsupported = _DB(artifact=unsupported_artifact)
    unsupported = await train_service.cancel_ticket(db_unsupported, artifact_id=uuid4(), user=user)
    assert unsupported.status == "not_supported"

    base_artifact = Artifact(
        task_id=uuid4(),
        module="train",
        kind="ticket",
        data_json_safe={"provider": "SRT", "reservation_id": "PNR-5"},
    )
    db = _DB(artifact=base_artifact)
    async def _redis_obj():
        return object()

    async def _refresh_false(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return False

    monkeypatch.setattr(train_service, "get_redis_client", _redis_obj)
    monkeypatch.setattr(train_service, "RedisTokenBucketLimiter", lambda _redis: _Limiter())
    monkeypatch.setattr(train_service, "_refresh_ticket_artifact_status", _refresh_false)
    monkeypatch.setattr(train_service, "_build_task_attempt", lambda **_kwargs: SimpleNamespace())  # noqa: ANN003
    monkeypatch.setattr(train_service, "validate_safe_metadata", lambda payload: payload)

    class _NotSupportedClient:
        async def cancel(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=False, error_code="not_supported", error_message_safe="no cancel")

    async def _client_not_supported(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return _NotSupportedClient()

    monkeypatch.setattr(train_service, "_get_logged_in_provider_client", _client_not_supported)
    not_supported = await train_service.cancel_ticket(db, artifact_id=uuid4(), user=user)
    assert not_supported.status == "not_supported"

    class _ReservationGoneClient:
        async def cancel(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=False, error_code="reservation_not_found", error_message_safe="gone")

    async def _client_gone(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return _ReservationGoneClient()

    monkeypatch.setattr(train_service, "_get_logged_in_provider_client", _client_gone)
    gone = await train_service.cancel_ticket(db, artifact_id=uuid4(), user=user)
    assert gone.status == "not_found"

    class _CancelFailClient:
        async def cancel(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=False, error_code="cancel_failed", error_message_safe="failed")

    async def _client_fail(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return _CancelFailClient()

    monkeypatch.setattr(train_service, "_get_logged_in_provider_client", _client_fail)
    with pytest.raises(HTTPException) as cancel_fail:
        await train_service.cancel_ticket(db, artifact_id=uuid4(), user=user)
    assert cancel_fail.value.status_code == 400

    class _CancelSuccessClient:
        async def cancel(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(ok=True, data={})

    async def _client_success(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return _CancelSuccessClient()

    monkeypatch.setattr(train_service, "_get_logged_in_provider_client", _client_success)
    cancelled = await train_service.cancel_ticket(db, artifact_id=uuid4(), user=user)
    assert cancelled.status == "cancelled"


@pytest.mark.asyncio
async def test_pay_task_success_sync_error_and_sync_merge_paths(monkeypatch):
    user = SimpleNamespace(id=uuid4())
    now = datetime(2026, 2, 22, 12, 0, tzinfo=timezone.utc)
    task = _task(state="FAILED")
    artifact = Artifact(
        task_id=task.id,
        module="train",
        kind="ticket",
        data_json_safe={
            "provider": "SRT",
            "reservation_id": "PNR-10",
            "status": "awaiting_payment",
            "paid": False,
            "provider_http": {},
        },
    )
    task.artifacts = [artifact]
    db = _DB()

    async def _task_lookup(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return task

    async def _redis_obj():
        return object()

    async def _refresh_false(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return False

    async def _card_ok(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {
            "card_number": "4111111111111111",
            "card_password": "12",
            "validation_number": "900101",
            "card_expire": "2501",
        }

    class _Client:
        async def pay(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(
                ok=True,
                data={
                    "payment_id": "PAY-1",
                    "ticket_no": "TICKET-1",
                    "http_trace": {"request": {"pnrNo": "PNR-10"}, "response": {"result": "ok"}},
                },
            )

    async def _client_ok(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return _Client()

    async def _last_attempts(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return {task.id: now}

    monkeypatch.setattr(train_service, "get_task_for_user", _task_lookup)
    monkeypatch.setattr(train_service, "get_redis_client", _redis_obj)
    monkeypatch.setattr(train_service, "RedisTokenBucketLimiter", lambda _redis: _Limiter())
    monkeypatch.setattr(train_service, "_refresh_ticket_artifact_status", _refresh_false)
    monkeypatch.setattr(train_service, "get_payment_card_for_execution", _card_ok)
    monkeypatch.setattr(train_service, "_get_logged_in_provider_client", _client_ok)
    monkeypatch.setattr(train_service, "_last_attempt_map", _last_attempts)
    monkeypatch.setattr(train_service, "_ticket_summary_from_artifact", lambda _artifact: {"ticket_status": "paid", "ticket_paid": True})
    monkeypatch.setattr(train_service, "task_to_summary", lambda *_args, **_kwargs: _summary(ticket_status="paid", ticket_paid=True))
    monkeypatch.setattr(train_service, "validate_safe_metadata", lambda payload: payload)
    monkeypatch.setattr(train_service, "redact_sensitive", lambda payload: payload)

    async def _sync_error(**_kwargs):  # noqa: ANN003
        raise RuntimeError("sync down")

    monkeypatch.setattr(train_service, "fetch_ticket_sync_snapshot", _sync_error)
    result_sync_error = await train_service.pay_task(db, task_id=task.id, user=user)
    assert result_sync_error.task.ticket_paid is True
    assert artifact.data_json_safe["paid"] is True
    assert artifact.data_json_safe["provider_sync"]["pay_sync_error"] == "provider_sync_error:RuntimeError"
    assert artifact.data_json_safe["provider_http"]["pay"]["response"]["result"] == "ok"
    assert task.state == "COMPLETED"
    assert task.failed_at is None
    assert task.completed_at is not None

    task.state = "FAILED"
    task.failed_at = now
    artifact.data_json_safe = {
        "provider": "SRT",
        "reservation_id": "PNR-10",
        "status": "awaiting_payment",
        "paid": False,
        "provider_http": {},
    }

    async def _sync_ok(**_kwargs):  # noqa: ANN003
        return {
            "status": "paid",
            "paid": True,
            "waiting": False,
            "payment_deadline_at": now.isoformat(),
            "seat_count": 1,
            "tickets": [{"seat_no": "1A"}],
            "reservation_snapshot": {"reservation_id": "PNR-10"},
            "provider_sync": {"state": "ok"},
            "provider_http": {"tickets": {"status_code": 200}},
            "synced_at": now.isoformat(),
        }

    monkeypatch.setattr(train_service, "fetch_ticket_sync_snapshot", _sync_ok)
    result_sync_ok = await train_service.pay_task(db, task_id=task.id, user=user)
    assert result_sync_ok.task.ticket_paid is True
    assert artifact.data_json_safe["status"] == "paid"
    assert artifact.data_json_safe["tickets"][0]["seat_no"] == "1A"
    assert artifact.data_json_safe["provider_http"]["tickets"]["status_code"] == 200


@pytest.mark.asyncio
async def test_delete_task_and_cancel_ticket_transport_and_trace_paths(monkeypatch):
    user = SimpleNamespace(id=uuid4())
    db = _DB()

    active_task = _task(state="POLLING")

    async def _active_lookup(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return active_task

    monkeypatch.setattr(train_service, "get_task_for_user", _active_lookup)
    with pytest.raises(HTTPException) as cannot_delete:
        await train_service.delete_task(db, task_id=uuid4(), user=user)
    assert cannot_delete.value.status_code == 400

    terminal_task = _task(state="COMPLETED")

    async def _terminal_lookup(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return terminal_task

    monkeypatch.setattr(train_service, "get_task_for_user", _terminal_lookup)
    monkeypatch.setattr(train_service, "task_to_summary", lambda *_args, **_kwargs: _summary())
    deleted = await train_service.delete_task(db, task_id=uuid4(), user=user)
    assert deleted.task.state == "COMPLETED"
    assert terminal_task.hidden_at is not None

    artifact = Artifact(
        task_id=uuid4(),
        module="train",
        kind="ticket",
        data_json_safe={"provider": "SRT", "reservation_id": "PNR-20"},
    )
    db_transport = _DB(artifact=artifact)

    async def _redis_obj():
        return object()

    async def _refresh_false(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return False

    monkeypatch.setattr(train_service, "get_redis_client", _redis_obj)
    monkeypatch.setattr(train_service, "RedisTokenBucketLimiter", lambda _redis: _Limiter())
    monkeypatch.setattr(train_service, "_refresh_ticket_artifact_status", _refresh_false)
    monkeypatch.setattr(train_service, "_build_task_attempt", lambda **_kwargs: SimpleNamespace())  # noqa: ANN003
    monkeypatch.setattr(train_service, "validate_safe_metadata", lambda payload: payload)
    monkeypatch.setattr(train_service, "redact_sensitive", lambda payload: payload)

    class _ClientRaise:
        async def cancel(self, **_kwargs):  # noqa: ANN003
            raise RuntimeError("transport down")

    async def _client_raise(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return _ClientRaise()

    monkeypatch.setattr(train_service, "_get_logged_in_provider_client", _client_raise)
    with pytest.raises(HTTPException) as transport_error:
        await train_service.cancel_ticket(db_transport, artifact_id=uuid4(), user=user)
    assert transport_error.value.status_code == 502

    class _ClientNotSupportedWithTrace:
        async def cancel(self, **_kwargs):  # noqa: ANN003
            return ProviderOutcome(
                ok=False,
                error_code="not_supported",
                error_message_safe="no cancel",
                data={"http_trace": {"endpoint": "cancel", "status": "not_supported"}},
            )

    async def _client_not_supported(*_args, **_kwargs):  # noqa: ANN002, ANN003
        return _ClientNotSupportedWithTrace()

    monkeypatch.setattr(train_service, "_get_logged_in_provider_client", _client_not_supported)
    not_supported = await train_service.cancel_ticket(db_transport, artifact_id=uuid4(), user=user)
    assert not_supported.status == "not_supported"
    assert artifact.data_json_safe["provider_http"]["cancel"]["endpoint"] == "cancel"
