from __future__ import annotations

import asyncio
import hashlib
import json
import time
from datetime import datetime, timedelta, timezone
from uuid import UUID

from fastapi import HTTPException, status
from sqlalchemy import Select, and_, delete, func, or_, select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from app.core.config import get_settings
from app.core.crypto.secrets_store import build_encrypted_secret, decrypt_secret
from app.core.crypto.redaction import redact_sensitive
from app.core.redis import get_redis_client
from app.core.time import utc_now
from app.db.models import Artifact, Secret, Task, TaskAttempt, User
from app.modules.train.constants import (
    ACTIVE_TASK_STATES,
    SECRET_KIND_KTX_CREDENTIALS,
    SECRET_KIND_SRT_CREDENTIALS,
    TASK_MODULE,
    TERMINAL_TASK_STATES,
    credential_kind,
)
from app.modules.train.providers import get_provider_client
from app.modules.train.providers.base import ProviderSchedule
from app.modules.train.queue import enqueue_train_task
from app.modules.train.rate_limiter import RedisTokenBucketLimiter
from app.modules.train.schemas import (
    ArtifactOut,
    KTXCredentialStatusResponse,
    KTXCredentialsSetRequest,
    ProviderReservationCancelResponse,
    ProviderReservationsResponse,
    ProviderTicketInfoResponse,
    ProviderCredentialStatus,
    ProviderCredentialsStatusResponse,
    ScheduleOut,
    SRTCredentialsSetRequest,
    SRTCredentialStatusResponse,
    TaskActionResponse,
    TaskAttemptOut,
    TaskDetailOut,
    TaskListResponse,
    TaskSummaryOut,
    TicketCancelResponse,
    TrainSearchRequest,
    TrainSearchResponse,
    TrainStationOut,
    TrainStationsResponse,
    TrainTaskCreateRequest,
    TrainTaskCreateResponse,
)
from app.modules.train.stations import ALL_STATIONS, station_code_for_name, station_exists
from app.modules.train.ticket_sync import fetch_ticket_sync_snapshot
from app.modules.train.timezone import KST
from app.services.wallet import get_payment_card_for_execution

settings = get_settings()
TASK_VISIBILITY_RETENTION_DAYS = 365
TICKET_SYNC_MIN_SECONDS = 15


def _parse_iso_datetime(value: str | None) -> datetime | None:
    if not value:
        return None
    try:
        parsed = datetime.fromisoformat(value)
    except ValueError:
        return None
    if parsed.tzinfo is None:
        return parsed.replace(tzinfo=timezone.utc)
    return parsed


def _build_task_attempt(
    *,
    task_id: UUID,
    action: str,
    provider: str,
    ok: bool,
    retryable: bool,
    error_code: str | None,
    error_message_safe: str | None,
    duration_ms: int,
    meta_json_safe: dict | None,
    started_at: datetime,
    finished_at: datetime | None = None,
) -> TaskAttempt:
    return TaskAttempt(
        task_id=task_id,
        action=action,
        provider=provider,
        ok=ok,
        retryable=retryable,
        error_code=error_code,
        error_message_safe=error_message_safe,
        duration_ms=duration_ms,
        meta_json_safe=redact_sensitive(meta_json_safe) if meta_json_safe else None,
        started_at=started_at,
        finished_at=finished_at or utc_now(),
    )


async def _latest_secret_for_user(db: AsyncSession, *, user_id: UUID, kind: str) -> Secret | None:
    stmt = (
        select(Secret)
        .where(Secret.user_id == user_id)
        .where(Secret.kind == kind)
        .order_by(Secret.updated_at.desc())
        .limit(1)
    )
    return (await db.execute(stmt)).scalar_one_or_none()


def _credential_missing_code(provider: str) -> str:
    return "srt_credentials_missing" if provider == "SRT" else "ktx_credentials_missing"


async def _load_provider_credentials(
    db: AsyncSession,
    *,
    user_id: UUID,
    provider: str,
) -> dict[str, str] | None:
    secret = await _latest_secret_for_user(db, user_id=user_id, kind=credential_kind(provider))
    if secret is None:
        return None

    try:
        payload = decrypt_secret(secret)
    except Exception:
        return None

    username = str(payload.get("username") or "").strip()
    password = str(payload.get("password") or "")
    if not username or not password:
        return None

    verified_at = str(payload.get("verified_at") or "")
    return {"username": username, "password": password, "verified_at": verified_at}


async def _save_provider_credentials(
    db: AsyncSession,
    *,
    user_id: UUID,
    provider: str,
    payload: dict,
) -> None:
    encrypted_secret = build_encrypted_secret(
        user_id=user_id,
        kind=credential_kind(provider),
        payload=payload,
    )
    existing_secret = await _latest_secret_for_user(db, user_id=user_id, kind=credential_kind(provider))
    now = utc_now()
    if existing_secret is None:
        db.add(encrypted_secret)
    else:
        existing_secret.ciphertext = encrypted_secret.ciphertext
        existing_secret.nonce = encrypted_secret.nonce
        existing_secret.wrapped_dek = encrypted_secret.wrapped_dek
        existing_secret.dek_nonce = encrypted_secret.dek_nonce
        existing_secret.aad = encrypted_secret.aad
        existing_secret.kek_version = encrypted_secret.kek_version
        existing_secret.updated_at = now


def _parse_verified_at(value: str | None) -> datetime | None:
    if not value:
        return None
    try:
        return datetime.fromisoformat(value)
    except ValueError:
        return None


def _is_recent_verification(verified_at: datetime | None) -> bool:
    if verified_at is None:
        return False
    cache_seconds = max(settings.train_credential_cache_seconds, 0)
    if cache_seconds <= 0:
        return False
    return (utc_now() - verified_at) <= timedelta(seconds=cache_seconds)


