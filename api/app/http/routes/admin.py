from datetime import datetime, timedelta, timezone
from typing import Literal
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, status
from pydantic import BaseModel, EmailStr, Field, field_validator
from sqlalchemy import delete, func, select, update
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import joinedload

from app.core.redis import get_redis_client
from app.db.models import Artifact, Role, Secret, Session, Task, TaskAttempt, User
from app.db.session import get_db
from app.http.deps import get_current_admin
from app.modules.train.constants import (
    ACTIVE_TASK_STATES,
    SPEC_KEY_NEXT_RUN_AT,
    TASK_MODULE as TRAIN_TASK_MODULE,
    TERMINAL_TASK_STATES,
)
from app.modules.train.queue import enqueue_train_task
from app.modules.train.worker import enqueue_recoverable_tasks
from app.schemas.auth import MessageResponse
from app.services.system_payment import (
    clear_system_payment_card,
    get_system_payment_settings_status,
    set_system_payment_card,
    set_system_payment_enabled,
)
from app.worker import HEARTBEAT_KEY

router = APIRouter(dependencies=[Depends(get_current_admin)])

STALE_TASK_WINDOW = timedelta(minutes=10)


# ---------- Schemas ----------


class SystemStats(BaseModel):
    total_users: int
    active_users_24h: int
    total_sessions: int
    active_sessions: int
    total_tasks: int
    tasks_by_state: dict[str, int]
    tasks_completed_24h: int


class AdminUserSummary(BaseModel):
    id: UUID
    email: str
    display_name: str | None
    role: str
    access_status: Literal["pending", "approved", "rejected"]
    access_reviewed_at: datetime | None
    created_at: datetime
    last_seen_at: datetime | None
    session_count: int
    task_count: int


class AdminUserList(BaseModel):
    users: list[AdminUserSummary]
    total: int
    page: int
    page_size: int


class AdminUserDetail(BaseModel):
    id: UUID
    email: str
    display_name: str | None
    phone_number: str | None
    role: str
    access_status: Literal["pending", "approved", "rejected"]
    access_reviewed_at: datetime | None
    created_at: datetime
    updated_at: datetime
    email_verified_at: datetime | None
    session_count: int
    active_session_count: int
    task_count: int
    secret_count: int


class UpdateUserRole(BaseModel):
    role: Literal["admin", "user"]


class UpdateUserAccess(BaseModel):
    access_status: Literal["pending", "approved", "rejected"]


class RevokeSessionsRequest(BaseModel):
    user_id: UUID


class OpsRedisStatus(BaseModel):
    ok: bool
    detail: str | None = None


class OpsArqQueueStatus(BaseModel):
    queued: int
    in_progress: int


class OpsWorkerHeartbeatStatus(BaseModel):
    online: bool
    last_heartbeat_at: datetime | None = None


class OpsTrainStatus(BaseModel):
    active_task_count: int
    stale_task_count: int


class OpsStatusResponse(BaseModel):
    redis: OpsRedisStatus
    arq: OpsArqQueueStatus
    worker: OpsWorkerHeartbeatStatus
    train: OpsTrainStatus


class OpsStaleTaskOut(BaseModel):
    task_id: UUID
    state: str
    created_at: datetime
    updated_at: datetime
    deadline_at: datetime
    user_id: UUID
    user_email: EmailStr
    last_attempt_at: datetime | None
    last_error_code: str | None
    last_error_message_safe: str | None


class OpsStaleTasksResponse(BaseModel):
    tasks: list[OpsStaleTaskOut]


class OpsRecentFailureOut(BaseModel):
    task_id: UUID
    user_id: UUID
    user_email: EmailStr
    action: str
    provider: str
    error_code: str | None
    error_message_safe: str | None
    started_at: datetime
    finished_at: datetime


class OpsRecentFailuresResponse(BaseModel):
    failures: list[OpsRecentFailureOut]


class OpsRecoverResponse(BaseModel):
    enqueued_count: int


class AdminPaymentSettingsResponse(BaseModel):
    payment_enabled: bool
    payment_enabled_env: bool
    payment_enabled_override: bool
    configured: bool
    source: Literal["server_override", "pay_env", "none"]
    card_masked: str | None = None
    updated_at: datetime | None = None
    updated_by_user_id: UUID | None = None


