from __future__ import annotations

from app.services import email_template
from app.services.email_template import Block, format_completion_summary, render_email


def test_render_email_generates_html_and_text_from_blocks():
    rendered = render_email(
        subject="Verify your email for bominal",
        preheader="Verify with the button or enter the code.",
        theme="spring",
        blocks=[
            Block(type="hero", data={"title": "Welcome to bominal", "subtitle": "Verify your email to finish setup."}),
            Block(type="cta", data={"label": "Verify email", "url": "https://www.bominal.com/api/auth/verify-email?email=test%40example.com&code=123456"}),
            Block(type="otp", data={"code": "123456", "ttl_minutes": 10}),
        ],
    )

    assert rendered.subject == "Verify your email for bominal"
    assert "Welcome to bominal" in rendered.html
    assert "Verify email" in rendered.html
    assert "123456" in rendered.text


def test_render_email_falls_back_to_spring_theme():
    rendered = render_email(
        subject="Theme fallback",
        preheader="fallback",
        theme="not-a-season",  # type: ignore[arg-type]
        blocks=[Block(type="paragraph", data={"text": "Body"})],
    )

    assert "bominal" in rendered.html
    assert "Body" in rendered.text


def test_render_email_resolves_context_data_pointers():
    rendered = render_email(
        subject="Pointer test",
        preheader="context preheader",
        theme="winter",
        context={
            "user": {"display_name": "Jason"},
            "verify": {"url": "https://www.bominal.com/verify?code=abc", "otp": "981233"},
            "summary": "Successfully completed reservation for train on 2026-02-11 14:05 - SRT345 2026-02-11 for 2 people.",
        },
        blocks=[
            Block(type="hero", data={"title": "Welcome {{ user.display_name }}"}),
            Block(type="cta", data={"label": "Verify", "url": {"$ref": "verify.url"}}),
            Block(type="otp", data={"code": {"$ref": "verify.otp"}, "ttl_minutes": 10}),
            Block(type="mono", data={"text": {"$ref": "summary"}}),
        ],
    )

    assert "Welcome Jason" in rendered.html
    assert "https://www.bominal.com/verify?code=abc" in rendered.text
    assert "981233" in rendered.text
    assert "Successfully completed reservation for train" in rendered.text


def test_render_email_resolves_context_in_top_level_fields():
    rendered = render_email(
        subject="{{ mail.subject }}",
        preheader="{{ mail.preheader }}",
        theme="summer",
        context={
            "mail": {
                "subject": "Verify your account",
                "preheader": "Use the link below to continue.",
                "support_text": "Contact bominal support",
                "support_url": "https://www.bominal.com/help",
            }
        },
        blocks=[Block(type="paragraph", data={"text": "Body"})],
        footer_support_text="{{ mail.support_text }}",
        footer_support_url="{{ mail.support_url }}",
    )

    assert rendered.subject == "Verify your account"
    assert "Use the link below to continue." in rendered.text
    assert "Contact bominal support" in rendered.html
    assert "https://www.bominal.com/help" in rendered.text


def test_format_completion_summary_uses_canonical_field_order():
    summary = format_completion_summary(
        status="Successfully completed",
        task="reservation",
        module="train",
        completion_date="2026-02-11",
        completion_time="14:05",
        item="SRT345",
        target_date="2026-02-11",
        people=2,
    )

    assert summary == (
        "Successfully completed reservation for train on 2026-02-11 14:05 - "
        "SRT345 2026-02-11 for 2 people."
    )


def test_render_email_covers_kv_bullets_divider_and_helper_paths():
    rendered = render_email(
        subject="Status for {{ user.name }}",
        preheader="Details",
        theme="autumn",
        context={
            "user": {"name": "Casey"},
            "cta": {"url": "https://www.bominal.com/details"},
            "summary": {"status": "Done"},
        },
        blocks=[
            Block(type="paragraph", data={"text": "Line one"}),
            Block(
                type="cta",
                data={
                    "label": "Open",
                    "url": {"$ref": "cta.url"},
                    "helper_text": "Link expires soon.",
                },
            ),
            Block(
                type="kv",
                data={
                    "rows": [
                        {"k": "Task", "v": "Train"},
                        {"k": "Status", "v": {"$ref": "summary.status"}},
                    ]
                },
            ),
            Block(type="bullets", data={"items": ["First", "Second"]}),
            Block(type="divider", data={}),
            Block(type="mono", data={"text": "trace-id: abc-123"}),
        ],
        footer_support_url=None,
    )

    assert rendered.subject == "Status for Casey"
    assert "Link expires soon." in rendered.text
    assert "Task: Train" in rendered.text
    assert "- First" in rendered.text
    assert "------------------------" in rendered.text
    assert "Need help?" in rendered.text
    assert "trace-id: abc-123" in rendered.html


def test_resolve_with_context_handles_missing_refs_defaults_and_lists():
    assert email_template._lookup_pointer({"a": {"b": "c"}}, "a.b") == "c"
    assert email_template._lookup_pointer({"a": {"b": "c"}}, "a.x") is None

    resolved_missing_default = email_template._resolve_with_context(
        {"$ref": "missing.path", "default": "fallback"},
        {},
    )
    assert resolved_missing_default == "fallback"

    resolved_list = email_template._resolve_with_context(
        [
            "Hello {{ user.name }}",
            {"$ref": "summary.status"},
        ],
        {
            "user": {"name": "Avery"},
            "summary": {"status": "ok"},
        },
    )
    assert resolved_list == ["Hello Avery", "ok"]
