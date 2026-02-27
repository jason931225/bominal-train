from __future__ import annotations

import asyncio
import hashlib
import json
import logging
import time
from datetime import datetime, timedelta, timezone
from typing import Any
from uuid import UUID, uuid4

from fastapi import HTTPException, status
from sqlalchemy import Select, and_, delete, func, or_, select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from app.core.config import get_settings
from app.core.crypto import validate_safe_metadata
from app.core.crypto.secrets_store import build_encrypted_secret, decrypt_secret
from app.core.crypto.redaction import redact_sensitive
from app.core.redis import get_redis_client
from app.core.time import utc_now
from app.db.models import Artifact, Secret, Task, TaskAttempt, User
from app.modules.train.constants import (
    ACTIVE_TASK_STATES,
    SECRET_KIND_KTX_CREDENTIALS,
    SECRET_KIND_SRT_CREDENTIALS,
    SPEC_KEY_MANUAL_RETRY_LAST_AT,
    SPEC_KEY_NEXT_RUN_AT,
    SPEC_KEY_NOTIFY_EMAIL_JOB_ID,
    SPEC_KEY_NOTIFY_EMAIL_SENT_AT,
    SPEC_KEY_NOTIFY_EMAIL_STATE,
    SPEC_KEY_RETRY_ON_EXPIRY,
    TASK_MODULE,
    TERMINAL_TASK_STATES,
    credential_kind,
)
from app.modules.train.events import publish_task_state_event
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
    TrainTaskDuplicateCheckResponse,
    TrainTaskDuplicateMatchOut,
    TrainTaskDuplicateSummaryOut,
    TrainTaskCreateRequest,
    TrainTaskCreateResponse,
)
from app.modules.train.stations import ALL_STATIONS, station_code_for_name, station_exists
from app.modules.train.ticket_sync import fetch_ticket_sync_snapshot
from app.modules.train.timezone import KST
from app.services.system_payment import is_payment_runtime_enabled
from app.services.wallet import get_payment_card_for_execution

settings = get_settings()
logger = logging.getLogger(__name__)
TASK_VISIBILITY_RETENTION_DAYS = 365
MANUAL_RETRY_COOLDOWN_SECONDS = 15
MANUAL_RETRY_LAST_AT_KEY = SPEC_KEY_MANUAL_RETRY_LAST_AT
NEXT_RUN_AT_KEY = SPEC_KEY_NEXT_RUN_AT
RETRY_ON_EXPIRY_KEY = SPEC_KEY_RETRY_ON_EXPIRY
SYNC_PROVIDER_META_STABLE_KEYS = frozenset(
    {
        "provider",
        "reservation_id",
        "reservations_ok",
        "reservations_error",
        "ticket_info_ok",
        "ticket_info_error",
        "error",
        "pay_sync_error",
    }
)


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


def _as_aware_utc_datetime(value: datetime) -> datetime:
    if value.tzinfo is None:
        return value.replace(tzinfo=timezone.utc)
    return value.astimezone(timezone.utc)


def _compute_retry_now_status(
    task: Task,
    *,
    now: datetime,
) -> tuple[bool, str | None, datetime | None]:
    now = _as_aware_utc_datetime(now)

    if task.state == "PAUSED" or task.paused_at is not None:
        return False, "paused_use_resume", None
    if task.state in {"RUNNING", "RESERVING", "PAYING"}:
        return False, "task_running", None
    if task.state not in {"QUEUED", "POLLING", "EXPIRED", "CANCELLED", "FAILED"}:
        return False, "not_eligible_state", None

    deadline_at = _as_aware_utc_datetime(task.deadline_at)
    if now >= deadline_at:
        return False, "deadline_passed", None

    last_manual_retry_at = _parse_iso_datetime(str((task.spec_json or {}).get(MANUAL_RETRY_LAST_AT_KEY) or ""))
    if last_manual_retry_at is None:
        return True, None, None

    available_at = _as_aware_utc_datetime(last_manual_retry_at) + timedelta(seconds=MANUAL_RETRY_COOLDOWN_SECONDS)
    if now < available_at:
        return False, "cooldown_active", available_at

    return True, None, None


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
        meta_json_safe=validate_safe_metadata(meta_json_safe) if meta_json_safe else None,
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


def _is_provider_enabled(provider: str) -> bool:
    normalized = str(provider).upper()
    if normalized == "KTX":
        return bool(settings.train_ktx_enabled)
    return normalized == "SRT"


def _provider_disabled_detail(provider: str) -> str:
    normalized = str(provider).upper()
    if normalized == "KTX":
        return "KTX provider is temporarily unavailable"
    return f"{normalized} provider is temporarily unavailable"


def _provider_disabled_error_code(provider: str) -> str:
    return f"{str(provider).lower()}_provider_disabled"


def _disabled_provider_status(provider: str) -> ProviderCredentialStatus:
    return ProviderCredentialStatus(
        configured=False,
        verified=False,
        detail=_provider_disabled_detail(provider),
    )


def _require_provider_enabled(provider: str, *, status_code: int = status.HTTP_503_SERVICE_UNAVAILABLE) -> None:
    if _is_provider_enabled(provider):
        return
    raise HTTPException(status_code=status_code, detail=_provider_disabled_detail(provider))


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
                detail=None,
            )
        return ProviderCredentialStatus(
            configured=True,
            verified=False,
            detail=f"{provider} login timed out. Try again.",
        )
    except Exception:
        if verified_at is not None:
            return ProviderCredentialStatus(
                configured=True,
                verified=True,
                detail=None,
            )
        return ProviderCredentialStatus(
            configured=True,
            verified=False,
            detail=f"{provider} login check failed",
        )

    if not login_outcome.ok:
        if login_outcome.retryable and verified_at is not None:
            return ProviderCredentialStatus(
                configured=True,
                verified=True,
                detail=None,
            )
        return ProviderCredentialStatus(
            configured=True,
            verified=False,
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
            detail="Credentials are missing",
        )

    verified_at = _parse_verified_at(creds.get("verified_at"))
    verified = verified_at is not None
    return ProviderCredentialStatus(
        configured=True,
        verified=verified,
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
    if _is_provider_enabled("KTX"):
        ktx = await _verify_provider_credentials_guarded(db, user=user, provider="KTX")
    else:
        ktx = _disabled_provider_status("KTX")
    srt = await _verify_provider_credentials_guarded(db, user=user, provider="SRT")
    return ProviderCredentialsStatusResponse(ktx=ktx, srt=srt)


async def get_srt_credential_status(db: AsyncSession, *, user: User) -> SRTCredentialStatusResponse:
    status_info = await _verify_provider_credentials_guarded(db, user=user, provider="SRT")
    return SRTCredentialStatusResponse(**status_info.model_dump())


async def get_ktx_credential_status(db: AsyncSession, *, user: User) -> KTXCredentialStatusResponse:
    if not _is_provider_enabled("KTX"):
        return KTXCredentialStatusResponse(**_disabled_provider_status("KTX").model_dump())
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
        logger.warning(
            "SRT credential verification failed",
            extra={
                "provider": "SRT",
                "error_code": login_outcome.error_code,
                "retryable": login_outcome.retryable,
            },
        )
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
    return SRTCredentialStatusResponse(configured=True, verified=True, detail=None)


async def set_ktx_credentials(
    db: AsyncSession,
    *,
    user: User,
    payload: KTXCredentialsSetRequest,
) -> KTXCredentialStatusResponse:
    _require_provider_enabled("KTX")
    username = payload.username.strip()
    password = payload.password

    client = get_provider_client("KTX")
    login_outcome = await client.login(
        user_id=str(user.id),
        credentials={"username": username, "password": password},
    )
    if not login_outcome.ok:
        logger.warning(
            "KTX credential verification failed",
            extra={
                "provider": "KTX",
                "error_code": login_outcome.error_code,
                "retryable": login_outcome.retryable,
            },
        )
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
    return KTXCredentialStatusResponse(configured=True, verified=True, detail=None)


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
    _require_provider_enabled(provider)
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
                **({"arrival_at": item.arrival_at.isoformat()} if item.arrival_at is not None else {}),
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
    for provider in providers:
        if not _is_provider_enabled(provider):
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail=_provider_disabled_detail(provider),
            )
    return providers


