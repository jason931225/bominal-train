from __future__ import annotations

import json
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from types import SimpleNamespace
from uuid import uuid4

import pytest

import app.modules.train.providers.ktx_client as ktx
import app.modules.train.providers.srt_client as srt
from app.modules.train.providers.base import ProviderOutcome
from app.modules.train.providers.ktx_client import KTXClient
from app.modules.train.providers.srt_client import SRTNetFunnelHelper
from app.modules.train.providers.transport import TransportResponse
from app.modules.train.timezone import KST


@dataclass(slots=True)
class _QueuedResponse:
    status_code: int
    body: str


class _QueueTransport:
    def __init__(self, responses: list[_QueuedResponse]) -> None:
        self._responses = list(responses)
        self.requests: list[dict] = []

    async def request(self, **kwargs) -> TransportResponse:
        self.requests.append(kwargs)
        if not self._responses:
            raise AssertionError("No queued transport response available")
        response = self._responses.pop(0)
        return TransportResponse(status_code=response.status_code, text=response.body, headers={})


def _payload_response(payload: dict, *, status_code: int = 200) -> _QueuedResponse:
    return _QueuedResponse(status_code=status_code, body=json.dumps(payload))


def test_srt_helper_primitives_and_ticket_mapping():
    assert srt._as_list([1, 2]) == [1, 2]
    assert srt._as_list({"a": 1}) == [{"a": 1}]
    assert srt._as_list("x") == []

    assert srt._to_int("12") == 12
    assert srt._to_int("bad", default=7) == 7

    parsed = srt._parse_srt_datetime("20260222", "123406")
    assert parsed is not None
    assert parsed.tzinfo == KST
    assert srt._parse_srt_datetime("2026", "12") is None

    assert srt._hhmmss_from_iso("2026-02-22T12:34:56+09:00") == "123456"
    assert srt._hhmmss_from_iso("not-iso") == ""

    stable_a = srt._stable_schedule_id("SRT", "수서", "부산", parsed, "381")
    stable_b = srt._stable_schedule_id("SRT", "수서", "부산", parsed, "381")
    assert stable_a == stable_b
    assert len(stable_a) == 24

    parsed_yyyymm = srt._parse_datetime_yyyymmdd_hhmmss("20260222", "123456")
    assert parsed_yyyymm.tzinfo == KST

    ticket = srt._build_srt_ticket_dict(
        {
            "scarNo": "3",
            "seatNo": "",
            "psrmClCd": "2",
            "psgTpCd": "4",
            "dcntKndCd": "000",
            "rcvdAmt": "10000",
            "stdrPrc": "12000",
            "dcntPrc": "2000",
        }
    )
    assert ticket["seat_class_name"] == "특실"
    assert ticket["passenger_type_name"] == "경로"
    assert ticket["waiting"] is True


