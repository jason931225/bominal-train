from __future__ import annotations

import asyncio
import logging
import smtplib
import ssl
import uuid
from email.message import EmailMessage
from email.utils import formataddr

import httpx

from app.core.config import get_settings
from app.schemas.notification import EmailJobPayload, EmailSendResult

settings = get_settings()
logger = logging.getLogger(__name__)


class EmailDeliveryError(RuntimeError):
    """Raised when email delivery cannot be completed."""


def _from_header() -> str:
    return formataddr((settings.email_from_name, settings.email_from_address))


def _build_message(payload: EmailJobPayload) -> EmailMessage:
    message = EmailMessage()
    message["Subject"] = payload.subject
    message["From"] = _from_header()
    message["To"] = str(payload.to_email)
    if settings.email_reply_to:
        message["Reply-To"] = settings.email_reply_to

    message.set_content(payload.text_body)
    if payload.html_body:
        message.add_alternative(payload.html_body, subtype="html")
    return message


def _resend_payload(payload: EmailJobPayload) -> dict:
    body: dict = {
        "from": _from_header(),
        "to": [str(payload.to_email)],
        "subject": payload.subject,
        "text": payload.text_body,
        "headers": {
            "X-Bominal-Message-Id": str(uuid.uuid4()),
        },
    }
    if payload.html_body:
        body["html"] = payload.html_body
    if payload.tags:
        # Resend supports tags with {name,value}; keep value stable for filters.
        body["tags"] = [{"name": tag, "value": "true"} for tag in payload.tags]
    return body


async def _send_resend(payload: EmailJobPayload) -> EmailSendResult:
    if not settings.resend_api_key:
        raise EmailDeliveryError("Resend API key is not configured")

    endpoint = f"{settings.resend_api_base_url.rstrip('/')}/emails"
    timeout_seconds = max(1.0, float(settings.smtp_timeout_seconds))
    headers = {
        "Authorization": f"Bearer {settings.resend_api_key}",
        "Content-Type": "application/json",
    }

    try:
        async with httpx.AsyncClient(timeout=timeout_seconds) as client:
            response = await client.post(endpoint, headers=headers, json=_resend_payload(payload))
    except httpx.HTTPError as exc:  # pragma: no cover - network failures are environment-dependent
        raise EmailDeliveryError(f"Resend delivery failed: {type(exc).__name__}") from exc

    if response.status_code >= 400:
        # Keep error safe: avoid leaking provider internals or payload.
        raise EmailDeliveryError(f"Resend rejected email ({response.status_code})")

    response_payload = response.json() if response.content else {}
    provider_message_id = str(response_payload.get("id") or "")

    logger.info(
        "Email sent via Resend: to=%s subject=%s tags=%s",
        payload.to_email,
        payload.subject,
        ",".join(payload.tags),
    )
    return EmailSendResult(
        status="sent",
        recipient=payload.to_email,
        provider="resend",
        metadata={
            "mode": "resend",
            "provider_message_id": provider_message_id or None,
        },
    )


def _send_smtp_sync(message: EmailMessage) -> None:
    timeout_seconds = max(1.0, float(settings.smtp_timeout_seconds))

    if settings.smtp_use_ssl:
        smtp: smtplib.SMTP = smtplib.SMTP_SSL(
            host=settings.smtp_host,
            port=settings.smtp_port,
            timeout=timeout_seconds,
            context=ssl.create_default_context(),
        )
    else:
        smtp = smtplib.SMTP(
            host=settings.smtp_host,
            port=settings.smtp_port,
            timeout=timeout_seconds,
        )

    try:
        if settings.smtp_starttls and not settings.smtp_use_ssl:
            smtp.ehlo()
            smtp.starttls(context=ssl.create_default_context())
            smtp.ehlo()

        if settings.smtp_username:
            smtp.login(settings.smtp_username, settings.smtp_password or "")

        smtp.send_message(message)
    finally:
        try:
            smtp.quit()
        except Exception:
            smtp.close()


async def send_email(payload: EmailJobPayload) -> EmailSendResult:
    provider = settings.email_provider

    if provider == "disabled":
        raise EmailDeliveryError("Email delivery is disabled")

    if provider == "log":
        logger.info(
            "Email(log provider): to=%s subject=%s tags=%s",
            payload.to_email,
            payload.subject,
            ",".join(payload.tags),
        )
        return EmailSendResult(
            status="sent",
            recipient=payload.to_email,
            provider=provider,
            metadata={"mode": "log"},
        )

    if provider == "resend":
        return await _send_resend(payload)

    if provider != "smtp":
        raise EmailDeliveryError(f"Unsupported email provider: {provider}")

    message = _build_message(payload)

    try:
        await asyncio.to_thread(_send_smtp_sync, message)
    except Exception as exc:  # pragma: no cover - network failures are environment-dependent
        raise EmailDeliveryError(f"SMTP delivery failed: {type(exc).__name__}") from exc

    logger.info(
        "Email sent: to=%s subject=%s tags=%s",
        payload.to_email,
        payload.subject,
        ",".join(payload.tags),
    )
    return EmailSendResult(
        status="sent",
        recipient=payload.to_email,
        provider=provider,
        metadata={"mode": "smtp"},
    )