def normalize_task_spec(payload: TrainTaskCreateRequest, *, ranked_trains: list[dict]) -> dict:
    dep_srt_code = station_code_for_name(payload.dep)
    arr_srt_code = station_code_for_name(payload.arr)
    providers = _resolve_task_providers(ranked_trains)
    effective_auto_pay = bool(settings.payment_enabled and payload.auto_pay)
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
        "auto_pay": effective_auto_pay,
        "retry_on_expiry": bool(payload.retry_on_expiry),
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


def _force_unique_active_idempotency_key(user_id: UUID, spec_json: dict) -> str:
    base_key = compute_idempotency_key(user_id, spec_json)
    return hashlib.sha256(f"{base_key}:{uuid4().hex}".encode("utf-8")).hexdigest()


def _build_retry_on_expiry_spec(task: Task) -> dict:
    retry_spec = dict(task.spec_json or {})
    retry_spec.pop(MANUAL_RETRY_LAST_AT_KEY, None)
    retry_spec.pop(NEXT_RUN_AT_KEY, None)
    retry_spec.pop(SPEC_KEY_NOTIFY_EMAIL_SENT_AT, None)
    retry_spec.pop(SPEC_KEY_NOTIFY_EMAIL_STATE, None)
    retry_spec.pop(SPEC_KEY_NOTIFY_EMAIL_JOB_ID, None)
    return validate_safe_metadata(retry_spec)


def _apply_retry_reset(task: Task, *, retry_spec: dict, now: datetime) -> None:
    task.spec_json = validate_safe_metadata(retry_spec)
    task.state = "QUEUED"
    task.paused_at = None
    task.completed_at = None
    task.failed_at = None
    task.cancelled_at = None
    task.updated_at = now


async def _requeue_task_after_expiry_in_place(*, task: Task, now: datetime) -> bool:
    spec = dict(task.spec_json or {})
    if not bool(spec.get(RETRY_ON_EXPIRY_KEY)):
        return False

    retry_spec = _build_retry_on_expiry_spec(task)
    try:
        retry_deadline_at = _as_aware_utc_datetime(compute_deadline_from_spec(retry_spec))
    except Exception:
        return False

    if now >= retry_deadline_at:
        return False

    _apply_retry_reset(task, retry_spec=retry_spec, now=now)
    task.deadline_at = retry_deadline_at
    return True


async def _activate_requeued_task(task: Task) -> None:
    try:
        enqueued = await enqueue_train_task(str(task.id))
        if enqueued is False:
            await enqueue_train_task(str(task.id), defer_seconds=0.01)
    except Exception as exc:
        logger.warning("Failed to enqueue retry-on-expiry task %s: %s", task.id, type(exc).__name__)
        return

    try:
        await publish_task_state_event(
            user_id=task.user_id,
            task_id=task.id,
            state=task.state,
            updated_at=task.updated_at,
        )
    except Exception as exc:
        logger.warning("Failed to publish retry-on-expiry task event %s: %s", task.id, type(exc).__name__)


def _departure_key_map_from_spec(spec_json: dict) -> dict[int, datetime]:
    ranked = spec_json.get("selected_trains_ranked", [])
    if not isinstance(ranked, list):
        return {}

    keys: dict[int, datetime] = {}
    for row in ranked:
        if not isinstance(row, dict):
            continue
        departure_raw = str(row.get("departure_at") or "")
        departure_at = _parse_iso_datetime(departure_raw)
        if departure_at is None:
            continue
        aware = _as_aware_utc_datetime(departure_at).replace(microsecond=0)
        keys[int(aware.timestamp())] = aware
    return keys


def _duplicate_category_for_task(task: Task, ticket_summary: dict | None) -> str | None:
    summary = ticket_summary or {}
    ticket_status = str(summary.get("ticket_status") or "")
    ticket_paid = summary.get("ticket_paid")

    if ticket_status == "waiting" and ticket_paid is not True:
        return "waiting"
    if ticket_status in {"awaiting_payment", "reserved"} and ticket_paid is not True:
        return "already_reserved"
    if task.state in ACTIVE_TASK_STATES:
        return "polling"
    return None


async def _duplicate_match_rows_for_spec(
    db: AsyncSession,
    *,
    user: User,
    spec_json: dict,
) -> list[tuple[Task, str, datetime, str | None]]:
    incoming_dep = str(spec_json.get("dep") or "")
    incoming_arr = str(spec_json.get("arr") or "")
    incoming_date = str(spec_json.get("date") or "")
    incoming_departure_keys = _departure_key_map_from_spec(spec_json)
    if not incoming_dep or not incoming_arr or not incoming_date or not incoming_departure_keys:
        return []

    stmt = (
        select(Task)
        .where(Task.user_id == user.id)
        .where(Task.module == TASK_MODULE)
        .where(Task.hidden_at.is_(None))
        .where(Task.state.in_(ACTIVE_TASK_STATES | {"COMPLETED"}))
        .order_by(Task.created_at.desc(), Task.id.desc())
    )
    tasks = (await db.execute(stmt)).scalars().all()
    if not tasks:
        return []

    ticket_artifacts = await _latest_ticket_artifact_map(db, [task.id for task in tasks])
    ticket_summaries: dict[UUID, dict | None] = {
        task.id: _ticket_summary_from_artifact(ticket_artifacts.get(task.id))
        for task in tasks
    }

    matches: list[tuple[Task, str, datetime, str | None]] = []
    for task in tasks:
        task_spec = dict(task.spec_json or {})
        if str(task_spec.get("dep") or "") != incoming_dep:
            continue
        if str(task_spec.get("arr") or "") != incoming_arr:
            continue
        if str(task_spec.get("date") or "") != incoming_date:
            continue

        task_departure_keys = _departure_key_map_from_spec(task_spec)
        overlap = set(task_departure_keys).intersection(incoming_departure_keys)
        if not overlap:
            continue

        category = _duplicate_category_for_task(task, ticket_summaries.get(task.id))
        if category is None:
            continue

        matched_key = min(overlap)
        departure_at = task_departure_keys[matched_key]
        ticket_status = str((ticket_summaries.get(task.id) or {}).get("ticket_status") or "") or None
        matches.append((task, category, departure_at, ticket_status))

    return matches