def test_srt_status_and_reservation_helpers_cover_fallback_paths():
    result_map_list = {"resultMap": [{"strResult": "FAIL", "msgCd": "A01", "msgTxt": "bad"}]}
    out_sets = {"outDataSets": {"dsOutput0": [{"strResult": "FAIL", "msgCd": "B01", "msgTxt": "bad2"}]}}
    assert srt._extract_srt_status_item(result_map_list)["msgCd"] == "A01"
    assert srt._extract_srt_status_item({"resultMap": {"strResult": "FAIL", "msgCd": "A02"}})["msgCd"] == "A02"
    assert srt._extract_srt_status_item(out_sets)["msgCd"] == "B01"
    assert srt._extract_srt_status_item({"resultMap": []}) is None

    failure = srt._srt_status_failure(
        {"resultMap": [{"strResult": "FAIL", "msgCd": "E01", "msgTxt": "denied"}]},
        error_prefix="srt_fail",
        default_message="failed",
    )
    assert isinstance(failure, ProviderOutcome)
    assert failure.error_code == "srt_fail_E01"

    assert (
        srt._extract_srt_reservation_id({"reservListMap": [{"pnrNo": "PNR-1"}]})
        == "PNR-1"
    )
    assert (
        srt._extract_srt_reservation_id({"outDataSets": {"dsOutput1": [{"pnrNo": "PNR-2"}]}})
        == "PNR-2"
    )
    assert srt._extract_srt_reservation_id({"outDataSets": {"dsOutput2": [{"pnrNo": "PNR-3"}]}}) == "PNR-3"
    assert srt._extract_srt_reservation_id({}) is None

    payload = srt._build_srt_passenger_payload(adults=1, children=2, special_seat=True)
    assert payload["totPrnb"] == "3"
    assert payload["psgGridcnt"] == "2"
    assert payload["psgTpCd1"] == "1"
    assert payload["psgTpCd2"] == "5"
    assert payload["psrmClCd1"] == "2"
    assert srt._build_srt_passenger_payload(adults=0, children=0, special_seat=False) == {}

    assert srt._seat_class_is_special("special") is True
    assert srt._seat_class_is_special("general") is False
    assert srt._standby_available_from_code("019") is True
    assert srt._standby_available_from_code("000") is False

    unpaid_row = {"stlFlg": "N", "iseLmtDt": "20260222", "iseLmtTm": "132906"}
    paid_row = {"stlFlg": "Y", "iseLmtDt": "20260222", "iseLmtTm": "132906"}
    now = datetime(2026, 2, 22, 13, 33, 26, tzinfo=KST)
    assert srt._reservation_paid(unpaid_row) is False
    assert srt._reservation_paid(paid_row) is True
    assert srt._reservation_payment_deadline(unpaid_row) == datetime(2026, 2, 22, 13, 29, 6, tzinfo=KST)
    assert srt._reservation_expired_unpaid(unpaid_row, now=now) is True
    assert srt._reservation_expired_unpaid(paid_row, now=now) is False
    assert srt._reservation_expired_unpaid({"stlFlg": "N"}, now=now) is False

    assert srt._is_srt_reservation_not_found_message("조회자료가 없습니다.") is True
    assert srt._is_srt_reservation_not_found_message("Reservation not found") is True
    assert srt._is_srt_reservation_not_found_message("ok") is False


def test_srt_parse_functions_cover_invalid_json_and_fail_status():
    invalid = srt.parse_srt_search_response("not-json", dep="수서", arr="부산")
    assert invalid.ok is False
    assert invalid.retryable is True

    failed = srt.parse_srt_search_response(
        json.dumps(
            {"outDataSets": {"dsOutput0": [{"strResult": "FAIL", "msgCd": "E01", "msgTxt": "failed"}]}}
        ),
        dep="수서",
        arr="부산",
    )
    assert failed.ok is False
    assert failed.error_code == "srt_api_fail_E01"

    mixed = srt.parse_srt_search_response(
        json.dumps(
            {
                "outDataSets": {
                    "dsOutput0": [{"strResult": "SUCC"}],
                    "dsOutput1": [
                        {
                            "stlbTrnClsfCd": "00",
                            "trnNo": "KTX-1",
                            "dptDt": "20260223",
                            "dptTm": "080000",
                            "arvDt": "20260223",
                            "arvTm": "100000",
                            "gnrmRsvPsbStr": "예약가능",
                            "sprmRsvPsbStr": "매진",
                        },
                        {
                            "stlbTrnClsfCd": "17",
                            "trnNo": "381",
                            "dptDt": "20260223",
                            "dptTm": "080000",
                            "arvDt": "20260223",
                            "arvTm": "100000",
                            "gnrmRsvPsbStr": "예약가능",
                            "sprmRsvPsbStr": "매진",
                        },
                    ],
                }
            }
        ),
        dep="수서",
        arr="부산",
    )
    assert mixed.ok is True
    assert len(mixed.data["schedules"]) == 1
    assert mixed.data["schedules"][0].provider == "SRT"

    login_invalid_json = srt.parse_srt_login_response("not-json")
    assert login_invalid_json.ok is False
    assert login_invalid_json.error_code == "invalid_json"

    login_ip_block = srt.parse_srt_login_response("Your IP Address Blocked")
    assert login_ip_block.error_code == "ip_blocked"

    login_fail = srt.parse_srt_login_response(json.dumps({"strResult": "FAIL", "RTNCD": "N", "MSG": "실패"}))
    assert login_fail.ok is False
    assert login_fail.error_code == "login_failed"