async def _verify_provider_credentials(
    db: AsyncSession,
    *,
    user: User,
    provider: str,
    force_live: bool = False,
) -> ProviderCredentialStatus:
    creds = await _load_provider_credentials(db, user_id=user.id, provider=provider)
    if creds is None:
        return ProviderCredentialStatus(
            configured=False,
            verified=False,
            detail="Credentials are missing",
        )

    verified_at = _parse_verified_at(creds.get("verified_at"))
    if not force_live and _is_recent_verification(verified_at):
        return ProviderCredentialStatus(
            configured=True,
            verified=True,
            username=creds["username"],
            verified_at=verified_at,
            detail=None,
        )

    client = get_provider_client(provider)
    try:
        login_outcome = await asyncio.wait_for(
            client.login(
                user_id=str(user.id),
                credentials={"username": creds["username"], "password": creds["password"]},
            ),
            timeout=settings.train_credential_verify_timeout_seconds,
        )
    except asyncio.TimeoutError:
        if verified_at is not None:
            return ProviderCredentialStatus(
                configured=True,
                verified=True,
                username=creds["username"],
                verified_at=verified_at,
                detail=None,
            )
        return ProviderCredentialStatus(
            configured=True,
            verified=False,
            username=creds["username"],
            detail=f"{provider} login timed out. Try again.",
        )
    except Exception:
        if verified_at is not None:
            return ProviderCredentialStatus(
                configured=True,
                verified=True,
                username=creds["username"],
                verified_at=verified_at,
                detail=None,
            )
        return ProviderCredentialStatus(
            configured=True,
            verified=False,
            username=creds["username"],
            detail=f"{provider} login check failed",
        )

    if not login_outcome.ok:
        if login_outcome.retryable and verified_at is not None:
            return ProviderCredentialStatus(
                configured=True,
                verified=True,
                username=creds["username"],
                verified_at=verified_at,
                detail=None,
            )
        return ProviderCredentialStatus(
            configured=True,
            verified=False,
            username=creds["username"],
            verified_at=verified_at,
            detail=login_outcome.error_message_safe or f"{provider} login failed",
        )

    fresh_verified_at = utc_now()
    secret_payload = {
        "username": creds["username"],
        "password": creds["password"],
        "verified_at": fresh_verified_at.isoformat(),
        "membership_number": login_outcome.data.get("membership_number"),
        "membership_name": login_outcome.data.get("membership_name")
        or login_outcome.data.get("name"),
    }
    await _save_provider_credentials(db, user_id=user.id, provider=provider, payload=secret_payload)
    await db.commit()

    return ProviderCredentialStatus(
        configured=True,
        verified=True,
        username=creds["username"],
        verified_at=fresh_verified_at,
        detail=None,
    )


async def _status_from_saved_credentials(
    db: AsyncSession,
    *,
    user: User,
    provider: str,
    fallback_detail: str | None = None,
) -> ProviderCredentialStatus:
    creds = await _load_provider_credentials(db, user_id=user.id, provider=provider)
    if creds is None:
        return ProviderCredentialStatus(
            configured=False,
            verified=False,
            username=None,
            verified_at=None,
            detail="Credentials are missing",
        )

    verified_at = _parse_verified_at(creds.get("verified_at"))
    verified = verified_at is not None
    return ProviderCredentialStatus(
        configured=True,
        verified=verified,
        username=creds["username"],
        verified_at=verified_at,
        detail=None if verified else fallback_detail,
    )


async def _verify_provider_credentials_guarded(
    db: AsyncSession,
    *,
    user: User,
    provider: str,
) -> ProviderCredentialStatus:
    try:
        return await _verify_provider_credentials(db, user=user, provider=provider)
    except Exception:
        return await _status_from_saved_credentials(
            db,
            user=user,
            provider=provider,
            fallback_detail=f"{provider} login check failed. Reconnect if needed.",
        )


async def get_train_credentials_status(
    db: AsyncSession,
    *,
    user: User,
) -> ProviderCredentialsStatusResponse:
    # Keep checks sequential on one AsyncSession to avoid concurrent DB/session usage errors.
    ktx = await _verify_provider_credentials_guarded(db, user=user, provider="KTX")
    srt = await _verify_provider_credentials_guarded(db, user=user, provider="SRT")
    return ProviderCredentialsStatusResponse(ktx=ktx, srt=srt)


async def get_srt_credential_status(db: AsyncSession, *, user: User) -> SRTCredentialStatusResponse:
    status_info = await _verify_provider_credentials_guarded(db, user=user, provider="SRT")
    return SRTCredentialStatusResponse(**status_info.model_dump())


async def get_ktx_credential_status(db: AsyncSession, *, user: User) -> KTXCredentialStatusResponse:
    status_info = await _verify_provider_credentials_guarded(db, user=user, provider="KTX")
    return KTXCredentialStatusResponse(**status_info.model_dump())


async def set_srt_credentials(
    db: AsyncSession,
    *,
    user: User,
    payload: SRTCredentialsSetRequest,
) -> SRTCredentialStatusResponse:
    username = payload.username.strip()
    password = payload.password

    client = get_provider_client("SRT")
    login_outcome = await client.login(
        user_id=str(user.id),
        credentials={"username": username, "password": password},
    )
    if not login_outcome.ok:
        status_code = status.HTTP_400_BAD_REQUEST
        if login_outcome.retryable:
            status_code = status.HTTP_502_BAD_GATEWAY
        raise HTTPException(
            status_code=status_code,
            detail=login_outcome.error_message_safe or "SRT login failed",
        )

    now = utc_now()
    secret_payload = {
        "username": username,
        "password": password,
        "verified_at": now.isoformat(),
        "membership_number": login_outcome.data.get("membership_number"),
        "membership_name": login_outcome.data.get("membership_name"),
    }
    await _save_provider_credentials(db, user_id=user.id, provider="SRT", payload=secret_payload)
    await db.commit()
    return SRTCredentialStatusResponse(configured=True, verified=True, username=username, verified_at=now)


async def set_ktx_credentials(
    db: AsyncSession,
    *,
    user: User,
    payload: KTXCredentialsSetRequest,
) -> KTXCredentialStatusResponse:
    username = payload.username.strip()
    password = payload.password

    client = get_provider_client("KTX")
    login_outcome = await client.login(
        user_id=str(user.id),
        credentials={"username": username, "password": password},
    )
    if not login_outcome.ok:
        status_code = status.HTTP_400_BAD_REQUEST
        if login_outcome.retryable:
            status_code = status.HTTP_502_BAD_GATEWAY
        raise HTTPException(
            status_code=status_code,
            detail=login_outcome.error_message_safe or "KTX login failed",
        )

    now = utc_now()
    secret_payload = {
        "username": username,
        "password": password,
        "verified_at": now.isoformat(),
        "membership_number": login_outcome.data.get("membership_number"),
        "membership_name": login_outcome.data.get("name"),
    }
    await _save_provider_credentials(db, user_id=user.id, provider="KTX", payload=secret_payload)
    await db.commit()
    return KTXCredentialStatusResponse(configured=True, verified=True, username=username, verified_at=now)


