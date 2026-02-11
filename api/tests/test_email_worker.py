from __future__ import annotations

import pytest

from app.schemas.notification import (
    EmailJobPayload,
    EmailSendResult,
    EmailTemplateBlock,
    EmailTemplateJobPayload,
)
from app.services import email_worker


@pytest.mark.asyncio
async def test_deliver_email_job_supports_raw_payload(monkeypatch):
    captured: dict[str, str] = {}

    async def _fake_send(payload: EmailJobPayload):
        captured["subject"] = payload.subject
        captured["text_body"] = payload.text_body
        return EmailSendResult(status="sent", recipient=payload.to_email, provider="log")

    monkeypatch.setattr(email_worker, "send_email", _fake_send)

    result = await email_worker.deliver_email_job(
        {},
        EmailJobPayload(
            to_email="raw@example.com",
            subject="Raw",
            text_body="Raw body",
        ).model_dump(mode="json"),
    )

    assert result["status"] == "sent"
    assert captured["subject"] == "Raw"
    assert captured["text_body"] == "Raw body"


@pytest.mark.asyncio
async def test_deliver_email_job_renders_template_payload(monkeypatch):
    captured: dict[str, str] = {}

    async def _fake_send(payload: EmailJobPayload):
        captured["subject"] = payload.subject
        captured["html"] = payload.html_body or ""
        captured["text"] = payload.text_body
        return EmailSendResult(status="sent", recipient=payload.to_email, provider="log")

    monkeypatch.setattr(email_worker, "send_email", _fake_send)

    payload = EmailTemplateJobPayload(
        to_email="template@example.com",
        subject="Verify",
        preheader="Verify your email",
        theme="spring",
        blocks=[
            EmailTemplateBlock(type="hero", data={"title": "Welcome {{ user.name }}"}),
            EmailTemplateBlock(type="otp", data={"code": {"$ref": "verify.code"}, "ttl_minutes": 10}),
        ],
        context={"user": {"name": "Template User"}, "verify": {"code": "123456"}},
        tags=["verify"],
    )

    result = await email_worker.deliver_email_job({"job_id": "job-123"}, payload.model_dump(mode="json"))

    assert result["status"] == "sent"
    assert captured["subject"] == "Verify"
    assert "Welcome" in captured["html"]
    assert "123456" in captured["text"]


@pytest.mark.asyncio
async def test_deliver_email_job_accepts_unknown_theme_with_fallback(monkeypatch):
    captured: dict[str, str] = {}

    async def _fake_send(payload: EmailJobPayload):
        captured["subject"] = payload.subject
        captured["html"] = payload.html_body or ""
        captured["text"] = payload.text_body
        return EmailSendResult(status="sent", recipient=payload.to_email, provider="log")

    monkeypatch.setattr(email_worker, "send_email", _fake_send)

    payload = {
        "to_email": "template@example.com",
        "subject": "{{ mail.subject }}",
        "preheader": "{{ mail.preheader }}",
        "theme": "ocean",
        "blocks": [{"type": "paragraph", "data": {"text": "Theme fallback body"}}],
        "context": {
            "mail": {
                "subject": "Template Subject",
                "preheader": "Template Preheader",
            }
        },
        "tags": ["verify"],
    }

    result = await email_worker.deliver_email_job({"job_id": "job-456"}, payload)

    assert result["status"] == "sent"
    assert captured["subject"] == "Template Subject"
    assert "Template Preheader" in captured["text"]
    assert "Theme fallback body" in captured["html"]