@pytest.mark.asyncio
async def test_srt_netfunnel_parse_build_params_and_run_cache(monkeypatch):
    transport = _QueueTransport([])
    helper = SRTNetFunnelHelper(transport=transport)
    helper._cached_key = "CACHE-KEY"

    parsed = helper._parse(
        "NetFunnel.gControl.result='5002:201:key=NEWKEY&nwait=2&ip=127.0.0.1';"
    )
    assert parsed["status"] == "201"
    assert parsed["key"] == "NEWKEY"
    assert parsed["ip"] == "127.0.0.1"
    assert helper._parse("invalid")["status"] == helper.WAIT_STATUS_PASS

    params_get = helper._build_params("getTidchkEnter")
    assert params_get["opcode"] == helper.OP_CODE["getTidchkEnter"]
    assert params_get["sid"] == "service_1"
    assert params_get["aid"] == "act_10"

    params_chk = helper._build_params("chkEnter")
    assert params_chk["key"] == "CACHE-KEY"
    assert params_chk["ttl"] == "1"

    params_complete = helper._build_params("setComplete")
    assert params_complete["key"] == "CACHE-KEY"

    # _make_request fallback on HTTP error.
    error_transport = _QueueTransport([_QueuedResponse(status_code=503, body="{}")])
    error_helper = SRTNetFunnelHelper(error_transport)
    error_helper._cached_key = "OLD"
    status, key, nwait, ip = await error_helper._make_request("getTidchkEnter")
    assert status == error_helper.WAIT_STATUS_PASS
    assert key == "OLD"
    assert nwait is None
    assert ip is None

    # run() wait-loop path with cache update + complete.
    calls = [
        ("201", "KEY-1", "3", "1.1.1.1"),
        ("200", "KEY-2", "0", "1.1.1.1"),
        ("200", "KEY-2", None, "1.1.1.1"),
    ]

    async def _fake_make_request(opcode: str, *, ip: str | None = None):  # noqa: ARG001
        return calls.pop(0)

    sleep_calls: list[float] = []

    async def _fake_sleep(delay: float):
        sleep_calls.append(delay)

    monotonic_values = [100.0, 120.0, 121.0, 121.5, 121.8]
    monotonic_index = {"value": 0}

    def _fake_monotonic() -> float:
        idx = monotonic_index["value"]
        monotonic_index["value"] = idx + 1
        if idx >= len(monotonic_values):
            return monotonic_values[-1]
        return monotonic_values[idx]

    monkeypatch.setattr(srt.time, "monotonic", _fake_monotonic)
    monkeypatch.setattr(srt.asyncio, "sleep", _fake_sleep)
    monkeypatch.setattr(helper, "_make_request", _fake_make_request)

    key = await helper.run()
    assert key == "KEY-2"
    assert helper._cached_key == "KEY-2"
    assert sleep_calls == [1.0]

    # Cache hit should return without calling _make_request again.
    monkeypatch.setattr(helper, "_make_request", lambda *args, **kwargs: (_ for _ in ()).throw(AssertionError("no call")))
    cached_key = await helper.run()
    assert cached_key == "KEY-2"


def test_ktx_helper_primitives_parse_and_payload_builders():
    assert ktx._as_list([1]) == [1]
    assert ktx._as_list({"a": 1}) == [{"a": 1}]
    assert ktx._as_list("x") == []

    assert ktx._to_int("7") == 7
    assert ktx._to_int("bad", default=3) == 3

    parsed = ktx._parse_ktx_datetime("20260222", "123406")
    assert parsed is not None
    assert parsed.tzinfo == KST
    assert ktx._parse_ktx_datetime("2026", "12") is None

    stable_a = ktx._stable_schedule_id("KTX", "수서", "부산", parsed, "123")
    stable_b = ktx._stable_schedule_id("KTX", "수서", "부산", parsed, "123")
    assert stable_a == stable_b
    assert len(stable_a) == 24

    parsed_yyyymm = ktx._parse_datetime_yyyymmdd_hhmmss("20260223", "081000")
    assert parsed_yyyymm.tzinfo == KST

    failure = ktx._ktx_failure(
        {"strResult": "FAIL", "h_msg_cd": "E01", "h_msg_txt": "failed"},
        error_prefix="ktx_fail",
        default_message="nope",
    )
    assert failure.error_code == "ktx_fail_E01"
    assert ktx._ktx_failure({"strResult": "SUCC"}, error_prefix="x", default_message="y") is None

    payload = ktx._build_ktx_passenger_payload(adults=1, children=2)
    assert payload["txtPsgTpCd1"] == "1"
    assert payload["txtPsgTpCd2"] == "3"
    assert payload["txtCompaCnt2"] == "2"

    assert ktx._seat_class_is_special("special_preferred") is True
    assert ktx._seat_class_is_special("general") is False

    ticket = ktx._build_ktx_ticket_dict(
        {
            "h_srcar_no": "4",
            "h_seat_no": "",
            "h_seat_cnt": "2",
            "h_psrm_cl_nm": "특실",
            "h_psg_tp_dv_nm": "어른",
            "h_rcvd_amt": "12000",
            "h_seat_prc": "14000",
            "h_dcnt_amt": "2000",
        }
    )
    assert ticket["car_no"] == "4"
    assert ticket["seat_count"] == 2
    assert ticket["waiting"] is True