async def _clear_provider_credentials(
    db: AsyncSession,
    *,
    user: User,
    provider: str,
) -> ProviderCredentialStatus:
    await db.execute(
        delete(Secret)
        .where(Secret.user_id == user.id)
        .where(Secret.kind == credential_kind(provider))
    )
    await db.commit()
    return ProviderCredentialStatus(
        configured=False,
        verified=False,
        username=None,
        verified_at=None,
        detail=f"{provider} credentials signed out",
    )


async def clear_srt_credentials(
    db: AsyncSession,
    *,
    user: User,
) -> SRTCredentialStatusResponse:
    status_info = await _clear_provider_credentials(db, user=user, provider="SRT")
    return SRTCredentialStatusResponse(**status_info.model_dump())


async def clear_ktx_credentials(
    db: AsyncSession,
    *,
    user: User,
) -> KTXCredentialStatusResponse:
    status_info = await _clear_provider_credentials(db, user=user, provider="KTX")
    return KTXCredentialStatusResponse(**status_info.model_dump())


async def _get_logged_in_provider_client(
    db: AsyncSession,
    *,
    user: User,
    provider: str,
):
    creds = await _load_provider_credentials(db, user_id=user.id, provider=provider)
    if creds is None:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=f"Connect {provider} credentials first",
        )

    client = get_provider_client(provider)
    login_outcome = await client.login(
        user_id=str(user.id),
        credentials={"username": creds["username"], "password": creds["password"]},
    )
    if not login_outcome.ok:
        status_code = status.HTTP_502_BAD_GATEWAY if login_outcome.retryable else status.HTTP_400_BAD_REQUEST
        raise HTTPException(
            status_code=status_code,
            detail=login_outcome.error_message_safe or f"{provider} login failed",
        )

    return client


def _sorted_ranked_trains(payload: TrainTaskCreateRequest) -> list[dict]:
    ranked: list[dict] = []
    for item in sorted(payload.selected_trains_ranked, key=lambda row: row.rank):
        provider = item.provider or payload.provider
        if provider is None:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Each selected schedule must include provider metadata. Re-run search and select schedules again.",
            )
        if payload.provider and item.provider and item.provider != payload.provider:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Selected schedules include a different provider than the requested provider.",
            )
        ranked.append(
            {
                "schedule_id": item.schedule_id,
                "departure_at": item.departure_at.isoformat(),
                "rank": item.rank,
                "provider": provider,
            }
        )
    return ranked


def _resolve_task_providers(ranked_trains: list[dict]) -> list[str]:
    providers = sorted({str(item.get("provider")) for item in ranked_trains if item.get("provider")})
    if not providers:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="No provider could be resolved from selected schedules.",
        )
    return providers


def normalize_task_spec(payload: TrainTaskCreateRequest, *, ranked_trains: list[dict]) -> dict:
    dep_srt_code = station_code_for_name(payload.dep)
    arr_srt_code = station_code_for_name(payload.arr)
    providers = _resolve_task_providers(ranked_trains)
    return {
        "module": TASK_MODULE,
        "provider": providers[0] if len(providers) == 1 else None,
        "providers": providers,
        "dep": payload.dep,
        "arr": payload.arr,
        "dep_srt_code": dep_srt_code,
        "arr_srt_code": arr_srt_code,
        "date": payload.date.isoformat(),
        "selected_trains_ranked": ranked_trains,
        "passengers": {
            "adults": payload.passengers.adults,
            "children": payload.passengers.children,
        },
        "seat_class": payload.seat_class,
        "auto_pay": payload.auto_pay,
        "notify": payload.notify,
    }


def compute_deadline_from_spec(spec_json: dict) -> datetime:
    ranked = spec_json.get("selected_trains_ranked", [])
    if not ranked:
        raise ValueError("selected_trains_ranked cannot be empty")

    earliest = min(datetime.fromisoformat(item["departure_at"]) for item in ranked)
    if earliest.tzinfo is None:
        earliest = earliest.replace(tzinfo=KST)
    return earliest


def compute_idempotency_key(user_id: UUID, spec_json: dict) -> str:
    serialized = json.dumps(spec_json, sort_keys=True, separators=(",", ":"), ensure_ascii=True)
    return hashlib.sha256(f"{user_id}:{serialized}".encode("utf-8")).hexdigest()


def _ticket_summary_from_artifact(artifact: Artifact | None) -> dict | None:
    if artifact is None:
        return None
    payload = artifact.data_json_safe or {}
    return {
        "ticket_status": payload.get("status"),
        "ticket_paid": payload.get("paid"),
        "ticket_payment_deadline_at": _parse_iso_datetime(str(payload.get("payment_deadline_at") or "")),
        "ticket_reservation_id": payload.get("reservation_id"),
    }


def task_to_summary(
    task: Task,
    last_attempt_at: datetime | None = None,
    ticket_summary: dict | None = None,
) -> TaskSummaryOut:
    ticket_summary = ticket_summary or {}
    return TaskSummaryOut(
        id=task.id,
        module=task.module,
        state=task.state,
        deadline_at=task.deadline_at,
        created_at=task.created_at,
        updated_at=task.updated_at,
        paused_at=task.paused_at,
        cancelled_at=task.cancelled_at,
        completed_at=task.completed_at,
        failed_at=task.failed_at,
        hidden_at=task.hidden_at,
        last_attempt_at=last_attempt_at,
        spec_json=task.spec_json,
        ticket_status=ticket_summary.get("ticket_status"),
        ticket_paid=ticket_summary.get("ticket_paid"),
        ticket_payment_deadline_at=ticket_summary.get("ticket_payment_deadline_at"),
        ticket_reservation_id=ticket_summary.get("ticket_reservation_id"),
    )


