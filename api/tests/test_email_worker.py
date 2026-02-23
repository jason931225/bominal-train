from __future__ import annotations

import pytest
from pydantic import ValidationError

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


@pytest.mark.asyncio
async def test_deliver_email_job_accepts_legacy_template_payload_without_render_fields(monkeypatch):
    captured: dict[str, str] = {}

    async def _fake_send(payload: EmailJobPayload):
        captured["subject"] = payload.subject
        captured["text"] = payload.text_body
        captured["html"] = payload.html_body or ""
        return EmailSendResult(status="sent", recipient=payload.to_email, provider="log")

    monkeypatch.setattr(email_worker, "send_email", _fake_send)

    payload = {
        "to_email": "legacy-template@example.com",
        "subject": "Verify your email for bominal",
        "template_id": "onboarding_verify",
        "metadata": {"kind": "onboarding_verify"},
    }

    result = await email_worker.deliver_email_job({"job_id": "legacy-template-job"}, payload)

    assert result["status"] == "sent"
    assert captured["subject"] == "Verify your email for bominal"
    assert "Verify your email for bominal" in captured["text"]
    assert "Verify your email for bominal" in captured["html"]


@pytest.mark.asyncio
async def test_deliver_email_job_accepts_legacy_raw_payload_without_text_body(monkeypatch):
    captured: dict[str, str] = {}

    async def _fake_send(payload: EmailJobPayload):
        captured["subject"] = payload.subject
        captured["text"] = payload.text_body
        return EmailSendResult(status="sent", recipient=payload.to_email, provider="log")

    monkeypatch.setattr(email_worker, "send_email", _fake_send)

    payload = {
        "to_email": "legacy-raw@example.com",
        "subject": "Legacy subject only",
    }

    result = await email_worker.deliver_email_job({"job_id": "legacy-raw-job"}, payload)

    assert result["status"] == "sent"
    assert captured["subject"] == "Legacy subject only"
    assert captured["text"] == "Legacy subject only"


def test_build_email_payload_template_detection_and_validation_fallback_paths():
    assert email_worker._looks_like_template_payload({"metadata": {"kind": "password_reset"}}) is True
    assert email_worker._looks_like_template_payload({"subject": "x"}) is False

    # Template validation fails due invalid blocks type; falls back to raw payload.
    fallback = email_worker._build_email_payload(
        {
            "to_email": "fallback@example.com",
            "subject": "Fallback Subject",
            "blocks": "invalid",
        }
    )
    assert fallback.subject == "Fallback Subject"
    assert fallback.text_body == "Fallback Subject"

    # Both template and raw validation fail; template validation error is re-raised.
    with pytest.raises(ValidationError) as exc_info:
        email_worker._build_email_payload(
            {
                "to_email": "not-an-email",
                "subject": "Broken",
                "blocks": "invalid",
            }
        )
    assert "blocks" in str(exc_info.value)

    # Not template-like and invalid raw payload => raw validation error re-raised.
    with pytest.raises(ValidationError):
        email_worker._build_email_payload(
            {
                "to_email": "not-an-email",
                "subject": "",
            }
        )
