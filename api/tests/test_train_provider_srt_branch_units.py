from __future__ import annotations

import json
from datetime import datetime
from typing import Any

import pytest

import app.modules.train.providers.srt_client as srt_module
from app.modules.train.providers.base import ProviderOutcome, ProviderSchedule
from app.modules.train.providers.srt_client import (
    SRTClient,
    _hhmmss_from_iso,
    _parse_srt_datetime,
    parse_srt_login_response,
    parse_srt_search_response,
)
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


async def _netfunnel_key() -> str:
    return "NET000001"


def _schedule(*, general: bool = True, special: bool = True, reserve_wait_code: str = "9") -> ProviderSchedule:
    return ProviderSchedule(
        schedule_id="sched-srt-1",
        provider="SRT",
        dep="수서",
        arr="부산",
        departure_at=datetime(2026, 2, 23, 9, 0, tzinfo=KST),
        arrival_at=datetime(2026, 2, 23, 11, 30, tzinfo=KST),
        train_no="381",
        availability={"general": general, "special": special},
        metadata={
            "train_code": "17",
            "dep_station_code": "0551",
            "arr_station_code": "0020",
            "dep_date": "20260223",
            "dep_time": "090000",
            "arr_time": "113000",
            "dep_station_constitution_order": "0001",
            "arr_station_constitution_order": "0002",
            "dep_station_run_order": "0001",
            "arr_station_run_order": "0002",
            "reserve_wait_code": reserve_wait_code,
        },
    )


def test_srt_helper_parsers_and_login_response_branches():
    assert _parse_srt_datetime(None, "090000") is None
    assert _parse_srt_datetime("2026", "0900") is None
    assert _parse_srt_datetime("20260230", "090000") is None

    assert _hhmmss_from_iso(None) == ""
    assert _hhmmss_from_iso("not-iso") == ""
    assert _hhmmss_from_iso("2026-02-23T09:10:11+09:00") == "091011"

    text_invalid_user = parse_srt_login_response("존재하지않는 회원입니다")
    assert text_invalid_user.ok is False
    assert text_invalid_user.error_code == "invalid_credentials"

    ip_blocked = parse_srt_login_response("Your IP Address Blocked")
    assert ip_blocked.ok is False
    assert ip_blocked.error_code == "ip_blocked"

    invalid_json = parse_srt_login_response("{not-json")
    assert invalid_json.ok is False
    assert invalid_json.error_code == "invalid_json"

    fail_rtn = parse_srt_login_response(json.dumps({"strResult": "FAIL", "RTNCD": "N", "MSG": "bad"}))
    assert fail_rtn.ok is False
    assert fail_rtn.error_code == "login_failed"

    missing_user_map = parse_srt_login_response(json.dumps({"strResult": "SUCC", "RTNCD": "Y", "MSG": "ok"}))
    assert missing_user_map.ok is False
    assert missing_user_map.error_code == "login_failed"

    parse_fail = parse_srt_search_response(
        json.dumps({"outDataSets": {"dsOutput0": {"strResult": "FAIL", "msgCd": "X", "msgTxt": "bad"}}}),
        dep="수서",
        arr="부산",
    )
    assert parse_fail.ok is False
    assert parse_fail.error_code == "srt_api_fail_X"

    parse_rows_dict = parse_srt_search_response(
        json.dumps(
            {
                "outDataSets": {
                    "dsOutput0": [{"strResult": "SUCC"}],
                    "dsOutput1": {
                        "stlbTrnClsfCd": "17",
                        "dptDt": "20260223",
                        "dptTm": "090000",
                        "arvDt": "20260223",
                        "arvTm": "113000",
                        "trnNo": "381",
                        "gnrmRsvPsbStr": "예약가능",
                        "sprmRsvPsbStr": "매진",
                    },
                }
            }
        ),
        dep="수서",
        arr="부산",
    )
    assert parse_rows_dict.ok is True
    assert len(parse_rows_dict.data["schedules"]) == 1

    parse_rows_invalid = parse_srt_search_response(
        json.dumps({"outDataSets": {"dsOutput0": [{"strResult": "SUCC"}], "dsOutput1": "invalid"}}),
        dep="수서",
        arr="부산",
    )
    assert parse_rows_invalid.ok is True
    assert parse_rows_invalid.data["schedules"] == []


