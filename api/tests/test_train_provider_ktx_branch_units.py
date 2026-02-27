from __future__ import annotations

import json
from datetime import datetime
from typing import Any
from types import SimpleNamespace

import pytest

from app.modules.train.providers.base import ProviderOutcome, ProviderSchedule
from app.modules.train.providers.ktx_client import KTXClient, _parse_ktx_datetime, parse_ktx_search_response
from app.modules.train.providers.transport import TransportResponse
from app.modules.train.timezone import KST


class _QueuedTransport:
    def __init__(self, responses: list[tuple[int, Any]]) -> None:
        self._responses = list(responses)
        self.requests: list[dict[str, Any]] = []

    async def request(self, **kwargs) -> TransportResponse:  # noqa: ANN003
        self.requests.append(kwargs)
        if not self._responses:
            raise AssertionError("No queued transport response available")
        status_code, payload = self._responses.pop(0)
        text = payload if isinstance(payload, str) else json.dumps(payload)
        return TransportResponse(status_code=status_code, text=text, headers={})


def _schedule(*, general: bool = True, special: bool = True, wait_flag: str = "-1") -> ProviderSchedule:
    return ProviderSchedule(
        schedule_id="sched-1",
        provider="KTX",
        dep="서울",
        arr="부산",
        departure_at=datetime(2026, 2, 23, 9, 0, tzinfo=KST),
        arrival_at=datetime(2026, 2, 23, 11, 30, tzinfo=KST),
        train_no="305",
        availability={"general": general, "special": special},
        metadata={
            "dep_date": "20260223",
            "dep_station_code": "0001",
            "arr_station_code": "0020",
            "dep_time": "090000",
            "arr_time": "113000",
            "run_date": "20260223",
            "train_type_code": "100",
            "train_group_code": "109",
            "wait_reserve_flag": wait_flag,
        },
    )


def test_ktx_datetime_parse_guards_and_search_rows_normalization():
    assert _parse_ktx_datetime(None, "090000") is None
    assert _parse_ktx_datetime("2026", "0900") is None
    assert _parse_ktx_datetime("20260230", "090000") is None

    as_dict = parse_ktx_search_response(
        json.dumps(
            {
                "strResult": "SUCC",
                "trn_infos": {
                    "trn_info": {
                        "h_trn_no": "305",
                        "h_dpt_dt": "20260223",
                        "h_dpt_tm": "090000",
                        "h_arv_dt": "20260223",
                        "h_arv_tm": "113000",
                        "h_gen_rsv_cd": "11",
                        "h_spe_rsv_cd": "00",
                    }
                },
            }
        ),
        dep="서울",
        arr="부산",
    )
    assert as_dict.ok is True
    assert len(as_dict.data["schedules"]) == 1

    as_invalid = parse_ktx_search_response(
        json.dumps({"strResult": "SUCC", "trn_infos": {"trn_info": "invalid"}}),
        dep="서울",
        arr="부산",
    )
    assert as_invalid.ok is True
    assert as_invalid.data["schedules"] == []


