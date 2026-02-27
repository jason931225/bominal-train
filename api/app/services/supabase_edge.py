from __future__ import annotations

import logging
from typing import Any

import httpx

from app.core.config import get_settings
from app.schemas.notification import EmailJobPayload, EmailTag

settings = get_settings()
logger = logging.getLogger(__name__)


def _tag_payload(tags: list[str | EmailTag]) -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    for tag in tags or []:
        if isinstance(tag, str):
            rows.append({"name": tag, "value": "true"})
            continue
        if isinstance(tag, EmailTag):
            rows.append({"name": tag.name, "value": tag.value})
    return rows


def _edge_notify_endpoint() -> str | None:
    base = settings.resolved_supabase_edge_functions_base_url
    function_name = str(settings.supabase_edge_task_notify_function_name or "").strip()
    if not base or not function_name:
        return None
    return f"{base}/{function_name}"


async def send_task_notification_via_edge(payload: EmailJobPayload) -> bool:
    if not settings.edge_task_notify_enabled:
        return False
    endpoint = _edge_notify_endpoint()
    service_key = str(settings.supabase_service_role_key or "").strip()
    if not endpoint or not service_key:
        return False

    body: dict[str, Any] = {
        "to_email": str(payload.to_email),
        "subject": payload.subject,
        "text_body": payload.text_body,
        "html_body": payload.html_body,
        "tags": _tag_payload(payload.tags),
        "headers": payload.headers,
        "metadata": payload.metadata,
        "message_id": payload.message_id,
        "idempotency_key": payload.idempotency_key,
    }

    timeout_seconds = max(float(settings.supabase_edge_timeout_seconds), 1.0)
    try:
        async with httpx.AsyncClient(timeout=timeout_seconds) as client:
            response = await client.post(
                endpoint,
                headers={
                    "Authorization": f"Bearer {service_key}",
                    "apikey": service_key,
                    "Content-Type": "application/json",
                },
                json=body,
            )
    except Exception as exc:  # noqa: BLE001 - edge invoke fallback should stay soft-fail
        logger.warning("Edge task-notify invoke failed: %s", type(exc).__name__)
        return False

    if response.status_code >= 400:
        logger.warning("Edge task-notify rejected request: status=%s", response.status_code)
        return False

    try:
        payload_json = response.json() if response.content else {}
    except ValueError:
        logger.warning("Edge task-notify returned invalid JSON")
        return False
    if isinstance(payload_json, dict) and payload_json.get("ok") is False:
        logger.warning("Edge task-notify returned ok=false")
        return False
    return True