def _duplicate_check_response_from_rows(
    rows: list[tuple[Task, str, datetime, str | None]],
) -> TrainTaskDuplicateCheckResponse:
    summary = TrainTaskDuplicateSummaryOut()
    matches: list[TrainTaskDuplicateMatchOut] = []

    for task, category, departure_at, ticket_status in rows:
        if category == "already_reserved":
            summary.already_reserved += 1
        elif category == "waiting":
            summary.waiting += 1
        elif category == "polling":
            summary.polling += 1

        matches.append(
            TrainTaskDuplicateMatchOut(
                task_id=task.id,
                state=task.state,
                category=category,  # type: ignore[arg-type]
                departure_at=departure_at,
                ticket_status=ticket_status,
            )
        )

    return TrainTaskDuplicateCheckResponse(
        has_duplicate=bool(matches),
        summary=summary,
        matches=matches,
    )


async def check_task_duplicates(
    db: AsyncSession,
    *,
    user: User,
    payload: TrainTaskCreateRequest,
) -> TrainTaskDuplicateCheckResponse:
    if not station_exists(payload.dep) or not station_exists(payload.arr):
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Unknown station name")

    ranked_trains = _sorted_ranked_trains(payload)
    spec_json = normalize_task_spec(payload, ranked_trains=ranked_trains)
    rows = await _duplicate_match_rows_for_spec(db, user=user, spec_json=spec_json)
    return _duplicate_check_response_from_rows(rows)


def _ticket_summary_from_artifact(artifact: Artifact | None) -> dict | None:
    if artifact is None:
        return None
    payload = artifact.data_json_safe or {}
    reservation_snapshot = payload.get("reservation_snapshot")

    ticket_train_no_raw = payload.get("train_no")
    if (ticket_train_no_raw is None or str(ticket_train_no_raw).strip() == "") and isinstance(reservation_snapshot, dict):
        ticket_train_no_raw = reservation_snapshot.get("train_no")
    ticket_train_no = str(ticket_train_no_raw).strip() if ticket_train_no_raw is not None else ""
    if ticket_train_no == "":
        ticket_train_no = None

    ticket_seats: list[str] = []
    seen_seats: set[str] = set()
    tickets_raw = payload.get("tickets")
    if isinstance(tickets_raw, list):
        for row in tickets_raw:
            if not isinstance(row, dict):
                continue
            seat_no_raw = row.get("seat_no")
            if seat_no_raw is None:
                continue
            seat_no = str(seat_no_raw).strip()
            if seat_no == "":
                continue
            car_no_raw = row.get("car_no")
            car_no = str(car_no_raw).strip() if car_no_raw is not None else ""
            label = f"{car_no}-{seat_no}" if car_no else seat_no
            if label in seen_seats:
                continue
            seen_seats.add(label)
            ticket_seats.append(label)

    ticket_seat_count = None
    seat_count_raw = payload.get("seat_count")
    if isinstance(seat_count_raw, int) and not isinstance(seat_count_raw, bool):
        ticket_seat_count = max(0, seat_count_raw)
    elif ticket_seats:
        ticket_seat_count = len(ticket_seats)

    return {
        "ticket_status": payload.get("status"),
        "ticket_paid": payload.get("paid"),
        "ticket_payment_deadline_at": _parse_iso_datetime(str(payload.get("payment_deadline_at") or "")),
        "ticket_reservation_id": payload.get("reservation_id"),
        "ticket_train_no": ticket_train_no,
        "ticket_seat_count": ticket_seat_count,
        "ticket_seats": ticket_seats or None,
    }


def _is_manual_payment_pending(ticket_summary: dict | None) -> bool:
    if not ticket_summary:
        return False
    ticket_status = str(ticket_summary.get("ticket_status") or "")
    ticket_paid = ticket_summary.get("ticket_paid")
    return ticket_status in {"awaiting_payment", "waiting"} and ticket_paid is not True


def _is_manual_payment_expired(
    ticket_summary: dict | None,
    *,
    now: datetime,
    include_reserved: bool = False,
) -> bool:
    if not ticket_summary:
        return False
    ticket_status = str(ticket_summary.get("ticket_status") or "")
    ticket_paid = ticket_summary.get("ticket_paid")
    pending_statuses = {"awaiting_payment", "waiting"}
    if include_reserved:
        pending_statuses.add("reserved")
    if ticket_status not in pending_statuses or ticket_paid is True:
        return False
    payment_deadline_at = ticket_summary.get("ticket_payment_deadline_at") if ticket_summary else None
    if not isinstance(payment_deadline_at, datetime):
        return False
    return now >= _as_aware_utc_datetime(payment_deadline_at)


def _is_waitlisted_unpaid(ticket_summary: dict | None) -> bool:
    if not ticket_summary:
        return False
    ticket_status = str(ticket_summary.get("ticket_status") or "")
    ticket_paid = ticket_summary.get("ticket_paid")
    return ticket_status == "waiting" and ticket_paid is not True


def _should_refresh_pending_ticket_status(ticket_summary: dict | None) -> bool:
    if not ticket_summary:
        return False
    ticket_status = str(ticket_summary.get("ticket_status") or "")
    ticket_paid = ticket_summary.get("ticket_paid")
    # "waiting" tickets are polled by worker every 5 minutes. Skip read-path
    # provider sync calls for waitlisted tasks to avoid noisy DB writes.
    return ticket_status in {"awaiting_payment"} and ticket_paid is not True


def _is_active_for_listing(task: Task, ticket_summary: dict | None) -> bool:
    if task.state in ACTIVE_TASK_STATES:
        return True
    if task.state == "COMPLETED" and _is_manual_payment_pending(ticket_summary):
        return True
    return False


def _expire_manual_payment_task_if_due(
    task: Task,
    *,
    ticket_summary: dict | None,
    ticket_artifact: Artifact | None = None,
    now: datetime,
) -> bool:
    if task.state not in ACTIVE_TASK_STATES and task.state != "COMPLETED":
        return False
    include_reserved = task.state in ACTIVE_TASK_STATES
    if not _is_manual_payment_expired(ticket_summary, now=now, include_reserved=include_reserved):
        return False
    task.state = "EXPIRED"
    task.completed_at = None
    task.paused_at = None
    task.updated_at = now
    if ticket_artifact is not None:
        artifact_data = dict(ticket_artifact.data_json_safe or {})
        artifact_data["status"] = "expired"
        artifact_data["expired"] = True
        if artifact_data.get("waiting") is True:
            artifact_data["waiting"] = False
        ticket_artifact.data_json_safe = validate_safe_metadata(artifact_data)
    return True


