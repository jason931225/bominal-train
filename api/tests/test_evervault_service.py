from __future__ import annotations

import pytest

from app.services.evervault import (
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
