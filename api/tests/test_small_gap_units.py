from __future__ import annotations

from datetime import date, datetime, timezone
from types import SimpleNamespace
from uuid import uuid4

import fakeredis.aioredis
import pytest
from argon2.exceptions import InvalidHashError

from app.core import security
from app.core.config import Settings
from app.http.routes.notifications import get_email_status
from app.modules.restaurant.lease import acquire_payment_lease, release_payment_lease
from app.modules.restaurant.providers.scaffold import ScaffoldRestaurantProviderClient
from app.modules.restaurant.queue import enqueue_restaurant_task
from app.modules.restaurant.router import restaurant_health
from app.modules.train.constants import credential_kind
from app.modules.train.schemas import RankedTrainSelection, TimeWindow, TrainTaskCreateRequest
from app.schemas.notification import EmailJobPayload, EmailTemplateJobPayload
from app.schemas.wallet import PaymentCardSetRequest


@pytest.mark.asyncio
async def test_scaffold_provider_methods_return_not_implemented():
    client = ScaffoldRestaurantProviderClient()
    slot = SimpleNamespace(provider_slot_id="slot-1", provider="RESY", restaurant_id="r1")

    outcomes = [
        await client.authenticate_start(account_identifier="u@example.com"),
        await client.authenticate_complete(account_identifier="u@example.com", otp_code="123456"),
        await client.refresh_auth(account_ref="acct"),
        await client.get_user_profile(account_ref="acct"),
        await client.search_availability(
            account_ref="acct",
            restaurant_id="r1",
            party_size=2,
            date_time_local=datetime.now(timezone.utc),
        ),
        await client.create_reservation(account_ref="acct", slot=slot),  # type: ignore[arg-type]
        await client.cancel_reservation(account_ref="acct", restaurant_id="r1", confirmation_number="c1"),
    ]
    assert all(outcome.ok is False for outcome in outcomes)
    assert all(outcome.error_code == "not_implemented" for outcome in outcomes)


@pytest.mark.asyncio
async def test_notification_email_status_and_restaurant_health_return_payloads():
    status = await get_email_status(_=SimpleNamespace())
    assert status.provider
    assert isinstance(status.enabled, bool)

    health = await restaurant_health(_=SimpleNamespace())
    assert health["status"] == "ok"
    assert health["module"] == "restaurant"
    assert health["enabled"] is False


@pytest.mark.asyncio
async def test_restaurant_lease_paths_and_queue_immediate_branch(monkeypatch):
    redis = fakeredis.aioredis.FakeRedis()
    acquired = await acquire_payment_lease(
        redis,
        lease_key="lease:test",
        holder_token="token-a",
        ttl_seconds=10,
    )
    assert acquired is True

    mismatch = await release_payment_lease(redis, lease_key="lease:test", holder_token="token-b")
    assert mismatch is False

    released = await release_payment_lease(redis, lease_key="lease:test", holder_token="token-a")
    assert released is True

    calls: list[tuple] = []

    class _Pool:
        async def enqueue_job(self, *args, **kwargs):  # noqa: ANN002, ANN003
            calls.append((args, kwargs))

    async def _fake_pool():
        return _Pool()

    monkeypatch.setattr("app.modules.restaurant.queue.get_restaurant_queue_pool", _fake_pool)
    task_id = str(uuid4())
    await enqueue_restaurant_task(task_id, defer_seconds=0)
    assert calls[0][0] == ("run_restaurant_task", task_id)
    assert calls[0][1]["_job_id"] == f"restaurant:{task_id}"
    assert "_defer_by" not in calls[0][1]


def test_constants_and_schema_validator_error_paths():
    with pytest.raises(ValueError, match="Unsupported provider"):
        credential_kind("UNKNOWN")

    with pytest.raises(ValueError, match="time_window.start must be <= time_window.end"):
        TimeWindow(start="23:59", end="00:00")

    selection = RankedTrainSelection(
        schedule_id="sched-1",
        departure_at=datetime.now(timezone.utc),
        rank=1,
        provider="SRT",
    )
    with pytest.raises(ValueError, match="ranks must be unique"):
        TrainTaskCreateRequest(
            provider="SRT",
            dep="수서",
            arr="부산",
            date=date(2026, 2, 23),
            selected_trains_ranked=[selection, selection.model_copy(update={"schedule_id": "sched-2"})],
            passengers={"adults": 1, "children": 0},
            seat_class="general",
            auto_pay=False,
            notify=False,
        )
    with pytest.raises(ValueError, match="schedule_id must be unique"):
        TrainTaskCreateRequest(
            provider="SRT",
            dep="수서",
            arr="부산",
            date=date(2026, 2, 23),
            selected_trains_ranked=[selection, selection.model_copy(update={"rank": 2})],
            passengers={"adults": 1, "children": 0},
            seat_class="general",
            auto_pay=False,
            notify=False,
        )


def test_config_and_schema_validation_small_gaps():
    assert Settings.parse_optional_proxy_url(None) is None

    with pytest.raises(ValueError, match="(12 to 19 digits|at least 12 characters)"):
        PaymentCardSetRequest(
            card_number="1234 5678 901",
            expiry_month=1,
            expiry_year=2030,
            birth_date=date(1990, 1, 1),
            pin2="12",
        )

    with pytest.raises(ValueError, match="subject cannot be blank"):
        EmailJobPayload(
            to_email="user@example.com",
            subject="   ",
            text_body="body",
        )
    with pytest.raises(ValueError, match="subject cannot be blank"):
        EmailTemplateJobPayload(
            to_email="user@example.com",
            subject="   ",
            blocks=[],
        )


def test_password_needs_rehash_handles_invalid_hash_errors(monkeypatch: pytest.MonkeyPatch) -> None:
    class _BrokenHasher:
        @staticmethod
        def check_needs_rehash(_password_hash: str) -> bool:
            raise InvalidHashError("invalid hash")

    monkeypatch.setattr(security, "password_hasher", _BrokenHasher())
    assert security.password_needs_rehash("not-a-valid-hash") is False
