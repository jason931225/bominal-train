from __future__ import annotations

from app.core.config import get_settings
from app.modules.train.providers.base import TrainProviderClient
from app.modules.train.providers.hybrid import HybridKTXClient, HybridSRTClient
from app.modules.train.providers.ktx_client import KTXClient
from app.modules.train.providers.mock import MockKTXClient, MockSRTClient
from app.modules.train.providers.srt_client import SRTClient
from app.modules.train.providers.transport import CurlCffiTransport, HttpxTransport


def _build_transport(provider: str):
    settings = get_settings()
    mode = settings.train_provider_transport.lower()
    impersonate = "chrome" if provider == "SRT" else "chrome131_android"

    if mode in {"curl", "curl_cffi"}:
        return CurlCffiTransport(impersonate=impersonate)
    if mode == "httpx":
        return HttpxTransport()

    # auto: try curl_cffi first, then fallback to httpx.
    try:
        return CurlCffiTransport(impersonate=impersonate)
    except RuntimeError:
        return HttpxTransport()


def _build_live_client(provider: str) -> TrainProviderClient:
    transport = _build_transport(provider)
    if provider == "SRT":
        return SRTClient(transport=transport)
    if provider == "KTX":
        return KTXClient(transport=transport)
    raise ValueError(f"Unsupported provider: {provider}")


def get_provider_client(provider: str) -> TrainProviderClient:
    settings = get_settings()
    mode = settings.train_provider_mode.lower()

    if mode == "mock":
        if provider == "SRT":
            return MockSRTClient()
        if provider == "KTX":
            return MockKTXClient()

    if mode in {"hybrid", "real_search_mock_execute"}:
        if provider == "SRT":
            return HybridSRTClient(live_client=_build_live_client("SRT"))
        if provider == "KTX":
            return HybridKTXClient(live_client=_build_live_client("KTX"))

    if provider in {"SRT", "KTX"}:
        return _build_live_client(provider)

    raise ValueError(f"Unsupported provider: {provider}")