def task_to_summary(
    task: Task,
    last_attempt_at: datetime | None = None,
    latest_attempt: TaskAttempt | None = None,
    ticket_summary: dict | None = None,
    now: datetime | None = None,
) -> TaskSummaryOut:
    ticket_summary = ticket_summary or {}
    now = now or utc_now()
    next_run_at = (
        _parse_iso_datetime(str((task.spec_json or {}).get(NEXT_RUN_AT_KEY) or ""))
        if task.state == "POLLING"
        else None
    )
    retry_now_allowed, retry_now_reason, retry_now_available_at = _compute_retry_now_status(task, now=now)
    if latest_attempt is not None:
        last_attempt_at = latest_attempt.finished_at
    last_attempt_finished_at = last_attempt_at
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
        last_attempt_action=latest_attempt.action if latest_attempt else None,
        last_attempt_ok=latest_attempt.ok if latest_attempt else None,
        last_attempt_error_code=latest_attempt.error_code if latest_attempt else None,
        last_attempt_error_message_safe=latest_attempt.error_message_safe if latest_attempt else None,
        last_attempt_finished_at=last_attempt_finished_at,
        next_run_at=next_run_at,
        retry_now_allowed=retry_now_allowed,
        retry_now_reason=retry_now_reason,
        retry_now_available_at=retry_now_available_at,
        spec_json=task.spec_json,
        ticket_status=ticket_summary.get("ticket_status"),
        ticket_paid=ticket_summary.get("ticket_paid"),
        ticket_payment_deadline_at=ticket_summary.get("ticket_payment_deadline_at"),
        ticket_reservation_id=ticket_summary.get("ticket_reservation_id"),
        ticket_train_no=ticket_summary.get("ticket_train_no"),
        ticket_seat_count=ticket_summary.get("ticket_seat_count"),
        ticket_seats=ticket_summary.get("ticket_seats"),
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
        if not _is_provider_enabled(provider):
            provider_errors[provider] = {
                "error_code": _provider_disabled_error_code(provider),
                "error_message": _provider_disabled_detail(provider),
            }
            continue

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
    return TrainSearchResponse(schedules=schedules, provider_errors=provider_errors)


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

    if not payload.confirm_duplicate:
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

        duplicate_rows = await _duplicate_match_rows_for_spec(db, user=user, spec_json=spec_json)
        if duplicate_rows:
            duplicate_task = duplicate_rows[0][0]
            return TrainTaskCreateResponse(task=task_to_summary(duplicate_task), queued=False, deduplicated=True)
    else:
        idempotency_key = _force_unique_active_idempotency_key(user.id, spec_json)

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


async def _latest_attempt_map(db: AsyncSession, task_ids: list[UUID]) -> dict[UUID, TaskAttempt]:
    if not task_ids:
        return {}

    is_postgres = bool(db.bind is not None and db.bind.dialect.name == "postgresql")
    if is_postgres:
        stmt = (
            select(TaskAttempt)
            .where(TaskAttempt.task_id.in_(task_ids))
            .distinct(TaskAttempt.task_id)
            .order_by(TaskAttempt.task_id.asc(), TaskAttempt.finished_at.desc(), TaskAttempt.id.desc())
        )
    else:
        ranked_attempts = (
            select(
                TaskAttempt.id.label("attempt_id"),
                TaskAttempt.task_id.label("task_id"),
                func.row_number()
                .over(
                    partition_by=TaskAttempt.task_id,
                    order_by=(TaskAttempt.finished_at.desc(), TaskAttempt.id.desc()),
                )
                .label("attempt_rank"),
            )
            .where(TaskAttempt.task_id.in_(task_ids))
            .subquery()
        )
        stmt = (
            select(TaskAttempt)
            .join(ranked_attempts, TaskAttempt.id == ranked_attempts.c.attempt_id)
            .where(ranked_attempts.c.attempt_rank == 1)
        )
    attempts = (await db.execute(stmt)).scalars().all()
    return {attempt.task_id: attempt for attempt in attempts}


async def _latest_ticket_artifact_map(db: AsyncSession, task_ids: list[UUID]) -> dict[UUID, Artifact]:
    if not task_ids:
        return {}

    is_postgres = bool(db.bind is not None and db.bind.dialect.name == "postgresql")
    if is_postgres:
        stmt = (
            select(Artifact)
            .where(Artifact.task_id.in_(task_ids))
            .where(Artifact.kind == "ticket")
            .distinct(Artifact.task_id)
            .order_by(Artifact.task_id.asc(), Artifact.created_at.desc(), Artifact.id.desc())
        )
    else:
        ranked_artifacts = (
            select(
                Artifact.id.label("artifact_id"),
                Artifact.task_id.label("task_id"),
                func.row_number()
                .over(
                    partition_by=Artifact.task_id,
                    order_by=(Artifact.created_at.desc(), Artifact.id.desc()),
                )
                .label("artifact_rank"),
            )
            .where(Artifact.task_id.in_(task_ids))
            .where(Artifact.kind == "ticket")
            .subquery()
        )
        stmt = (
            select(Artifact)
            .join(ranked_artifacts, Artifact.id == ranked_artifacts.c.artifact_id)
            .where(ranked_artifacts.c.artifact_rank == 1)
        )
    artifacts = (await db.execute(stmt)).scalars().all()
    return {artifact.task_id: artifact for artifact in artifacts}