async def search_schedules(
    db: AsyncSession,
    *,
    payload: TrainSearchRequest,
    user: User,
) -> TrainSearchResponse:
    if not station_exists(payload.dep) or not station_exists(payload.arr):
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Unknown station name")

    schedules: list[ScheduleOut] = []
    provider_errors: dict[str, dict[str, str | None]] = {}
    redis = await get_redis_client()
    limiter = RedisTokenBucketLimiter(redis)

    for provider in payload.providers:
            creds = await _load_provider_credentials(db, user_id=user.id, provider=provider)
            if creds is None:
                provider_errors[provider] = {
                    "error_code": _credential_missing_code(provider),
                    "error_message": f"Connect {provider} credentials first",
                }
                continue

            client = get_provider_client(provider)
            try:
                login_outcome = await client.login(
                    user_id=str(user.id),
                    credentials={"username": creds["username"], "password": creds["password"]},
                )
            except Exception as exc:
                provider_errors[provider] = {
                    "error_code": "provider_login_transport_error",
                    "error_message": f"{provider} login transport error: {type(exc).__name__}",
                }
                continue

            if not login_outcome.ok:
                provider_errors[provider] = {
                    "error_code": login_outcome.error_code or f"{provider.lower()}_login_failed",
                    "error_message": login_outcome.error_message_safe or f"{provider} login failed",
                }
                continue

            await limiter.acquire_provider_call(
                provider=provider,
                user_bucket_key=str(user.id),
                host_bucket_key="default-host",
            )
            try:
                outcome = await client.search(
                    dep=payload.dep,
                    arr=payload.arr,
                    date_value=payload.date,
                    time_window_start=payload.time_window.start,
                    time_window_end=payload.time_window.end,
                    user_id=str(user.id),
                )
            except Exception as exc:
                provider_errors[provider] = {
                    "error_code": "provider_transport_error",
                    "error_message": f"{provider} search transport error: {type(exc).__name__}",
                }
                continue

            if not outcome.ok:
                provider_errors[provider] = {
                    "error_code": outcome.error_code,
                    "error_message": outcome.error_message_safe,
                }
                continue

            provider_schedules = outcome.data.get("schedules", [])
            for schedule in provider_schedules:
                if not isinstance(schedule, ProviderSchedule):
                    continue
                schedules.append(
                    ScheduleOut(
                        schedule_id=schedule.schedule_id,
                        provider=schedule.provider,
                        departure_at=schedule.departure_at,
                        arrival_at=schedule.arrival_at,
                        train_no=schedule.train_no,
                        dep=schedule.dep,
                        arr=schedule.arr,
                        availability=schedule.availability,
                        metadata=schedule.metadata,
                    )
                )

    if not schedules and provider_errors and len(provider_errors) == len(payload.providers):
        detail = "; ".join(
            f"{provider}: {info.get('error_code') or 'error'} ({info.get('error_message') or 'search failed'})"
            for provider, info in provider_errors.items()
        )
        raise HTTPException(
            status_code=status.HTTP_502_BAD_GATEWAY,
            detail=f"All provider searches failed: {detail}",
        )

    schedules.sort(key=lambda row: row.departure_at)
    return TrainSearchResponse(schedules=schedules)


async def create_task(db: AsyncSession, *, user: User, payload: TrainTaskCreateRequest) -> TrainTaskCreateResponse:
    if not station_exists(payload.dep) or not station_exists(payload.arr):
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Unknown station name")
    ranked_trains = _sorted_ranked_trains(payload)
    providers = _resolve_task_providers(ranked_trains)

    if "SRT" in providers and (
        station_code_for_name(payload.dep) is None or station_code_for_name(payload.arr) is None
    ):
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Selected stations are not supported by SRT",
        )

    for provider in providers:
        credential_status = await _verify_provider_credentials(db, user=user, provider=provider)
        if not credential_status.configured:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail=f"Connect {provider} credentials before creating a {provider} Task",
            )
        if not credential_status.verified:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail=credential_status.detail or f"{provider} login failed. Update credentials.",
            )

    spec_json = normalize_task_spec(payload, ranked_trains=ranked_trains)
    deadline_at = compute_deadline_from_spec(spec_json)
    idempotency_key = compute_idempotency_key(user.id, spec_json)

    existing_stmt = (
        select(Task)
        .where(Task.user_id == user.id)
        .where(Task.module == TASK_MODULE)
        .where(Task.idempotency_key == idempotency_key)
        .where(Task.state.in_(ACTIVE_TASK_STATES))
        .order_by(Task.created_at.desc())
        .limit(1)
    )
    existing = (await db.execute(existing_stmt)).scalar_one_or_none()
    if existing is not None:
        return TrainTaskCreateResponse(task=task_to_summary(existing), queued=False, deduplicated=True)

    task = Task(
        user_id=user.id,
        module=TASK_MODULE,
        state="QUEUED",
        deadline_at=deadline_at,
        spec_json=spec_json,
        idempotency_key=idempotency_key,
    )
    db.add(task)
    await db.commit()
    await db.refresh(task)

    await enqueue_train_task(str(task.id))

    return TrainTaskCreateResponse(task=task_to_summary(task), queued=True, deduplicated=False)


async def _last_attempt_map(db: AsyncSession, task_ids: list[UUID]) -> dict[UUID, datetime]:
    if not task_ids:
        return {}

    stmt = (
        select(TaskAttempt.task_id, func.max(TaskAttempt.finished_at))
        .where(TaskAttempt.task_id.in_(task_ids))
        .group_by(TaskAttempt.task_id)
    )
    rows = (await db.execute(stmt)).all()
    return {task_id: last_at for task_id, last_at in rows}


async def _latest_ticket_artifact_map(db: AsyncSession, task_ids: list[UUID]) -> dict[UUID, Artifact]:
    if not task_ids:
        return {}

    stmt = (
        select(Artifact)
        .where(Artifact.task_id.in_(task_ids))
        .where(Artifact.kind == "ticket")
        .order_by(Artifact.task_id.asc(), Artifact.created_at.desc())
    )
    artifacts = (await db.execute(stmt)).scalars().all()
    latest: dict[UUID, Artifact] = {}
    for artifact in artifacts:
        latest.setdefault(artifact.task_id, artifact)
    return latest


