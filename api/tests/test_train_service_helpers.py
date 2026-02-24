from __future__ import annotations

from datetime import date, datetime, timedelta, timezone
from types import SimpleNamespace
from uuid import uuid4

import pytest
from fastapi import HTTPException

from app.db.models import Artifact, Task
from app.modules.train import service as train_service
from app.modules.train.providers.base import ProviderOutcome
from app.modules.train.schemas import RankedTrainSelection, TrainPassengers, TrainTaskCreateRequest
from app.modules.train.timezone import KST


def _make_task(
    *,
    state: str = "QUEUED",
    spec_json: dict | None = None,
    deadline_at: datetime | None = None,
    completed_at: datetime | None = None,
    failed_at: datetime | None = None,
    cancelled_at: datetime | None = None,
    paused_at: datetime | None = None,
) -> Task:
    now = datetime.now(timezone.utc)
    return Task(
        user_id=uuid4(),
        module="train",
        state=state,
        deadline_at=deadline_at or (now + timedelta(hours=2)),
        spec_json=spec_json or {},
        idempotency_key=uuid4().hex,
        created_at=now,
        updated_at=now,
        paused_at=paused_at,
        completed_at=completed_at,
        failed_at=failed_at,
        cancelled_at=cancelled_at,
    )


def _make_ticket_artifact(*, created_at: datetime, data: dict | None = None) -> Artifact:
    return Artifact(
        task_id=uuid4(),
        module="train",
        kind="ticket",
        data_json_safe=data or {},
        created_at=created_at,
    )


def test_datetime_parsing_and_normalization_helpers():
    assert train_service._parse_iso_datetime(None) is None
    assert train_service._parse_iso_datetime("bad") is None

    naive = train_service._parse_iso_datetime("2026-02-22T12:00:00")
    assert naive is not None
    assert naive.tzinfo == timezone.utc

    aware = train_service._parse_iso_datetime("2026-02-22T12:00:00+09:00")
    assert aware is not None
    assert aware.utcoffset() == timedelta(hours=9)

    utc_naive = train_service._as_aware_utc_datetime(datetime(2026, 2, 22, 12, 0, 0))
    assert utc_naive.tzinfo == timezone.utc

    converted = train_service._as_aware_utc_datetime(datetime(2026, 2, 22, 21, 0, 0, tzinfo=KST))
    assert converted.tzinfo == timezone.utc
    assert converted.hour == 12


def test_compute_retry_now_status_covers_all_major_branches():
    now = datetime(2026, 2, 22, 12, 0, tzinfo=timezone.utc)

    paused = _make_task(state="PAUSED")
    assert train_service._compute_retry_now_status(paused, now=now) == (False, "paused_use_resume", None)

    terminal = _make_task(state="FAILED")
    assert train_service._compute_retry_now_status(terminal, now=now) == (False, "terminal_state", None)

    running = _make_task(state="RUNNING")
    assert train_service._compute_retry_now_status(running, now=now) == (False, "task_running", None)

    other_state = _make_task(state="RESERVED")
    assert train_service._compute_retry_now_status(other_state, now=now) == (False, "not_eligible_state", None)

    expired_deadline = _make_task(state="QUEUED", deadline_at=now - timedelta(seconds=1))
    assert train_service._compute_retry_now_status(expired_deadline, now=now) == (False, "deadline_passed", None)

    no_retry = _make_task(state="QUEUED", deadline_at=now + timedelta(hours=1), spec_json={})
    assert train_service._compute_retry_now_status(no_retry, now=now) == (True, None, None)

    cooldown_active = _make_task(
        state="POLLING",
        deadline_at=now + timedelta(hours=1),
        spec_json={train_service.MANUAL_RETRY_LAST_AT_KEY: (now - timedelta(seconds=5)).isoformat()},
    )
    allowed, reason, available_at = train_service._compute_retry_now_status(cooldown_active, now=now)
    assert allowed is False
    assert reason == "cooldown_active"
    assert available_at == now + timedelta(seconds=10)

    cooldown_passed = _make_task(
        state="POLLING",
        deadline_at=now + timedelta(hours=1),
        spec_json={train_service.MANUAL_RETRY_LAST_AT_KEY: (now - timedelta(seconds=30)).isoformat()},
    )
    assert train_service._compute_retry_now_status(cooldown_passed, now=now) == (True, None, None)


