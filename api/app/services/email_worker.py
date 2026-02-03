from __future__ import annotations

import logging
from typing import Any

from app.schemas.notification import EmailJobPayload
from app.services.email import send_email

logger = logging.getLogger(__name__)


async def deliver_email_job(ctx: dict, payload: dict[str, Any]) -> dict[str, Any]:
    email_payload = EmailJobPayload.model_validate(payload)
    result = await send_email(email_payload)
    logger.info("Email job complete: recipient=%s provider=%s", result.recipient, result.provider)
    return result.model_dump(mode="json")
