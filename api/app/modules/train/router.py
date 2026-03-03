from __future__ import annotations

import json
import logging
from collections.abc import AsyncIterator
from uuid import UUID

from fastapi import APIRouter, Depends, Path, Query, Request
from fastapi.responses import StreamingResponse
from sqlalchemy.ext.asyncio import AsyncSession

from app.http.deps import get_current_user
from app.core.redis import get_redis_client
from app.db.models import User
from app.db.session import SessionLocal, get_db
from app.modules.train.events import task_events_channel
from app.modules.train.schemas import (
    KTXCredentialStatusResponse,
    KTXCredentialsSetRequest,
    ProviderReservationCancelResponse,
    ProviderReservationsResponse,
    ProviderTicketInfoResponse,
    ProviderCredentialsStatusResponse,
    SRTCredentialsSetRequest,
    SRTCredentialStatusResponse,
    TaskActionResponse,
    TaskDetailOut,
    TaskLastAttemptRuntimeOut,
    TaskListResponse,
    TicketCancelResponse,
    TrainStationsResponse,
    TrainSearchRequest,
    TrainSearchResponse,
    TrainTaskDuplicateCheckResponse,
    TrainTaskCreateRequest,
    TrainTaskCreateResponse,
)
from app.modules.train.service import (
    check_task_duplicates,
    clear_ktx_credentials,
    clear_srt_credentials,
    cancel_provider_reservation,
    cancel_task,
    cancel_ticket,
    create_task,
    delete_task,
    get_provider_ticket_info,
    get_ktx_credential_status,
    get_task_last_attempt_runtime,
    get_train_credentials_status,
    get_task_detail,
    get_srt_credential_status,
    list_provider_reservations,
    list_station_options,
    list_tasks,
    pause_task,
    pay_task,
    refresh_task_detail,
    retry_task_now,
    resume_task,
    search_schedules,
    set_ktx_credentials,
    set_srt_credentials,
)

router = APIRouter(prefix="/api/train", tags=["train"])
TRAIN_TASK_EVENTS_PING_SECONDS = 10.0
TRAIN_TASK_ATTENTION_SNAPSHOT_LIMIT = 200
ATTENTION_REFRESH_TASK_STATES = frozenset({"COMPLETED", "CANCELLED", "EXPIRED", "FAILED"})
logger = logging.getLogger(__name__)


def _sse(event: str, data: str, *, event_id: int | None = None, retry_ms: int | None = None) -> str:
    message = [f"event: {event}"]
    if event_id is not None:
        message.append(f"id: {event_id}")
    if retry_ms is not None:
        message.append(f"retry: {retry_ms}")
    message.append(f"data: {data}")
    return "\n".join(message) + "\n\n"


def _event_name_for_payload(payload_text: str) -> str:
    try:
        payload = json.loads(payload_text)
    except Exception:
        return "task_state"
    if not isinstance(payload, dict):
        return "task_state"
    payload_type = str(payload.get("type") or "")
    if payload_type == "task_ticket_status_changed":
        return "task_ticket_status"
    return "task_state"


async def _task_events_stream(
    request: Request,
    *,
    user_id: UUID,
    attention_snapshot_payload: str | None = None,
) -> AsyncIterator[str]:
    redis = await get_redis_client()
    pubsub = redis.pubsub()
    channel = task_events_channel(user_id)
    await pubsub.subscribe(channel)
    request_headers = getattr(request, "headers", None)
    last_event_id_header = request_headers.get("last-event-id") if request_headers is not None else None
    try:
        sequence = int(last_event_id_header) if last_event_id_header else 0
    except (TypeError, ValueError):
        sequence = 0
    try:
        sequence += 1
        yield _sse(
            "connected",
            json.dumps({"channel": channel}, separators=(",", ":")),
            event_id=sequence,
            retry_ms=3000,
        )
        if attention_snapshot_payload is not None:
            sequence += 1
            yield _sse("attention_snapshot", attention_snapshot_payload, event_id=sequence)
        while True:
            if await request.is_disconnected():
                break
            message = await pubsub.get_message(
                ignore_subscribe_messages=True,
                timeout=TRAIN_TASK_EVENTS_PING_SECONDS,
            )
            if message is None:
                # Keep SSE connections warm through proxies/LBs.
                yield ": keepalive\n\n"
                continue
            payload_raw = message.get("data")
            if payload_raw is None:
                continue
            if isinstance(payload_raw, bytes):
                payload_text = payload_raw.decode("utf-8", errors="replace")
            else:
                payload_text = str(payload_raw)
            sequence += 1
            yield _sse(_event_name_for_payload(payload_text), payload_text, event_id=sequence)
    finally:
        try:
            await pubsub.unsubscribe(channel)
        finally:
            await pubsub.aclose()