def _task_list_stmt(user: User, status_filter: str, *, limit: int) -> Select[tuple[Task]]:
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
        # Include COMPLETED candidates so manual-payment-pending reservations can be
        # reclassified into active after ticket metadata inspection.
        stmt = stmt.where(Task.state.in_(ACTIVE_TASK_STATES | {"COMPLETED"}))
    elif status_filter == "completed":
        stmt = stmt.where(Task.state.in_(TERMINAL_TASK_STATES)).where(terminal_visible_expr)
    else:
        stmt = stmt.where(
            or_(
                Task.state.in_(ACTIVE_TASK_STATES),
                and_(Task.state.in_(TERMINAL_TASK_STATES), terminal_visible_expr),
            )
        )

    bounded_limit = max(1, limit)
    if status_filter == "active":
        # Over-fetch so we can filter out non-pending COMPLETED rows without starving
        # the active list.
        bounded_limit = bounded_limit * 4
    return stmt.order_by(Task.created_at.desc(), Task.id.desc()).limit(bounded_limit)


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
    limit: int = 200,
    refresh_completed: bool = False,
) -> TaskListResponse:
    stmt = _task_list_stmt(user, status_filter, limit=limit)
    tasks = (await db.execute(stmt)).scalars().all()
    now = utc_now()

    latest_attempts = await _latest_attempt_map(db, [task.id for task in tasks])
    ticket_artifacts = await _latest_ticket_artifact_map(db, [task.id for task in tasks])

    # Read paths are intentionally DB-only snapshots.
    # Provider sync occurs in worker/manual actions so list queries stay cheap
    # and state updates remain strictly state-change driven.
    _ = refresh_completed

    ticket_summaries: dict[UUID, dict | None] = {
        task.id: _ticket_summary_from_artifact(ticket_artifacts.get(task.id)) for task in tasks
    }
    expired_updates = False
    requeued_retry_tasks: list[Task] = []
    for task in tasks:
        ticket_artifact = ticket_artifacts.get(task.id)
        expired_changed = _expire_manual_payment_task_if_due(
            task,
            ticket_summary=ticket_summaries.get(task.id),
            ticket_artifact=ticket_artifact,
            now=now,
        )
        if expired_changed and ticket_artifact is not None:
            ticket_summaries[task.id] = _ticket_summary_from_artifact(ticket_artifact)
        if expired_changed:
            if await _requeue_task_after_expiry_in_place(task=task, now=now):
                requeued_retry_tasks.append(task)
        expired_updates = expired_changed or expired_updates
    if expired_updates:
        await db.commit()
        for retry_task in requeued_retry_tasks:
            await _activate_requeued_task(retry_task)

    if status_filter == "active":
        tasks = [task for task in tasks if _is_active_for_listing(task, ticket_summaries.get(task.id))][: max(1, limit)]
    elif status_filter == "completed":
        tasks = [
            task
            for task in tasks
            if task.state in TERMINAL_TASK_STATES and not _is_active_for_listing(task, ticket_summaries.get(task.id))
        ][: max(1, limit)]

    return TaskListResponse(
        tasks=[
            task_to_summary(
                task,
                latest_attempt=latest_attempts.get(task.id),
                ticket_summary=ticket_summaries.get(task.id),
                now=now,
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
    _ = artifact_data
    _ = force
    # Refresh cooldown intentionally removed: sync execution is state-change
    # driven by worker/manual actions rather than read-path timing windows.
    return True


def _latest_ticket_artifact_for_task(task: Task) -> Artifact | None:
    ticket_artifacts = [artifact for artifact in task.artifacts if artifact.kind == "ticket"]
    if not ticket_artifacts:
        return None
    return max(ticket_artifacts, key=lambda artifact: artifact.created_at)


def _compact_provider_sync_payload(value: Any) -> dict[str, Any] | None:
    if not isinstance(value, dict):
        return None
    compact: dict[str, Any] = {}
    for key in SYNC_PROVIDER_META_STABLE_KEYS:
        if key in value:
            compact[key] = value[key]
    return compact or None


def _merge_provider_http_once(*, current_payload: dict[str, Any] | None, incoming_payload: Any) -> dict[str, Any] | None:
    existing = dict(current_payload or {})
    if not isinstance(incoming_payload, dict):
        return existing or None
    for key, value in incoming_payload.items():
        key_str = str(key)
        # Keep first-seen trace per operation to avoid high-churn rewrites
        # caused by volatile provider tracing fields.
        if key_str not in existing and value is not None:
            existing[key_str] = value
    return existing or None


def _ticket_state_change_fingerprint(payload: dict[str, Any]) -> tuple[Any, ...]:
    provider_sync = payload.get("provider_sync") if isinstance(payload.get("provider_sync"), dict) else {}

    normalized_seats: list[tuple[str, str]] = []
    tickets = payload.get("tickets")
    if isinstance(tickets, list):
        for row in tickets:
            if not isinstance(row, dict):
                continue
            car = str(row.get("car_no") or "")
            seat = str(row.get("seat_no") or "")
            if seat:
                normalized_seats.append((car, seat))
    normalized_seats.sort()

    return (
        str(payload.get("status") or "").strip().lower(),
        payload.get("paid"),
        payload.get("waiting"),
        payload.get("expired"),
        bool(payload.get("cancelled")),
        str(payload.get("payment_deadline_at") or ""),
        payload.get("seat_count"),
        tuple(normalized_seats),
        str(payload.get("reservation_id") or ""),
        str(payload.get("payment_id") or ""),
        str(payload.get("ticket_no") or ""),
        str(provider_sync.get("error") or ""),
        str(provider_sync.get("reservations_error") or ""),
        str(provider_sync.get("ticket_info_error") or ""),
        str(provider_sync.get("pay_sync_error") or ""),
    )


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
            sync_meta = dict(_compact_provider_sync_payload(current_data.get("provider_sync")) or {})
            sync_meta["error"] = exc.detail
            if sync_meta == (current_data.get("provider_sync") or {}):
                return False
            current_data["provider_sync"] = sync_meta
            current_data["last_provider_sync_at"] = utc_now().isoformat()
            artifact.data_json_safe = validate_safe_metadata(current_data)
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
        sync_meta = dict(_compact_provider_sync_payload(current_data.get("provider_sync")) or {})
        sync_meta["error"] = f"provider_sync_error:{type(exc).__name__}"
        if sync_meta == (current_data.get("provider_sync") or {}):
            return False
        current_data["provider_sync"] = sync_meta
        current_data["last_provider_sync_at"] = utc_now().isoformat()
        artifact.data_json_safe = validate_safe_metadata(current_data)
        return True

    merged_status = snapshot.get("status", current_data.get("status"))
    if current_data.get("cancelled"):
        merged_status = "cancelled"

    merged_provider_sync = _compact_provider_sync_payload(snapshot.get("provider_sync"))
    merged_provider_http = _merge_provider_http_once(
        current_payload=dict(current_data.get("provider_http") or {}),
        incoming_payload=snapshot.get("provider_http"),
    )

    merged_data = {
        **current_data,
        "status": merged_status,
        "paid": snapshot.get("paid", current_data.get("paid")),
        "waiting": snapshot.get("waiting", current_data.get("waiting")),
        "expired": snapshot.get("expired", current_data.get("expired")),
        "payment_deadline_at": snapshot.get("payment_deadline_at", current_data.get("payment_deadline_at")),
        "seat_count": snapshot.get("seat_count", current_data.get("seat_count")),
        "tickets": snapshot.get("tickets", current_data.get("tickets", [])),
        "reservation_snapshot": snapshot.get("reservation_snapshot", current_data.get("reservation_snapshot")),
    }
    if merged_provider_sync is not None:
        merged_data["provider_sync"] = merged_provider_sync
    if merged_provider_http is not None:
        merged_data["provider_http"] = merged_provider_http

    current_fingerprint = _ticket_state_change_fingerprint(current_data)
    merged_fingerprint = _ticket_state_change_fingerprint(merged_data)
    if merged_fingerprint != current_fingerprint:
        merged_data["last_provider_sync_at"] = snapshot.get("synced_at", utc_now().isoformat())
        artifact.data_json_safe = validate_safe_metadata(merged_data)
        return True
    return False


async def get_task_detail(db: AsyncSession, *, task_id: UUID, user: User) -> TaskDetailOut:
    task = await get_task_for_user(db, task_id=task_id, user=user)

    attempts = sorted(task.attempts, key=lambda row: row.started_at)
    artifacts = sorted(task.artifacts, key=lambda row: row.created_at)
    latest_ticket_artifact = _latest_ticket_artifact_for_task(task)
    ticket_summary = _ticket_summary_from_artifact(latest_ticket_artifact)

    latest_attempt = max(attempts, key=lambda row: row.finished_at) if attempts else None
    last_attempt_at = latest_attempt.finished_at if latest_attempt else None
    now = utc_now()
    requeued_retry = False
    if _expire_manual_payment_task_if_due(
        task,
        ticket_summary=ticket_summary,
        ticket_artifact=latest_ticket_artifact,
        now=now,
    ):
        requeued_retry = await _requeue_task_after_expiry_in_place(task=task, now=now)
        await db.commit()
        if requeued_retry:
            await _activate_requeued_task(task)
        ticket_summary = _ticket_summary_from_artifact(latest_ticket_artifact)

    return TaskDetailOut(
        task=task_to_summary(
            task,
            last_attempt_at=last_attempt_at,
            latest_attempt=latest_attempt,
            ticket_summary=ticket_summary,
            now=now,
        ),
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


async def refresh_task_detail(db: AsyncSession, *, task_id: UUID, user: User) -> TaskDetailOut:
    task = await get_task_for_user(db, task_id=task_id, user=user)
    artifact = _latest_ticket_artifact_for_task(task)
    if artifact is not None:
        redis = await get_redis_client()
        limiter = RedisTokenBucketLimiter(redis)
        updated = await _refresh_ticket_artifact_status(
            db,
            user=user,
            artifact=artifact,
            limiter=limiter,
            force=True,
        )
        if updated:
            task.updated_at = utc_now()
            await db.commit()
    return await get_task_detail(db, task_id=task_id, user=user)


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
    if task.state in TERMINAL_TASK_STATES:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Cannot resume terminal task")
    if task.state != "PAUSED" and task.paused_at is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Task is not paused")

    next_spec = dict(task.spec_json or {})
    next_spec.pop(NEXT_RUN_AT_KEY, None)
    task.spec_json = next_spec
    task.state = "QUEUED"
    task.paused_at = None
    task.updated_at = utc_now()
    await db.commit()
    await db.refresh(task)

    enqueued = await enqueue_train_task(str(task.id))
    if enqueued is False:
        # Deterministic job-id dedupe can reject immediate re-enqueue when a
        # stale queued job key exists; fall back to a near-immediate deferred
        # enqueue (non-deterministic id) so manual retry actually executes.
        await enqueue_train_task(str(task.id), defer_seconds=0.01)

    return TaskActionResponse(task=task_to_summary(task))


async def retry_task_now(db: AsyncSession, *, task_id: UUID, user: User) -> TaskActionResponse:
    task = await get_task_for_user(db, task_id=task_id, user=user)
    now = utc_now()

    allowed, reason, _available_at = _compute_retry_now_status(task, now=now)
    if not allowed:
        if reason == "paused_use_resume":
            raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Task is paused. Use Resume instead.")
        if reason == "task_running":
            raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Task is currently running.")
        if reason == "deadline_passed":
            task.state = "EXPIRED"
            task.updated_at = now
            await db.commit()
            await db.refresh(task)
            raise HTTPException(status_code=status.HTTP_410_GONE, detail="Task deadline has passed.")
        if reason == "cooldown_active":
            raise HTTPException(status_code=status.HTTP_429_TOO_MANY_REQUESTS, detail="Retry cooldown active.")
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Task is not eligible for retry.")

    next_spec = _build_retry_on_expiry_spec(task)
    next_spec[MANUAL_RETRY_LAST_AT_KEY] = now.isoformat()

    _apply_retry_reset(task, retry_spec=next_spec, now=now)
    await db.commit()
    await db.refresh(task)

    enqueued = await enqueue_train_task(str(task.id))
    if enqueued is False:
        # Retry now must still run even if deterministic job-id dedupe collides
        # with an existing queued record for this task.
        await enqueue_train_task(str(task.id), defer_seconds=0.01)

    return TaskActionResponse(task=task_to_summary(task))


async def cancel_task(db: AsyncSession, *, task_id: UUID, user: User) -> TaskActionResponse:
    task = await get_task_for_user(db, task_id=task_id, user=user)
    latest_ticket_artifact = _latest_ticket_artifact_for_task(task)
    ticket_summary = _ticket_summary_from_artifact(latest_ticket_artifact)

    if task.state in TERMINAL_TASK_STATES and task.state != "COMPLETED":
        return TaskActionResponse(task=task_to_summary(task, ticket_summary=ticket_summary))

    if task.state == "COMPLETED" and not _is_manual_payment_pending(ticket_summary):
        return TaskActionResponse(task=task_to_summary(task, ticket_summary=ticket_summary))

    if latest_ticket_artifact is not None:
        await cancel_ticket(db, artifact_id=latest_ticket_artifact.id, user=user)
        task = await get_task_for_user(db, task_id=task_id, user=user)
        latest_ticket_artifact = _latest_ticket_artifact_for_task(task)
        ticket_summary = _ticket_summary_from_artifact(latest_ticket_artifact)
        return TaskActionResponse(task=task_to_summary(task, ticket_summary=ticket_summary))

    task.state = "CANCELLED"
    task.cancelled_at = task.cancelled_at or utc_now()
    task.completed_at = None
    task.failed_at = None
    task.paused_at = None
    task.updated_at = utc_now()
    await db.commit()
    await db.refresh(task)
    return TaskActionResponse(task=task_to_summary(task, ticket_summary=ticket_summary))


async def pay_task(db: AsyncSession, *, task_id: UUID, user: User) -> TaskActionResponse:
    if not await is_payment_runtime_enabled(db):
        raise HTTPException(status_code=status.HTTP_403_FORBIDDEN, detail="Payment features are currently disabled")

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
        reconciled_to_completed = False
        if task.state in ACTIVE_TASK_STATES or task.state in {"FAILED", "PAUSED"}:
            task.state = "COMPLETED"
            task.completed_at = task.completed_at or utc_now()
            task.failed_at = None
            task.updated_at = utc_now()
            reconciled_to_completed = True
        if updated or reconciled_to_completed:
            await db.commit()
        task = await get_task_for_user(db, task_id=task_id, user=user)
        last_attempt_at = (await _last_attempt_map(db, [task.id])).get(task.id)
        ticket_summary = _ticket_summary_from_artifact(_latest_ticket_artifact_for_task(task))
        return TaskActionResponse(
            task=task_to_summary(task, last_attempt_at=last_attempt_at, ticket_summary=ticket_summary)
        )

    if ticket_status == "expired":
        if updated:
            await db.commit()
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail="Reservation payment window has expired. Refresh reservation status and try again.",
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
            detail="Payment settings are not configured for this account.",
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
        meta_json_safe=validate_safe_metadata(
            {
                "manual_trigger": True,
                "rate_limit_wait_ms": limit.waited_ms,
                "rate_limit_rounds": limit.rounds,
                "reservation_id": reservation_id,
                "payment_card_configured": bool(payment_card),
                "payment_id": outcome.data.get("payment_id"),
            }
        ),
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

    artifact.data_json_safe = validate_safe_metadata(
        {
            **artifact_data,
            "paid": True,
            "status": "paid",
            "payment_id": outcome.data.get("payment_id"),
            "ticket_no": outcome.data.get("ticket_no"),
            "provider_http": provider_http,
        }
    )
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
        artifact.data_json_safe = validate_safe_metadata(
            {
                **artifact.data_json_safe,
                "provider_sync": sync_meta,
                "last_provider_sync_at": utc_now().isoformat(),
            }
        )
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
        artifact.data_json_safe = validate_safe_metadata(merged_data)

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
        select(Artifact, Task)
        .join(Task, Task.id == Artifact.task_id)
        .where(Artifact.id == artifact_id)
        .where(Task.user_id == user.id)
        .where(Task.module == TASK_MODULE)
    )
    row = (await db.execute(stmt)).one_or_none()
    if row is None:
        return TicketCancelResponse(status="not_found", detail="Artifact not found")
    artifact, task = row

    if artifact.data_json_safe.get("cancelled"):
        if task.state != "CANCELLED":
            task.state = "CANCELLED"
            task.cancelled_at = task.cancelled_at or utc_now()
            task.completed_at = None
            task.failed_at = None
            task.paused_at = None
            task.updated_at = utc_now()
            await db.commit()
        return TicketCancelResponse(status="already_cancelled", detail="Ticket is already cancelled")

    provider = str(artifact.data_json_safe.get("provider") or "")
    if provider not in {"SRT", "KTX"}:
        if task.state != "CANCELLED":
            task.state = "CANCELLED"
            task.cancelled_at = task.cancelled_at or utc_now()
            task.completed_at = None
            task.failed_at = None
            task.paused_at = None
            task.updated_at = utc_now()
            await db.commit()
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
        if task.state != "CANCELLED":
            task.state = "CANCELLED"
            task.cancelled_at = task.cancelled_at or utc_now()
            task.completed_at = None
            task.failed_at = None
            task.paused_at = None
            task.updated_at = utc_now()
        artifact.data_json_safe = validate_safe_metadata(
            {
                **artifact.data_json_safe,
                "provider_http": provider_http,
            }
        )
        await db.commit()
        return TicketCancelResponse(status="not_supported", detail=outcome.error_message_safe or "not supported")

    if not outcome.ok and outcome.error_code == "reservation_not_found":
        if task.state != "CANCELLED":
            task.state = "CANCELLED"
            task.cancelled_at = task.cancelled_at or utc_now()
            task.completed_at = None
            task.failed_at = None
            task.paused_at = None
            task.updated_at = utc_now()
        artifact.data_json_safe = validate_safe_metadata(
            {
                **artifact.data_json_safe,
                "status": "reservation_not_found",
                "provider_http": provider_http,
            }
        )
        await db.commit()
        return TicketCancelResponse(status="not_found", detail=outcome.error_message_safe or "Reservation not found")

    if not outcome.ok:
        artifact.data_json_safe = validate_safe_metadata(
            {
                **artifact.data_json_safe,
                "provider_http": provider_http,
            }
        )
        await db.commit()
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail=outcome.error_message_safe or "Cancel failed")

    artifact.data_json_safe = validate_safe_metadata(
        {
            **artifact.data_json_safe,
            "cancelled": True,
            "status": "cancelled",
            "provider_http": provider_http,
        }
    )
    task.state = "CANCELLED"
    task.cancelled_at = task.cancelled_at or utc_now()
    task.completed_at = None
    task.failed_at = None
    task.paused_at = None
    task.updated_at = utc_now()
    updated = True
    updated = await _refresh_ticket_artifact_status(
        db,
        user=user,
        artifact=artifact,
        limiter=limiter,
        force=True,
        client_cache=client_cache,
    ) or updated

    artifact.data_json_safe = validate_safe_metadata(
        {
            **artifact.data_json_safe,
            "cancelled": True,
            "status": "cancelled",
        }
    )
    await db.commit()
    return TicketCancelResponse(status="cancelled", detail="Ticket cancelled")


def _provider_reservation_ticket_status(reservation: dict) -> str:
    status_raw = str(reservation.get("status") or "").strip().lower()
    if status_raw == "reserved":
        return "awaiting_payment"
    if status_raw in {"awaiting_payment", "waiting", "paid", "expired", "cancelled", "reservation_not_found"}:
        return status_raw
    if bool(reservation.get("paid")):
        return "paid"
    if bool(reservation.get("expired")):
        return "expired"
    if bool(reservation.get("waiting")):
        return "waiting"
    return "awaiting_payment"


def _provider_reservation_task_state(ticket_status: str) -> str:
    if ticket_status == "expired":
        return "EXPIRED"
    if ticket_status in {"cancelled", "reservation_not_found"}:
        return "CANCELLED"
    return "COMPLETED"


def _provider_reservation_deadline_at(reservation: dict, *, now: datetime) -> datetime:
    for key in ("departure_at", "payment_deadline_at", "arrival_at"):
        parsed = _parse_iso_datetime(str(reservation.get(key) or ""))
        if parsed is not None:
            return _as_aware_utc_datetime(parsed)
    return now + timedelta(days=365)


def _provider_reservation_idempotency_key(*, user_id: UUID, provider: str, reservation_id: str) -> str:
    raw = f"provider_discovery:{user_id}:{provider}:{reservation_id}"
    return hashlib.sha256(raw.encode("utf-8")).hexdigest()


def _provider_reservation_task_spec(*, provider: str, reservation_id: str, reservation: dict) -> dict:
    spec: dict[str, Any] = {
        "provider": provider,
        "reservation_id": reservation_id,
        "discovered_from_provider": True,
    }

    dep = str(reservation.get("dep") or "").strip()
    arr = str(reservation.get("arr") or "").strip()
    departure_at = _parse_iso_datetime(str(reservation.get("departure_at") or ""))
    arrival_at = _parse_iso_datetime(str(reservation.get("arrival_at") or ""))
    train_no = str(reservation.get("train_no") or "").strip()

    if dep:
        spec["dep"] = dep
    if arr:
        spec["arr"] = arr
    if departure_at is not None:
        # Preserve the provider-declared departure date context without
        # forcing an additional timezone conversion.
        spec["date"] = departure_at.date().isoformat()

    if dep and arr and departure_at is not None:
        ranked: dict[str, Any] = {
            "rank": 1,
            "provider": provider,
            "dep": dep,
            "arr": arr,
            "departure_at": departure_at.isoformat(),
        }
        if arrival_at is not None:
            ranked["arrival_at"] = arrival_at.isoformat()
        if train_no:
            ranked["train_no"] = train_no
        spec["selected_trains_ranked"] = [ranked]

    return validate_safe_metadata(spec)


def _provider_reservation_ticket_payload(*, provider: str, reservation: dict, ticket_status: str) -> dict:
    reservation_id = str(reservation.get("reservation_id") or "").strip()
    departure_at = _parse_iso_datetime(str(reservation.get("departure_at") or ""))
    arrival_at = _parse_iso_datetime(str(reservation.get("arrival_at") or ""))
    payment_deadline_at = _parse_iso_datetime(str(reservation.get("payment_deadline_at") or ""))

    tickets_raw = reservation.get("tickets")
    tickets = [row for row in tickets_raw if isinstance(row, dict)] if isinstance(tickets_raw, list) else []

    seat_count = reservation.get("seat_count")
    if not isinstance(seat_count, int) or isinstance(seat_count, bool):
        seat_count = len(tickets) if tickets else None

    total_cost = reservation.get("total_cost")
    if not isinstance(total_cost, int) or isinstance(total_cost, bool):
        total_cost = None

    metadata = reservation.get("metadata")
    metadata_safe = metadata if isinstance(metadata, dict) else None

    payload: dict[str, Any] = {
        "provider": provider,
        "reservation_id": reservation_id,
        "status": ticket_status,
        "paid": bool(reservation.get("paid")),
        "waiting": bool(reservation.get("waiting")),
        "expired": bool(reservation.get("expired")),
        "train_no": reservation.get("train_no"),
        "train_code": reservation.get("train_code"),
        "train_type_code": reservation.get("train_type_code"),
        "train_type_name": reservation.get("train_type_name"),
        "dep": reservation.get("dep"),
        "arr": reservation.get("arr"),
        "departure_at": departure_at.isoformat() if departure_at is not None else None,
        "arrival_at": arrival_at.isoformat() if arrival_at is not None else None,
        "payment_deadline_at": payment_deadline_at.isoformat() if payment_deadline_at is not None else None,
        "seat_count": seat_count,
        "total_cost": total_cost,
        "journey_no": reservation.get("journey_no"),
        "journey_cnt": reservation.get("journey_cnt"),
        "rsv_chg_no": reservation.get("rsv_chg_no"),
        "wct_no": reservation.get("wct_no"),
        "tickets": tickets,
        "reservation_snapshot": redact_sensitive(reservation),
    }
    if metadata_safe is not None:
        payload["metadata"] = metadata_safe
    return validate_safe_metadata(payload)


async def _upsert_discovered_provider_reservations(
    db: AsyncSession,
    *,
    user: User,
    provider: str,
    reservations: list[dict],
) -> bool:
    if not reservations:
        return False

    now = utc_now()
    stmt = (
        select(Task, Artifact)
        .join(Artifact, Artifact.task_id == Task.id)
        .where(Task.user_id == user.id)
        .where(Task.module == TASK_MODULE)
        .where(Task.hidden_at.is_(None))
        .where(Artifact.module == TASK_MODULE)
        .where(Artifact.kind == "ticket")
    )
    existing_rows = (await db.execute(stmt)).all()
    existing_by_reservation: dict[str, tuple[Task, Artifact]] = {}
    min_aware = datetime.min.replace(tzinfo=timezone.utc)
    for task, artifact in existing_rows:
        data = artifact.data_json_safe or {}
        if str(data.get("provider") or "") != provider:
            continue
        reservation_id = str(data.get("reservation_id") or "").strip()
        if not reservation_id:
            continue
        previous = existing_by_reservation.get(reservation_id)
        if previous is None:
            existing_by_reservation[reservation_id] = (task, artifact)
            continue
        previous_created = previous[1].created_at or min_aware
        current_created = artifact.created_at or min_aware
        if current_created >= previous_created:
            existing_by_reservation[reservation_id] = (task, artifact)

    changed = False
    for reservation in reservations:
        if not isinstance(reservation, dict):
            continue
        reservation_id = str(reservation.get("reservation_id") or "").strip()
        if not reservation_id:
            continue

        ticket_status = _provider_reservation_ticket_status(reservation)
        task_state = _provider_reservation_task_state(ticket_status)
        ticket_payload = _provider_reservation_ticket_payload(
            provider=provider,
            reservation=reservation,
            ticket_status=ticket_status,
        )

        existing = existing_by_reservation.get(reservation_id)
        if existing is None:
            task = Task(
                user_id=user.id,
                module=TASK_MODULE,
                state=task_state,
                deadline_at=_provider_reservation_deadline_at(reservation, now=now),
                spec_json=_provider_reservation_task_spec(
                    provider=provider,
                    reservation_id=reservation_id,
                    reservation=reservation,
                ),
                idempotency_key=_provider_reservation_idempotency_key(
                    user_id=user.id,
                    provider=provider,
                    reservation_id=reservation_id,
                ),
            )
            if task_state == "COMPLETED":
                task.completed_at = now
            elif task_state == "EXPIRED":
                task.completed_at = None
            elif task_state == "CANCELLED":
                task.cancelled_at = now
            db.add(task)
            await db.flush()

            artifact = Artifact(
                task_id=task.id,
                module=TASK_MODULE,
                kind="ticket",
                data_json_safe=ticket_payload,
            )
            db.add(artifact)
            existing_by_reservation[reservation_id] = (task, artifact)
            changed = True
            continue

        task, artifact = existing
        merged_payload = validate_safe_metadata(
            {
                **dict(artifact.data_json_safe or {}),
                **ticket_payload,
            }
        )
        if merged_payload != artifact.data_json_safe:
            artifact.data_json_safe = merged_payload
            changed = True

        if task.state == "CANCELLED":
            continue

        task_changed = False
        if task_state == "EXPIRED":
            if task.state != "EXPIRED":
                task.state = "EXPIRED"
                task_changed = True
            if task.completed_at is not None:
                task.completed_at = None
                task_changed = True
            if task.failed_at is not None:
                task.failed_at = None
                task_changed = True
            if task.paused_at is not None:
                task.paused_at = None
                task_changed = True
        elif task_state == "COMPLETED":
            # Preserve active worker lifecycle for already-tracked unpaid reservations.
            # Newly discovered reservations are created as COMPLETED + pending ticket
            # so they appear in the active tab without entering worker polling.
            if ticket_status in {"awaiting_payment", "waiting"} and task.state in ACTIVE_TASK_STATES:
                task_changed = False
            else:
                if task.state != "COMPLETED":
                    task.state = "COMPLETED"
                    task_changed = True
                if task.completed_at is None:
                    task.completed_at = now
                    task_changed = True
                if task.failed_at is not None:
                    task.failed_at = None
                    task_changed = True
                if task.paused_at is not None:
                    task.paused_at = None
                    task_changed = True
        elif task_state == "CANCELLED":
            if task.state != "CANCELLED":
                task.state = "CANCELLED"
                task_changed = True
            if task.cancelled_at is None:
                task.cancelled_at = now
                task_changed = True
            if task.completed_at is not None:
                task.completed_at = None
                task_changed = True
            if task.failed_at is not None:
                task.failed_at = None
                task_changed = True
            if task.paused_at is not None:
                task.paused_at = None
                task_changed = True

        if task_changed:
            task.updated_at = now
            changed = True

    if changed:
        await db.commit()
    return changed


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

    reservation_rows = [row for row in outcome.data.get("reservations", []) if isinstance(row, dict)]
    if reservation_rows:
        try:
            await _upsert_discovered_provider_reservations(
                db,
                user=user,
                provider=provider,
                reservations=reservation_rows,
            )
        except Exception:
            logger.warning(
                "Failed to upsert discovered provider reservations",
                extra={"user_id": str(user.id), "provider": provider},
            )

    return ProviderReservationsResponse(reservations=reservation_rows)


async def refresh_train_reservations_after_sign_in(
    db: AsyncSession,
    *,
    user: User,
) -> None:
    for provider in ("SRT", "KTX"):
        if not _is_provider_enabled(provider):
            continue
        creds = await _load_provider_credentials(db, user_id=user.id, provider=provider)
        if creds is None:
            continue
        try:
            await list_provider_reservations(
                db,
                user=user,
                provider=provider,
                paid_only=False,
            )
        except HTTPException as exc:
            await db.rollback()
            logger.warning(
                "Sign-in reservation refresh failed for configured provider",
                extra={
                    "user_id": str(user.id),
                    "provider": provider,
                    "status_code": exc.status_code,
                },
            )
        except Exception:
            await db.rollback()
            logger.warning(
                "Sign-in reservation refresh failed unexpectedly",
                extra={"user_id": str(user.id), "provider": provider},
            )


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
