from __future__ import annotations

import base64
import json
import threading
import time
from dataclasses import dataclass
from urllib import parse, request

from app.core.config import Settings

GCE_METADATA_TOKEN_URL = (
    "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token"
)
SECRET_MANAGER_ACCESS_URL_TEMPLATE = (
    "https://secretmanager.googleapis.com/v1/projects/{project}/secrets/{secret}/versions/{version}:access"
)
SECRET_MANAGER_TIMEOUT_SECONDS = 5.0
SECRET_MANAGER_CACHE_TTL_SECONDS = 300.0


class MasterKeyResolutionError(RuntimeError):
    """Raised when a valid master key cannot be resolved."""


@dataclass(frozen=True)
class ResolvedMasterKey:
    master_key_b64: str
    source: str
    project_id: str | None = None
    secret_id: str | None = None
    secret_version: str | None = None


_cache_lock = threading.Lock()
_secret_cache: dict[tuple[str, str, str], tuple[str, float]] = {}


def _validate_master_key_b64(value: str, *, source: str) -> str:
    normalized = str(value or "").strip()
    if not normalized:
        raise MasterKeyResolutionError(f"MASTER_KEY from {source} is empty")
    try:
        decoded = base64.b64decode(normalized.encode("utf-8"), validate=True)
    except Exception as exc:
        raise MasterKeyResolutionError(f"MASTER_KEY from {source} is not valid base64") from exc
    if len(decoded) != 32:
        raise MasterKeyResolutionError(
            f"MASTER_KEY from {source} must decode to 32 bytes (got {len(decoded)})"
        )
    return normalized


def _http_get_json(url: str, *, headers: dict[str, str]) -> dict:
    req = request.Request(url=url, headers=headers, method="GET")
    with request.urlopen(req, timeout=SECRET_MANAGER_TIMEOUT_SECONDS) as response:
        payload_raw = response.read()
    payload = json.loads(payload_raw.decode("utf-8"))
    if not isinstance(payload, dict):
        raise MasterKeyResolutionError("Unexpected JSON payload while resolving master key")
    return payload


def _decode_secret_payload_data(encoded_payload: str) -> str:
    value = str(encoded_payload or "").strip()
    if not value:
        raise MasterKeyResolutionError("Secret Manager payload data is empty")
    # Secret Manager returns base64-encoded payload bytes.
    padded = value + "=" * (-len(value) % 4)
    try:
        decoded = base64.b64decode(padded.encode("utf-8"))
    except Exception as exc:
        raise MasterKeyResolutionError("Secret Manager payload is not valid base64") from exc
    try:
        return decoded.decode("utf-8").strip()
    except Exception as exc:
        raise MasterKeyResolutionError("Secret Manager payload is not valid UTF-8 text") from exc


def _fetch_metadata_access_token() -> str:
    payload = _http_get_json(
        GCE_METADATA_TOKEN_URL,
        headers={"Metadata-Flavor": "Google"},
    )
    token = str(payload.get("access_token") or "").strip()
    if not token:
        raise MasterKeyResolutionError("Metadata server token response did not include access_token")
    return token


def _fetch_master_key_from_secret_manager(*, project_id: str, secret_id: str, version: str) -> str:
    cache_key = (project_id, secret_id, version)
    now = time.monotonic()
    with _cache_lock:
        cached = _secret_cache.get(cache_key)
        if cached is not None:
            cached_key, expires_at = cached
            if now < expires_at:
                return cached_key
            _secret_cache.pop(cache_key, None)

    access_token = _fetch_metadata_access_token()
    access_url = SECRET_MANAGER_ACCESS_URL_TEMPLATE.format(
        project=parse.quote(project_id, safe=""),
        secret=parse.quote(secret_id, safe=""),
        version=parse.quote(version, safe=""),
    )
    payload = _http_get_json(
        access_url,
        headers={"Authorization": f"Bearer {access_token}"},
    )
    payload_obj = payload.get("payload")
    if not isinstance(payload_obj, dict):
        raise MasterKeyResolutionError("Secret Manager response missing payload object")
    secret_data = _decode_secret_payload_data(str(payload_obj.get("data") or ""))
    validated_key = _validate_master_key_b64(
        secret_data,
        source=f"gsm:{project_id}/{secret_id}@{version}",
    )
    with _cache_lock:
        _secret_cache[cache_key] = (validated_key, now + SECRET_MANAGER_CACHE_TTL_SECONDS)
    return validated_key


def resolve_master_key(*, settings: Settings) -> ResolvedMasterKey:
    override = str(settings.master_key_override or "").strip()
    if override:
        return ResolvedMasterKey(
            master_key_b64=_validate_master_key_b64(override, source="MASTER_KEY_OVERRIDE"),
            source="deploy_override",
        )

    env_master_key = str(settings.master_key or "").strip()
    if settings.gsm_master_key_enabled:
        project_id = str(settings.resolved_gsm_master_key_project_id or "").strip()
        secret_id = str(settings.gsm_master_key_secret_id or "").strip()
        secret_version = str(settings.gsm_master_key_version or "").strip()
        if not project_id:
            raise MasterKeyResolutionError(
                "GSM master-key resolution requires GSM_MASTER_KEY_PROJECT_ID or GCP_PROJECT_ID"
            )
        if not secret_id:
            raise MasterKeyResolutionError("GSM master-key resolution requires GSM_MASTER_KEY_SECRET_ID")
        if not secret_version:
            raise MasterKeyResolutionError("GSM master-key resolution requires GSM_MASTER_KEY_VERSION")
        try:
            resolved = _fetch_master_key_from_secret_manager(
                project_id=project_id,
                secret_id=secret_id,
                version=secret_version,
            )
        except Exception as exc:
            if settings.gsm_master_key_allow_env_fallback and env_master_key:
                return ResolvedMasterKey(
                    master_key_b64=_validate_master_key_b64(env_master_key, source="MASTER_KEY fallback"),
                    source="env_fallback",
                )
            raise MasterKeyResolutionError("Failed to resolve MASTER_KEY from Secret Manager") from exc
        return ResolvedMasterKey(
            master_key_b64=resolved,
            source="gcp_secret_manager",
            project_id=project_id,
            secret_id=secret_id,
            secret_version=secret_version,
        )

    if env_master_key:
        return ResolvedMasterKey(
            master_key_b64=_validate_master_key_b64(env_master_key, source="MASTER_KEY"),
            source="env",
        )

    raise MasterKeyResolutionError("MASTER_KEY is not configured")