class AdminPaymentEnabledRequest(BaseModel):
    enabled: bool


class AdminPaymentCardUpdateRequest(BaseModel):
    card_number: str = Field(min_length=12, max_length=24)
    expiry_mm: str = Field(pattern=r"^\d{2}$")
    expiry_yy: str = Field(pattern=r"^\d{2}$")
    dob: str = Field(pattern=r"^\d{8}$")
    pin2: str = Field(pattern=r"^\d{2}$")

    @field_validator("card_number")
    @classmethod
    def normalize_card_number(cls, value: str) -> str:
        digits_only = "".join(ch for ch in value if ch.isdigit())
        if len(digits_only) < 13 or len(digits_only) > 19:
            raise ValueError("card_number must contain 13 to 19 digits")
        return digits_only

    @field_validator("expiry_mm")
    @classmethod
    def validate_expiry_month(cls, value: str) -> str:
        month = int(value)
        if month < 1 or month > 12:
            raise ValueError("expiry_mm must be between 01 and 12")
        return value

    @field_validator("dob")
    @classmethod
    def validate_dob(cls, value: str) -> str:
        datetime.strptime(value, "%Y%m%d")
        return value


def _payment_settings_response(payload: dict) -> AdminPaymentSettingsResponse:
    source_value = str(payload.get("source") or "none")
    if source_value not in {"server_override", "pay_env", "none"}:
        source_value = "none"

    return AdminPaymentSettingsResponse(
        payment_enabled=bool(payload.get("payment_enabled")),
        payment_enabled_env=bool(payload.get("payment_enabled_env")),
        payment_enabled_override=bool(payload.get("payment_enabled_override")),
        configured=bool(payload.get("configured")),
        source=source_value,
        card_masked=payload.get("card_masked"),
        updated_at=payload.get("updated_at"),
        updated_by_user_id=payload.get("updated_by_user_id"),
    )


# ---------- Endpoints ----------


@router.get("", response_model=MessageResponse)
async def admin_only() -> MessageResponse:
    return MessageResponse(message="Admin access granted")


@router.get("/payment-settings", response_model=AdminPaymentSettingsResponse)
async def get_payment_settings(db: AsyncSession = Depends(get_db)) -> AdminPaymentSettingsResponse:
    payload = await get_system_payment_settings_status(db)
    return _payment_settings_response(payload)


@router.patch("/payment-settings/enabled", response_model=AdminPaymentSettingsResponse)
async def set_payment_settings_enabled(
    body: AdminPaymentEnabledRequest,
    db: AsyncSession = Depends(get_db),
    admin_user: User = Depends(get_current_admin),
) -> AdminPaymentSettingsResponse:
    payload = await set_system_payment_enabled(
        db,
        enabled=body.enabled,
        updated_by_user_id=admin_user.id,
    )
    return _payment_settings_response(payload)


@router.put("/payment-settings/card", response_model=AdminPaymentSettingsResponse)
async def set_payment_settings_card(
    body: AdminPaymentCardUpdateRequest,
    db: AsyncSession = Depends(get_db),
    admin_user: User = Depends(get_current_admin),
) -> AdminPaymentSettingsResponse:
    payload = await set_system_payment_card(
        db,
        card_number=body.card_number,
        expiry_mm=body.expiry_mm,
        expiry_yy=body.expiry_yy,
        dob=body.dob,
        pin2=body.pin2,
        updated_by_user_id=admin_user.id,
    )
    if payload is None:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Invalid payment settings payload")
    return _payment_settings_response(payload)


@router.delete("/payment-settings/card", response_model=AdminPaymentSettingsResponse)
async def delete_payment_settings_card(
    db: AsyncSession = Depends(get_db),
    admin_user: User = Depends(get_current_admin),
) -> AdminPaymentSettingsResponse:
    payload = await clear_system_payment_card(
        db,
        updated_by_user_id=admin_user.id,
    )
    return _payment_settings_response(payload)