def test_build_task_attempt_sanitizes_metadata_and_defaults_finished_at(monkeypatch):
    fixed_now = datetime(2026, 2, 22, 0, 0, tzinfo=timezone.utc)
    monkeypatch.setattr(train_service, "utc_now", lambda: fixed_now)
    monkeypatch.setattr(train_service, "validate_safe_metadata", lambda payload: {"sanitized": payload})

    attempt = train_service._build_task_attempt(
        task_id=uuid4(),
        action="PAY",
        provider="SRT",
        ok=True,
        retryable=False,
        error_code=None,
        error_message_safe=None,
        duration_ms=123,
        meta_json_safe={"token": "secret"},
        started_at=fixed_now,
    )
    assert attempt.meta_json_safe == {"sanitized": {"token": "secret"}}
    assert attempt.finished_at == fixed_now

    attempt_no_meta = train_service._build_task_attempt(
        task_id=uuid4(),
        action="SEARCH",
        provider="KTX",
        ok=False,
        retryable=True,
        error_code="timeout",
        error_message_safe="safe",
        duration_ms=5,
        meta_json_safe=None,
        started_at=fixed_now,
        finished_at=fixed_now + timedelta(seconds=1),
    )
    assert attempt_no_meta.meta_json_safe is None
    assert attempt_no_meta.finished_at == fixed_now + timedelta(seconds=1)


def test_provider_credential_helpers_and_recent_verification_logic(monkeypatch):
    assert train_service._credential_missing_code("SRT") == "srt_credentials_missing"
    assert train_service._credential_missing_code("KTX") == "ktx_credentials_missing"

    assert train_service._parse_verified_at(None) is None
    assert train_service._parse_verified_at("bad") is None
    parsed = train_service._parse_verified_at("2026-02-22T12:00:00+00:00")
    assert parsed is not None

    fixed_now = datetime(2026, 2, 22, 12, 0, tzinfo=timezone.utc)
    monkeypatch.setattr(train_service, "utc_now", lambda: fixed_now)
    monkeypatch.setattr(train_service.settings, "train_credential_cache_seconds", 60)

    assert train_service._is_recent_verification(None) is False
    assert train_service._is_recent_verification(fixed_now - timedelta(seconds=30)) is True
    assert train_service._is_recent_verification(fixed_now - timedelta(seconds=90)) is False

    monkeypatch.setattr(train_service.settings, "train_credential_cache_seconds", 0)
    assert train_service._is_recent_verification(fixed_now) is False


def test_ranked_train_and_spec_helpers_cover_provider_validation_and_deadline_rules():
    dep_time_1 = datetime(2026, 2, 23, 8, 0, tzinfo=KST)
    dep_time_2 = datetime(2026, 2, 23, 8, 30, tzinfo=KST)
    arr_time_2 = datetime(2026, 2, 23, 11, 10, tzinfo=KST)
    payload = TrainTaskCreateRequest(
        provider=None,
        dep="수서",
        arr="부산",
        date=date(2026, 2, 23),
        selected_trains_ranked=[
            RankedTrainSelection(
                schedule_id="KTX-100",
                departure_at=dep_time_2,
                arrival_at=arr_time_2,
                rank=2,
                provider="KTX",
            ),
            RankedTrainSelection(schedule_id="SRT-200", departure_at=dep_time_1, rank=1, provider="SRT"),
        ],
        passengers=TrainPassengers(adults=1, children=0),
        seat_class="general",
        auto_pay=False,
        notify=True,
    )

    ranked = train_service._sorted_ranked_trains(payload)
    assert [row["schedule_id"] for row in ranked] == ["SRT-200", "KTX-100"]
    assert ranked[0].get("arrival_at") is None
    assert ranked[1].get("arrival_at") == arr_time_2.isoformat()

    providers = train_service._resolve_task_providers(ranked)
    assert providers == ["KTX", "SRT"]

    spec = train_service.normalize_task_spec(payload, ranked_trains=ranked)
    assert spec["dep_srt_code"] == "0551"
    assert spec["arr_srt_code"] == "0020"
    assert spec["provider"] is None
    assert spec["providers"] == ["KTX", "SRT"]
    assert spec["passengers"] == {"adults": 1, "children": 0}

    deadline = train_service.compute_deadline_from_spec(spec)
    assert deadline == dep_time_1

    key1 = train_service.compute_idempotency_key(uuid4(), spec)
    key2 = train_service.compute_idempotency_key(uuid4(), spec)
    assert key1 != key2
    assert len(key1) == 64

    with pytest.raises(ValueError, match="selected_trains_ranked cannot be empty"):
        train_service.compute_deadline_from_spec({"selected_trains_ranked": []})

    naive_deadline = train_service.compute_deadline_from_spec(
        {"selected_trains_ranked": [{"departure_at": "2026-02-23T09:00:00"}]}
    )
    assert naive_deadline.tzinfo == KST


