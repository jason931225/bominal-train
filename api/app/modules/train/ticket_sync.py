from __future__ import annotations

from datetime import datetime, timezone
from typing import Any
from uuid import UUID

from app.core.crypto.redaction import redact_sensitive
from app.core.time import utc_now
from app.modules.train.rate_limiter import RedisTokenBucketLimiter


def find_reservation_by_number(reservations: list[dict[str, Any]], reservation_id: str) -> dict[str, Any] | None:
    for ticket in reservations:
        if str(ticket.get("reservation_id") or "") == reservation_id:
            return ticket
    return None


def _status_from_snapshot(
    *,
    paid: bool | None,
    waiting: bool | None,
    expired: bool | None,
    reservation_found: bool,
) -> str:
    if not reservation_found:
        return "reservation_not_found"
    if bool(paid):
        return "paid"
    if bool(expired):
        return "expired"
    if bool(waiting):
        return "waiting"
    return "awaiting_payment"


async def fetch_ticket_sync_snapshot(
    *,
    client: Any,
    provider: str,
    reservation_id: str,
    user_id: UUID,
    limiter: RedisTokenBucketLimiter | None,
) -> dict[str, Any]:
    snapshot: dict[str, Any] = {
        "provider": provider,
        "reservation_id": reservation_id,
        "synced_at": utc_now().isoformat(),
    }

    provider_http: dict[str, Any] = {}
    sync_meta: dict[str, Any] = {
        "provider": provider,
        "reservation_id": reservation_id,
    }

    reservations_outcome = None
    reservation_rows: list[dict[str, Any]] = []

    if limiter is not None:
        limit = await limiter.acquire_provider_call(
            provider=provider,
            user_bucket_key=str(user_id),
            host_bucket_key="default-host",
        )
        sync_meta["reservations_rate_limit_wait_ms"] = limit.waited_ms

    try:
        reservations_outcome = await client.get_reservations(
            user_id=str(user_id),
            reservation_id=reservation_id,
        )
    except Exception as exc:
        sync_meta["reservations_error"] = f"provider_reservations_transport_error:{type(exc).__name__}"
    else:
        sync_meta["reservations_ok"] = bool(reservations_outcome.ok)
        if reservations_outcome.ok:
            reservation_rows = [
                row for row in reservations_outcome.data.get("reservations", []) if isinstance(row, dict)
            ]
        else:
            sync_meta["reservations_error"] = reservations_outcome.error_message_safe or reservations_outcome.error_code

        trace = reservations_outcome.data.get("http_trace") if reservations_outcome else None
        if trace:
            provider_http["get_reservations"] = redact_sensitive(trace)

    reservation_row = find_reservation_by_number(reservation_rows, reservation_id)

    ticket_outcome = None
    ticket_rows: list[dict[str, Any]] = []
    if limiter is not None:
        limit = await limiter.acquire_provider_call(
            provider=provider,
            user_bucket_key=str(user_id),
            host_bucket_key="default-host",
        )
        sync_meta["ticket_info_rate_limit_wait_ms"] = limit.waited_ms

    try:
        ticket_outcome = await client.ticket_info(
            reservation_id=reservation_id,
            user_id=str(user_id),
        )
    except Exception as exc:
        sync_meta["ticket_info_error"] = f"provider_ticket_info_transport_error:{type(exc).__name__}"
    else:
        sync_meta["ticket_info_ok"] = bool(ticket_outcome.ok)
        if ticket_outcome.ok:
            ticket_rows = [row for row in ticket_outcome.data.get("tickets", []) if isinstance(row, dict)]
        else:
            sync_meta["ticket_info_error"] = ticket_outcome.error_message_safe or ticket_outcome.error_code

        trace = ticket_outcome.data.get("http_trace") if ticket_outcome else None
        if trace:
            provider_http["ticket_info"] = redact_sensitive(trace)

    if not ticket_rows and reservation_row:
        ticket_rows = [row for row in reservation_row.get("tickets", []) if isinstance(row, dict)]

    reservation_found = reservation_row is not None
    paid = bool(reservation_row.get("paid")) if reservation_row else None
    waiting = bool(reservation_row.get("waiting")) if reservation_row else None
    expired = bool(reservation_row.get("expired")) if reservation_row else None

    if reservation_row:
        snapshot["reservation_snapshot"] = redact_sensitive(reservation_row)
        snapshot["payment_deadline_at"] = reservation_row.get("payment_deadline_at")
        snapshot["dep"] = reservation_row.get("dep") or snapshot.get("dep")
        snapshot["arr"] = reservation_row.get("arr") or snapshot.get("arr")
        snapshot["train_no"] = reservation_row.get("train_no") or snapshot.get("train_no")
        if reservation_row.get("journey_no"):
            snapshot["journey_no"] = reservation_row.get("journey_no")
        if reservation_row.get("journey_cnt"):
            snapshot["journey_cnt"] = reservation_row.get("journey_cnt")
        if reservation_row.get("rsv_chg_no"):
            snapshot["rsv_chg_no"] = reservation_row.get("rsv_chg_no")
        if reservation_row.get("wct_no"):
            snapshot["wct_no"] = reservation_row.get("wct_no")

    if ticket_rows:
        snapshot["tickets"] = redact_sensitive(ticket_rows)

    seat_count = len(ticket_rows)
    if reservation_row and isinstance(reservation_row.get("seat_count"), int):
        seat_count = max(seat_count, int(reservation_row["seat_count"]))
    if seat_count > 0:
        snapshot["seat_count"] = seat_count

    if paid is not None:
        snapshot["paid"] = paid
    if waiting is not None:
        snapshot["waiting"] = waiting
    if expired is not None:
        snapshot["expired"] = expired

    snapshot["status"] = _status_from_snapshot(
        paid=paid,
        waiting=waiting,
        expired=expired,
        reservation_found=reservation_found,
    )
    snapshot["provider_sync"] = redact_sensitive(sync_meta)
    if provider_http:
        snapshot["provider_http"] = provider_http

    return snapshot