def _is_attention_task(task: TaskSummaryOut) -> bool:
    if task.state in ATTENTION_REFRESH_TASK_STATES:
        return True
    if task.ticket_status in {"awaiting_payment", "waiting"} and task.ticket_paid is not True:
        return True
    return False


async def _build_attention_snapshot_payload(*, user: User) -> str | None:
    try:
        async with SessionLocal() as db:
            task_list = await list_tasks(
                db,
                user=user,
                status_filter="all",
                limit=TRAIN_TASK_ATTENTION_SNAPSHOT_LIMIT,
                refresh_completed=False,
            )
        snapshot_tasks = [task.model_dump(mode="json") for task in task_list.tasks if _is_attention_task(task)]
        return json.dumps(
            {"type": "attention_snapshot", "tasks": snapshot_tasks},
            separators=(",", ":"),
        )
    except Exception:
        logger.warning("Failed to build train attention snapshot payload", extra={"user_id": str(user.id)})
        return None


@router.get("/stations", response_model=TrainStationsResponse)
async def list_train_stations(_: User = Depends(get_current_user)) -> TrainStationsResponse:
    return list_station_options()


@router.get("/credentials/srt", response_model=SRTCredentialStatusResponse)
async def get_srt_credentials(
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> SRTCredentialStatusResponse:
    return await get_srt_credential_status(db, user=user)


@router.get("/credentials/ktx", response_model=KTXCredentialStatusResponse)
async def get_ktx_credentials(
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> KTXCredentialStatusResponse:
    return await get_ktx_credential_status(db, user=user)


@router.get("/credentials/status", response_model=ProviderCredentialsStatusResponse)
async def get_train_credentials_status_route(
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> ProviderCredentialsStatusResponse:
    return await get_train_credentials_status(db, user=user)


@router.post("/credentials/srt", response_model=SRTCredentialStatusResponse)
async def save_srt_credentials(
    payload: SRTCredentialsSetRequest,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> SRTCredentialStatusResponse:
    return await set_srt_credentials(db, user=user, payload=payload)


@router.post("/credentials/srt/signout", response_model=SRTCredentialStatusResponse)
async def signout_srt_credentials(
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> SRTCredentialStatusResponse:
    return await clear_srt_credentials(db, user=user)


@router.post("/credentials/ktx", response_model=KTXCredentialStatusResponse)
async def save_ktx_credentials(
    payload: KTXCredentialsSetRequest,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> KTXCredentialStatusResponse:
    return await set_ktx_credentials(db, user=user, payload=payload)


@router.post("/credentials/ktx/signout", response_model=KTXCredentialStatusResponse)
async def signout_ktx_credentials(
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> KTXCredentialStatusResponse:
    return await clear_ktx_credentials(db, user=user)


@router.post("/search", response_model=TrainSearchResponse)
async def search_train(
    payload: TrainSearchRequest,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> TrainSearchResponse:
    return await search_schedules(db, payload=payload, user=user)


@router.post("/tasks/duplicate-check", response_model=TrainTaskDuplicateCheckResponse)
async def check_train_task_duplicates(
    payload: TrainTaskCreateRequest,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> TrainTaskDuplicateCheckResponse:
    return await check_task_duplicates(db, user=user, payload=payload)


@router.post("/tasks", response_model=TrainTaskCreateResponse)
async def create_train_task(
    payload: TrainTaskCreateRequest,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> TrainTaskCreateResponse:
    return await create_task(db, user=user, payload=payload)


@router.get("/tasks", response_model=TaskListResponse)
async def list_train_tasks(
    status: str = Query(default="active", pattern=r"^(active|completed|all)$"),
    refresh_completed: bool = Query(default=False),
    limit: int = Query(default=200, ge=1, le=500),
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> TaskListResponse:
    return await list_tasks(
        db,
        user=user,
        status_filter=status,
        limit=limit,
        refresh_completed=refresh_completed,
    )


@router.get("/tasks/events")
async def stream_train_task_events(
    request: Request,
    user: User = Depends(get_current_user),
) -> StreamingResponse:
    attention_snapshot_payload = await _build_attention_snapshot_payload(user=user)

    return StreamingResponse(
        _task_events_stream(
            request,
            user_id=user.id,
            attention_snapshot_payload=attention_snapshot_payload,
        ),
        media_type="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
            "X-Accel-Buffering": "no",
        },
    )


@router.get("/tasks/{task_id}", response_model=TaskDetailOut)
async def get_train_task(
    task_id: UUID,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> TaskDetailOut:
    return await get_task_detail(db, task_id=task_id, user=user)


@router.get("/tasks/{task_id}/last-attempt", response_model=TaskLastAttemptRuntimeOut)
async def get_train_task_last_attempt(
    task_id: UUID,
    user: User = Depends(get_current_user),
) -> TaskLastAttemptRuntimeOut:
    return await get_task_last_attempt_runtime(task_id=task_id, user=user)


@router.post("/tasks/{task_id}/refresh", response_model=TaskDetailOut)
async def refresh_train_task(
    task_id: UUID,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> TaskDetailOut:
    return await refresh_task_detail(db, task_id=task_id, user=user)


@router.post("/tasks/{task_id}/pause", response_model=TaskActionResponse)
async def pause_train_task(
    task_id: UUID,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> TaskActionResponse:
    return await pause_task(db, task_id=task_id, user=user)


@router.post("/tasks/{task_id}/resume", response_model=TaskActionResponse)
async def resume_train_task(
    task_id: UUID,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> TaskActionResponse:
    return await resume_task(db, task_id=task_id, user=user)


@router.post("/tasks/{task_id}/retry", response_model=TaskActionResponse)
async def retry_train_task(
    task_id: UUID,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> TaskActionResponse:
    return await retry_task_now(db, task_id=task_id, user=user)


@router.post("/tasks/{task_id}/cancel", response_model=TaskActionResponse)
async def cancel_train_task(
    task_id: UUID,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
    ) -> TaskActionResponse:
    return await cancel_task(db, task_id=task_id, user=user)


@router.post("/tasks/{task_id}/pay", response_model=TaskActionResponse)
async def pay_train_task(
    task_id: UUID,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> TaskActionResponse:
    return await pay_task(db, task_id=task_id, user=user)


@router.post("/tasks/{task_id}/delete", response_model=TaskActionResponse)
async def delete_train_task(
    task_id: UUID,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> TaskActionResponse:
    return await delete_task(db, task_id=task_id, user=user)


@router.post("/tickets/{artifact_id}/cancel", response_model=TicketCancelResponse)
async def cancel_train_ticket(
    artifact_id: UUID,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> TicketCancelResponse:
    return await cancel_ticket(db, artifact_id=artifact_id, user=user)


@router.get("/providers/{provider}/reservations", response_model=ProviderReservationsResponse)
async def list_train_provider_reservations(
    provider: str = Path(..., pattern=r"^(SRT|KTX)$"),
    paid_only: bool = Query(default=False),
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> ProviderReservationsResponse:
    return await list_provider_reservations(
        db,
        user=user,
        provider=provider,
        paid_only=paid_only,
    )


@router.get("/providers/{provider}/reservations/{reservation_id}/tickets", response_model=ProviderTicketInfoResponse)
async def get_train_provider_ticket_info(
    provider: str = Path(..., pattern=r"^(SRT|KTX)$"),
    reservation_id: str = Path(..., min_length=1, max_length=64),
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> ProviderTicketInfoResponse:
    return await get_provider_ticket_info(
        db,
        user=user,
        provider=provider,
        reservation_id=reservation_id,
    )


@router.post("/providers/{provider}/reservations/{reservation_id}/cancel", response_model=ProviderReservationCancelResponse)
async def cancel_train_provider_reservation(
    provider: str = Path(..., pattern=r"^(SRT|KTX)$"),
    reservation_id: str = Path(..., min_length=1, max_length=64),
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> ProviderReservationCancelResponse:
    return await cancel_provider_reservation(
        db,
        user=user,
        provider=provider,
        reservation_id=reservation_id,
    )
