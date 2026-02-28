from __future__ import annotations

import pytest

from app.services.evervault import (
    EvervaultRelayError,
    _resolve_relay_runtime,
    build_payment_relay_definition,
)


def test_build_payment_relay_definition_ktx_uses_exact_form_decrypt_selectors():
    payload = build_payment_relay_definition(
        provider="KTX",
        payment_url="https://smart.letskorail.com/classes/com.korail.mobile.payment.ReservationPayment",
    )

    assert payload["destinationDomain"] == "smart.letskorail.com"
    assert payload["authentication"] == "api-key"
    routes = payload["routes"]
    assert isinstance(routes, list) and len(routes) == 1

    route = routes[0]
    assert route["method"] == "POST"
    assert route["path"] == "/classes/com.korail.mobile.payment.ReservationPayment"
    assert route["response"] == []

    request_actions = route["request"]
    assert isinstance(request_actions, list) and len(request_actions) == 1
    action = request_actions[0]
    assert action["action"] == "decrypt"
    selectors = action["selections"]
    assert [row["selector"] for row in selectors] == [
        "hidStlCrCrdNo1",
        "hidVanPwd1",
        "hidAthnVal1",
        "hidCrdVlidTrm1",
    ]
    assert all(row["type"] == "form" for row in selectors)
    assert all("*" not in str(row["selector"]) for row in selectors)


def test_build_payment_relay_definition_srt_uses_exact_form_decrypt_selectors():
    payload = build_payment_relay_definition(
        provider="SRT",
        payment_url="https://app.srail.or.kr/ata/selectListAta09036_n.do",
    )

    assert payload["destinationDomain"] == "app.srail.or.kr"
    selectors = payload["routes"][0]["request"][0]["selections"]
    assert [row["selector"] for row in selectors] == [
        "stlCrCrdNo1",
        "vanPwd1",
        "athnVal1",
        "crdVlidTrm1",
    ]
    assert all(row["type"] == "form" for row in selectors)
    assert all("*" not in str(row["selector"]) for row in selectors)


def test_build_payment_relay_definition_rejects_unknown_provider():
    with pytest.raises(ValueError, match="Unsupported provider"):
        build_payment_relay_definition(
            provider="UNKNOWN",
            payment_url="https://example.com/payment",
        )


@pytest.mark.asyncio
async def test_resolve_relay_runtime_uses_pinned_id_and_domain_without_management(monkeypatch: pytest.MonkeyPatch):
    monkeypatch.setattr("app.services.evervault.settings.evervault_ktx_payment_relay_id", "relay_destination_testktx")
    monkeypatch.setattr(
        "app.services.evervault.settings.evervault_ktx_payment_relay_domain",
        "smart-letskorail-com-app-d7431f3114b5.relay.evervault.app",
    )

    async def _management_request_should_not_run(**_kwargs):
        raise AssertionError("management API should not be called when relay ID/domain are pinned")

    monkeypatch.setattr("app.services.evervault._management_request", _management_request_should_not_run)

    runtime = await _resolve_relay_runtime(
        provider="KTX",
        payment_url="https://smart.letskorail.com/classes/com.korail.mobile.payment.ReservationPayment",
    )

    assert runtime.relay_id == "relay_destination_testktx"
    assert runtime.relay_domain == "smart-letskorail-com-app-d7431f3114b5.relay.evervault.app"


@pytest.mark.asyncio
async def test_resolve_relay_runtime_rejects_invalid_pinned_relay_domain(monkeypatch: pytest.MonkeyPatch):
    monkeypatch.setattr("app.services.evervault.settings.evervault_srt_payment_relay_id", "relay_destination_testsrt")
    monkeypatch.setattr("app.services.evervault.settings.evervault_srt_payment_relay_domain", "app.srail.or.kr")

    with pytest.raises(EvervaultRelayError, match="Evervault relay domain is invalid"):
        await _resolve_relay_runtime(
            provider="SRT",
            payment_url="https://app.srail.or.kr/ata/selectListAta09036_n.do",
        )