@pytest.mark.asyncio
async def test_srt_reserve_paths_and_pay_paths(monkeypatch):
    client = SRTClient(transport=_QueuedTransport([]))
    monkeypatch.setattr(client._netfunnel, "run", _netfunnel_key)  # noqa: SLF001

    invalid_context_schedule = ProviderSchedule(
        schedule_id="broken",
        provider="SRT",
        dep="수서",
        arr="부산",
        departure_at=datetime(2026, 2, 23, 9, 0, tzinfo=KST),
        arrival_at=datetime(2026, 2, 23, 11, 30, tzinfo=KST),
        train_no="381",
        availability={"general": True, "special": False},
        metadata={},
    )
    invalid_context = await client._reserve_request(  # noqa: SLF001
        schedule=invalid_context_schedule,
        seat_class="general",
        passengers={"adults": 1},
        user_id="u1",
        standby=False,
    )
    assert invalid_context.ok is False
    assert invalid_context.error_code == "invalid_schedule_context"

    invalid_passengers = await client._reserve_request(  # noqa: SLF001
        schedule=_schedule(),
        seat_class="general",
        passengers={"adults": 0, "children": 0},
        user_id="u1",
        standby=False,
    )
    assert invalid_passengers.ok is False
    assert invalid_passengers.error_code == "invalid_passengers"

    server_error_client = SRTClient(transport=_QueuedTransport([(503, {})]))
    monkeypatch.setattr(server_error_client._netfunnel, "run", _netfunnel_key)  # noqa: SLF001
    server_error = await server_error_client._reserve_request(  # noqa: SLF001
        schedule=_schedule(),
        seat_class="general",
        passengers={"adults": 1},
        user_id="u1",
        standby=False,
    )
    assert server_error.ok is False
    assert server_error.error_code == "srt_server_error"

    invalid_json_client = SRTClient(transport=_QueuedTransport([(200, "{oops")]))
    monkeypatch.setattr(invalid_json_client._netfunnel, "run", _netfunnel_key)  # noqa: SLF001
    invalid_json = await invalid_json_client._reserve_request(  # noqa: SLF001
        schedule=_schedule(),
        seat_class="general",
        passengers={"adults": 1},
        user_id="u1",
        standby=False,
    )
    assert invalid_json.ok is False
    assert invalid_json.error_code == "invalid_json"

    failure_client = SRTClient(
        transport=_QueuedTransport([(200, {"resultMap": [{"strResult": "FAIL", "msgCd": "X", "msgTxt": "bad"}]})])
    )
    monkeypatch.setattr(failure_client._netfunnel, "run", _netfunnel_key)  # noqa: SLF001
    reserve_fail = await failure_client._reserve_request(  # noqa: SLF001
        schedule=_schedule(),
        seat_class="general",
        passengers={"adults": 1},
        user_id="u1",
        standby=False,
    )
    assert reserve_fail.ok is False
    assert reserve_fail.error_code == "srt_reserve_fail_X"

    missing_pnr_client = SRTClient(transport=_QueuedTransport([(200, {"resultMap": [{"strResult": "SUCC"}]})]))
    monkeypatch.setattr(missing_pnr_client._netfunnel, "run", _netfunnel_key)  # noqa: SLF001
    missing_pnr = await missing_pnr_client._reserve_request(  # noqa: SLF001
        schedule=_schedule(),
        seat_class="general",
        passengers={"adults": 1},
        user_id="u1",
        standby=False,
    )
    assert missing_pnr.ok is False
    assert missing_pnr.error_code == "reservation_id_missing"

    success_client = SRTClient(
        transport=_QueuedTransport([(200, {"resultMap": [{"strResult": "SUCC"}], "reservListMap": [{"pnrNo": "PNR-1"}]})])
    )
    monkeypatch.setattr(success_client._netfunnel, "run", _netfunnel_key)  # noqa: SLF001
    reserve_ok = await success_client._reserve_request(  # noqa: SLF001
        schedule=_schedule(),
        seat_class="general",
        passengers={"adults": 1},
        user_id="u1",
        standby=False,
    )
    assert reserve_ok.ok is True
    assert reserve_ok.data["reservation_id"] == "PNR-1"

    not_logged_in = await client.reserve(
        schedule_id="sched",
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

    client._schedule_cache["u1"] = {"sched": _schedule(general=False, special=False, reserve_wait_code="9")}  # noqa: SLF001

    async def _reserve_standby_success(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=True, data={"reservation_id": "PNR-S"})

    monkeypatch.setattr(client, "reserve_standby", _reserve_standby_success)
    sold_out_with_standby = await client.reserve(
        schedule_id="sched",
        seat_class="general",
        passengers={"adults": 1},
        user_id="u1",
    )
    assert sold_out_with_standby.ok is True

    client._schedule_cache["u1"]["sched"] = _schedule(general=True, special=False, reserve_wait_code="9")  # noqa: SLF001
    special_unavailable = await client.reserve(
        schedule_id="sched",
        seat_class="special",
        passengers={"adults": 1},
        user_id="u1",
    )
    assert special_unavailable.ok is False
    assert special_unavailable.error_code == "special_seat_unavailable"

    client._schedule_cache["u1"]["sched"] = _schedule(general=False, special=True, reserve_wait_code="9")  # noqa: SLF001
    general_unavailable = await client.reserve(
        schedule_id="sched",
        seat_class="general",
        passengers={"adults": 1},
        user_id="u1",
    )
    assert general_unavailable.ok is False
    assert general_unavailable.error_code == "general_seat_unavailable"

    client_pay = SRTClient(transport=_QueuedTransport([]))
    no_login_pay = await client_pay.pay(
        reservation_id="PNR-1",
        user_id="u1",
        payment_card={"card_number": "1"},
    )
    assert no_login_pay.ok is False
    assert no_login_pay.error_code == "not_logged_in"

    client_pay._logged_in_user_ids.add("u1")  # noqa: SLF001
    no_card = await client_pay.pay(reservation_id="PNR-1", user_id="u1", payment_card=None)
    assert no_card.ok is False
    assert no_card.error_code == "payment_card_missing"

    async def _reservation_lookup_fail(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=False, retryable=True, error_code="lookup_fail")

    monkeypatch.setattr(client_pay, "get_reservations", _reservation_lookup_fail)
    lookup_fail = await client_pay.pay(
        reservation_id="PNR-1",
        user_id="u1",
        payment_card={"card_number": "1", "card_password": "12", "validation_number": "900101", "card_expire": "2501"},
    )
    assert lookup_fail.ok is False
    assert lookup_fail.error_code == "lookup_fail"

    async def _reservation_missing(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=True, data={"reservations": []})

    monkeypatch.setattr(client_pay, "get_reservations", _reservation_missing)
    missing = await client_pay.pay(
        reservation_id="PNR-1",
        user_id="u1",
        payment_card={"card_number": "1", "card_password": "12", "validation_number": "900101", "card_expire": "2501"},
    )
    assert missing.ok is False
    assert missing.error_code == "reservation_not_found"

    async def _reservation_paid(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=True, data={"reservations": [{"paid": True}]})

    monkeypatch.setattr(client_pay, "get_reservations", _reservation_paid)
    already_paid = await client_pay.pay(
        reservation_id="PNR-1",
        user_id="u1",
        payment_card={"card_number": "1", "card_password": "12", "validation_number": "900101", "card_expire": "2501"},
    )
    assert already_paid.ok is True
    assert already_paid.data["already_paid"] is True

    async def _reservation_unpaid(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=True, data={"reservations": [{"paid": False, "seat_count": 1, "total_cost": 12000}]})

    monkeypatch.setattr(client_pay, "get_reservations", _reservation_unpaid)
    membership_missing = await client_pay.pay(
        reservation_id="PNR-1",
        user_id="u1",
        payment_card={"card_number": "1", "card_password": "12", "validation_number": "900101", "card_expire": "2501"},
    )
    assert membership_missing.ok is False
    assert membership_missing.error_code == "membership_number_missing"

    client_pay._membership_number_by_user_id["u1"] = "M-1"  # noqa: SLF001
    incomplete_card = await client_pay.pay(
        reservation_id="PNR-1",
        user_id="u1",
        payment_card={"card_number": "1"},
    )
    assert incomplete_card.ok is False
    assert incomplete_card.error_code == "payment_card_incomplete"

    pay_server_error_client = SRTClient(transport=_QueuedTransport([(503, {})]))
    pay_server_error_client._logged_in_user_ids.add("u1")  # noqa: SLF001
    pay_server_error_client._membership_number_by_user_id["u1"] = "M-1"  # noqa: SLF001
    monkeypatch.setattr(pay_server_error_client, "get_reservations", _reservation_unpaid)
    pay_server_error = await pay_server_error_client.pay(
        reservation_id="PNR-1",
        user_id="u1",
        payment_card={
            "card_number": "4111111111111111",
            "card_password": "12",
            "validation_number": "900101",
            "card_expire": "2501",
        },
    )
    assert pay_server_error.ok is False
    assert pay_server_error.error_code == "srt_server_error"

    pay_invalid_json_client = SRTClient(transport=_QueuedTransport([(200, "{oops")]))
    pay_invalid_json_client._logged_in_user_ids.add("u1")  # noqa: SLF001
    pay_invalid_json_client._membership_number_by_user_id["u1"] = "M-1"  # noqa: SLF001
    monkeypatch.setattr(pay_invalid_json_client, "get_reservations", _reservation_unpaid)
    pay_invalid_json = await pay_invalid_json_client.pay(
        reservation_id="PNR-1",
        user_id="u1",
        payment_card={
            "card_number": "4111111111111111",
            "card_password": "12",
            "validation_number": "900101",
            "card_expire": "2501",
        },
    )
    assert pay_invalid_json.ok is False
    assert pay_invalid_json.error_code == "invalid_json"

    pay_fail_client = SRTClient(
        transport=_QueuedTransport([(200, {"outDataSets": {"dsOutput0": [{"strResult": "FAIL", "msgCd": "X", "msgTxt": "bad"}]}})])
    )
    pay_fail_client._logged_in_user_ids.add("u1")  # noqa: SLF001
    pay_fail_client._membership_number_by_user_id["u1"] = "M-1"  # noqa: SLF001
    monkeypatch.setattr(pay_fail_client, "get_reservations", _reservation_unpaid)
    pay_fail = await pay_fail_client.pay(
        reservation_id="PNR-1",
        user_id="u1",
        payment_card={
            "card_number": "4111111111111111",
            "card_password": "12",
            "validation_number": "900101",
            "card_expire": "2501",
        },
    )
    assert pay_fail.ok is False
    assert pay_fail.error_code == "srt_payment_fail_X"

    pay_success_client = SRTClient(transport=_QueuedTransport([(200, {"resultMap": [{"strResult": "SUCC"}]})]))
    pay_success_client._logged_in_user_ids.add("u1")  # noqa: SLF001
    pay_success_client._membership_number_by_user_id["u1"] = "M-1"  # noqa: SLF001
    monkeypatch.setattr(pay_success_client, "get_reservations", _reservation_unpaid)
    pay_ok = await pay_success_client.pay(
        reservation_id="PNR-1",
        user_id="u1",
        payment_card={
            "card_number": "4111111111111111",
            "card_password": "12",
            "validation_number": "900101",
            "card_expire": "2501",
        },
    )
    assert pay_ok.ok is True
    assert pay_ok.data["paid"] is True


@pytest.mark.asyncio
async def test_srt_reservations_ticket_info_reserve_info_and_cancel_branches(monkeypatch):
    user_id = "u1"

    not_logged_res = await SRTClient(transport=_QueuedTransport([])).get_reservations(user_id=user_id)
    assert not_logged_res.ok is False

    reservations_server_error_client = SRTClient(transport=_QueuedTransport([(503, {})]))
    reservations_server_error_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    reservations_server_error = await reservations_server_error_client.get_reservations(user_id=user_id)
    assert reservations_server_error.ok is False
    assert reservations_server_error.error_code == "srt_server_error"

    reservations_invalid_json_client = SRTClient(transport=_QueuedTransport([(200, "{oops")]))
    reservations_invalid_json_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    reservations_invalid_json = await reservations_invalid_json_client.get_reservations(user_id=user_id)
    assert reservations_invalid_json.ok is False
    assert reservations_invalid_json.error_code == "invalid_json"

    reservations_fail_client = SRTClient(transport=_QueuedTransport([(200, {"resultMap": [{"strResult": "FAIL", "msgCd": "X"}]})]))
    reservations_fail_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    reservations_fail = await reservations_fail_client.get_reservations(user_id=user_id)
    assert reservations_fail.ok is False
    assert reservations_fail.error_code == "srt_reservations_fail_X"

    reservations_ok_client = SRTClient(
        transport=_QueuedTransport(
            [
                (
                    200,
                    {
                        "resultMap": [{"strResult": "SUCC"}],
                        "trainListMap": [
                            "invalid",
                            {"pnrNo": "PNR-1", "rcvdAmt": "12000", "tkSpecNum": "1", "rsvWaitPsbCd": "9"},
                        ],
                        "payListMap": [
                            "invalid",
                            {
                                "pnrNo": "PNR-1",
                                "stlFlg": "Y",
                                "dptDt": "20260223",
                                "dptTm": "090000",
                                "arvTm": "113000",
                                "dptRsStnCd": "0551",
                                "arvRsStnCd": "0020",
                                "iseLmtDt": "20260222",
                                "iseLmtTm": "132906",
                            },
                        ],
                    },
                )
            ]
        )
    )
    reservations_ok_client._logged_in_user_ids.add(user_id)  # noqa: SLF001

    async def _ticket_info_ok(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=True, data={"tickets": [{"seat_no": "1A"}]})

    monkeypatch.setattr(reservations_ok_client, "ticket_info", _ticket_info_ok)
    reservations_ok = await reservations_ok_client.get_reservations(
        user_id=user_id,
        paid_only=True,
        reservation_id="PNR-1",
    )
    assert reservations_ok.ok is True
    assert len(reservations_ok.data["reservations"]) == 1

    ticket_not_logged = await SRTClient(transport=_QueuedTransport([])).ticket_info(reservation_id="PNR-1", user_id="u2")
    assert ticket_not_logged.ok is False

    ticket_server_error_client = SRTClient(transport=_QueuedTransport([(503, {})]))
    ticket_server_error_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    ticket_server_error = await ticket_server_error_client.ticket_info(reservation_id="PNR-1", user_id=user_id)
    assert ticket_server_error.ok is False

    ticket_invalid_json_client = SRTClient(transport=_QueuedTransport([(200, "{oops")]))
    ticket_invalid_json_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    ticket_invalid_json = await ticket_invalid_json_client.ticket_info(reservation_id="PNR-1", user_id=user_id)
    assert ticket_invalid_json.ok is False

    ticket_not_found_client = SRTClient(
        transport=_QueuedTransport([(200, {"resultMap": [{"strResult": "FAIL", "msgTxt": "조회자료가 없습니다."}]})])
    )
    ticket_not_found_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    ticket_not_found = await ticket_not_found_client.ticket_info(reservation_id="PNR-1", user_id=user_id)
    assert ticket_not_found.ok is False
    assert ticket_not_found.error_code == "reservation_not_found"

    ticket_ok_client = SRTClient(
        transport=_QueuedTransport(
            [
                (
                    200,
                    {
                        "resultMap": [{"strResult": "SUCC"}],
                        "trainListMap": [{"scarNo": "3", "seatNo": "1A", "psrmClCd": "1", "psgTpCd": "1"}],
                    },
                )
            ]
        )
    )
    ticket_ok_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    ticket_ok = await ticket_ok_client.ticket_info(reservation_id="PNR-1", user_id=user_id)
    assert ticket_ok.ok is True
    assert len(ticket_ok.data["tickets"]) == 1

    reserve_info_server_error_client = SRTClient(transport=_QueuedTransport([(503, {})]))
    reserve_info_server_error = await reserve_info_server_error_client._reserve_info(  # noqa: SLF001
        reservation_id="PNR-1",
        user_id=user_id,
    )
    assert reserve_info_server_error.ok is False
    assert reserve_info_server_error.error_code == "srt_server_error"

    reserve_info_invalid_json_client = SRTClient(transport=_QueuedTransport([(200, "{oops")]))
    reserve_info_invalid_json = await reserve_info_invalid_json_client._reserve_info(  # noqa: SLF001
        reservation_id="PNR-1",
        user_id=user_id,
    )
    assert reserve_info_invalid_json.ok is False
    assert reserve_info_invalid_json.error_code == "invalid_json"

    reserve_info_not_found_client = SRTClient(transport=_QueuedTransport([(200, {"ErrorMsg": "조회자료가 없습니다."})]))
    reserve_info_not_found = await reserve_info_not_found_client._reserve_info(  # noqa: SLF001
        reservation_id="PNR-1",
        user_id=user_id,
    )
    assert reserve_info_not_found.ok is False
    assert reserve_info_not_found.error_code == "reservation_not_found"

    reserve_info_error_code_client = SRTClient(transport=_QueuedTransport([(200, {"ErrorCode": "9", "ErrorMsg": "bad"})]))
    reserve_info_error_code = await reserve_info_error_code_client._reserve_info(  # noqa: SLF001
        reservation_id="PNR-1",
        user_id=user_id,
    )
    assert reserve_info_error_code.ok is False
    assert reserve_info_error_code.error_code == "srt_reserve_info_fail_9"

    reserve_info_missing_client = SRTClient(transport=_QueuedTransport([(200, {"resultMap": [{"strResult": "SUCC"}]})]))
    reserve_info_missing = await reserve_info_missing_client._reserve_info(  # noqa: SLF001
        reservation_id="PNR-1",
        user_id=user_id,
    )
    assert reserve_info_missing.ok is False
    assert reserve_info_missing.error_code == "reserve_info_missing"

    reserve_info_ok_client = SRTClient(
        transport=_QueuedTransport(
            [
                (
                    200,
                    {
                        "resultMap": [{"strResult": "SUCC"}],
                        "outDataSets": {"dsOutput1": [{"pnrNo": "PNR-1"}]},
                    },
                )
            ]
        )
    )
    reserve_info_ok = await reserve_info_ok_client._reserve_info(reservation_id="PNR-1", user_id=user_id)  # noqa: SLF001
    assert reserve_info_ok.ok is True

    refund_server_error_client = SRTClient(transport=_QueuedTransport([(503, {})]))
    async def _reserve_info_ok(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=True, data={"reserve_info": {}})

    monkeypatch.setattr(refund_server_error_client, "_reserve_info", _reserve_info_ok)
    refund_server_error = await refund_server_error_client._refund_paid_reservation(  # noqa: SLF001
        reservation_id="PNR-1",
        user_id=user_id,
    )
    assert refund_server_error.ok is False
    assert refund_server_error.error_code == "srt_server_error"

    refund_invalid_json_client = SRTClient(transport=_QueuedTransport([(200, "{oops")]))
    monkeypatch.setattr(refund_invalid_json_client, "_reserve_info", _reserve_info_ok)
    refund_invalid_json = await refund_invalid_json_client._refund_paid_reservation(  # noqa: SLF001
        reservation_id="PNR-1",
        user_id=user_id,
    )
    assert refund_invalid_json.ok is False
    assert refund_invalid_json.error_code == "invalid_json"

    refund_fail_client = SRTClient(transport=_QueuedTransport([(200, {"resultMap": [{"strResult": "FAIL", "msgCd": "X"}]})]))
    monkeypatch.setattr(refund_fail_client, "_reserve_info", _reserve_info_ok)
    refund_fail = await refund_fail_client._refund_paid_reservation(reservation_id="PNR-1", user_id=user_id)  # noqa: SLF001
    assert refund_fail.ok is False
    assert refund_fail.error_code == "srt_refund_fail_X"

    refund_ok_client = SRTClient(transport=_QueuedTransport([(200, {"resultMap": [{"strResult": "SUCC"}]})]))
    async def _reserve_info_ok_with_trace(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=True, data={"reserve_info": {}, "http_trace": {}})

    monkeypatch.setattr(refund_ok_client, "_reserve_info", _reserve_info_ok_with_trace)
    refund_ok = await refund_ok_client._refund_paid_reservation(reservation_id="PNR-1", user_id=user_id)  # noqa: SLF001
    assert refund_ok.ok is True
    assert refund_ok.data["refunded"] is True

    cancel_client = SRTClient(transport=_QueuedTransport([]))
    cancel_not_logged = await cancel_client.cancel(artifact_data={"reservation_id": "PNR-1"}, user_id="u2")
    assert cancel_not_logged.ok is False
    cancel_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    cancel_missing = await cancel_client.cancel(artifact_data={}, user_id=user_id)
    assert cancel_missing.ok is False

    async def _refund_ok(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=True, data={"refunded": True})

    monkeypatch.setattr(cancel_client, "_refund_paid_reservation", _refund_ok)
    cancel_paid = await cancel_client.cancel(
        artifact_data={"reservation_id": "PNR-1", "status": "paid"},
        user_id=user_id,
    )
    assert cancel_paid.ok is True

    cancel_server_error_client = SRTClient(transport=_QueuedTransport([(503, {})]))
    cancel_server_error_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    cancel_server_error = await cancel_server_error_client.cancel(
        artifact_data={"reservation_id": "PNR-1"},
        user_id=user_id,
    )
    assert cancel_server_error.ok is False
    assert cancel_server_error.error_code == "srt_server_error"

    cancel_invalid_json_client = SRTClient(transport=_QueuedTransport([(200, "{oops")]))
    cancel_invalid_json_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    cancel_invalid_json = await cancel_invalid_json_client.cancel(
        artifact_data={"reservation_id": "PNR-1"},
        user_id=user_id,
    )
    assert cancel_invalid_json.ok is False
    assert cancel_invalid_json.error_code == "invalid_json"

    cancel_fail_client = SRTClient(transport=_QueuedTransport([(200, {"resultMap": [{"strResult": "FAIL", "msgCd": "X"}]})]))
    cancel_fail_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    cancel_fail = await cancel_fail_client.cancel(
        artifact_data={"reservation_id": "PNR-1"},
        user_id=user_id,
    )
    assert cancel_fail.ok is False
    assert cancel_fail.error_code == "srt_cancel_fail_X"

    cancel_ok_client = SRTClient(transport=_QueuedTransport([(200, {"resultMap": [{"strResult": "SUCC"}]})]))
    cancel_ok_client._logged_in_user_ids.add(user_id)  # noqa: SLF001
    cancel_ok = await cancel_ok_client.cancel(
        artifact_data={"reservation_id": "PNR-1"},
        user_id=user_id,
    )
    assert cancel_ok.ok is True
    assert cancel_ok.data["cancelled"] is True


@pytest.mark.asyncio
async def test_srt_additional_branch_coverage_for_helpers_filters_and_standby(monkeypatch):
    parse_missing_fields = parse_srt_search_response(
        json.dumps(
            {
                "outDataSets": {
                    "dsOutput0": [{"strResult": "SUCC"}],
                    "dsOutput1": [{"stlbTrnClsfCd": "17", "dptDt": "20260223", "arvDt": "20260223", "arvTm": "113000"}],
                }
            }
        ),
        dep="수서",
        arr="부산",
    )
    assert parse_missing_fields.ok is True
    assert parse_missing_fields.data["schedules"] == []

    status_item = srt_module._extract_srt_status_item(  # noqa: SLF001
        {"outDataSets": {"dsOutput0": {"strResult": "FAIL", "msgCd": "E"}}}
    )
    assert status_item == {"strResult": "FAIL", "msgCd": "E"}

    helper_error_transport = srt_module.SRTNetFunnelHelper(_QueuedTransport([(500, {})]))
    status, key, nwait, ip = await helper_error_transport._make_request("getTidchkEnter")  # noqa: SLF001
    assert status == helper_error_transport.WAIT_STATUS_PASS
    assert key is None
    assert nwait is None
    assert ip is None

    helper_parse = srt_module.SRTNetFunnelHelper(_QueuedTransport([]))
    invalid_chunks = helper_parse._parse("NetFunnel.gControl.result='broken:format';")  # noqa: SLF001
    assert invalid_chunks["status"] == helper_parse.WAIT_STATUS_PASS

    helper_success_transport = srt_module.SRTNetFunnelHelper(
        _QueuedTransport(
            [
                (
                    200,
                    "NetFunnel.gControl.result='5002:201:key=KEY-2&nwait=1&ip=1.1.1.1';",
                )
            ]
        )
    )
    parsed_status, parsed_key, parsed_nwait, parsed_ip = await helper_success_transport._make_request("chkEnter")  # noqa: SLF001
    assert parsed_status == "201"
    assert parsed_key == "KEY-2"
    assert parsed_nwait == "1"
    assert parsed_ip == "1.1.1.1"

    async def _sleep_noop(_seconds: float):  # noqa: ANN001
        return None

    helper_run = srt_module.SRTNetFunnelHelper(_QueuedTransport([]))
    helper_run._cache_ttl = 0  # noqa: SLF001
    calls: list[str] = []

    async def _make_request(opcode: str, *, ip: str | None = None):  # noqa: ANN001
        _ = ip
        calls.append(opcode)
        if opcode == "getTidchkEnter":
            return helper_run.WAIT_STATUS_FAIL, "KEY-1", "3", "1.1.1.1"
        if opcode == "chkEnter":
            return helper_run.WAIT_STATUS_FAIL, "KEY-1", "1", "1.1.1.1"
        return "500", "KEY-1", None, None

    monkeypatch.setattr(srt_module.asyncio, "sleep", _sleep_noop)
    monkeypatch.setattr(helper_run, "_make_request", _make_request)
    run_key = await helper_run.run()
    assert run_key == "KEY-1"
    assert "setComplete" in calls

    login_client = SRTClient(transport=_QueuedTransport([]))
    missing_creds = await login_client.login(user_id="u1", credentials={"username": "", "password": ""})
    assert missing_creds.ok is False
    assert missing_creds.error_code == "not_configured"

    phone_login_client = SRTClient(transport=_QueuedTransport([(200, {"strResult": "SUCC", "RTNCD": "Y", "userMap": {}})]))
    phone_login = await phone_login_client.login(
        user_id="u1",
        credentials={"username": "010-1234-5678", "password": "pw"},
    )
    assert phone_login.ok is True
    assert phone_login_client._transport.requests[0]["data"]["srchDvCd"] == "3"  # noqa: SLF001
    assert phone_login_client._transport.requests[0]["data"]["srchDvNm"] == "01012345678"  # noqa: SLF001

    login_server_error_client = SRTClient(transport=_QueuedTransport([(503, {})]))
    login_server_error = await login_server_error_client.login(
        user_id="u1",
        credentials={"username": "user", "password": "pw"},
    )
    assert login_server_error.ok is False
    assert login_server_error.error_code == "srt_server_error"

    search_client = SRTClient(transport=_QueuedTransport([(200, {"resultMap": [{"strResult": "SUCC"}], "outDataSets": {"dsOutput1": []}})]))
    invalid_station = await search_client.search(
        dep="UNKNOWN",
        arr="부산",
        date_value=datetime(2026, 2, 23, tzinfo=KST).date(),
        time_window_start="00:00",
        time_window_end="23:59",
        user_id="u1",
    )
    assert invalid_station.ok is False
    assert invalid_station.error_code == "invalid_station"

    monkeypatch.setattr(srt_module, "now_kst", lambda: datetime(2026, 2, 23, 12, 30, tzinfo=KST))
    same_day_client = SRTClient(transport=_QueuedTransport([(200, {"resultMap": [{"strResult": "SUCC"}], "outDataSets": {"dsOutput1": []}})]))
    monkeypatch.setattr(same_day_client._netfunnel, "run", _netfunnel_key)  # noqa: SLF001
    same_day = await same_day_client.search(
        dep="수서",
        arr="부산",
        date_value=datetime(2026, 2, 23, tzinfo=KST).date(),
        time_window_start="00:00",
        time_window_end="23:59",
        user_id="u1",
    )
    assert same_day.ok is True
    assert same_day_client._transport.requests[0]["data"]["dptTm"] >= "123000"  # noqa: SLF001

    search_server_error_client = SRTClient(transport=_QueuedTransport([(503, {})]))
    monkeypatch.setattr(search_server_error_client._netfunnel, "run", _netfunnel_key)  # noqa: SLF001
    search_server_error = await search_server_error_client.search(
        dep="수서",
        arr="부산",
        date_value=datetime(2026, 2, 23, tzinfo=KST).date(),
        time_window_start="00:00",
        time_window_end="23:59",
        user_id="u1",
    )
    assert search_server_error.ok is False
    assert search_server_error.error_code == "srt_server_error"

    search_parse_fail_client = SRTClient(
        transport=_QueuedTransport([(200, {"outDataSets": {"dsOutput0": {"strResult": "FAIL", "msgCd": "NO"}}})])
    )
    monkeypatch.setattr(search_parse_fail_client._netfunnel, "run", _netfunnel_key)  # noqa: SLF001
    search_parse_fail = await search_parse_fail_client.search(
        dep="수서",
        arr="부산",
        date_value=datetime(2026, 2, 23, tzinfo=KST).date(),
        time_window_start="00:00",
        time_window_end="23:59",
        user_id="u1",
    )
    assert search_parse_fail.ok is False
    assert search_parse_fail.error_code == "srt_api_fail_NO"

    reserve_client = SRTClient(transport=_QueuedTransport([]))
    reserve_client._logged_in_user_ids.add("u1")  # noqa: SLF001
    reserve_client._schedule_cache["u1"] = {  # noqa: SLF001
        "pref-general": _schedule(general=False, special=True, reserve_wait_code="-1"),
        "pref-special": _schedule(general=True, special=False, reserve_wait_code="-1"),
    }

    captured: list[dict[str, Any]] = []

    async def _capture_reserve(**kwargs):  # noqa: ANN003
        captured.append(kwargs)
        return ProviderOutcome(ok=True, data={"reservation_id": f"PNR-{len(captured)}"})

    monkeypatch.setattr(reserve_client, "_reserve_request", _capture_reserve)
    preferred_general = await reserve_client.reserve(
        schedule_id="pref-general",
        seat_class="general_preferred",
        passengers={"adults": 1},
        user_id="u1",
    )
    assert preferred_general.ok is True
    assert captured[-1]["seat_class"] == "special"

    preferred_special = await reserve_client.reserve(
        schedule_id="pref-special",
        seat_class="special_preferred",
        passengers={"adults": 1},
        user_id="u1",
    )
    assert preferred_special.ok is True
    assert captured[-1]["seat_class"] == "general"

    standby_not_logged = await SRTClient(transport=_QueuedTransport([])).reserve_standby(
        schedule_id="missing",
        seat_class="general",
        passengers={"adults": 1},
        user_id="u2",
    )
    assert standby_not_logged.ok is False
    assert standby_not_logged.error_code == "not_logged_in"

    standby_missing_context_client = SRTClient(transport=_QueuedTransport([]))
    standby_missing_context_client._logged_in_user_ids.add("u1")  # noqa: SLF001
    standby_missing_context = await standby_missing_context_client.reserve_standby(
        schedule_id="missing",
        seat_class="general",
        passengers={"adults": 1},
        user_id="u1",
    )
    assert standby_missing_context.ok is False
    assert standby_missing_context.error_code == "schedule_context_missing"

    standby_failure_client = SRTClient(transport=_QueuedTransport([]))
    standby_failure_client._logged_in_user_ids.add("u1")  # noqa: SLF001
    standby_failure_client._schedule_cache["u1"] = {"sched": _schedule()}  # noqa: SLF001

    async def _reserve_failure(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=False, retryable=True, error_code="reserve_failed")

    monkeypatch.setattr(standby_failure_client, "_reserve_request", _reserve_failure)
    standby_failure = await standby_failure_client.reserve_standby(
        schedule_id="sched",
        seat_class="general",
        passengers={"adults": 1},
        user_id="u1",
    )
    assert standby_failure.ok is False
    assert standby_failure.error_code == "reserve_failed"

    pay_status_rows_dict_client = SRTClient(
        transport=_QueuedTransport([(200, {"outDataSets": {"dsOutput0": {"strResult": "FAIL", "msgCd": "P"}}})])
    )
    pay_status_rows_dict_client._logged_in_user_ids.add("u1")  # noqa: SLF001
    pay_status_rows_dict_client._membership_number_by_user_id["u1"] = "M-1"  # noqa: SLF001
    async def _reservation_unpaid_min(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=True, data={"reservations": [{"paid": False, "seat_count": 1, "total_cost": 1000}]})

    monkeypatch.setattr(pay_status_rows_dict_client, "get_reservations", _reservation_unpaid_min)
    pay_status_rows_dict = await pay_status_rows_dict_client.pay(
        reservation_id="PNR-1",
        user_id="u1",
        payment_card={
            "card_number": "4111111111111111",
            "card_password": "12",
            "validation_number": "900101",
            "card_expire": "2501",
        },
    )
    assert pay_status_rows_dict.ok is False
    assert pay_status_rows_dict.error_code == "srt_payment_fail_P"

    reservations_filter_client = SRTClient(
        transport=_QueuedTransport(
            [
                (
                    200,
                    {
                        "resultMap": [{"strResult": "SUCC"}],
                        "trainListMap": [{"pnrNo": ""}, {"pnrNo": "PNR-2"}],
                        "payListMap": [{"pnrNo": ""}, {"pnrNo": "PNR-2", "stlFlg": "N"}],
                    },
                )
            ]
        )
    )
    reservations_filter_client._logged_in_user_ids.add("u1")  # noqa: SLF001
    async def _ticket_info_empty(**_kwargs):  # noqa: ANN003
        return ProviderOutcome(ok=True, data={"tickets": []})

    monkeypatch.setattr(reservations_filter_client, "ticket_info", _ticket_info_empty)
    filtered = await reservations_filter_client.get_reservations(
        user_id="u1",
        reservation_id="OTHER",
        paid_only=True,
    )
    assert filtered.ok is True
    assert filtered.data["reservations"] == []

    paid_only_filter_client = SRTClient(
        transport=_QueuedTransport(
            [
                (
                    200,
                    {
                        "resultMap": [{"strResult": "SUCC"}],
                        "trainListMap": [{"pnrNo": "PNR-3"}],
                        "payListMap": [{"pnrNo": "PNR-3", "stlFlg": "N"}],
                    },
                )
            ]
        )
    )
    paid_only_filter_client._logged_in_user_ids.add("u1")  # noqa: SLF001
    monkeypatch.setattr(paid_only_filter_client, "ticket_info", _ticket_info_empty)
    paid_only_filtered = await paid_only_filter_client.get_reservations(
        user_id="u1",
        paid_only=True,
    )
    assert paid_only_filtered.ok is True
    assert paid_only_filtered.data["reservations"] == []

    ticket_generic_failure_client = SRTClient(
        transport=_QueuedTransport([(200, {"resultMap": [{"strResult": "FAIL", "msgCd": "GEN", "msgTxt": "generic"}]})])
    )
    ticket_generic_failure_client._logged_in_user_ids.add("u1")  # noqa: SLF001
    ticket_generic_failure = await ticket_generic_failure_client.ticket_info(
        reservation_id="PNR-1",
        user_id="u1",
    )
    assert ticket_generic_failure.ok is False
    assert ticket_generic_failure.error_code == "srt_ticket_info_fail_GEN"

    reserve_info_status_failure_client = SRTClient(
        transport=_QueuedTransport([(200, {"resultMap": [{"strResult": "FAIL", "msgCd": "GEN", "msgTxt": "generic"}]})])
    )
    reserve_info_status_failure = await reserve_info_status_failure_client._reserve_info(  # noqa: SLF001
        reservation_id="PNR-1",
        user_id="u1",
    )
    assert reserve_info_status_failure.ok is False
    assert reserve_info_status_failure.error_code == "srt_reserve_info_fail_GEN"

    reserve_info_message_only_client = SRTClient(transport=_QueuedTransport([(200, {"ErrorCode": "0", "ErrorMsg": "message-only"})]))
    reserve_info_message_only = await reserve_info_message_only_client._reserve_info(  # noqa: SLF001
        reservation_id="PNR-1",
        user_id="u1",
    )
    assert reserve_info_message_only.ok is False
    assert reserve_info_message_only.error_code == "srt_reserve_info_fail"