def _task_list_stmt(user: User, status_filter: str) -> Select[tuple[Task]]:
    stmt = (
        select(Task)
        .where(Task.user_id == user.id)
        .where(Task.module == TASK_MODULE)
        .where(Task.hidden_at.is_(None))
    )
    cutoff = utc_now() - timedelta(days=TASK_VISIBILITY_RETENTION_DAYS)
    terminal_visible_expr = func.coalesce(
        Task.completed_at,
        Task.failed_at,
        Task.cancelled_at,
        Task.updated_at,
    ) >= cutoff

    if status_filter == "active":
        stmt = stmt.where(Task.state.in_(ACTIVE_TASK_STATES))
    elif status_filter == "completed":
        stmt = stmt.where(Task.state.in_(TERMINAL_TASK_STATES)).where(terminal_visible_expr)
    else:
        stmt = stmt.where(
            or_(
                Task.state.in_(ACTIVE_TASK_STATES),
                and_(Task.state.in_(TERMINAL_TASK_STATES), terminal_visible_expr),
            )
        )

    return stmt.order_by(Task.created_at.desc())


def _is_terminal_task_expired_for_visibility(task: Task) -> bool:
    if task.state not in TERMINAL_TASK_STATES:
        return False

    completed_at = task.completed_at or task.failed_at or task.cancelled_at or task.updated_at
    if completed_at is None:
        return False
    if completed_at.tzinfo is None:
        completed_at = completed_at.replace(tzinfo=timezone.utc)
    return completed_at < (utc_now() - timedelta(days=TASK_VISIBILITY_RETENTION_DAYS))


async def list_tasks(
    db: AsyncSession,
    *,
    user: User,
    status_filter: str,
    refresh_completed: bool = False,
) -> TaskListResponse:
    stmt = _task_list_stmt(user, status_filter)
    tasks = (await db.execute(stmt)).scalars().all()

    last_attempts = await _last_attempt_map(db, [task.id for task in tasks])
    ticket_artifacts = await _latest_ticket_artifact_map(db, [task.id for task in tasks])

    should_refresh_completed = refresh_completed and status_filter in {"completed", "all"}
    if should_refresh_completed and ticket_artifacts:
        redis = await get_redis_client()
        limiter = RedisTokenBucketLimiter(redis)
        updated = False
        client_cache: dict[str, object] = {}
        for task in tasks:
                if task.state not in TERMINAL_TASK_STATES:
                    continue
                artifact = ticket_artifacts.get(task.id)
                if artifact is None:
                    continue
                updated = (
                    await _refresh_ticket_artifact_status(
                        db,
                        user=user,
                        artifact=artifact,
                        limiter=limiter,
                        force=False,
                        client_cache=client_cache,
                    )
                    or updated
                )

        if updated:
            await db.commit()
            ticket_artifacts = await _latest_ticket_artifact_map(db, [task.id for task in tasks])

    return TaskListResponse(
        tasks=[
            task_to_summary(
                task,
                last_attempt_at=last_attempts.get(task.id),
                ticket_summary=_ticket_summary_from_artifact(ticket_artifacts.get(task.id)),
            )
            for task in tasks
        ]
    )


async def get_task_for_user(db: AsyncSession, *, task_id: UUID, user: User) -> Task:
    stmt = (
        select(Task)
        .options(selectinload(Task.attempts), selectinload(Task.artifacts))
        .where(Task.id == task_id)
        .where(Task.user_id == user.id)
        .where(Task.module == TASK_MODULE)
        .where(Task.hidden_at.is_(None))
    )
    task = (await db.execute(stmt)).scalar_one_or_none()
    if task is None or _is_terminal_task_expired_for_visibility(task):
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Task not found")
    return task


def _should_refresh_ticket_artifact(artifact_data: dict, *, force: bool) -> bool:
    if force:
        return True
    synced_at = _parse_iso_datetime(str(artifact_data.get("last_provider_sync_at") or ""))
    if synced_at is None:
        return True
    return (utc_now() - synced_at) >= timedelta(seconds=TICKET_SYNC_MIN_SECONDS)


def _latest_ticket_artifact_for_task(task: Task) -> Artifact | None:
    ticket_artifacts = [artifact for artifact in task.artifacts if artifact.kind == "ticket"]
    if not ticket_artifacts:
        return None
    return max(ticket_artifacts, key=lambda artifact: artifact.created_at)


async def _refresh_ticket_artifact_status(
    db: AsyncSession,
    *,
    user: User,
    artifact: Artifact,
    limiter: RedisTokenBucketLimiter,
    force: bool = False,
    client_cache: dict[str, object] | None = None,
) -> bool:
    if artifact.kind != "ticket":
        return False

    current_data = dict(artifact.data_json_safe or {})
    provider = str(current_data.get("provider") or "")
    reservation_id = str(current_data.get("reservation_id") or "")
    if provider not in {"SRT", "KTX"} or not reservation_id:
        return False
    if not _should_refresh_ticket_artifact(current_data, force=force):
        return False

    clients = client_cache if client_cache is not None else {}
    client = clients.get(provider)
    if client is None:
        try:
            client = await _get_logged_in_provider_client(db, user=user, provider=provider)
        except HTTPException as exc:
            sync_meta = dict(current_data.get("provider_sync") or {})
            sync_meta["error"] = exc.detail
            current_data["provider_sync"] = sync_meta
            current_data["last_provider_sync_at"] = utc_now().isoformat()
            artifact.data_json_safe = current_data
            return True
        clients[provider] = client

    try:
        snapshot = await fetch_ticket_sync_snapshot(
            client=client,
            provider=provider,
            reservation_id=reservation_id,
            user_id=user.id,
            limiter=limiter,
        )
    except Exception as exc:
        sync_meta = dict(current_data.get("provider_sync") or {})
        sync_meta["error"] = f"provider_sync_error:{type(exc).__name__}"
        current_data["provider_sync"] = sync_meta
        current_data["last_provider_sync_at"] = utc_now().isoformat()
        artifact.data_json_safe = current_data
        return True

    merged_status = snapshot.get("status", current_data.get("status"))
    if current_data.get("cancelled"):
        merged_status = "cancelled"

    merged_data = {
        **current_data,
        "status": merged_status,
        "paid": snapshot.get("paid", current_data.get("paid")),
        "waiting": snapshot.get("waiting", current_data.get("waiting")),
        "payment_deadline_at": snapshot.get("payment_deadline_at", current_data.get("payment_deadline_at")),
        "seat_count": snapshot.get("seat_count", current_data.get("seat_count")),
        "tickets": snapshot.get("tickets", current_data.get("tickets", [])),
        "reservation_snapshot": snapshot.get("reservation_snapshot", current_data.get("reservation_snapshot")),
        "provider_sync": snapshot.get("provider_sync", current_data.get("provider_sync")),
        "last_provider_sync_at": snapshot.get("synced_at", utc_now().isoformat()),
    }
    snapshot_http = snapshot.get("provider_http")
    if isinstance(snapshot_http, dict):
        merged_data["provider_http"] = {
            **dict(current_data.get("provider_http") or {}),
            **snapshot_http,
        }

    if merged_data != current_data:
        artifact.data_json_safe = merged_data
        return True
    return False


