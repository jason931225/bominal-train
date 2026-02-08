from __future__ import annotations

from uuid import UUID

from fastapi import APIRouter, Depends, Path, Query
from sqlalchemy.ext.asyncio import AsyncSession

from app.http.deps import get_current_user
from app.db.models import User
from app.db.session import get_db
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
    TaskListResponse,
    TicketCancelResponse,
    TrainStationsResponse,
    TrainSearchRequest,
    TrainSearchResponse,
    TrainTaskCreateRequest,
    TrainTaskCreateResponse,
)
from app.modules.train.service import (
    clear_ktx_credentials,
    clear_srt_credentials,
    cancel_provider_reservation,
    cancel_task,
    cancel_ticket,
    create_task,
    delete_task,
    get_provider_ticket_info,
    get_ktx_credential_status,
    get_train_credentials_status,
    get_task_detail,
    get_srt_credential_status,
    list_provider_reservations,
    list_station_options,
    list_tasks,
    pause_task,
    pay_task,
    resume_task,
    retry_task_now,
    search_schedules,
    set_ktx_credentials,
    set_srt_credentials,
)

router = APIRouter(prefix="/api/train", tags=["train"])


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
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> TaskListResponse:
    return await list_tasks(
        db,
        user=user,
        status_filter=status,
        refresh_completed=refresh_completed,
    )


@router.get("/tasks/{task_id}", response_model=TaskDetailOut)
async def get_train_task(
    task_id: UUID,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> TaskDetailOut:
    return await get_task_detail(db, task_id=task_id, user=user)


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