def test_ktx_parse_functions_cover_failure_and_success_paths():
    invalid = ktx.parse_ktx_search_response("not-json", dep="수서", arr="부산")
    assert invalid.ok is False
    assert invalid.error_code == "invalid_json"

    failed = ktx.parse_ktx_search_response(
        json.dumps({"strResult": "FAIL", "h_msg_cd": "E01", "h_msg_txt": "failed"}),
        dep="수서",
        arr="부산",
    )
    assert failed.ok is False
    assert failed.error_code == "ktx_api_fail_E01"

    parsed = ktx.parse_ktx_search_response(
        json.dumps(
            {
                "strResult": "SUCC",
                "trn_infos": {
                    "trn_info": [
                        {
                            "h_trn_no": "100",
                            "h_dpt_dt": "20260223",
                            "h_dpt_tm": "081000",
                            "h_arv_dt": "20260223",
                            "h_arv_tm": "100000",
                            "h_gen_rsv_cd": "11",
                            "h_spe_rsv_cd": "00",
                        },
                        {"h_trn_no": "missing-fields"},
                    ]
                },
            }
        ),
        dep="수서",
        arr="부산",
    )
    assert parsed.ok is True
    assert len(parsed.data["schedules"]) == 1

    login_invalid_json = ktx.parse_ktx_login_response("not-json")
    assert login_invalid_json.ok is False
    assert login_invalid_json.error_code == "invalid_json"

    login_failed = ktx.parse_ktx_login_response(json.dumps({"strResult": "FAIL", "h_msg_cd": "A01"}))
    assert login_failed.error_code == "ktx_login_fail_A01"

    login_missing_membership = ktx.parse_ktx_login_response(json.dumps({"strResult": "SUCC"}))
    assert login_missing_membership.error_code == "ktx_login_no_membership"

    login_ok = ktx.parse_ktx_login_response(
        json.dumps({"strResult": "SUCC", "strMbCrdNo": "1234", "strCustNm": "Tester"})
    )
    assert login_ok.ok is True
    assert login_ok.data["membership_number"] == "1234"