def test_ranked_train_helper_raises_for_missing_or_mismatched_providers():
    base_kwargs = {
        "dep": "수서",
        "arr": "부산",
        "date": date(2026, 2, 23),
        "passengers": TrainPassengers(adults=1, children=0),
        "seat_class": "general",
        "auto_pay": False,
        "notify": False,
    }
    with pytest.raises(HTTPException, match="include provider metadata"):
        train_service._sorted_ranked_trains(
            TrainTaskCreateRequest(
                provider=None,
                selected_trains_ranked=[
                    RankedTrainSelection(
                        schedule_id="NO-PROVIDER",
                        departure_at=datetime(2026, 2, 23, 9, 0, tzinfo=KST),
                        rank=1,
                        provider=None,
                    )
                ],
                **base_kwargs,
            )
        )

    with pytest.raises(HTTPException, match="different provider"):
        train_service._sorted_ranked_trains(
            TrainTaskCreateRequest(
                provider="SRT",
                selected_trains_ranked=[
                    RankedTrainSelection(
                        schedule_id="MISMATCH",
                        departure_at=datetime(2026, 2, 23, 9, 0, tzinfo=KST),
                        rank=1,
                        provider="KTX",
                    )
                ],
                **base_kwargs,
            )
        )

    with pytest.raises(HTTPException, match="No provider could be resolved"):
        train_service._resolve_task_providers([{"provider": None}])


def test_ticket_summary_and_active_listing_helpers():
    assert train_service._ticket_summary_from_artifact(None) is None

    deadline = datetime(2026, 2, 23, 10, 0, tzinfo=timezone.utc)
    artifact = _make_ticket_artifact(
        created_at=datetime(2026, 2, 22, 10, 0, tzinfo=timezone.utc),
        data={
            "status": "awaiting_payment",
            "paid": False,
            "payment_deadline_at": deadline.isoformat(),
            "reservation_id": "PNR-1",
            "train_no": "301",
            "seat_count": 2,
            "tickets": [
                {"car_no": "8", "seat_no": "12A"},
                {"car_no": "8", "seat_no": "12A"},
                {"seat_no": "14A"},
            ],
        },
    )
    summary = train_service._ticket_summary_from_artifact(artifact)
    assert summary == {
        "ticket_status": "awaiting_payment",
        "ticket_paid": False,
        "ticket_payment_deadline_at": deadline,
        "ticket_reservation_id": "PNR-1",
        "ticket_train_no": "301",
        "ticket_seat_count": 2,
        "ticket_seats": ["8-12A", "14A"],
    }

    assert train_service._is_manual_payment_pending(summary) is True
    assert train_service._is_manual_payment_pending({"ticket_status": "waiting", "ticket_paid": False}) is True
    assert train_service._is_manual_payment_pending({"ticket_status": "paid", "ticket_paid": True}) is False
    assert train_service._is_manual_payment_pending(None) is False
    assert train_service._is_waitlisted_unpaid({"ticket_status": "waiting", "ticket_paid": False}) is True  # noqa: SLF001
    assert train_service._is_waitlisted_unpaid({"ticket_status": "awaiting_payment", "ticket_paid": False}) is False  # noqa: SLF001
    assert train_service._should_refresh_pending_ticket_status({"ticket_status": "awaiting_payment", "ticket_paid": False}) is True  # noqa: SLF001
    assert train_service._should_refresh_pending_ticket_status({"ticket_status": "waiting", "ticket_paid": False}) is False  # noqa: SLF001

    active_task = _make_task(state="POLLING")
    completed_task = _make_task(state="COMPLETED")
    assert train_service._is_active_for_listing(active_task, summary) is True
    assert train_service._is_active_for_listing(completed_task, summary) is True
    assert train_service._is_active_for_listing(completed_task, {"ticket_status": "paid", "ticket_paid": True}) is False