@router.get("/stats", response_model=SystemStats)
async def get_system_stats(db: AsyncSession = Depends(get_db)) -> SystemStats:
    """Get system-wide statistics for the admin dashboard."""
    now = datetime.now(timezone.utc)
    day_ago = now - timedelta(days=1)

    # Total users
    total_users = (await db.execute(select(func.count(User.id)))).scalar_one()

    # Active users in last 24h (users with sessions seen in last 24h)
    active_users_24h = (
        await db.execute(
            select(func.count(func.distinct(Session.user_id))).where(Session.last_seen_at >= day_ago)
        )
    ).scalar_one()

    # Total sessions
    total_sessions = (await db.execute(select(func.count(Session.id)))).scalar_one()

    # Active sessions (not expired, not revoked)
    active_sessions = (
        await db.execute(
            select(func.count(Session.id)).where(
                Session.expires_at > now,
                Session.revoked_at.is_(None),
            )
        )
    ).scalar_one()

    # Total tasks
    total_tasks = (await db.execute(select(func.count(Task.id)))).scalar_one()

    # Tasks by state
    state_counts = (await db.execute(select(Task.state, func.count(Task.id)).group_by(Task.state))).all()
    tasks_by_state = {state: count for state, count in state_counts}

    # Tasks completed in last 24h
    tasks_completed_24h = (
        await db.execute(select(func.count(Task.id)).where(Task.completed_at >= day_ago))
    ).scalar_one()

    return SystemStats(
        total_users=total_users,
        active_users_24h=active_users_24h,
        total_sessions=total_sessions,
        active_sessions=active_sessions,
        total_tasks=total_tasks,
        tasks_by_state=tasks_by_state,
        tasks_completed_24h=tasks_completed_24h,
    )


@router.get("/ops/status", response_model=OpsStatusResponse)
async def get_ops_status(db: AsyncSession = Depends(get_db)) -> OpsStatusResponse:
    now = datetime.now(timezone.utc)
    stale_cutoff = now - STALE_TASK_WINDOW

    redis_ok = True
    redis_detail: str | None = None
    arq_queue = 0
    arq_in_progress = 0
    heartbeat_online = False
    heartbeat_at: datetime | None = None

    try:
        redis = await get_redis_client()
        await redis.ping()
        arq_queue = int(await redis.zcard(b"arq:queue"))
        arq_in_progress = int(await redis.zcard(b"arq:in-progress"))
        heartbeat_raw = await redis.get(HEARTBEAT_KEY)
        if isinstance(heartbeat_raw, (bytes, bytearray)) and heartbeat_raw:
            heartbeat_online = True
            try:
                heartbeat_at = datetime.fromisoformat(heartbeat_raw.decode("utf-8"))
                if heartbeat_at.tzinfo is None:
                    heartbeat_at = heartbeat_at.replace(tzinfo=timezone.utc)
            except ValueError:
                heartbeat_at = None
    except Exception as exc:
        redis_ok = False
        redis_detail = f"{type(exc).__name__}"

    active_count = (
        await db.execute(
            select(func.count(Task.id))
            .where(Task.module == TRAIN_TASK_MODULE)
            .where(Task.state.in_(ACTIVE_TASK_STATES))
            .where(Task.hidden_at.is_(None))
            .where(Task.cancelled_at.is_(None))
        )
    ).scalar_one()

    stale_count = (
        await db.execute(
            select(func.count(Task.id))
            .where(Task.module == TRAIN_TASK_MODULE)
            .where(Task.hidden_at.is_(None))
            .where(Task.cancelled_at.is_(None))
            .where(Task.state.in_(("RUNNING", "POLLING", "RESERVING", "PAYING")))
            .where(Task.updated_at < stale_cutoff)
        )
    ).scalar_one()

    return OpsStatusResponse(
        redis=OpsRedisStatus(ok=redis_ok, detail=redis_detail),
        arq=OpsArqQueueStatus(queued=arq_queue, in_progress=arq_in_progress),
        worker=OpsWorkerHeartbeatStatus(online=heartbeat_online, last_heartbeat_at=heartbeat_at),
        train=OpsTrainStatus(active_task_count=int(active_count), stale_task_count=int(stale_count)),
    )


