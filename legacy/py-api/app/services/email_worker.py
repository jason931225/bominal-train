from __future__ import annotations

import logging
from typing import Any

from pydantic import ValidationError

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


def _looks_like_template_payload(payload: dict[str, Any]) -> bool:
    template_keys = {"preheader", "theme", "blocks", "context", "template_id"}
    if bool(template_keys.intersection(payload.keys())):
        return True

    metadata = payload.get("metadata")
    if isinstance(metadata, dict):
        kind = str(metadata.get("kind") or "").strip().lower()
        if kind in {"onboarding_verify", "password_reset"}:
            return True

    return False


def _normalize_template_payload(payload: dict[str, Any]) -> EmailTemplateJobPayload:
    normalized = dict(payload)
    normalized.setdefault("preheader", "")
    normalized.setdefault("theme", "spring")
    normalized.setdefault("context", {})

    if not normalized.get("blocks"):
        fallback_text = (
            str(normalized.get("message") or "")
            or str(normalized.get("text_body") or "")
            or str(normalized.get("subject") or "")
        ).strip()
        normalized["blocks"] = (
            [{"type": "paragraph", "data": {"text": fallback_text}}]
            if fallback_text
            else []
        )

    return EmailTemplateJobPayload.model_validate(normalized)


def _normalize_raw_payload(payload: dict[str, Any]) -> EmailJobPayload:
    normalized = dict(payload)
    if not normalized.get("text_body"):
        fallback_text = (
            str(normalized.get("message") or "")
            or str(normalized.get("preheader") or "")
            or str(normalized.get("subject") or "")
        ).strip()
        if fallback_text:
            normalized["text_body"] = fallback_text
    return EmailJobPayload.model_validate(normalized)


def _build_email_payload(payload: dict[str, Any]) -> EmailJobPayload:
    template_error: ValidationError | None = None
    if _looks_like_template_payload(payload):
        try:
            return _from_template_payload(_normalize_template_payload(payload))
        except ValidationError as exc:
            template_error = exc

    try:
        return _normalize_raw_payload(payload)
    except ValidationError:
        if template_error is not None:
            raise template_error
        raise


async def deliver_email_job(ctx: dict, payload: dict[str, Any]) -> dict[str, Any]:
    email_payload = _build_email_payload(payload)

    job_id = ctx.get("job_id")
    if job_id and not email_payload.message_id:
        email_payload = email_payload.model_copy(update={"message_id": str(job_id)})
    if job_id and not email_payload.idempotency_key:
        email_payload = email_payload.model_copy(update={"idempotency_key": str(job_id)})

    result = await send_email(email_payload)
    logger.info("Email job complete: recipient=%s provider=%s", result.recipient, result.provider)
    return result.model_dump(mode="json")