def test_terminal_visibility_refresh_and_latest_ticket_helpers(monkeypatch):
    fixed_now = datetime(2026, 2, 22, 12, 0, tzinfo=timezone.utc)
    monkeypatch.setattr(train_service, "utc_now", lambda: fixed_now)

    assert train_service._should_refresh_ticket_artifact({}, force=True) is True
    assert train_service._should_refresh_ticket_artifact({}, force=False) is True
    assert train_service._should_refresh_ticket_artifact(
        {"last_provider_sync_at": (fixed_now - timedelta(seconds=5)).isoformat()},
        force=False,
    )
    assert train_service._should_refresh_ticket_artifact(
        {"last_provider_sync_at": (fixed_now - timedelta(seconds=11)).isoformat()},
        force=False,
    )

    non_terminal = _make_task(state="QUEUED")
    assert train_service._is_terminal_task_expired_for_visibility(non_terminal) is False

    recent_terminal = _make_task(state="COMPLETED", completed_at=fixed_now - timedelta(days=1))
    assert train_service._is_terminal_task_expired_for_visibility(recent_terminal) is False

    old_terminal = _make_task(state="FAILED", failed_at=fixed_now - timedelta(days=500))
    assert train_service._is_terminal_task_expired_for_visibility(old_terminal) is True

    now = datetime(2026, 2, 22, 13, 0, tzinfo=timezone.utc)
    older_ticket = _make_ticket_artifact(created_at=now - timedelta(hours=1), data={"reservation_id": "old"})
    newer_ticket = _make_ticket_artifact(created_at=now, data={"reservation_id": "new"})
    non_ticket = Artifact(
        task_id=uuid4(),
        module="train",
        kind="reservation",
        data_json_safe={},
        created_at=now + timedelta(hours=1),
    )
    task = _make_task(state="COMPLETED")
    task.artifacts = [older_ticket, non_ticket, newer_ticket]

    assert train_service._latest_ticket_artifact_for_task(task) == newer_ticket

    no_ticket_task = _make_task(state="COMPLETED")
    no_ticket_task.artifacts = [non_ticket]
    assert train_service._latest_ticket_artifact_for_task(no_ticket_task) is None


@pytest.mark.asyncio
async def test_cancel_provider_reservation_status_mapping(monkeypatch):
    user = SimpleNamespace(id=uuid4())

    class _Client:
        def __init__(self, outcome: ProviderOutcome) -> None:
            self._outcome = outcome

        async def cancel(self, *, artifact_data: dict, user_id: str) -> ProviderOutcome:  # noqa: ARG002
            return self._outcome

    async def _client_ok(*args, **kwargs):  # noqa: ANN002, ANN003
        return _Client(ProviderOutcome(ok=True))

    monkeypatch.setattr(train_service, "_get_logged_in_provider_client", _client_ok)
    cancelled = await train_service.cancel_provider_reservation(
        db=None,  # type: ignore[arg-type]
        user=user,  # type: ignore[arg-type]
        provider="SRT",
        reservation_id="PNR-1",
    )
    assert cancelled.status == "cancelled"

    async def _client_not_found(*args, **kwargs):  # noqa: ANN002, ANN003
        return _Client(
            ProviderOutcome(
                ok=False,
                error_code="reservation_not_found",
                error_message_safe="not found",
            )
        )

    monkeypatch.setattr(train_service, "_get_logged_in_provider_client", _client_not_found)
    not_found = await train_service.cancel_provider_reservation(
        db=None,  # type: ignore[arg-type]
        user=user,  # type: ignore[arg-type]
        provider="SRT",
        reservation_id="PNR-2",
    )
    assert not_found.status == "not_found"
    assert not_found.detail == "not found"

    async def _client_failed(*args, **kwargs):  # noqa: ANN002, ANN003
        return _Client(ProviderOutcome(ok=False, error_code="provider_error", error_message_safe="failed"))

    monkeypatch.setattr(train_service, "_get_logged_in_provider_client", _client_failed)
    failed = await train_service.cancel_provider_reservation(
        db=None,  # type: ignore[arg-type]
        user=user,  # type: ignore[arg-type]
        provider="KTX",
        reservation_id="PNR-3",
    )
    assert failed.status == "failed"
    assert failed.detail == "failed"


def test_list_station_options_exposes_station_metadata():
    stations = train_service.list_station_options().stations
    assert stations
    suseo = next((station for station in stations if station.name == "수서"), None)
    assert suseo is not None
    assert suseo.srt_supported is True