async def get_task_detail(db: AsyncSession, *, task_id: UUID, user: User) -> TaskDetailOut:
    task = await get_task_for_user(db, task_id=task_id, user=user)
    redis = await get_redis_client()
    limiter = RedisTokenBucketLimiter(redis)
    updated = False
    client_cache: dict[str, object] = {}
    for artifact in task.artifacts:
            updated = (
                await _refresh_ticket_artifact_status(
                    db,
                    user=user,
                    artifact=artifact,
                    limiter=limiter,
                    force=False,
                    client_cache=client_cache,
                )
                or updated
            )

    if updated:
        await db.commit()
        task = await get_task_for_user(db, task_id=task_id, user=user)

    attempts = sorted(task.attempts, key=lambda row: row.started_at)
    artifacts = sorted(task.artifacts, key=lambda row: row.created_at)

    last_attempt_at = attempts[-1].finished_at if attempts else None

    return TaskDetailOut(
        task=task_to_summary(task, last_attempt_at=last_attempt_at),
        attempts=[
            TaskAttemptOut(
                id=attempt.id,
                action=attempt.action,
                provider=attempt.provider,
                ok=attempt.ok,
                retryable=attempt.retryable,
                error_code=attempt.error_code,
                error_message_safe=attempt.error_message_safe,
                duration_ms=attempt.duration_ms,
                meta_json_safe=attempt.meta_json_safe,
                started_at=attempt.started_at,
                finished_at=attempt.finished_at,
            )
            for attempt in attempts
        ],
        artifacts=[
            ArtifactOut(
                id=artifact.id,
                module=artifact.module,
                kind=artifact.kind,
                data_json_safe=artifact.data_json_safe,
                created_at=artifact.created_at,
            )
            for artifact in artifacts
        ],
    )


async def pause_task(db: AsyncSession, *, task_id: UUID, user: User) -> TaskActionResponse:
    task = await get_task_for_user(db, task_id=task_id, user=user)
    if task.state in TERMINAL_TASK_STATES:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Cannot pause terminal task")

    task.state = "PAUSED"
    task.paused_at = utc_now()
    await db.commit()
    await db.refresh(task)
    return TaskActionResponse(task=task_to_summary(task))


async def resume_task(db: AsyncSession, *, task_id: UUID, user: User) -> TaskActionResponse:
    task = await get_task_for_user(db, task_id=task_id, user=user)
    if task.state != "PAUSED":
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Task is not paused")

    task.state = "QUEUED"
    task.paused_at = None
    await db.commit()
    await db.refresh(task)

    await enqueue_train_task(str(task.id))

    return TaskActionResponse(task=task_to_summary(task))


async def cancel_task(db: AsyncSession, *, task_id: UUID, user: User) -> TaskActionResponse:
    task = await get_task_for_user(db, task_id=task_id, user=user)
    if task.state in TERMINAL_TASK_STATES:
        return TaskActionResponse(task=task_to_summary(task))

    task.state = "CANCELLED"
    task.cancelled_at = utc_now()
    await db.commit()
    await db.refresh(task)
    return TaskActionResponse(task=task_to_summary(task))


