from __future__ import annotations

from datetime import datetime, timezone

from fastapi import APIRouter, Depends, HTTPException, status

from app.http.deps import auth_rate_limit, get_current_user
from app.core.config import get_settings
from app.core.time import utc_now
from app.db.models import User
from app.schemas.notification import EmailJobPayload, EmailStatusResponse, EmailTestRequest, EmailTestResponse
from app.services.email_queue import enqueue_email

router = APIRouter(prefix="/api/notifications", tags=["notifications"])
settings = get_settings()


@router.get("/email/status", response_model=EmailStatusResponse)
async def get_email_status(_: User = Depends(get_current_user)) -> EmailStatusResponse:
    return EmailStatusResponse(
        enabled=settings.email_provider != "disabled",
        provider=settings.email_provider,
        from_name=settings.email_from_name,
        from_address=settings.email_from_address,
    )


@router.post(
    "/email/test",
    response_model=EmailTestResponse,
    dependencies=[Depends(auth_rate_limit)],
)
async def send_test_email(
    payload: EmailTestRequest,
    user: User = Depends(get_current_user),
) -> EmailTestResponse:
    if settings.email_provider == "disabled":
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail="Email delivery is disabled",
        )

    recipient = payload.to_email or user.email
    display_name = user.display_name or "there"
    subject = (payload.subject or "bominal test email").strip() or "bominal test email"

    body = (payload.message or "").strip()
    if not body:
        body = (
            f"Hi {display_name},\n\n"
            "This is a bominal test email.\n"
            "Your notification pipeline is ready for module events and service alerts.\n\n"
            f"Requested at: {utc_now().isoformat()}\n"
            f"User id: {user.id}\n"
        )

    email_payload = EmailJobPayload(
        to_email=recipient,
        subject=subject,
        text_body=body,
        tags=["test", "manual"],
        metadata={
            "source": "notifications.test",
            "user_id": str(user.id),
        },
    )

    job_id = await enqueue_email(email_payload)

    return EmailTestResponse(
        queued=True,
        job_id=job_id,
        recipient=recipient,
        provider=settings.email_provider,
        detail="Test email queued",
        queued_at=utc_now(),
    )