@pytest.mark.asyncio
async def test_ktx_login_bootstrap_and_login_edge_paths():
    user_id = str(uuid4())

    missing = await KTXClient(transport=_QueueTransport([])).login(
        user_id=user_id,
        credentials={"username": "", "password": ""},
    )
    assert missing.error_code == "not_configured"

    server_error = await KTXClient(transport=_QueueTransport([_QueuedResponse(503, "{}")])).login(
        user_id=user_id,
        credentials={"username": "user@example.com", "password": "pw"},
    )
    assert server_error.error_code == "ktx_server_error"

    bootstrap_invalid_json = await KTXClient(
        transport=_QueueTransport([_QueuedResponse(200, "not-json")])
    ).login(
        user_id=user_id,
        credentials={"username": "user@example.com", "password": "pw"},
    )
    assert bootstrap_invalid_json.error_code == "ktx_login_bootstrap_invalid_json"

    bootstrap_failed = await KTXClient(
        transport=_QueueTransport([_payload_response({"strResult": "FAIL", "h_msg_txt": "nope"})])
    ).login(
        user_id=user_id,
        credentials={"username": "user@example.com", "password": "pw"},
    )
    assert bootstrap_failed.error_code == "ktx_login_bootstrap_failed"

    bootstrap_missing_keys = await KTXClient(
        transport=_QueueTransport([_payload_response({"strResult": "SUCC", "app.login.cphd": {"idx": "1"}})])
    ).login(
        user_id=user_id,
        credentials={"username": "user@example.com", "password": "pw"},
    )
    assert bootstrap_missing_keys.error_code == "ktx_login_bootstrap_missing_keys"

    login_server_error = await KTXClient(
        transport=_QueueTransport(
            [
                _payload_response(
                    {
                        "strResult": "SUCC",
                        "app.login.cphd": {"idx": "1", "key": "1234567890abcdef"},
                    }
                ),
                _QueuedResponse(500, "{}"),
            ]
        )
    ).login(
        user_id=user_id,
        credentials={"username": "user@example.com", "password": "pw"},
    )
    assert login_server_error.error_code == "ktx_server_error"

    transport = _QueueTransport(
        [
            _payload_response(
                {
                    "strResult": "SUCC",
                    "app.login.cphd": {"idx": "1", "key": "1234567890abcdef"},
                }
            ),
            _payload_response(
                {
                    "strResult": "SUCC",
                    "strMbCrdNo": "MEM-1",
                    "strCustNm": "Tester",
                    "strCpNo": "01012341234",
                }
            ),
        ]
    )
    client = KTXClient(transport=transport)
    success = await client.login(
        user_id=user_id,
        credentials={"username": "user@example.com", "password": "pw"},
    )
    assert success.ok is True
    assert client._membership_number_by_user_id[user_id] == "MEM-1"
    assert user_id in client._logged_in_user_ids

    bootstrap_request = transport.requests[0]
    assert bootstrap_request["method"] == "GET"
    assert bootstrap_request["url"].endswith("common.code.do")
    assert bootstrap_request["params"]["code"] == "app.login.cphd"
    assert bootstrap_request["params"]["Device"] == "IP"
    assert bootstrap_request["params"]["Version"] == "250601002"
    assert bootstrap_request["params"]["srtCheckYn"] == "Y"

    login_request = transport.requests[1]
    assert login_request["method"] == "GET"
    assert login_request["url"].endswith("login.Login")
    assert login_request["params"]["txtInputFlg"] == "5"
    assert login_request["params"]["txtPwd"]
    assert login_request["params"]["txtDv"] == "2"
    assert login_request["params"]["checkValidPw"] == "Y"
    assert login_request["params"]["srtCheckYn"] == "Y"
    assert login_request["params"]["Device"] == "IP"
    assert login_request["params"]["Version"] == "250601002"
    assert login_request["params"]["Key"] == "korail1234567890"


def test_ktx_encrypt_password_changes_when_key_changes():
    client = KTXClient()
    password = "Test1234!"

    encrypted_with_key_a = client._encrypt_password(password, "2bed45a56c3c150dc86e218d0429e152")  # noqa: SLF001
    encrypted_with_key_b = client._encrypt_password(password, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")  # noqa: SLF001

    assert encrypted_with_key_a != encrypted_with_key_b


@pytest.mark.asyncio
async def test_ktx_login_honors_endpoint_overrides(monkeypatch):
    monkeypatch.setenv("TRAIN_KTX_ENDPOINT_CODE", "https://localhost:9443/bootstrap")
    monkeypatch.setenv("TRAIN_KTX_ENDPOINT_LOGIN", "https://localhost:9443/login")

    transport = _QueueTransport(
        [
            _payload_response(
                {
                    "strResult": "SUCC",
                    "app.login.cphd": {"idx": "1", "key": "1234567890abcdef"},
                }
            ),
            _payload_response({"strResult": "SUCC", "strMbCrdNo": "MEM-2"}),
        ]
    )
    client = KTXClient(transport=transport)
    outcome = await client.login(
        user_id=str(uuid4()),
        credentials={"username": "010-1111-2222", "password": "pw"},
    )

    assert outcome.ok is True
    assert transport.requests[0]["url"] == "https://localhost:9443/bootstrap"
    assert transport.requests[1]["url"] == "https://localhost:9443/login"


@pytest.mark.asyncio
async def test_ktx_login_honors_dynapath_header_override(monkeypatch):
    monkeypatch.setenv("TRAIN_KTX_AUTH_DYNAPATH_TOKEN", "token-for-local-debug")

    transport = _QueueTransport(
        [
            _payload_response(
                {
                    "strResult": "SUCC",
                    "app.login.cphd": {"idx": "1", "key": "1234567890abcdef"},
                }
            ),
            _payload_response({"strResult": "SUCC", "strMbCrdNo": "MEM-3"}),
        ]
    )

    outcome = await KTXClient(transport=transport).login(
        user_id=str(uuid4()),
        credentials={"username": "010-1111-2222", "password": "pw"},
    )

    assert outcome.ok is True
    assert transport.requests[0]["headers"]["x-dynapath-m-token"] == "token-for-local-debug"
    assert transport.requests[1]["headers"]["x-dynapath-m-token"] == "token-for-local-debug"