@router.get("/ops/train/stale-tasks", response_model=OpsStaleTasksResponse)
async def list_stale_train_tasks(
    limit: int = Query(default=20, ge=1, le=200),
    db: AsyncSession = Depends(get_db),
) -> OpsStaleTasksResponse:
    now = datetime.now(timezone.utc)
    stale_cutoff = now - STALE_TASK_WINDOW

    stmt = (
        select(Task, User.email)
        .join(User, User.id == Task.user_id)
        .where(Task.module == TRAIN_TASK_MODULE)
        .where(Task.hidden_at.is_(None))
        .where(Task.cancelled_at.is_(None))
        .where(Task.state.in_(("RUNNING", "POLLING", "RESERVING", "PAYING")))
        .where(Task.updated_at < stale_cutoff)
        .order_by(Task.updated_at.asc())
        .limit(limit)
    )
    rows = (await db.execute(stmt)).all()
    tasks = [row[0] for row in rows]
    email_by_task = {row[0].id: row[1] for row in rows}

    task_ids = [task.id for task in tasks]
    attempts_stmt = (
        select(TaskAttempt)
        .where(TaskAttempt.task_id.in_(task_ids))
        .order_by(TaskAttempt.task_id.asc(), TaskAttempt.finished_at.desc(), TaskAttempt.started_at.desc())
    )
    attempts = (await db.execute(attempts_stmt)).scalars().all()
    latest_attempt: dict[UUID, TaskAttempt] = {}
    for attempt in attempts:
        latest_attempt.setdefault(attempt.task_id, attempt)

    out: list[OpsStaleTaskOut] = []
    for task in tasks:
        attempt = latest_attempt.get(task.id)
        out.append(
            OpsStaleTaskOut(
                task_id=task.id,
                state=task.state,
                created_at=task.created_at,
                updated_at=task.updated_at,
                deadline_at=task.deadline_at,
                user_id=task.user_id,
                user_email=email_by_task[task.id],
                last_attempt_at=attempt.finished_at if attempt else None,
                last_error_code=attempt.error_code if attempt else None,
                last_error_message_safe=attempt.error_message_safe if attempt else None,
            )
        )

    return OpsStaleTasksResponse(tasks=out)


@router.get("/ops/train/recent-failures", response_model=OpsRecentFailuresResponse)
async def list_recent_train_failures(
    hours: int = Query(default=24, ge=1, le=168),
    limit: int = Query(default=50, ge=1, le=500),
    db: AsyncSession = Depends(get_db),
) -> OpsRecentFailuresResponse:
    now = datetime.now(timezone.utc)
    cutoff = now - timedelta(hours=hours)

    stmt = (
        select(TaskAttempt, Task.user_id, User.email)
        .join(Task, Task.id == TaskAttempt.task_id)
        .join(User, User.id == Task.user_id)
        .where(Task.module == TRAIN_TASK_MODULE)
        .where(TaskAttempt.ok.is_(False))
        .where(TaskAttempt.started_at >= cutoff)
        .order_by(TaskAttempt.started_at.desc())
        .limit(limit)
    )
    rows = (await db.execute(stmt)).all()

    failures = [
        OpsRecentFailureOut(
            task_id=attempt.task_id,
            user_id=user_id,
            user_email=email,
            action=attempt.action,
            provider=attempt.provider,
            error_code=attempt.error_code,
            error_message_safe=attempt.error_message_safe,
            started_at=attempt.started_at,
            finished_at=attempt.finished_at,
        )
        for (attempt, user_id, email) in rows
    ]
    return OpsRecentFailuresResponse(failures=failures)


@router.post("/ops/train/recover", response_model=OpsRecoverResponse)
async def recover_train_tasks(db: AsyncSession = Depends(get_db)) -> OpsRecoverResponse:
    enqueued = await enqueue_recoverable_tasks(db)
    return OpsRecoverResponse(enqueued_count=int(enqueued))