async def pay_task(db: AsyncSession, *, task_id: UUID, user: User) -> TaskActionResponse:
    task = await get_task_for_user(db, task_id=task_id, user=user)
    if task.state in {"EXPIRED", "CANCELLED"}:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="This task can no longer be paid")

    artifact = _latest_ticket_artifact_for_task(task)
    if artifact is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="No ticket artifact found for this task")

    redis = await get_redis_client()
    limiter = RedisTokenBucketLimiter(redis)
    client_cache: dict[str, object] = {}
    updated = False

    updated = await _refresh_ticket_artifact_status(
        db,
        user=user,
        artifact=artifact,
        limiter=limiter,
        force=True,
        client_cache=client_cache,
    ) or updated

    artifact_data = dict(artifact.data_json_safe or {})
    provider = str(artifact_data.get("provider") or "")
    reservation_id = str(artifact_data.get("reservation_id") or "")
    ticket_status = str(artifact_data.get("status") or "")
    ticket_paid = bool(artifact_data.get("paid"))
    ticket_cancelled = bool(artifact_data.get("cancelled"))

    if provider not in {"SRT", "KTX"} or not reservation_id:
        if updated:
            await db.commit()
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Ticket artifact is missing provider reservation data")

    if ticket_cancelled or ticket_status == "cancelled":
        if updated:
            await db.commit()
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Reservation is already cancelled")

    if ticket_paid:
        if updated:
            await db.commit()
        task = await get_task_for_user(db, task_id=task_id, user=user)
        last_attempt_at = (await _last_attempt_map(db, [task.id])).get(task.id)
        ticket_summary = _ticket_summary_from_artifact(_latest_ticket_artifact_for_task(task))
        return TaskActionResponse(
            task=task_to_summary(task, last_attempt_at=last_attempt_at, ticket_summary=ticket_summary)
        )

    if ticket_status not in {"awaiting_payment", "reserved"}:
        if updated:
            await db.commit()
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail=f"Reservation is not payable in its current state ({ticket_status or 'unknown'})",
        )

    payment_deadline_at = _parse_iso_datetime(str(artifact_data.get("payment_deadline_at") or ""))
    if payment_deadline_at is not None and utc_now() >= payment_deadline_at:
        if updated:
            await db.commit()
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail="Reservation payment window has expired. Refresh reservation status and try again.",
        )

    payment_card = await get_payment_card_for_execution(db, user_id=user.id)
    if payment_card is None:
        if updated:
            await db.commit()
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Configure payment settings before paying for reservations.",
        )

    client = client_cache.get(provider)
    if client is None:
        client = await _get_logged_in_provider_client(db, user=user, provider=provider)
        client_cache[provider] = client

    started_at = utc_now()
    timer = time.perf_counter()
    limit = await limiter.acquire_provider_call(
        provider=provider,
        user_bucket_key=str(user.id),
        host_bucket_key="default-host",
    )
    outcome = await client.pay(
        reservation_id=reservation_id,
        user_id=str(user.id),
        payment_card=payment_card,
    )
    finished_at = utc_now()
    duration_ms = int((time.perf_counter() - timer) * 1000)

    attempt = TaskAttempt(
        task_id=task.id,
        action="PAY",
        provider=provider,
        ok=outcome.ok,
        retryable=bool(outcome.retryable),
        error_code=outcome.error_code,
        error_message_safe=outcome.error_message_safe,
        duration_ms=duration_ms,
        meta_json_safe={
            "manual_trigger": True,
            "rate_limit_wait_ms": limit.waited_ms,
            "rate_limit_rounds": limit.rounds,
            "reservation_id": reservation_id,
            "payment_card_configured": bool(payment_card),
            "payment_id": outcome.data.get("payment_id"),
        },
        started_at=started_at,
        finished_at=finished_at,
    )
    db.add(attempt)

    if not outcome.ok:
        await db.commit()
        raise HTTPException(
            status_code=status.HTTP_502_BAD_GATEWAY if outcome.retryable else status.HTTP_400_BAD_REQUEST,
            detail=outcome.error_message_safe or "Payment failed",
        )

    provider_http = dict(artifact_data.get("provider_http") or {})
    pay_trace = outcome.data.get("http_trace")
    if pay_trace:
        provider_http["pay"] = redact_sensitive(pay_trace)

    artifact.data_json_safe = {
        **artifact_data,
        "paid": True,
        "status": "paid",
        "payment_id": outcome.data.get("payment_id"),
        "ticket_no": outcome.data.get("ticket_no"),
        "provider_http": provider_http,
    }
    updated = True

    try:
        snapshot = await fetch_ticket_sync_snapshot(
            client=client,
            provider=provider,
            reservation_id=reservation_id,
            user_id=user.id,
            limiter=limiter,
        )
    except Exception as exc:
        sync_meta = dict(artifact.data_json_safe.get("provider_sync") or {})
        sync_meta["pay_sync_error"] = f"provider_sync_error:{type(exc).__name__}"
        artifact.data_json_safe = {
            **artifact.data_json_safe,
            "provider_sync": sync_meta,
            "last_provider_sync_at": utc_now().isoformat(),
        }
    else:
        merged_data = dict(artifact.data_json_safe)
        for key in (
            "waiting",
            "payment_deadline_at",
            "seat_count",
            "tickets",
            "reservation_snapshot",
            "provider_sync",
        ):
            value = snapshot.get(key)
            if value is not None:
                merged_data[key] = value

        snapshot_http = snapshot.get("provider_http")
        if isinstance(snapshot_http, dict):
            merged_data["provider_http"] = {
                **dict(merged_data.get("provider_http") or {}),
                **snapshot_http,
            }

        merged_data["paid"] = True
        merged_data["status"] = "paid"
        merged_data["last_provider_sync_at"] = snapshot.get("synced_at", utc_now().isoformat())
        artifact.data_json_safe = merged_data

    if task.state in ACTIVE_TASK_STATES or task.state in {"FAILED", "PAUSED"}:
        task.state = "COMPLETED"
    task.completed_at = task.completed_at or utc_now()
    task.failed_at = None
    task.updated_at = utc_now()

    await db.commit()

    task = await get_task_for_user(db, task_id=task_id, user=user)
    last_attempt_at = (await _last_attempt_map(db, [task.id])).get(task.id)
    ticket_summary = _ticket_summary_from_artifact(_latest_ticket_artifact_for_task(task))
    return TaskActionResponse(task=task_to_summary(task, last_attempt_at=last_attempt_at, ticket_summary=ticket_summary))


async def delete_task(db: AsyncSession, *, task_id: UUID, user: User) -> TaskActionResponse:
    task = await get_task_for_user(db, task_id=task_id, user=user)
    if task.state not in TERMINAL_TASK_STATES:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Only completed tasks can be deleted")

    task.hidden_at = utc_now()
    task.updated_at = utc_now()
    await db.commit()
    await db.refresh(task)
    return TaskActionResponse(task=task_to_summary(task))