@pytest.mark.asyncio
async def test_ktx_reserve_request_branch_matrix(monkeypatch):
    client = KTXClient(transport=_QueuedTransport([]))
    sched = _schedule()

    missing_context = ProviderSchedule(
        schedule_id="broken",
        provider="KTX",
        dep="서울",
        arr="부산",
        departure_at=sched.departure_at,
        arrival_at=sched.arrival_at,
        train_no="305",
        availability={"general": True, "special": False},
        metadata={},
    )
    outcome = await client._reserve_request(  # noqa: SLF001
        schedule=missing_context,
        seat_class="general",
        passengers={"adults": 1},
        user_id="u1",
        standby=False,
    )
    assert outcome.ok is False
    assert outcome.error_code == "invalid_schedule_context"

    invalid_passengers = await client._reserve_request(  # noqa: SLF001
        schedule=sched,
        seat_class="general",
        passengers={"adults": 0, "children": 0},
        user_id="u1",
        standby=False,
    )
    assert invalid_passengers.ok is False
    assert invalid_passengers.error_code == "invalid_passengers"

    server_error_client = KTXClient(transport=_QueuedTransport([(503, {})]))
    server_error = await server_error_client._reserve_request(  # noqa: SLF001
        schedule=sched,
        seat_class="general",
        passengers={"adults": 1},
        user_id="u1",
        standby=False,
    )
    assert server_error.ok is False
    assert server_error.retryable is True

    invalid_json_client = KTXClient(transport=_QueuedTransport([(200, "{not-json")]))
    invalid_json = await invalid_json_client._reserve_request(  # noqa: SLF001
        schedule=sched,
        seat_class="general",
        passengers={"adults": 1},
        user_id="u1",
        standby=False,
    )
    assert invalid_json.ok is False
    assert invalid_json.error_code == "invalid_json"

    fail_payload_client = KTXClient(transport=_QueuedTransport([(200, {"strResult": "FAIL", "h_msg_cd": "X"})]))
    fail_payload = await fail_payload_client._reserve_request(  # noqa: SLF001
        schedule=sched,
        seat_class="general",
        passengers={"adults": 1},
        user_id="u1",
        standby=False,
    )
    assert fail_payload.ok is False
    assert fail_payload.error_code == "ktx_reserve_fail_X"

    missing_pnr_client = KTXClient(transport=_QueuedTransport([(200, {"strResult": "SUCC"})]))
    missing_pnr = await missing_pnr_client._reserve_request(  # noqa: SLF001
        schedule=sched,
        seat_class="general",
        passengers={"adults": 1},
        user_id="u1",
        standby=False,
    )
    assert missing_pnr.ok is False
    assert missing_pnr.error_code == "reservation_id_missing"

    success_client = KTXClient(transport=_QueuedTransport([(200, {"strResult": "SUCC", "h_pnr_no": "PNR-1"})]))

    async def _reservations(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(
            ok=True,
            data={
                "reservations": [
                    {
                        "journey_no": "001",
                        "journey_cnt": "01",
                        "rsv_chg_no": "00000",
                        "wct_no": "WCT-1",
                    }
                ]
            },
        )

    monkeypatch.setattr(success_client, "get_reservations", _reservations)
    success = await success_client._reserve_request(  # noqa: SLF001
        schedule=sched,
        seat_class="general",
        passengers={"adults": 1, "children": 1},
        user_id="u1",
        standby=False,
    )
    assert success.ok is True
    assert success.data["reservation_id"] == "PNR-1"
    assert success.data["journey_no"] == "001"


@pytest.mark.asyncio
async def test_ktx_reserve_and_standby_decision_paths(monkeypatch):
    client = KTXClient(transport=_QueuedTransport([]))

    not_logged_in = await client.reserve(
        schedule_id="sched-1",
        seat_class="general",
        passengers={"adults": 1},
        user_id="u1",
    )
    assert not_logged_in.ok is False
    assert not_logged_in.error_code == "not_logged_in"

    client._logged_in_user_ids.add("u1")  # noqa: SLF001

    missing_context = await client.reserve(
        schedule_id="missing",
        seat_class="general",
        passengers={"adults": 1},
        user_id="u1",
    )
    assert missing_context.ok is False
    assert missing_context.error_code == "schedule_context_missing"

    client._schedule_cache["u1"] = {"sched-1": _schedule(general=False, special=False, wait_flag="-1")}  # noqa: SLF001
    sold_out = await client.reserve(
        schedule_id="sched-1",
        seat_class="general",
        passengers={"adults": 1},
        user_id="u1",
    )
    assert sold_out.ok is False
    assert sold_out.error_code == "sold_out"

    client._schedule_cache["u1"]["sched-1"] = _schedule(general=True, special=False, wait_flag="-1")  # noqa: SLF001
    special_unavailable = await client.reserve(
        schedule_id="sched-1",
        seat_class="special",
        passengers={"adults": 1},
        user_id="u1",
    )
    assert special_unavailable.ok is False
    assert special_unavailable.error_code == "special_seat_unavailable"

    client._schedule_cache["u1"]["sched-1"] = _schedule(general=False, special=True, wait_flag="-1")  # noqa: SLF001
    general_unavailable = await client.reserve(
        schedule_id="sched-1",
        seat_class="general",
        passengers={"adults": 1},
        user_id="u1",
    )
    assert general_unavailable.ok is False
    assert general_unavailable.error_code == "general_seat_unavailable"

    captured: dict[str, Any] = {}

    async def _fake_reserve_request(**kwargs):  # noqa: ANN003
        captured.update(kwargs)
        return ProviderOutcome(ok=True, data={"reservation_id": "PNR-2"})

    monkeypatch.setattr(client, "_reserve_request", _fake_reserve_request)
    client._schedule_cache["u1"]["sched-1"] = _schedule(general=False, special=True, wait_flag="-1")  # noqa: SLF001
    preferred = await client.reserve(
        schedule_id="sched-1",
        seat_class="general_preferred",
        passengers={"adults": 1},
        user_id="u1",
    )
    assert preferred.ok is True
    assert captured["seat_class"] == "special"
    assert captured["standby"] is False

    client._schedule_cache["u1"]["sched-1"] = _schedule(general=False, special=False, wait_flag="0")  # noqa: SLF001
    standby = await client.reserve_standby(
        schedule_id="sched-1",
        seat_class="special",
        passengers={"adults": 1},
        user_id="u1",
    )
    assert standby.ok is True
    assert captured["seat_class"] == "special"
    assert captured["standby"] is True


@pytest.mark.asyncio
async def test_ktx_pay_get_reservations_ticket_info_cancel_and_refund_branches(monkeypatch):
    client = KTXClient(transport=_QueuedTransport([]))
    user_id = "u1"

    not_logged_in_pay = await client.pay(
        reservation_id="PNR-1",
        user_id=user_id,
        payment_card={"card_number": "1"},
    )
    assert not_logged_in_pay.ok is False
    assert not_logged_in_pay.error_code == "not_logged_in"

    client._logged_in_user_ids.add(user_id)  # noqa: SLF001

    no_card = await client.pay(reservation_id="PNR-1", user_id=user_id, payment_card=None)
    assert no_card.ok is False
    assert no_card.error_code == "payment_card_missing"

    async def _reservations_fail(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=False, retryable=True, error_code="upstream")

    monkeypatch.setattr(client, "get_reservations", _reservations_fail)
    propagated = await client.pay(
        reservation_id="PNR-1",
        user_id=user_id,
        payment_card={"card_number": "4111", "card_password": "12", "validation_number": "123456", "card_expire": "2501"},
    )
    assert propagated.ok is False
    assert propagated.error_code == "upstream"

    async def _reservation_missing(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=True, data={"reservations": []})

    monkeypatch.setattr(client, "get_reservations", _reservation_missing)
    missing = await client.pay(
        reservation_id="PNR-1",
        user_id=user_id,
        payment_card={"card_number": "4111", "card_password": "12", "validation_number": "123456", "card_expire": "2501"},
    )
    assert missing.ok is False
    assert missing.error_code == "reservation_not_found"

    async def _reservation_paid(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=True, data={"reservations": [{"paid": True}]})

    monkeypatch.setattr(client, "get_reservations", _reservation_paid)
    already_paid = await client.pay(
        reservation_id="PNR-1",
        user_id=user_id,
        payment_card={"card_number": "4111", "card_password": "12", "validation_number": "123456", "card_expire": "2501"},
    )
    assert already_paid.ok is True
    assert already_paid.data["already_paid"] is True

    async def _reservation_unpaid(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=True, data={"reservations": [{"paid": False, "total_cost": 12000, "rsv_chg_no": "00000"}]})

    async def _ticket_info_with_wct(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=True, data={"tickets": [], "wct_no": "WCT-1"})

    monkeypatch.setattr(client, "get_reservations", _reservation_unpaid)
    monkeypatch.setattr(client, "ticket_info", _ticket_info_with_wct)
    incomplete_card = await client.pay(
        reservation_id="PNR-1",
        user_id=user_id,
        payment_card={"card_number": "4111"},
    )
    assert incomplete_card.ok is False
    assert incomplete_card.error_code == "payment_card_incomplete"

    async def _ticket_info_missing(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=True, data={"tickets": [], "wct_no": ""})

    monkeypatch.setattr(client, "ticket_info", _ticket_info_missing)
    missing_wct = await client.pay(
        reservation_id="PNR-1",
        user_id=user_id,
        payment_card={
            "card_number": "4111111111111111",
            "card_password": "12",
            "validation_number": "900101",
            "card_expire": "2501",
        },
    )
    assert missing_wct.ok is False
    assert missing_wct.error_code == "ktx_wct_no_missing"

    monkeypatch.setattr(client, "ticket_info", _ticket_info_with_wct)

    payment_server_error_client = KTXClient(transport=_QueuedTransport([(503, {})]))
    payment_server_error_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    monkeypatch.setattr(payment_server_error_client, "get_reservations", _reservation_unpaid)
    monkeypatch.setattr(payment_server_error_client, "ticket_info", _ticket_info_with_wct)
    payment_server_error = await payment_server_error_client.pay(
        reservation_id="PNR-1",
        user_id=user_id,
        payment_card={
            "card_number": "4111111111111111",
            "card_password": "12",
            "validation_number": "900101",
            "card_expire": "2501",
        },
    )
    assert payment_server_error.ok is False
    assert payment_server_error.error_code == "ktx_server_error"

    payment_invalid_json_client = KTXClient(transport=_QueuedTransport([(200, "{not-json")]))
    payment_invalid_json_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    monkeypatch.setattr(payment_invalid_json_client, "get_reservations", _reservation_unpaid)
    monkeypatch.setattr(payment_invalid_json_client, "ticket_info", _ticket_info_with_wct)
    invalid_json = await payment_invalid_json_client.pay(
        reservation_id="PNR-1",
        user_id=user_id,
        payment_card={
            "card_number": "4111111111111111",
            "card_password": "12",
            "validation_number": "900101",
            "card_expire": "2501",
        },
    )
    assert invalid_json.ok is False
    assert invalid_json.error_code == "invalid_json"

    payment_fail_client = KTXClient(transport=_QueuedTransport([(200, {"strResult": "FAIL", "h_msg_cd": "X"})]))
    payment_fail_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    monkeypatch.setattr(payment_fail_client, "get_reservations", _reservation_unpaid)
    monkeypatch.setattr(payment_fail_client, "ticket_info", _ticket_info_with_wct)
    pay_failed = await payment_fail_client.pay(
        reservation_id="PNR-1",
        user_id=user_id,
        payment_card={
            "card_number": "4111111111111111",
            "card_password": "12",
            "validation_number": "900101",
            "card_expire": "2501",
        },
    )
    assert pay_failed.ok is False
    assert pay_failed.error_code == "ktx_pay_fail_X"

    payment_success_client = KTXClient(transport=_QueuedTransport([(200, {"strResult": "SUCC"})]))
    payment_success_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    monkeypatch.setattr(payment_success_client, "get_reservations", _reservation_unpaid)
    monkeypatch.setattr(payment_success_client, "ticket_info", _ticket_info_with_wct)
    pay_ok = await payment_success_client.pay(
        reservation_id="PNR-1",
        user_id=user_id,
        payment_card={
            "card_number": "4111111111111111",
            "card_password": "12",
            "validation_number": "900101",
            "card_expire": "2501",
        },
    )
    assert pay_ok.ok is True
    assert pay_ok.data["paid"] is True

    relay_client = KTXClient(transport=_QueuedTransport([]))
    relay_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    monkeypatch.setattr(relay_client, "get_reservations", _reservation_unpaid)
    monkeypatch.setattr(relay_client, "ticket_info", _ticket_info_with_wct)
    relay_captured: dict[str, Any] = {}

    async def _relay_submit(**kwargs):  # noqa: ANN003
        relay_captured.update(kwargs)
        return SimpleNamespace(
            status_code=200,
            text=json.dumps({"strResult": "SUCC"}),
            relay_url="https://relay.example/payment",
            relay_id="relay_ktx",
            relay_domain="relay.example",
        )

    monkeypatch.setattr(
        "app.modules.train.providers.ktx_client.submit_ktx_payment_via_evervault_relay",
        _relay_submit,
    )
    relay_ok = await relay_client.pay(
        reservation_id="PNR-1",
        user_id=user_id,
        payment_card={
            "source": "evervault",
            "card_number": "ev:card",
            "card_password": "ev:pin2",
            "validation_number": "ev:birth",
            "card_expire": "ev:expiry",
        },
    )
    assert relay_ok.ok is True
    assert relay_ok.data["paid"] is True
    assert relay_captured["form_data"]["hidStlCrCrdNo1"] == "ev:card"
    assert relay_client._transport.requests == []  # noqa: SLF001

    not_logged_in_reservations = await KTXClient(transport=_QueuedTransport([])).get_reservations(user_id="u2")
    assert not_logged_in_reservations.ok is False

    reservations_server_error_client = KTXClient(transport=_QueuedTransport([(503, {})]))
    reservations_server_error_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    reservations_server_error = await reservations_server_error_client.get_reservations(user_id=user_id)
    assert reservations_server_error.ok is False
    assert reservations_server_error.error_code == "ktx_server_error"

    reservations_invalid_json_client = KTXClient(transport=_QueuedTransport([(200, "{oops")]))
    reservations_invalid_json_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    reservations_invalid_json = await reservations_invalid_json_client.get_reservations(user_id=user_id)
    assert reservations_invalid_json.ok is False
    assert reservations_invalid_json.error_code == "invalid_json"

    reservations_fail_client = KTXClient(transport=_QueuedTransport([(200, {"strResult": "FAIL", "h_msg_cd": "X"})]))
    reservations_fail_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    reservations_fail = await reservations_fail_client.get_reservations(user_id=user_id)
    assert reservations_fail.ok is False
    assert reservations_fail.error_code == "ktx_reservations_fail_X"

    reservations_ok_client = KTXClient(
        transport=_QueuedTransport(
            [
                (
                    200,
                    {
                        "strResult": "SUCC",
                        "jrny_infos": {
                            "jrny_info": [
                                "invalid",
                                {
                                    "train_infos": {
                                        "train_info": [
                                            "bad",
                                            {"h_pnr_no": "", "h_pay_flg": "N"},
                                            {
                                                "h_pnr_no": "PNR-OK",
                                                "h_run_dt": "20260223",
                                                "h_dpt_tm": "090000",
                                                "h_arv_tm": "113000",
                                                "h_pay_flg": "Y",
                                                "h_tot_seat_cnt": "1",
                                                "h_rsv_amt": "12000",
                                            },
                                        ]
                                    }
                                },
                            ]
                        },
                    },
                )
            ]
        )
    )
    reservations_ok_client._logged_in_user_ids.add(user_id)  # noqa: SLF001

    async def _ticket_info_ok(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=True, data={"tickets": [{"seat_no": "1A"}], "wct_no": "WCT-1"})

    monkeypatch.setattr(reservations_ok_client, "ticket_info", _ticket_info_ok)
    reservations_ok = await reservations_ok_client.get_reservations(user_id=user_id, paid_only=True, reservation_id="PNR-OK")
    assert reservations_ok.ok is True
    assert len(reservations_ok.data["reservations"]) == 1

    ticket_not_logged = await KTXClient(transport=_QueuedTransport([])).ticket_info(reservation_id="PNR-1", user_id="u2")
    assert ticket_not_logged.ok is False

    ticket_server_error_client = KTXClient(transport=_QueuedTransport([(503, {})]))
    ticket_server_error_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    ticket_server_error = await ticket_server_error_client.ticket_info(reservation_id="PNR-1", user_id=user_id)
    assert ticket_server_error.ok is False

    ticket_invalid_json_client = KTXClient(transport=_QueuedTransport([(200, "{oops")]))
    ticket_invalid_json_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    ticket_invalid_json = await ticket_invalid_json_client.ticket_info(reservation_id="PNR-1", user_id=user_id)
    assert ticket_invalid_json.ok is False

    ticket_fail_client = KTXClient(transport=_QueuedTransport([(200, {"strResult": "FAIL", "h_msg_cd": "X"})]))
    ticket_fail_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    ticket_fail = await ticket_fail_client.ticket_info(reservation_id="PNR-1", user_id=user_id)
    assert ticket_fail.ok is False

    ticket_ok_client = KTXClient(
        transport=_QueuedTransport(
            [
                (
                    200,
                    {
                        "strResult": "SUCC",
                        "h_wct_no": "WCT-1",
                        "jrny_infos": {"jrny_info": [{"seat_infos": {"seat_info": [{"h_srcar_no": "3", "h_seat_no": "4A"}]}}]},
                    },
                )
            ]
        )
    )
    ticket_ok_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    ticket_ok = await ticket_ok_client.ticket_info(reservation_id="PNR-1", user_id=user_id)
    assert ticket_ok.ok is True
    assert ticket_ok.data["wct_no"] == "WCT-1"
    assert len(ticket_ok.data["tickets"]) == 1

    cancel_client = KTXClient(transport=_QueuedTransport([]))
    cancel_not_logged = await cancel_client.cancel(artifact_data={"reservation_id": "PNR-1"}, user_id="u2")
    assert cancel_not_logged.ok is False
    cancel_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    cancel_missing = await cancel_client.cancel(artifact_data={}, user_id=user_id)
    assert cancel_missing.ok is False

    async def _refund_ok(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=True, data={"cancelled": True, "refunded": True})

    monkeypatch.setattr(cancel_client, "_refund_paid_reservation", _refund_ok)
    cancel_paid = await cancel_client.cancel(artifact_data={"reservation_id": "PNR-1", "paid": True}, user_id=user_id)
    assert cancel_paid.ok is True

    cancel_lookup_fail_client = KTXClient(transport=_QueuedTransport([]))
    cancel_lookup_fail_client._logged_in_user_ids.add(user_id)  # noqa: SLF001

    async def _reservations_lookup_fail(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=False, retryable=True, error_code="lookup_fail")

    monkeypatch.setattr(cancel_lookup_fail_client, "get_reservations", _reservations_lookup_fail)
    lookup_fail = await cancel_lookup_fail_client.cancel(
        artifact_data={"reservation_id": "PNR-1"},
        user_id=user_id,
    )
    assert lookup_fail.ok is False
    assert lookup_fail.error_code == "lookup_fail"

    cancel_lookup_empty_client = KTXClient(transport=_QueuedTransport([]))
    cancel_lookup_empty_client._logged_in_user_ids.add(user_id)  # noqa: SLF001

    async def _reservations_lookup_empty(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=True, data={"reservations": []})

    monkeypatch.setattr(cancel_lookup_empty_client, "get_reservations", _reservations_lookup_empty)
    lookup_empty = await cancel_lookup_empty_client.cancel(
        artifact_data={"reservation_id": "PNR-1"},
        user_id=user_id,
    )
    assert lookup_empty.ok is False
    assert lookup_empty.error_code == "reservation_not_found"

    cancel_server_error_client = KTXClient(transport=_QueuedTransport([(503, {})]))
    cancel_server_error_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    cancel_server_error = await cancel_server_error_client.cancel(
        artifact_data={"reservation_id": "PNR-1", "journey_no": "001", "journey_cnt": "01", "rsv_chg_no": "00000"},
        user_id=user_id,
    )
    assert cancel_server_error.ok is False
    assert cancel_server_error.error_code == "ktx_server_error"

    cancel_invalid_json_client = KTXClient(transport=_QueuedTransport([(200, "{oops")]))
    cancel_invalid_json_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    cancel_invalid_json = await cancel_invalid_json_client.cancel(
        artifact_data={"reservation_id": "PNR-1", "journey_no": "001", "journey_cnt": "01", "rsv_chg_no": "00000"},
        user_id=user_id,
    )
    assert cancel_invalid_json.ok is False
    assert cancel_invalid_json.error_code == "invalid_json"

    cancel_fail_client = KTXClient(transport=_QueuedTransport([(200, {"strResult": "FAIL", "h_msg_cd": "X"})]))
    cancel_fail_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    cancel_fail = await cancel_fail_client.cancel(
        artifact_data={"reservation_id": "PNR-1", "journey_no": "001", "journey_cnt": "01", "rsv_chg_no": "00000"},
        user_id=user_id,
    )
    assert cancel_fail.ok is False
    assert cancel_fail.error_code == "ktx_cancel_fail_X"

    cancel_ok_client = KTXClient(transport=_QueuedTransport([(200, {"strResult": "SUCC"})]))
    cancel_ok_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    cancel_ok = await cancel_ok_client.cancel(
        artifact_data={"reservation_id": "PNR-1", "journey_no": "001", "journey_cnt": "01", "rsv_chg_no": "00000"},
        user_id=user_id,
    )
    assert cancel_ok.ok is True
    assert cancel_ok.data["cancelled"] is True

    refund_client = KTXClient(transport=_QueuedTransport([(503, {})]))
    refund_server_error = await refund_client._refund_paid_reservation(reservation_id="PNR-1", user_id=user_id)  # noqa: SLF001
    assert refund_server_error.ok is False
    assert refund_server_error.error_code == "ktx_server_error"

    refund_invalid_json_client = KTXClient(transport=_QueuedTransport([(200, "{oops")]))
    refund_invalid_json = await refund_invalid_json_client._refund_paid_reservation(  # noqa: SLF001
        reservation_id="PNR-1",
        user_id=user_id,
    )
    assert refund_invalid_json.ok is False
    assert refund_invalid_json.error_code == "invalid_json"

    refund_fail_client = KTXClient(transport=_QueuedTransport([(200, {"strResult": "FAIL", "h_msg_cd": "X"})]))
    refund_fail = await refund_fail_client._refund_paid_reservation(reservation_id="PNR-1", user_id=user_id)  # noqa: SLF001
    assert refund_fail.ok is False
    assert refund_fail.error_code == "ktx_ticket_list_fail_X"

    refund_not_found_client = KTXClient(
        transport=_QueuedTransport(
            [
                (
                    200,
                    {
                        "strResult": "SUCC",
                        "reservation_list": [
                            {
                                "ticket_list": [
                                    {
                                        "train_info": [
                                            {"h_pnr_no": "OTHER-PNR"},
                                        ]
                                    }
                                ]
                            }
                        ],
                    },
                )
            ]
        )
    )
    refund_not_found = await refund_not_found_client._refund_paid_reservation(  # noqa: SLF001
        reservation_id="PNR-1",
        user_id=user_id,
    )
    assert refund_not_found.ok is False
    assert refund_not_found.error_code == "reservation_not_found"

    refund_refund_server_error_client = KTXClient(
        transport=_QueuedTransport(
            [
                (
                    200,
                    {
                        "strResult": "SUCC",
                        "reservation_list": [
                            {
                                "ticket_list": [
                                    {
                                        "train_info": [
                                            {
                                                "h_pnr_no": "PNR-1",
                                                "h_orgtk_ret_sale_dt": "20260222",
                                                "h_orgtk_wct_no": "WCT-1",
                                                "h_orgtk_sale_sqno": "0001",
                                                "h_orgtk_ret_pwd": "1111",
                                                "h_trn_no": "305",
                                            }
                                        ]
                                    }
                                ]
                            }
                        ],
                    },
                ),
                (503, {}),
            ]
        )
    )
    refund_refund_server_error = await refund_refund_server_error_client._refund_paid_reservation(  # noqa: SLF001
        reservation_id="PNR-1",
        user_id=user_id,
    )
    assert refund_refund_server_error.ok is False
    assert refund_refund_server_error.error_code == "ktx_server_error"

    refund_success_client = KTXClient(
        transport=_QueuedTransport(
            [
                (
                    200,
                    {
                        "strResult": "SUCC",
                        "reservation_list": [
                            {
                                "ticket_list": [
                                    {
                                        "train_info": [
                                            {
                                                "h_pnr_no": "PNR-1",
                                                "h_orgtk_ret_sale_dt": "20260222",
                                                "h_orgtk_wct_no": "WCT-1",
                                                "h_orgtk_sale_sqno": "0001",
                                                "h_orgtk_ret_pwd": "1111",
                                                "h_trn_no": "305",
                                            }
                                        ]
                                    }
                                ]
                            }
                        ],
                    },
                ),
                (200, {"strResult": "SUCC"}),
            ]
        )
    )
    refund_success = await refund_success_client._refund_paid_reservation(reservation_id="PNR-1", user_id=user_id)  # noqa: SLF001
    assert refund_success.ok is True
    assert refund_success.data["refunded"] is True


@pytest.mark.asyncio
async def test_ktx_additional_branch_coverage_for_search_standby_and_refund_paths(monkeypatch):
    user_id = "u-branch"

    search_server_error_client = KTXClient(transport=_QueuedTransport([(503, {})]))
    search_server_error_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    search_server_error = await search_server_error_client.search(
        dep="서울",
        arr="부산",
        date_value=datetime(2026, 2, 23, tzinfo=KST).date(),
        time_window_start="00:00",
        time_window_end="23:59",
        user_id=user_id,
    )
    assert search_server_error.ok is False
    assert search_server_error.error_code == "ktx_server_error"

    search_parse_fail_client = KTXClient(transport=_QueuedTransport([(200, {"strResult": "FAIL", "h_msg_cd": "NO"})]))
    search_parse_fail_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    search_parse_fail = await search_parse_fail_client.search(
        dep="서울",
        arr="부산",
        date_value=datetime(2026, 2, 23, tzinfo=KST).date(),
        time_window_start="00:00",
        time_window_end="23:59",
        user_id=user_id,
    )
    assert search_parse_fail.ok is False
    assert search_parse_fail.error_code == "ktx_api_fail_NO"

    standby_client = KTXClient(transport=_QueuedTransport([]))
    standby_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    standby_client._schedule_cache[user_id] = {  # noqa: SLF001
        "sched-special": _schedule(general=False, special=False, wait_flag="0"),
        "sched-general": _schedule(general=False, special=False, wait_flag="0"),
        "sched-pref": _schedule(general=True, special=False, wait_flag="-1"),
    }

    captured: list[dict[str, Any]] = []

    async def _capture_reserve_request(**kwargs):  # noqa: ANN003
        captured.append(kwargs)
        return ProviderOutcome(ok=True, data={"reservation_id": f"PNR-{len(captured)}"})

    monkeypatch.setattr(standby_client, "_reserve_request", _capture_reserve_request)

    special_standby = await standby_client.reserve(
        schedule_id="sched-special",
        seat_class="special",
        passengers={"adults": 1},
        user_id=user_id,
    )
    assert special_standby.ok is True
    assert captured[-1]["standby"] is True
    assert captured[-1]["seat_class"] == "special"

    general_standby = await standby_client.reserve(
        schedule_id="sched-general",
        seat_class="general",
        passengers={"adults": 1},
        user_id=user_id,
    )
    assert general_standby.ok is True
    assert captured[-1]["standby"] is True
    assert captured[-1]["seat_class"] == "general"

    preferred_fallback = await standby_client.reserve(
        schedule_id="sched-pref",
        seat_class="special_preferred",
        passengers={"adults": 1},
        user_id=user_id,
    )
    assert preferred_fallback.ok is True
    assert captured[-1]["seat_class"] == "general"

    standby_not_logged = await KTXClient(transport=_QueuedTransport([])).reserve_standby(
        schedule_id="missing",
        seat_class="general",
        passengers={"adults": 1},
        user_id="u-missing",
    )
    assert standby_not_logged.ok is False
    assert standby_not_logged.error_code == "not_logged_in"

    standby_missing_context_client = KTXClient(transport=_QueuedTransport([]))
    standby_missing_context_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    standby_missing_context = await standby_missing_context_client.reserve_standby(
        schedule_id="missing",
        seat_class="general",
        passengers={"adults": 1},
        user_id=user_id,
    )
    assert standby_missing_context.ok is False
    assert standby_missing_context.error_code == "schedule_context_missing"

    reservations_filter_client = KTXClient(
        transport=_QueuedTransport(
            [
                (
                    200,
                    {
                        "strResult": "SUCC",
                        "jrny_infos": {
                            "jrny_info": [
                                {
                                    "train_infos": {
                                        "train_info": [
                                            {"h_pnr_no": "OTHER", "h_pay_flg": "Y"},
                                            {"h_pnr_no": "TARGET", "h_pay_flg": "N"},
                                        ]
                                    }
                                }
                            ]
                        },
                    },
                )
            ]
        )
    )
    reservations_filter_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    async def _ticket_info_empty(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=True, data={"tickets": []})

    monkeypatch.setattr(reservations_filter_client, "ticket_info", _ticket_info_empty)
    filtered = await reservations_filter_client.get_reservations(
        user_id=user_id,
        reservation_id="TARGET",
        paid_only=True,
    )
    assert filtered.ok is True
    assert filtered.data["reservations"] == []

    refund_structure_guard_client = KTXClient(
        transport=_QueuedTransport(
            [
                (
                    200,
                    {
                        "strResult": "SUCC",
                        "reservation_list": [
                            "invalid",
                            {"ticket_list": []},
                            {"ticket_list": ["invalid"]},
                        ],
                    },
                )
            ]
        )
    )
    refund_structure_guard = await refund_structure_guard_client._refund_paid_reservation(  # noqa: SLF001
        reservation_id="PNR-1",
        user_id=user_id,
    )
    assert refund_structure_guard.ok is False
    assert refund_structure_guard.error_code == "reservation_not_found"

    refund_train_info_guard_client = KTXClient(
        transport=_QueuedTransport(
            [
                (
                    200,
                    {
                        "strResult": "SUCC",
                        "reservation_list": [
                            {
                                "ticket_list": [
                                    {
                                        "train_info": ["not-a-dict"],
                                    }
                                ]
                            }
                        ],
                    },
                )
            ]
        )
    )
    refund_train_info_guard = await refund_train_info_guard_client._refund_paid_reservation(  # noqa: SLF001
        reservation_id="PNR-1",
        user_id=user_id,
    )
    assert refund_train_info_guard.ok is False
    assert refund_train_info_guard.error_code == "reservation_not_found"

    refund_invalid_json_response_client = KTXClient(
        transport=_QueuedTransport(
            [
                (
                    200,
                    {
                        "strResult": "SUCC",
                        "reservation_list": [
                            {
                                "ticket_list": [
                                    {
                                        "train_info": [
                                            {
                                                "h_pnr_no": "PNR-1",
                                                "h_orgtk_ret_sale_dt": "20260222",
                                                "h_orgtk_wct_no": "WCT-1",
                                                "h_orgtk_sale_sqno": "0001",
                                                "h_orgtk_ret_pwd": "1111",
                                                "h_trn_no": "305",
                                            }
                                        ]
                                    }
                                ]
                            }
                        ],
                    },
                ),
                (200, "{invalid-json"),
            ]
        )
    )
    refund_invalid_json_response = await refund_invalid_json_response_client._refund_paid_reservation(  # noqa: SLF001
        reservation_id="PNR-1",
        user_id=user_id,
    )
    assert refund_invalid_json_response.ok is False
    assert refund_invalid_json_response.error_code == "invalid_json"

    refund_failure_payload_client = KTXClient(
        transport=_QueuedTransport(
            [
                (
                    200,
                    {
                        "strResult": "SUCC",
                        "reservation_list": [
                            {
                                "ticket_list": [
                                    {
                                        "train_info": [
                                            {
                                                "h_pnr_no": "PNR-1",
                                                "h_orgtk_ret_sale_dt": "20260222",
                                                "h_orgtk_wct_no": "WCT-1",
                                                "h_orgtk_sale_sqno": "0001",
                                                "h_orgtk_ret_pwd": "1111",
                                                "h_trn_no": "305",
                                            }
                                        ]
                                    }
                                ]
                            }
                        ],
                    },
                ),
                (200, {"strResult": "FAIL", "h_msg_cd": "REFUND"}),
            ]
        )
    )
    refund_failure_payload = await refund_failure_payload_client._refund_paid_reservation(  # noqa: SLF001
        reservation_id="PNR-1",
        user_id=user_id,
    )
    assert refund_failure_payload.ok is False
    assert refund_failure_payload.error_code == "ktx_refund_fail_REFUND"
