from datetime import datetime, timedelta, timezone
from typing import Literal
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, status
from pydantic import BaseModel, EmailStr
from sqlalchemy import delete, func, select, update
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import joinedload

from app.db.models import Artifact, Role, Secret, Session, Task, TaskAttempt, User
from app.db.session import get_db
from app.http.deps import get_current_admin
from app.schemas.auth import MessageResponse

router = APIRouter(dependencies=[Depends(get_current_admin)])


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
    created_at: datetime
    updated_at: datetime
    email_verified_at: datetime | None
    session_count: int
    active_session_count: int
    task_count: int
    secret_count: int


class UpdateUserRole(BaseModel):
    role: Literal["admin", "user"]


class RevokeSessionsRequest(BaseModel):
    user_id: UUID


# ---------- Endpoints ----------


@router.get("", response_model=MessageResponse)
async def admin_only() -> MessageResponse:
    return MessageResponse(message="Admin access granted")


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


@router.get("/users", response_model=AdminUserList)
async def list_users(
    page: int = Query(1, ge=1),
    page_size: int = Query(20, ge=1, le=100),
    search: str | None = Query(None),
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
            User.created_at,
            func.max(Session.last_seen_at).label("last_seen_at"),
            func.count(func.distinct(Session.id)).label("session_count"),
            func.count(func.distinct(Task.id)).label("task_count"),
        )
        .join(Role, User.role_id == Role.id)
        .outerjoin(Session, Session.user_id == User.id)
        .outerjoin(Task, Task.user_id == User.id)
        .group_by(User.id, Role.name)
    )

    if search:
        search_pattern = f"%{search}%"
        query = query.where(
            (User.email.ilike(search_pattern)) | (User.display_name.ilike(search_pattern))
        )

    # Get total count
    count_query = select(func.count(func.distinct(User.id)))
    if search:
        search_pattern = f"%{search}%"
        count_query = count_query.where(
            (User.email.ilike(search_pattern)) | (User.display_name.ilike(search_pattern))
        )
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
