from __future__ import annotations

import logging
from typing import Any

from app.schemas.notification import EmailJobPayload, EmailTemplateJobPayload
from app.services.email_template import Block, render_email
from app.services.email import send_email

logger = logging.getLogger(__name__)


def _from_template_payload(payload: EmailTemplateJobPayload) -> EmailJobPayload:
    rendered = render_email(
        subject=payload.subject,
        preheader=payload.preheader,
        theme=payload.theme,
        context=payload.context,
        blocks=[Block(type=block.type, data=block.data) for block in payload.blocks],
    )
    return EmailJobPayload(
        to_email=payload.to_email,
        subject=rendered.subject,
        text_body=rendered.text,
        html_body=rendered.html,
        tags=payload.tags,
        headers=payload.headers,
        metadata=payload.metadata,
        message_id=payload.message_id,
        idempotency_key=payload.idempotency_key,
    )


async def deliver_email_job(ctx: dict, payload: dict[str, Any]) -> dict[str, Any]:
    if {"preheader", "theme", "blocks"}.issubset(payload.keys()):
        template_payload = EmailTemplateJobPayload.model_validate(payload)
        email_payload = _from_template_payload(template_payload)
    else:
        email_payload = EmailJobPayload.model_validate(payload)

    job_id = ctx.get("job_id")
    if job_id and not email_payload.message_id:
        email_payload = email_payload.model_copy(update={"message_id": str(job_id)})
    if job_id and not email_payload.idempotency_key:
        email_payload = email_payload.model_copy(update={"idempotency_key": str(job_id)})

    result = await send_email(email_payload)
    logger.info("Email job complete: recipient=%s provider=%s", result.recipient, result.provider)
    return result.model_dump(mode="json")