async def cancel_ticket(db: AsyncSession, *, artifact_id: UUID, user: User) -> TicketCancelResponse:
    stmt = (
        select(Artifact)
        .join(Task, Task.id == Artifact.task_id)
        .where(Artifact.id == artifact_id)
        .where(Task.user_id == user.id)
        .where(Task.module == TASK_MODULE)
    )
    artifact = (await db.execute(stmt)).scalar_one_or_none()
    if artifact is None:
        return TicketCancelResponse(status="not_found", detail="Artifact not found")

    if artifact.data_json_safe.get("cancelled"):
        return TicketCancelResponse(status="already_cancelled", detail="Ticket is already cancelled")

    provider = str(artifact.data_json_safe.get("provider") or "")
    if provider not in {"SRT", "KTX"}:
        return TicketCancelResponse(status="not_supported", detail="Provider does not support ticket cancellation")

    redis = await get_redis_client()
    limiter = RedisTokenBucketLimiter(redis)
    client_cache: dict[str, object] = {}
    updated = False
    cancel_started_at = utc_now()
    cancel_duration_ms = 0
    cancel_limit_wait_ms = 0
    cancel_limit_rounds = 0

    updated = await _refresh_ticket_artifact_status(
        db,
        user=user,
        artifact=artifact,
        limiter=limiter,
        force=True,
        client_cache=client_cache,
    ) or updated
    client = client_cache.get(provider)
    if client is None:
        client = await _get_logged_in_provider_client(db, user=user, provider=provider)
        client_cache[provider] = client

    cancel_timer = time.perf_counter()
    try:
        limit = await limiter.acquire_provider_call(
            provider=provider,
            user_bucket_key=str(user.id),
            host_bucket_key="default-host",
        )
        cancel_limit_wait_ms = int(limit.waited_ms)
        cancel_limit_rounds = int(limit.rounds)
        outcome = await client.cancel(artifact_data=artifact.data_json_safe, user_id=str(user.id))
    except Exception as exc:
        cancel_duration_ms = int((time.perf_counter() - cancel_timer) * 1000)
        db.add(
            _build_task_attempt(
                task_id=artifact.task_id,
                action="CANCEL",
                provider=provider,
                ok=False,
                retryable=True,
                error_code="provider_transport_error",
                error_message_safe=f"{provider} cancel transport error: {type(exc).__name__}",
                duration_ms=cancel_duration_ms,
                meta_json_safe={
                    "manual_trigger": True,
                    "rate_limit_wait_ms": cancel_limit_wait_ms,
                    "rate_limit_rounds": cancel_limit_rounds,
                    "reservation_id": artifact.data_json_safe.get("reservation_id"),
                    "artifact_id": str(artifact.id),
                },
                started_at=cancel_started_at,
            )
        )
        await db.commit()
        raise HTTPException(
            status_code=status.HTTP_502_BAD_GATEWAY,
            detail=f"{provider} cancel transport error",
        ) from exc
    cancel_duration_ms = int((time.perf_counter() - cancel_timer) * 1000)

    db.add(
        _build_task_attempt(
            task_id=artifact.task_id,
            action="CANCEL",
            provider=provider,
            ok=outcome.ok,
            retryable=bool(outcome.retryable),
            error_code=outcome.error_code,
            error_message_safe=outcome.error_message_safe,
            duration_ms=cancel_duration_ms,
            meta_json_safe={
                "manual_trigger": True,
                "rate_limit_wait_ms": cancel_limit_wait_ms,
                "rate_limit_rounds": cancel_limit_rounds,
                "reservation_id": artifact.data_json_safe.get("reservation_id"),
                "artifact_id": str(artifact.id),
            },
            started_at=cancel_started_at,
        )
    )

    provider_http = dict(artifact.data_json_safe.get("provider_http") or {})
    cancel_trace = outcome.data.get("http_trace")
    if cancel_trace:
        provider_http["cancel"] = redact_sensitive(cancel_trace)

    if not outcome.ok and outcome.error_code == "not_supported":
        artifact.data_json_safe = {
            **artifact.data_json_safe,
            "provider_http": provider_http,
        }
        await db.commit()
        return TicketCancelResponse(status="not_supported", detail=outcome.error_message_safe or "not supported")

    if not outcome.ok and outcome.error_code == "reservation_not_found":
        artifact.data_json_safe = {
            **artifact.data_json_safe,
            "status": "reservation_not_found",
            "provider_http": provider_http,
        }
        await db.commit()
        return TicketCancelResponse(status="not_found", detail=outcome.error_message_safe or "Reservation not found")

    if not outcome.ok:
        artifact.data_json_safe = {
            **artifact.data_json_safe,
            "provider_http": provider_http,
        }
        await db.commit()
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail=outcome.error_message_safe or "Cancel failed")

    artifact.data_json_safe = {
        **artifact.data_json_safe,
        "cancelled": True,
        "status": "cancelled",
        "provider_http": provider_http,
    }
    updated = True
    updated = await _refresh_ticket_artifact_status(
        db,
        user=user,
        artifact=artifact,
        limiter=limiter,
        force=True,
        client_cache=client_cache,
    ) or updated

    artifact.data_json_safe = {
        **artifact.data_json_safe,
        "cancelled": True,
        "status": "cancelled",
    }
    await db.commit()
    return TicketCancelResponse(status="cancelled", detail="Ticket cancelled")


async def list_provider_reservations(
    db: AsyncSession,
    *,
    user: User,
    provider: str,
    paid_only: bool = False,
) -> ProviderReservationsResponse:
    client = await _get_logged_in_provider_client(db, user=user, provider=provider)
    outcome = await client.get_reservations(
        user_id=str(user.id),
        paid_only=paid_only,
    )
    if not outcome.ok:
        status_code = status.HTTP_502_BAD_GATEWAY if outcome.retryable else status.HTTP_400_BAD_REQUEST
        raise HTTPException(
            status_code=status_code,
            detail=outcome.error_message_safe or f"{provider} failed to return reservations",
        )

    return ProviderReservationsResponse(reservations=outcome.data.get("reservations", []))


async def get_provider_ticket_info(
    db: AsyncSession,
    *,
    user: User,
    provider: str,
    reservation_id: str,
) -> ProviderTicketInfoResponse:
    client = await _get_logged_in_provider_client(db, user=user, provider=provider)
    outcome = await client.ticket_info(
        reservation_id=reservation_id,
        user_id=str(user.id),
    )
    if not outcome.ok:
        status_code = status.HTTP_502_BAD_GATEWAY if outcome.retryable else status.HTTP_400_BAD_REQUEST
        raise HTTPException(
            status_code=status_code,
            detail=outcome.error_message_safe or f"{provider} failed to return ticket info",
        )

    return ProviderTicketInfoResponse(
        reservation_id=reservation_id,
        tickets=outcome.data.get("tickets", []),
        wct_no=outcome.data.get("wct_no"),
    )


async def cancel_provider_reservation(
    db: AsyncSession,
    *,
    user: User,
    provider: str,
    reservation_id: str,
) -> ProviderReservationCancelResponse:
    client = await _get_logged_in_provider_client(db, user=user, provider=provider)
    outcome = await client.cancel(
        artifact_data={"reservation_id": reservation_id},
        user_id=str(user.id),
    )
    if outcome.ok:
        return ProviderReservationCancelResponse(status="cancelled", detail="Reservation cancelled")
    if outcome.error_code in {"reservation_not_found"}:
        return ProviderReservationCancelResponse(
            status="not_found",
            detail=outcome.error_message_safe or "Reservation not found",
        )
    return ProviderReservationCancelResponse(
        status="failed",
        detail=outcome.error_message_safe or "Reservation cancel failed",
    )


def list_station_options() -> TrainStationsResponse:
    return TrainStationsResponse(
        stations=[
            TrainStationOut(
                name=station.name,
                srt_code=station.srt_code,
                srt_supported=station.srt_code is not None,
            )
            for station in ALL_STATIONS
        ]
    )