@router.post("/ops/train/tasks/{task_id}/requeue", response_model=MessageResponse)
async def requeue_train_task(task_id: UUID, db: AsyncSession = Depends(get_db)) -> MessageResponse:
    task = (await db.execute(select(Task).where(Task.id == task_id))).scalar_one_or_none()
    if task is None or task.module != TRAIN_TASK_MODULE:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Task not found")
    if task.hidden_at is not None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Task not found")
    if task.cancelled_at is not None or task.state == "CANCELLED":
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Task is cancelled")
    if task.paused_at is not None or task.state == "PAUSED":
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Task is paused")
    if task.state in TERMINAL_TASK_STATES:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Task is terminal")

    next_spec = dict(task.spec_json or {})
    next_spec.pop(SPEC_KEY_NEXT_RUN_AT, None)
    task.spec_json = next_spec
    task.state = "QUEUED"
    task.updated_at = datetime.now(timezone.utc)
    await db.commit()
    await db.refresh(task)

    await enqueue_train_task(str(task.id))

    return MessageResponse(message="Task requeued")


@router.get("/users", response_model=AdminUserList)
async def list_users(
    page: int = Query(1, ge=1),
    page_size: int = Query(20, ge=1, le=100),
    search: str | None = Query(None),
    access_status: Literal["pending", "approved", "rejected"] | None = Query(default=None),
    db: AsyncSession = Depends(get_db),
) -> AdminUserList:
    """List all users with pagination and optional search."""
    now = datetime.now(timezone.utc)

    # Base query with aggregations
    query = (
        select(
            User.id,
            User.email,
            User.display_name,
            Role.name.label("role"),
            User.access_status,
            User.access_reviewed_at,
            User.created_at,
            func.max(Session.last_seen_at).label("last_seen_at"),
            func.count(func.distinct(Session.id)).label("session_count"),
            func.count(func.distinct(Task.id)).label("task_count"),
        )
        .join(Role, User.role_id == Role.id)
        .outerjoin(Session, Session.user_id == User.id)
        .outerjoin(Task, Task.user_id == User.id)
        .group_by(User.id, Role.name, User.access_status, User.access_reviewed_at)
    )

    if search:
        search_pattern = f"%{search}%"
        query = query.where(
            (User.email.ilike(search_pattern)) | (User.display_name.ilike(search_pattern))
        )
    if access_status is not None:
        query = query.where(User.access_status == access_status)

    # Get total count
    count_query = select(func.count(func.distinct(User.id))).select_from(User)
    if search:
        search_pattern = f"%{search}%"
        count_query = count_query.where(
            (User.email.ilike(search_pattern)) | (User.display_name.ilike(search_pattern))
        )
    if access_status is not None:
        count_query = count_query.where(User.access_status == access_status)
    total = (await db.execute(count_query)).scalar_one()

    # Paginate
    query = query.order_by(User.created_at.desc()).offset((page - 1) * page_size).limit(page_size)

    rows = (await db.execute(query)).all()

    users = [
        AdminUserSummary(
            id=row.id,
            email=row.email,
            display_name=row.display_name,
            role=row.role,
            access_status=row.access_status,
            access_reviewed_at=row.access_reviewed_at,
            created_at=row.created_at,
            last_seen_at=row.last_seen_at,
            session_count=row.session_count,
            task_count=row.task_count,
        )
        for row in rows
    ]

    return AdminUserList(users=users, total=total, page=page, page_size=page_size)


@router.get("/users/{user_id}", response_model=AdminUserDetail)
async def get_user_detail(user_id: UUID, db: AsyncSession = Depends(get_db)) -> AdminUserDetail:
    """Get detailed information about a specific user."""
    now = datetime.now(timezone.utc)

    user = (
        await db.execute(select(User).options(joinedload(User.role)).where(User.id == user_id))
    ).scalar_one_or_none()

    if not user:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="User not found")

    session_count = (
        await db.execute(select(func.count(Session.id)).where(Session.user_id == user_id))
    ).scalar_one()

    active_session_count = (
        await db.execute(
            select(func.count(Session.id)).where(
                Session.user_id == user_id,
                Session.expires_at > now,
                Session.revoked_at.is_(None),
            )
        )
    ).scalar_one()

    task_count = (
        await db.execute(select(func.count(Task.id)).where(Task.user_id == user_id))
    ).scalar_one()

    secret_count = (
        await db.execute(select(func.count(Secret.id)).where(Secret.user_id == user_id))
    ).scalar_one()

    return AdminUserDetail(
        id=user.id,
        email=user.email,
        display_name=user.display_name,
        phone_number=user.phone_number,
        role=user.role.name,
        access_status=user.access_status,
        access_reviewed_at=user.access_reviewed_at,
        created_at=user.created_at,
        updated_at=user.updated_at,
        email_verified_at=user.email_verified_at,
        session_count=session_count,
        active_session_count=active_session_count,
        task_count=task_count,
        secret_count=secret_count,
    )


