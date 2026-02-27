from __future__ import annotations

import logging
from dataclasses import dataclass

import httpx

from app.core.config import get_settings

settings = get_settings()
logger = logging.getLogger(__name__)


@dataclass(frozen=True)
class SupabasePasswordIdentity:
    user_id: str
    email: str


@dataclass(frozen=True)
class SupabaseCallbackSession:
    user_id: str
    email: str
    access_token: str


def _auth_base_url() -> str | None:
    base = str(settings.supabase_url or "").strip().rstrip("/")
    if not base:
        return None
    return f"{base}/auth/v1"


def _auth_api_key() -> str | None:
    value = str(settings.resolved_supabase_auth_api_key or "").strip()
    if not value:
        return None
    return value


def _auth_timeout_seconds() -> float:
    return max(float(settings.supabase_auth_timeout_seconds), 1.0)


def _extract_supabase_identity(payload: object, *, fallback_email: str = "") -> SupabasePasswordIdentity | None:
    if not isinstance(payload, dict):
        return None

    user_payload = payload.get("user") if isinstance(payload.get("user"), dict) else payload
    if not isinstance(user_payload, dict):
        return None

    user_id = str(user_payload.get("id") or "").strip()
    user_email = str(user_payload.get("email") or fallback_email).strip().lower()
    if not user_id or not user_email:
        return None
    return SupabasePasswordIdentity(user_id=user_id, email=user_email)


async def verify_supabase_password(*, email: str, password: str) -> SupabasePasswordIdentity | None:
    base_url = _auth_base_url()
    api_key = _auth_api_key()
    if not settings.supabase_auth_enabled or not base_url or not api_key:
        return None

    endpoint = f"{base_url}/token"
    params = {"grant_type": "password"}
    body = {"email": email, "password": password}
    headers = {
        "apikey": api_key,
        "Content-Type": "application/json",
    }

    try:
        async with httpx.AsyncClient(timeout=_auth_timeout_seconds()) as client:
            response = await client.post(endpoint, params=params, headers=headers, json=body)
    except Exception as exc:  # noqa: BLE001
        logger.warning("Supabase password verify transport failed: %s", type(exc).__name__)
        return None

    if response.status_code >= 400:
        return None

    try:
        payload = response.json()
    except ValueError:
        logger.warning("Supabase password verify returned invalid JSON")
        return None

    return _extract_supabase_identity(payload, fallback_email=email)


async def send_supabase_password_recovery(*, email: str, redirect_to: str | None = None) -> bool:
    base_url = _auth_base_url()
    api_key = _auth_api_key()
    if not settings.supabase_auth_enabled or not base_url or not api_key:
        return False

    endpoint = f"{base_url}/recover"
    body: dict[str, str] = {"email": email}
    normalized_redirect = str(redirect_to or "").strip()
    if normalized_redirect:
        body["redirect_to"] = normalized_redirect
    headers = {
        "apikey": api_key,
        "Content-Type": "application/json",
    }

    try:
        async with httpx.AsyncClient(timeout=_auth_timeout_seconds()) as client:
            response = await client.post(endpoint, headers=headers, json=body)
    except Exception as exc:  # noqa: BLE001
        logger.warning("Supabase password recovery transport failed: %s", type(exc).__name__)
        return False

    if response.status_code >= 400:
        logger.warning("Supabase password recovery rejected request: status=%s", response.status_code)
        return False
    return True


async def exchange_supabase_token_hash(*, token_hash: str, token_type: str) -> SupabaseCallbackSession | None:
    base_url = _auth_base_url()
    api_key = _auth_api_key()
    if not settings.supabase_auth_enabled or not base_url or not api_key:
        return None

    normalized_hash = str(token_hash or "").strip()
    normalized_type = str(token_type or "").strip().lower()
    if not normalized_hash or normalized_type not in {"recovery", "magiclink", "email", "signup"}:
        return None

    endpoint = f"{base_url}/verify"
    body = {"type": normalized_type, "token_hash": normalized_hash}
    headers = {
        "apikey": api_key,
        "Content-Type": "application/json",
    }

    try:
        async with httpx.AsyncClient(timeout=_auth_timeout_seconds()) as client:
            response = await client.post(endpoint, headers=headers, json=body)
    except Exception as exc:  # noqa: BLE001
        logger.warning("Supabase callback exchange transport failed: %s", type(exc).__name__)
        return None

    if response.status_code >= 400:
        logger.warning("Supabase callback exchange rejected request: status=%s", response.status_code)
        return None

    try:
        payload = response.json()
    except ValueError:
        logger.warning("Supabase callback exchange returned invalid JSON")
        return None

    access_token = str(payload.get("access_token") if isinstance(payload, dict) else "").strip()
    if not access_token:
        return None

    identity = _extract_supabase_identity(payload)
    if identity is None:
        return None

    return SupabaseCallbackSession(
        user_id=identity.user_id,
        email=identity.email,
        access_token=access_token,
    )


async def update_supabase_password(*, access_token: str, new_password: str) -> SupabasePasswordIdentity | None:
    base_url = _auth_base_url()
    api_key = _auth_api_key()
    if not settings.supabase_auth_enabled or not base_url or not api_key:
        return None

    normalized_access_token = str(access_token or "").strip()
    normalized_password = str(new_password or "")
    if not normalized_access_token or not normalized_password:
        return None

    endpoint = f"{base_url}/user"
    body = {"password": normalized_password}
    headers = {
        "apikey": api_key,
        "Authorization": f"Bearer {normalized_access_token}",
        "Content-Type": "application/json",
    }

    try:
        async with httpx.AsyncClient(timeout=_auth_timeout_seconds()) as client:
            response = await client.put(endpoint, headers=headers, json=body)
    except Exception as exc:  # noqa: BLE001
        logger.warning("Supabase password update transport failed: %s", type(exc).__name__)
        return None

    if response.status_code >= 400:
        logger.warning("Supabase password update rejected request: status=%s", response.status_code)
        return None

    try:
        payload = response.json()
    except ValueError:
        logger.warning("Supabase password update returned invalid JSON")
        return None

    return _extract_supabase_identity(payload)