@router.patch("/users/{user_id}/role", response_model=MessageResponse)
async def update_user_role(
    user_id: UUID,
    body: UpdateUserRole,
    db: AsyncSession = Depends(get_db),
) -> MessageResponse:
    """Update a user's role (admin/user)."""
    user = (await db.execute(select(User).where(User.id == user_id))).scalar_one_or_none()
    if not user:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="User not found")

    role = (await db.execute(select(Role).where(Role.name == body.role))).scalar_one_or_none()
    if not role:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Invalid role")

    user.role_id = role.id
    await db.commit()

    return MessageResponse(message=f"User role updated to {body.role}")


@router.patch("/users/{user_id}/access", response_model=MessageResponse)
async def update_user_access(
    user_id: UUID,
    body: UpdateUserAccess,
    db: AsyncSession = Depends(get_db),
    admin_user: User = Depends(get_current_admin),
) -> MessageResponse:
    """Update a user's access-review status (pending/approved/rejected)."""
    user = (
        await db.execute(select(User).options(joinedload(User.role)).where(User.id == user_id))
    ).scalar_one_or_none()
    if not user:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="User not found")

    next_status = body.access_status
    if user.id == admin_user.id and next_status != "approved":
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Cannot remove your own approved access status",
        )

    now = datetime.now(timezone.utc)
    user.access_status = next_status
    user.access_reviewed_at = None if next_status == "pending" else now

    revoked_count = 0
    if next_status != "approved":
        revoke_result = await db.execute(
            update(Session)
            .where(
                Session.user_id == user_id,
                Session.revoked_at.is_(None),
                Session.expires_at > now,
            )
            .values(revoked_at=now)
        )
        revoked_count = int(revoke_result.rowcount or 0)

    await db.commit()

    message = f"User access updated to {next_status}"
    if revoked_count:
        message = f"{message}; revoked {revoked_count} active session(s)"
    return MessageResponse(message=message)


@router.post("/users/{user_id}/revoke-sessions", response_model=MessageResponse)
async def revoke_user_sessions(user_id: UUID, db: AsyncSession = Depends(get_db)) -> MessageResponse:
    """Revoke all active sessions for a user (force logout)."""
    now = datetime.now(timezone.utc)

    user = (await db.execute(select(User).where(User.id == user_id))).scalar_one_or_none()
    if not user:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="User not found")

    result = await db.execute(
        update(Session)
        .where(
            Session.user_id == user_id,
            Session.revoked_at.is_(None),
            Session.expires_at > now,
        )
        .values(revoked_at=now)
    )
    await db.commit()

    return MessageResponse(message=f"Revoked {result.rowcount} active session(s)")


@router.delete("/users/{user_id}", response_model=MessageResponse)
async def delete_user(user_id: UUID, db: AsyncSession = Depends(get_db)) -> MessageResponse:
    """Permanently delete a user and all associated data. Use with caution."""
    user = (await db.execute(select(User).where(User.id == user_id))).scalar_one_or_none()
    if not user:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="User not found")

    # Delete in order due to foreign keys
    # Get task IDs first for cascading deletes
    task_ids = (await db.execute(select(Task.id).where(Task.user_id == user_id))).scalars().all()

    if task_ids:
        await db.execute(delete(Artifact).where(Artifact.task_id.in_(task_ids)))
        await db.execute(delete(TaskAttempt).where(TaskAttempt.task_id.in_(task_ids)))
        await db.execute(delete(Task).where(Task.user_id == user_id))

    await db.execute(delete(Secret).where(Secret.user_id == user_id))
    await db.execute(delete(Session).where(Session.user_id == user_id))
    await db.execute(delete(User).where(User.id == user_id))
    await db.commit()

    return MessageResponse(message=f"User {user.email} deleted")
