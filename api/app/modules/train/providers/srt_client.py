from __future__ import annotations

import asyncio
from collections import defaultdict
import hashlib
import json
import re
import time
from datetime import date, datetime
from typing import Any

from app.modules.train.providers.base import ProviderOutcome, ProviderSchedule
from app.modules.train.providers.transport import AsyncTransport, HttpxTransport
from app.modules.train.stations import SRT_STATION_CODE, station_code_for_name
from app.modules.train.timezone import KST, now_kst

SRT_MOBILE = "https://app.srail.or.kr:443"
SRT_API_ENDPOINTS = {
    "login": f"{SRT_MOBILE}/apb/selectListApb01080_n.do",
    "search_schedule": f"{SRT_MOBILE}/ara/selectListAra10007_n.do",
    "reserve": f"{SRT_MOBILE}/arc/selectListArc05013_n.do",
    "tickets": f"{SRT_MOBILE}/atc/selectListAtc14016_n.do",
    "ticket_info": f"{SRT_MOBILE}/ard/selectListArd02019_n.do",
    "payment": f"{SRT_MOBILE}/ata/selectListAta09036_n.do",
    "cancel": f"{SRT_MOBILE}/ard/selectListArd02045_n.do",
    "standby_option": f"{SRT_MOBILE}/ata/selectListAta01135_n.do",
    "reserve_info": f"{SRT_MOBILE}/atc/getListAtc14087.do",
    "reserve_info_referer": f"{SRT_MOBILE}/common/ATC/ATC0201L/view.do?pnrNo=",
    "refund": f"{SRT_MOBILE}/atc/selectListAtc02063_n.do",
}

SRT_DEFAULT_HEADERS = {
    "User-Agent": (
        "Mozilla/5.0 (Linux; Android 15; SM-S912N Build/AP3A.240905.015.A2; wv) "
        "AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/136.0.7103.125 "
        "Mobile Safari/537.36SRT-APP-Android V.2.0.38"
    ),
    "Accept": "application/json",
    "Referer": "https://app.srail.or.kr/main/main.do",
    "X-Requested-With": "kr.co.srail.newapp",
    "Accept-Language": "ko-KR,ko;q=0.9,en-US;q=0.8,en;q=0.7",
}

SRT_NETFUNNEL_HEADERS = {
    "Host": "nf.letskorail.com",
    "Connection": "keep-alive",
    "Pragma": "no-cache",
    "Cache-Control": "no-cache",
    "sec-ch-ua-platform": "Android",
    "User-Agent": SRT_DEFAULT_HEADERS["User-Agent"],
    "sec-ch-ua": '"Chromium";v="136", "Android WebView";v="136", "Not=A/Brand";v="99"',
    "sec-ch-ua-mobile": "?1",
    "Accept": "*/*",
    "X-Requested-With": "kr.co.srail.newapp",
    "Sec-Fetch-Site": "cross-site",
    "Sec-Fetch-Mode": "no-cors",
    "Sec-Fetch-Dest": "script",
    "Sec-Fetch-Storage-Access": "active",
    "Referer": "https://app.srail.or.kr/",
    "Accept-Encoding": "gzip, deflate, br, zstd",
    "Accept-Language": "ko-KR,ko;q=0.9,en-US;q=0.8,en;q=0.7",
}

EMAIL_REGEX = re.compile(r"[^@]+@[^@]+\.[^@]+")
# Korean mobile numbers: 01X-XXXX-XXXX (accepts with or without dashes)
PHONE_NUMBER_REGEX = re.compile(r"01[0-9]-?\d{3,4}-?\d{4}")
SRT_STATION_NAME_BY_CODE = {code: name for name, code in SRT_STATION_CODE.items()}
SRT_PASSENGER_TYPE_BY_CODE = {
    "1": "어른/청소년",
    "2": "장애 1~3급",
    "3": "장애 4~6급",
    "4": "경로",
    "5": "어린이",
}


def _as_list(value: Any) -> list[Any]:
    if isinstance(value, list):
        return value
    if isinstance(value, dict):
        return [value]
    return []


def _to_int(value: Any, default: int = 0) -> int:
    try:
        return int(str(value))
    except (TypeError, ValueError):
        return default


def _parse_srt_datetime(date_value: str | None, time_value: str | None) -> datetime | None:
    if not date_value or not time_value:
        return None
    if len(str(date_value)) < 8 or len(str(time_value)) < 4:
        return None
    try:
        dt = datetime.strptime(f"{date_value}{str(time_value)[:6]}", "%Y%m%d%H%M%S")
    except ValueError:
        return None
    return dt.replace(tzinfo=KST)


def _hhmmss_from_iso(value: str | None) -> str:
    if not value:
        return ""
    try:
        parsed = datetime.fromisoformat(value)
    except ValueError:
        return ""
    return parsed.strftime("%H%M%S")


def _stable_schedule_id(provider: str, dep: str, arr: str, departure_at: datetime, train_no: str) -> str:
    raw = f"{provider}|{dep}|{arr}|{departure_at.isoformat()}|{train_no}"
    return hashlib.sha256(raw.encode("utf-8")).hexdigest()[:24]


def _parse_datetime_yyyymmdd_hhmmss(date_value: str, time_value: str) -> datetime:
    dt = datetime.strptime(f"{date_value}{time_value[:6]}", "%Y%m%d%H%M%S")
    return dt.replace(tzinfo=KST)


def parse_srt_search_response(response_text: str, dep: str, arr: str) -> ProviderOutcome:
    try:
        payload = json.loads(response_text)
    except json.JSONDecodeError:
        return ProviderOutcome(
            ok=False,
            retryable=True,
            error_code="invalid_json",
            error_message_safe="SRT response was not valid JSON",
        )

    status_items = payload.get("outDataSets", {}).get("dsOutput0", [])
    if isinstance(status_items, dict):
        status_items = [status_items]
    if status_items:
        status_item = status_items[0]
        if status_item.get("strResult") == "FAIL":
            raw_code = status_item.get("msgCd")
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code=f"srt_api_fail_{raw_code}" if raw_code else "srt_api_fail",
                error_message_safe=status_item.get("msgTxt", "SRT API returned failure"),
            )

    rows = payload.get("outDataSets", {}).get("dsOutput1", [])
    if isinstance(rows, dict):
        rows = [rows]
    if not isinstance(rows, list):
        rows = []
    schedules: list[ProviderSchedule] = []

    for row in rows:
        if row.get("stlbTrnClsfCd") != "17":
            continue

        dep_date = row.get("dptDt")
        dep_time = row.get("dptTm")
        arr_date = row.get("arvDt")
        arr_time = row.get("arvTm")
        train_no = row.get("trnNo", "")
        if not dep_date or not dep_time or not arr_date or not arr_time:
            continue

        departure_at = _parse_datetime_yyyymmdd_hhmmss(dep_date, dep_time)
        arrival_at = _parse_datetime_yyyymmdd_hhmmss(arr_date, arr_time)
        schedule_id = _stable_schedule_id("SRT", dep, arr, departure_at, train_no)

        schedules.append(
            ProviderSchedule(
                schedule_id=schedule_id,
                provider="SRT",
                dep=dep,
                arr=arr,
                departure_at=departure_at,
                arrival_at=arrival_at,
                train_no=train_no,
                availability={
                    "general": "예약가능" in str(row.get("gnrmRsvPsbStr", "")),
                    "special": "예약가능" in str(row.get("sprmRsvPsbStr", "")),
                },
                metadata={
                    "reserve_wait_code": row.get("rsvWaitPsbCd"),
                    "reserve_wait_name": row.get("rsvWaitPsbCdNm"),
                    "train_code": row.get("stlbTrnClsfCd"),
                    "dep_station_code": row.get("dptRsStnCd"),
                    "arr_station_code": row.get("arvRsStnCd"),
                    "dep_station_run_order": row.get("dptStnRunOrdr"),
                    "arr_station_run_order": row.get("arvStnRunOrdr"),
                    "dep_station_constitution_order": row.get("dptStnConsOrdr"),
                    "arr_station_constitution_order": row.get("arvStnConsOrdr"),
                    "dep_date": row.get("dptDt"),
                    "dep_time": row.get("dptTm"),
                    "arr_date": row.get("arvDt"),
                    "arr_time": row.get("arvTm"),
                },
            )
        )

    return ProviderOutcome(ok=True, data={"schedules": schedules})


def parse_srt_login_response(response_text: str) -> ProviderOutcome:
    if "존재하지않는 회원입니다" in response_text or "비밀번호 오류" in response_text:
        return ProviderOutcome(
            ok=False,
            retryable=False,
            error_code="invalid_credentials",
            error_message_safe="SRT login failed. Check your username or password.",
        )
    if "Your IP Address Blocked" in response_text:
        return ProviderOutcome(
            ok=False,
            retryable=False,
            error_code="ip_blocked",
            error_message_safe="SRT blocked this IP address.",
        )

    try:
        payload = json.loads(response_text)
    except json.JSONDecodeError:
        return ProviderOutcome(
            ok=False,
            retryable=True,
            error_code="invalid_json",
            error_message_safe="SRT login response was not valid JSON",
        )

    msg_text = str(payload.get("MSG") or "")
    str_result = str(payload.get("strResult") or "")
    rtn_code = str(payload.get("RTNCD") or "")
    if "존재하지않는 회원입니다" in msg_text or "비밀번호 오류" in msg_text:
        return ProviderOutcome(
            ok=False,
            retryable=False,
            error_code="invalid_credentials",
            error_message_safe="SRT login failed. Check your username or password.",
        )
    if str_result.upper() == "FAIL" and rtn_code.upper() == "N":
        return ProviderOutcome(
            ok=False,
            retryable=False,
            error_code="login_failed",
            error_message_safe=msg_text or "SRT login failed",
        )

    user_map = payload.get("userMap")
    if not isinstance(user_map, dict):
        msg = msg_text or "SRT login failed"
        return ProviderOutcome(
            ok=False,
            retryable=False,
            error_code="login_failed",
            error_message_safe=str(msg),
        )

    return ProviderOutcome(
        ok=True,
        data={
            "membership_number": user_map.get("MB_CRD_NO"),
            "membership_name": user_map.get("CUST_NM"),
            "phone_number": user_map.get("MBL_PHONE"),
        },
    )


def _extract_srt_status_item(payload: dict[str, Any]) -> dict[str, Any] | None:
    result_map = payload.get("resultMap")
    if isinstance(result_map, list) and result_map:
        first = result_map[0]
        if isinstance(first, dict):
            return first
    if isinstance(result_map, dict):
        return result_map

    status_items = payload.get("outDataSets", {}).get("dsOutput0", [])
    if isinstance(status_items, dict):
        return status_items
    if isinstance(status_items, list) and status_items:
        first = status_items[0]
        if isinstance(first, dict):
            return first
    return None


def _srt_status_failure(
    payload: dict[str, Any],
    *,
    error_prefix: str,
    default_message: str,
) -> ProviderOutcome | None:
    status_item = _extract_srt_status_item(payload)
    if status_item and status_item.get("strResult") == "FAIL":
        raw_code = status_item.get("msgCd")
        return ProviderOutcome(
            ok=False,
            retryable=False,
            error_code=f"{error_prefix}_{raw_code}" if raw_code else error_prefix,
            error_message_safe=str(status_item.get("msgTxt", default_message)),
        )
    return None


def _extract_srt_reservation_id(payload: dict[str, Any]) -> str | None:
    reserv_list = _as_list(payload.get("reservListMap"))
    if reserv_list:
        for row in reserv_list:
            if isinstance(row, dict):
                pnr = row.get("pnrNo")
                if pnr:
                    return str(pnr)

    for dataset_key in ("dsOutput1", "dsOutput2"):
        rows = _as_list(payload.get("outDataSets", {}).get(dataset_key))
        if rows:
            for row in rows:
                if isinstance(row, dict):
                    pnr = row.get("pnrNo")
                    if pnr:
                        return str(pnr)
    return None


def _build_srt_passenger_payload(*, adults: int, children: int, special_seat: bool) -> dict[str, str]:
    total = adults + children
    if total <= 0:
        return {}

    rows: list[tuple[str, int]] = []
    if adults > 0:
        rows.append(("1", adults))
    if children > 0:
        rows.append(("5", children))

    payload: dict[str, str] = {
        "totPrnb": str(total),
        "psgGridcnt": str(len(rows)),
    }
    for idx, (passenger_type, count) in enumerate(rows, start=1):
        payload[f"psgTpCd{idx}"] = passenger_type
        payload[f"psgInfoPerPrnb{idx}"] = str(count)
        payload[f"locSeatAttCd{idx}"] = "000"
        payload[f"rqSeatAttCd{idx}"] = "015"
        payload[f"dirSeatAttCd{idx}"] = "009"
        payload[f"smkSeatAttCd{idx}"] = "000"
        payload[f"etcSeatAttCd{idx}"] = "000"
        payload[f"psrmClCd{idx}"] = "2" if special_seat else "1"
    return payload


def _seat_class_is_special(seat_class: str) -> bool:
    return seat_class in {"special", "special_preferred"}


def _standby_available_from_code(value: Any) -> bool:
    # srtgo parity: standby is available when wait code contains "9".
    return "9" in str(value or "")


def _reservation_paid(pay_row: dict[str, Any]) -> bool:
    return str(pay_row.get("stlFlg") or "").upper() == "Y"


def _reservation_payment_deadline(pay_row: dict[str, Any]) -> datetime | None:
    payment_date = str(pay_row.get("iseLmtDt") or "")
    payment_time = str(pay_row.get("iseLmtTm") or "")
    return _parse_srt_datetime(payment_date, payment_time)


def _reservation_expired_unpaid(
    pay_row: dict[str, Any],
    *,
    now: datetime | None = None,
) -> bool:
    # Primary expiry signal: unpaid + now(KST) > payment cutoff.
    if _reservation_paid(pay_row):
        return False
    payment_deadline_at = _reservation_payment_deadline(pay_row)
    if payment_deadline_at is None:
        return False
    current = now if now is not None else now_kst()
    return current > payment_deadline_at


def _is_srt_reservation_not_found_message(message: str | None) -> bool:
    message_text = str(message or "")
    message_lower = message_text.lower()
    return (
        "조회자료가 없습니다" in message_text
        or "reservation not found" in message_lower
        or "ticket not found" in message_lower
        or "not found" in message_lower
    )


def _build_srt_ticket_dict(raw: dict[str, Any]) -> dict[str, Any]:
    seat_class_code = str(raw.get("psrmClCd") or "")
    passenger_type_code = str(raw.get("psgTpCd") or "")
    return {
        "car_no": raw.get("scarNo"),
        "seat_no": raw.get("seatNo"),
        "seat_class_code": seat_class_code,
        "seat_class_name": "특실" if seat_class_code == "2" else ("일반실" if seat_class_code == "1" else ""),
        "passenger_type_code": passenger_type_code or None,
        "passenger_type_name": SRT_PASSENGER_TYPE_BY_CODE.get(passenger_type_code),
        "discount_type_code": raw.get("dcntKndCd"),
        "price": _to_int(raw.get("rcvdAmt"), 0),
        "original_price": _to_int(raw.get("stdrPrc"), 0),
        "discount_amount": _to_int(raw.get("dcntPrc"), 0),
        "waiting": not bool(raw.get("seatNo")),
    }


class SRTNetFunnelHelper:
    WAIT_STATUS_PASS = "200"
    WAIT_STATUS_FAIL = "201"
    ALREADY_COMPLETED = "502"

    OP_CODE = {
        "getTidchkEnter": "5101",
        "chkEnter": "5002",
        "setComplete": "5004",
    }

    def __init__(self, transport: AsyncTransport):
        self._transport = transport
        self._cached_key: str | None = None
        self._last_fetch_time: float = 0.0
        self._cache_ttl = 48.0

    async def run(self) -> str | None:
        now = time.monotonic()
        if self._cached_key and (now - self._last_fetch_time) < self._cache_ttl:
            return self._cached_key

        status, key, _nwait, ip = await self._make_request("getTidchkEnter")
        if key:
            self._cached_key = key
            self._last_fetch_time = now

        loops = 0
        while status == self.WAIT_STATUS_FAIL and loops < 10:
            await asyncio.sleep(1.0)
            status, key, _nwait, ip = await self._make_request("chkEnter", ip=ip)
            if key:
                self._cached_key = key
                self._last_fetch_time = time.monotonic()
            loops += 1

        complete_status, _key, _nwait, _ip = await self._make_request("setComplete", ip=ip)
        if complete_status in {self.WAIT_STATUS_PASS, self.ALREADY_COMPLETED} and self._cached_key:
            return self._cached_key
        return self._cached_key

    async def _make_request(self, opcode: str, *, ip: str | None = None) -> tuple[str, str | None, str | None, str | None]:
        url = f"https://{ip or 'nf.letskorail.com'}/ts.wseq"
        response = await self._transport.request(
            method="GET",
            url=url,
            headers=SRT_NETFUNNEL_HEADERS,
            params=self._build_params(opcode),
            timeout=8.0,
        )
        if response.status_code >= 400:
            return self.WAIT_STATUS_PASS, self._cached_key, None, None
        parsed = self._parse(response.text)
        return (
            parsed.get("status", self.WAIT_STATUS_PASS),
            parsed.get("key"),
            parsed.get("nwait"),
            parsed.get("ip"),
        )

    def _build_params(self, opcode_name: str) -> dict[str, str]:
        opcode = self.OP_CODE[opcode_name]
        params: dict[str, str] = {
            "opcode": opcode,
            "nfid": "0",
            "prefix": f"NetFunnel.gRtype={opcode};",
            "js": "true",
            str(int(time.time() * 1000)): "",
        }
        if opcode_name in {"getTidchkEnter", "chkEnter"}:
            params.update({"sid": "service_1", "aid": "act_10"})
            if opcode_name == "chkEnter" and self._cached_key:
                params.update({"key": self._cached_key, "ttl": "1"})
        elif opcode_name == "setComplete" and self._cached_key:
            params["key"] = self._cached_key
        return params

    def _parse(self, response: str) -> dict[str, str]:
        result_match = re.search(r"NetFunnel\.gControl\.result='([^']+)'", response)
        if not result_match:
            return {"status": self.WAIT_STATUS_PASS}

        chunks = result_match.group(1).split(":", 2)
        if len(chunks) != 3:
            return {"status": self.WAIT_STATUS_PASS}
        _code, status, params_str = chunks
        params = dict(
            param.split("=", 1) for param in params_str.split("&") if "=" in param
        )
        params["status"] = status
        return params


class SRTClient:
    provider_name = "SRT"

    def __init__(self, transport: AsyncTransport | None = None):
        self._transport = transport or HttpxTransport()
        self._netfunnel = SRTNetFunnelHelper(self._transport)
        self._logged_in_user_ids: set[str] = set()
        self._membership_number_by_user_id: dict[str, str] = {}
        self._phone_number_by_user_id: dict[str, str] = {}
        self._schedule_cache: dict[str, dict[str, ProviderSchedule]] = defaultdict(dict)

    async def login(self, *, user_id: str, credentials: dict[str, str] | None = None) -> ProviderOutcome:
        username = (credentials or {}).get("username", "").strip()
        password = (credentials or {}).get("password", "")
        if not username or not password:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="not_configured",
                error_message_safe="SRT credentials are required",
            )

        login_type = "2" if EMAIL_REGEX.match(username) else ("3" if PHONE_NUMBER_REGEX.match(username) else "1")
        if login_type == "3":
            username = re.sub("-", "", username)

        request_data = {
            "auto": "Y",
            "check": "Y",
            "page": "menu",
            "deviceKey": "-",
            "customerYn": "",
            "login_referer": f"{SRT_MOBILE}/main/main.do",
            "srchDvCd": login_type,
            "srchDvNm": username,
            "hmpgPwdCphd": password,
        }

        response = await self._transport.request(
            method="POST",
            url=SRT_API_ENDPOINTS["login"],
            headers=SRT_DEFAULT_HEADERS,
            data=request_data,
            timeout=20.0,
        )
        if response.status_code >= 500:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="srt_server_error",
                error_message_safe="SRT server error during login",
            )

        outcome = parse_srt_login_response(response.text)
        if outcome.ok:
            outcome.data["user_id"] = user_id
            outcome.data["username"] = username
            self._logged_in_user_ids.add(user_id)
            membership_number = str(outcome.data.get("membership_number") or "").strip()
            if membership_number:
                self._membership_number_by_user_id[user_id] = membership_number
            phone_number = str(outcome.data.get("phone_number") or "").strip()
            if phone_number:
                self._phone_number_by_user_id[user_id] = phone_number
        return outcome

    async def search(
        self,
        *,
        dep: str,
        arr: str,
        date_value: date,
        time_window_start: str,
        time_window_end: str,
        user_id: str,
    ) -> ProviderOutcome:
        dep_code = station_code_for_name(dep)
        arr_code = station_code_for_name(arr)
        if dep_code is None or arr_code is None:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="invalid_station",
                error_message_safe="SRT requires supported station names",
                data={"dep": dep, "arr": arr},
            )

        date_yyyymmdd = date_value.strftime("%Y%m%d")
        time_hhmmss = time_window_start.replace(":", "") + "00"
        kst_now = now_kst()
        if date_value == kst_now.date():
            time_hhmmss = max(time_hhmmss, kst_now.strftime("%H%M%S"))
        netfunnel_key = await self._netfunnel.run()

        request_data = {
            "chtnDvCd": "1",
            "dptDt": date_yyyymmdd,
            "dptTm": time_hhmmss,
            "dptDt1": date_yyyymmdd,
            "dptTm1": f"{time_hhmmss[:2]}0000",
            "dptRsStnCd": dep_code,
            "arvRsStnCd": arr_code,
            "stlbTrnClsfCd": "05",
            "trnGpCd": 109,
            "trnNo": "",
            "psgNum": "1",
            "seatAttCd": "015",
            "arriveTime": "N",
            "tkDptDt": "",
            "tkDptTm": "",
            "tkTrnNo": "",
            "tkTripChgFlg": "",
            "dlayTnumAplFlg": "Y",
            "netfunnelKey": netfunnel_key or "",
        }

        response = await self._transport.request(
            method="POST",
            url=SRT_API_ENDPOINTS["search_schedule"],
            headers=SRT_DEFAULT_HEADERS,
            data=request_data,
            timeout=20.0,
        )

        if response.status_code >= 500:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="srt_server_error",
                error_message_safe="SRT server error during search",
            )

        parsed = parse_srt_search_response(response.text, dep=dep, arr=arr)
        if not parsed.ok:
            return parsed

        filtered = [
            schedule
            for schedule in parsed.data.get("schedules", [])
            if time_window_start <= schedule.departure_at.strftime("%H:%M") <= time_window_end
        ]
        self._schedule_cache[user_id] = {schedule.schedule_id: schedule for schedule in filtered}
        return ProviderOutcome(ok=True, data={"schedules": filtered})

    async def _reserve_request(
        self,
        *,
        schedule: ProviderSchedule,
        seat_class: str,
        passengers: dict[str, int],
        user_id: str,
        standby: bool,
    ) -> ProviderOutcome:
        metadata = schedule.metadata or {}
        required_values = {
            "train_code": metadata.get("train_code"),
            "dep_station_code": metadata.get("dep_station_code"),
            "arr_station_code": metadata.get("arr_station_code"),
            "dep_date": metadata.get("dep_date"),
            "dep_time": metadata.get("dep_time"),
            "arr_time": metadata.get("arr_time"),
        }
        if any(not value for value in required_values.values()):
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="invalid_schedule_context",
                error_message_safe="Schedule details are incomplete for reserve",
                data={"schedule_id": schedule.schedule_id},
            )

        adults = max(0, int(passengers.get("adults", 0)))
        children = max(0, int(passengers.get("children", 0)))
        passenger_payload = _build_srt_passenger_payload(
            adults=adults,
            children=children,
            special_seat=_seat_class_is_special(seat_class),
        )
        if not passenger_payload:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="invalid_passengers",
                error_message_safe="At least one passenger is required for reserve",
            )

        netfunnel_key = await self._netfunnel.run()
        request_data = {
            "jobId": "1102" if standby else "1101",
            "jrnyCnt": "1",
            "jrnyTpCd": "11",
            "jrnySqno1": "001",
            "stndFlg": "N",
            "trnGpCd1": "300",
            "trnGpCd": "109",
            "grpDv": "0",
            "rtnDv": "0",
            "stlbTrnClsfCd1": str(required_values["train_code"]),
            "dptRsStnCd1": str(required_values["dep_station_code"]),
            "dptRsStnCdNm1": schedule.dep,
            "arvRsStnCd1": str(required_values["arr_station_code"]),
            "arvRsStnCdNm1": schedule.arr,
            "dptDt1": str(required_values["dep_date"]),
            "dptTm1": str(required_values["dep_time"]),
            "arvTm1": str(required_values["arr_time"]),
            "trnNo1": f"{int(schedule.train_no):05d}" if str(schedule.train_no).isdigit() else str(schedule.train_no),
            "runDt1": str(required_values["dep_date"]),
            "dptStnConsOrdr1": str(metadata.get("dep_station_constitution_order") or ""),
            "arvStnConsOrdr1": str(metadata.get("arr_station_constitution_order") or ""),
            "dptStnRunOrdr1": str(metadata.get("dep_station_run_order") or ""),
            "arvStnRunOrdr1": str(metadata.get("arr_station_run_order") or ""),
            "mblPhone": self._phone_number_by_user_id.get(user_id),
            "netfunnelKey": netfunnel_key or "",
        }
        if not standby:
            request_data["reserveType"] = "11"
        request_data.update(passenger_payload)

        response = await self._transport.request(
            method="POST",
            url=SRT_API_ENDPOINTS["reserve"],
            headers=SRT_DEFAULT_HEADERS,
            data=request_data,
            timeout=20.0,
        )
        if response.status_code >= 500:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="srt_server_error",
                error_message_safe="SRT server error during reserve",
            )

        try:
            payload = json.loads(response.text)
        except json.JSONDecodeError:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="invalid_json",
                error_message_safe="SRT reserve response was not valid JSON",
            )

        failure = _srt_status_failure(
            payload,
            error_prefix="srt_reserve_fail",
            default_message="SRT reserve failed",
        )
        if failure is not None:
            return failure

        reservation_id = _extract_srt_reservation_id(payload)
        if not reservation_id:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="reservation_id_missing",
                error_message_safe="SRT reserve succeeded but reservation id was missing",
            )

        return ProviderOutcome(
            ok=True,
            data={
                "reservation_id": reservation_id,
                "schedule_id": schedule.schedule_id,
                "seat_class": seat_class,
                "provider": "SRT",
                "standby": standby,
                "http_trace": {
                    "endpoint": "reserve",
                    "url": SRT_API_ENDPOINTS["reserve"],
                    "status_code": response.status_code,
                    "request": request_data,
                    "response": payload,
                },
            },
        )

    async def _set_standby_options(
        self,
        *,
        reservation_id: str,
        user_id: str,
        agree_sms: bool,
        agree_class_change: bool,
    ) -> bool:
        payload = {
            "pnrNo": reservation_id,
            "psrmClChgFlg": "Y" if agree_class_change else "N",
            "smsSndFlg": "Y" if agree_sms else "N",
            "telNo": self._phone_number_by_user_id.get(user_id, "") if agree_sms else "",
        }
        response = await self._transport.request(
            method="POST",
            url=SRT_API_ENDPOINTS["standby_option"],
            headers=SRT_DEFAULT_HEADERS,
            data=payload,
            timeout=20.0,
        )
        return response.status_code < 400

    async def reserve(
        self,
        *,
        schedule_id: str,
        seat_class: str,
        passengers: dict[str, int],
        user_id: str,
    ) -> ProviderOutcome:
        if user_id not in self._logged_in_user_ids:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="not_logged_in",
                error_message_safe="SRT login is required before reserve",
            )

        schedule = self._schedule_cache.get(user_id, {}).get(schedule_id)
        if schedule is None:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="schedule_context_missing",
                error_message_safe="Schedule context is missing. Search again before reserve.",
                data={"schedule_id": schedule_id},
            )

        has_general = bool(schedule.availability.get("general"))
        has_special = bool(schedule.availability.get("special"))
        standby_possible = _standby_available_from_code((schedule.metadata or {}).get("reserve_wait_code"))
        if not (has_general or has_special):
            if standby_possible:
                return await self.reserve_standby(
                    schedule_id=schedule_id,
                    seat_class=seat_class,
                    passengers=passengers,
                    user_id=user_id,
                )
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="sold_out",
                error_message_safe="No reservable seats are available for this schedule.",
            )

        chosen_seat_class = seat_class
        if seat_class == "special" and not has_special:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="special_seat_unavailable",
                error_message_safe="Special seat is unavailable for this schedule.",
            )
        if seat_class == "general" and not has_general:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="general_seat_unavailable",
                error_message_safe="General seat is unavailable for this schedule.",
            )
        if seat_class == "general_preferred" and not has_general and has_special:
            chosen_seat_class = "special"
        if seat_class == "special_preferred" and not has_special and has_general:
            chosen_seat_class = "general"

        return await self._reserve_request(
            schedule=schedule,
            seat_class=chosen_seat_class,
            passengers=passengers,
            user_id=user_id,
            standby=False,
        )

    async def reserve_standby(
        self,
        *,
        schedule_id: str,
        seat_class: str,
        passengers: dict[str, int],
        user_id: str,
    ) -> ProviderOutcome:
        if user_id not in self._logged_in_user_ids:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="not_logged_in",
                error_message_safe="SRT login is required before reserve standby",
            )

        schedule = self._schedule_cache.get(user_id, {}).get(schedule_id)
        if schedule is None:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="schedule_context_missing",
                error_message_safe="Schedule context is missing. Search again before reserve standby.",
                data={"schedule_id": schedule_id},
            )

        forced_seat_class = "special" if _seat_class_is_special(seat_class) else "general"
        outcome = await self._reserve_request(
            schedule=schedule,
            seat_class=forced_seat_class,
            passengers=passengers,
            user_id=user_id,
            standby=True,
        )
        if not outcome.ok:
            return outcome

        phone_number = self._phone_number_by_user_id.get(user_id, "")
        if phone_number:
            await self._set_standby_options(
                reservation_id=str(outcome.data.get("reservation_id")),
                user_id=user_id,
                agree_sms=True,
                agree_class_change=seat_class in {"general_preferred", "special_preferred"},
            )
        return outcome

    async def pay(
        self,
        *,
        reservation_id: str,
        user_id: str,
        payment_card: dict[str, Any] | None = None,
    ) -> ProviderOutcome:
        if user_id not in self._logged_in_user_ids:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="not_logged_in",
                error_message_safe="SRT login is required before payment",
                data={"reservation_id": reservation_id},
            )
        if not payment_card:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="payment_card_missing",
                error_message_safe="Payment card settings are required for SRT payment",
                data={"reservation_id": reservation_id},
            )

        reservations_outcome = await self.get_reservations(user_id=user_id, reservation_id=reservation_id)
        if not reservations_outcome.ok:
            return reservations_outcome
        reservations = reservations_outcome.data.get("reservations", [])
        reservation_row = reservations[0] if reservations else None
        if not isinstance(reservation_row, dict):
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reservation_not_found",
                error_message_safe="SRT reservation not found for payment",
                data={"reservation_id": reservation_id},
            )
        if bool(reservation_row.get("paid")):
            return ProviderOutcome(
                ok=True,
                data={
                    "reservation_id": reservation_id,
                    "payment_id": f"srt-paid-{reservation_id}",
                    "already_paid": True,
                },
            )

        membership_number = self._membership_number_by_user_id.get(user_id, "")
        card_number = str(payment_card.get("card_number") or "")
        card_password = str(payment_card.get("card_password") or "")
        validation_number = str(payment_card.get("validation_number") or "")
        card_expire = str(payment_card.get("card_expire") or "")
        card_type = str(payment_card.get("card_type") or "J")
        installment = int(payment_card.get("installment") or 0)

        if not membership_number:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="membership_number_missing",
                error_message_safe="SRT membership number is missing. Please reconnect SRT credentials.",
                data={"reservation_id": reservation_id},
            )
        if not card_number or not card_password or not validation_number or not card_expire:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="payment_card_incomplete",
                error_message_safe="Payment card details are incomplete",
                data={"reservation_id": reservation_id},
            )

        departure_hhmmss = _hhmmss_from_iso(str(reservation_row.get("departure_at") or ""))
        arrival_hhmmss = _hhmmss_from_iso(str(reservation_row.get("arrival_at") or ""))

        request_data = {
            "stlDmnDt": now_kst().strftime("%Y%m%d"),
            "mbCrdNo": membership_number,
            "stlMnsSqno1": "1",
            "ststlGridcnt": "1",
            "totNewStlAmt": str(reservation_row.get("total_cost") or 0),
            "athnDvCd1": card_type,
            "vanPwd1": card_password,
            "crdVlidTrm1": card_expire,
            "stlMnsCd1": "02",
            "rsvChgTno": "0",
            "chgMcs": "0",
            "ismtMnthNum1": str(installment),
            "ctlDvCd": "3102",
            "cgPsId": "korail",
            "pnrNo": reservation_id,
            "totPrnb": str(reservation_row.get("seat_count") or 1),
            "mnsStlAmt1": str(reservation_row.get("total_cost") or 0),
            "crdInpWayCd1": "@",
            "athnVal1": validation_number,
            "stlCrCrdNo1": card_number,
            "jrnyCnt": "1",
            "strJobId": "3102",
            "inrecmnsGridcnt": "1",
            "dptTm": departure_hhmmss,
            "arvTm": arrival_hhmmss,
            "dptStnConsOrdr2": "000000",
            "arvStnConsOrdr2": "000000",
            "trnGpCd": "300",
            "pageNo": "-",
            "rowCnt": "-",
            "pageUrl": "",
        }

        response = await self._transport.request(
            method="POST",
            url=SRT_API_ENDPOINTS["payment"],
            headers=SRT_DEFAULT_HEADERS,
            data=request_data,
            timeout=20.0,
        )
        if response.status_code >= 500:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="srt_server_error",
                error_message_safe="SRT server error during payment",
                data={"reservation_id": reservation_id},
            )

        try:
            payload = json.loads(response.text)
        except json.JSONDecodeError:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="invalid_json",
                error_message_safe="SRT payment response was not valid JSON",
                data={"reservation_id": reservation_id},
            )

        status_rows = payload.get("outDataSets", {}).get("dsOutput0", [])
        if isinstance(status_rows, dict):
            status_rows = [status_rows]
        if status_rows:
            first = status_rows[0]
            if isinstance(first, dict) and first.get("strResult") == "FAIL":
                raw_code = first.get("msgCd")
                return ProviderOutcome(
                    ok=False,
                    retryable=False,
                    error_code=f"srt_payment_fail_{raw_code}" if raw_code else "srt_payment_fail",
                    error_message_safe=str(first.get("msgTxt") or "SRT payment failed"),
                    data={"reservation_id": reservation_id},
                )

        return ProviderOutcome(
            ok=True,
            data={
                "reservation_id": reservation_id,
                "payment_id": f"srt-{reservation_id}",
                "paid": True,
                "http_trace": {
                    "endpoint": "payment",
                    "url": SRT_API_ENDPOINTS["payment"],
                    "status_code": response.status_code,
                    "request": {
                        "pnrNo": reservation_id,
                        "jrnyCnt": "1",
                        "totPrnb": request_data["totPrnb"],
                        "totNewStlAmt": request_data["totNewStlAmt"],
                        "mnsStlAmt1": request_data["mnsStlAmt1"],
                    },
                    "response": payload,
                },
            },
        )

    async def get_reservations(
        self,
        *,
        user_id: str,
        paid_only: bool = False,
        reservation_id: str | None = None,
    ) -> ProviderOutcome:
        if user_id not in self._logged_in_user_ids:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="not_logged_in",
                error_message_safe="SRT login is required before reading reservations",
            )

        response = await self._transport.request(
            method="POST",
            url=SRT_API_ENDPOINTS["tickets"],
            headers=SRT_DEFAULT_HEADERS,
            data={"pageNo": "0"},
            timeout=20.0,
        )
        if response.status_code >= 500:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="srt_server_error",
                error_message_safe="SRT server error while reading reservations",
            )

        try:
            payload = json.loads(response.text)
        except json.JSONDecodeError:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="invalid_json",
                error_message_safe="SRT reservation response was not valid JSON",
            )

        failure = _srt_status_failure(
            payload,
            error_prefix="srt_reservations_fail",
            default_message="SRT failed to return reservations",
        )
        if failure is not None:
            return failure

        train_list = _as_list(payload.get("trainListMap"))
        pay_list = _as_list(payload.get("payListMap"))
        kst_now = now_kst()
        reservations: list[dict[str, Any]] = []
        for train_row, pay_row in zip(train_list, pay_list):
            if not isinstance(train_row, dict) or not isinstance(pay_row, dict):
                continue

            current_reservation_id = str(train_row.get("pnrNo") or pay_row.get("pnrNo") or "")
            if not current_reservation_id:
                continue
            if reservation_id and current_reservation_id != reservation_id:
                continue

            is_paid = _reservation_paid(pay_row)
            if paid_only and not is_paid:
                continue

            dep_date = str(pay_row.get("dptDt") or train_row.get("dptDt") or "")
            dep_time = str(pay_row.get("dptTm") or train_row.get("dptTm") or "")
            arv_date = str(train_row.get("arvDt") or dep_date)
            arv_time = str(pay_row.get("arvTm") or train_row.get("arvTm") or "")
            departure_at = _parse_srt_datetime(dep_date, dep_time)
            arrival_at = _parse_srt_datetime(arv_date, arv_time)
            payment_date = str(pay_row.get("iseLmtDt") or "")
            payment_time = str(pay_row.get("iseLmtTm") or "")
            payment_deadline_at = _reservation_payment_deadline(pay_row)
            expired = _reservation_expired_unpaid(pay_row, now=kst_now)

            dep_code = str(pay_row.get("dptRsStnCd") or "")
            arr_code = str(pay_row.get("arvRsStnCd") or "")

            ticket_outcome = await self.ticket_info(
                reservation_id=current_reservation_id,
                user_id=user_id,
            )
            ticket_rows = ticket_outcome.data.get("tickets", []) if ticket_outcome.ok else []

            reservations.append(
                {
                    "reservation_id": current_reservation_id,
                    "provider": "SRT",
                    "paid": is_paid,
                    "waiting": not bool(is_paid or payment_date or payment_time),
                    "expired": expired,
                    "running": "tkSpecNum" not in train_row,
                    "train_no": pay_row.get("trnNo"),
                    "train_code": pay_row.get("stlbTrnClsfCd"),
                    "dep": SRT_STATION_NAME_BY_CODE.get(dep_code, dep_code),
                    "arr": SRT_STATION_NAME_BY_CODE.get(arr_code, arr_code),
                    "departure_at": departure_at.isoformat() if departure_at else None,
                    "arrival_at": arrival_at.isoformat() if arrival_at else None,
                    "payment_deadline_at": payment_deadline_at.isoformat() if payment_deadline_at else None,
                    "seat_count": _to_int(train_row.get("tkSpecNum") or train_row.get("seatNum"), 0),
                    "total_cost": _to_int(train_row.get("rcvdAmt"), 0),
                    "tickets": ticket_rows,
                    "metadata": {
                        "dep_station_code": dep_code,
                        "arr_station_code": arr_code,
                        "reserve_wait_name": train_row.get("rsvWaitPsbCdNm"),
                        "reserve_wait_code": train_row.get("rsvWaitPsbCd"),
                        "payment_cutoff_passed": expired,
                    },
                }
            )

        return ProviderOutcome(
            ok=True,
            data={
                "reservations": reservations,
                "http_trace": {
                    "endpoint": "get_reservations",
                    "url": SRT_API_ENDPOINTS["tickets"],
                    "status_code": response.status_code,
                    "request": {"pageNo": "0"},
                    "response": payload,
                },
            },
        )

    async def ticket_info(
        self,
        *,
        reservation_id: str,
        user_id: str,
    ) -> ProviderOutcome:
        if user_id not in self._logged_in_user_ids:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="not_logged_in",
                error_message_safe="SRT login is required before reading ticket info",
            )

        response = await self._transport.request(
            method="POST",
            url=SRT_API_ENDPOINTS["ticket_info"],
            headers=SRT_DEFAULT_HEADERS,
            data={"pnrNo": reservation_id, "jrnySqno": "1"},
            timeout=20.0,
        )
        if response.status_code >= 500:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="srt_server_error",
                error_message_safe="SRT server error while reading ticket info",
            )

        try:
            payload = json.loads(response.text)
        except json.JSONDecodeError:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="invalid_json",
                error_message_safe="SRT ticket info response was not valid JSON",
            )

        failure = _srt_status_failure(
            payload,
            error_prefix="srt_ticket_info_fail",
            default_message="SRT failed to return ticket info",
        )
        if failure is not None:
            if _is_srt_reservation_not_found_message(failure.error_message_safe):
                return ProviderOutcome(
                    ok=False,
                    retryable=False,
                    error_code="reservation_not_found",
                    error_message_safe="SRT reservation not found for ticket info",
                )
            return failure

        ticket_rows = [
            _build_srt_ticket_dict(ticket_row)
            for ticket_row in _as_list(payload.get("trainListMap"))
            if isinstance(ticket_row, dict)
        ]
        return ProviderOutcome(
            ok=True,
            data={
                "reservation_id": reservation_id,
                "tickets": ticket_rows,
                "http_trace": {
                    "endpoint": "ticket_info",
                    "url": SRT_API_ENDPOINTS["ticket_info"],
                    "status_code": response.status_code,
                    "request": {"pnrNo": reservation_id, "jrnySqno": "1"},
                    "response": payload,
                },
            },
        )

    async def _reserve_info(self, *, reservation_id: str, user_id: str) -> ProviderOutcome:
        headers = {
            **SRT_DEFAULT_HEADERS,
            "Referer": f"{SRT_API_ENDPOINTS['reserve_info_referer']}{reservation_id}",
        }
        response = await self._transport.request(
            method="POST",
            url=SRT_API_ENDPOINTS["reserve_info"],
            headers=headers,
            timeout=20.0,
        )
        if response.status_code >= 500:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="srt_server_error",
                error_message_safe="SRT server error while loading reservation refund info",
            )

        try:
            payload = json.loads(response.text)
        except json.JSONDecodeError:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="invalid_json",
                error_message_safe="SRT reserve info response was not valid JSON",
            )

        error_code = str(payload.get("ErrorCode") or "")
        error_message = str(payload.get("ErrorMsg") or "")
        if _is_srt_reservation_not_found_message(error_message):
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reservation_not_found",
                error_message_safe="SRT reservation not found",
            )
        status_failure = _srt_status_failure(
            payload,
            error_prefix="srt_reserve_info_fail",
            default_message="SRT reserve info failed",
        )
        if status_failure is not None:
            if _is_srt_reservation_not_found_message(status_failure.error_message_safe):
                return ProviderOutcome(
                    ok=False,
                    retryable=False,
                    error_code="reservation_not_found",
                    error_message_safe="SRT reservation not found",
                )
            return status_failure
        if error_code and error_code != "0":
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code=f"srt_reserve_info_fail_{error_code}",
                error_message_safe=error_message or "SRT reserve info failed",
            )
        if error_message:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="srt_reserve_info_fail",
                error_message_safe=error_message,
            )

        rows = _as_list(payload.get("outDataSets", {}).get("dsOutput1"))
        info_row = rows[0] if rows and isinstance(rows[0], dict) else None
        if not isinstance(info_row, dict):
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reserve_info_missing",
                error_message_safe="SRT reserve info is unavailable for refund",
            )

        return ProviderOutcome(
            ok=True,
            data={
                "reservation_id": reservation_id,
                "reserve_info": info_row,
                "http_trace": {
                    "endpoint": "reserve_info",
                    "url": SRT_API_ENDPOINTS["reserve_info"],
                    "status_code": response.status_code,
                    "request": {"pnrNo": reservation_id},
                    "response": payload,
                },
            },
        )

    async def _refund_paid_reservation(self, *, reservation_id: str, user_id: str) -> ProviderOutcome:
        reserve_info_outcome = await self._reserve_info(reservation_id=reservation_id, user_id=user_id)
        if not reserve_info_outcome.ok:
            return reserve_info_outcome

        info = reserve_info_outcome.data.get("reserve_info", {})
        refund_request = {
            "pnr_no": info.get("pnrNo"),
            "cnc_dmn_cont": "승차권 환불로 취소",
            "saleDt": info.get("ogtkSaleDt"),
            "saleWctNo": info.get("ogtkSaleWctNo"),
            "saleSqno": info.get("ogtkSaleSqno"),
            "tkRetPwd": info.get("ogtkRetPwd"),
            "psgNm": info.get("buyPsNm"),
        }
        response = await self._transport.request(
            method="POST",
            url=SRT_API_ENDPOINTS["refund"],
            headers=SRT_DEFAULT_HEADERS,
            data=refund_request,
            timeout=20.0,
        )
        if response.status_code >= 500:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="srt_server_error",
                error_message_safe="SRT server error while refunding paid ticket",
            )

        try:
            payload = json.loads(response.text)
        except json.JSONDecodeError:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="invalid_json",
                error_message_safe="SRT refund response was not valid JSON",
            )

        failure = _srt_status_failure(
            payload,
            error_prefix="srt_refund_fail",
            default_message="SRT refund failed",
        )
        if failure is not None:
            return failure

        return ProviderOutcome(
            ok=True,
            data={
                "reservation_id": reservation_id,
                "cancelled": True,
                "refunded": True,
                "http_trace": {
                    "reserve_info": reserve_info_outcome.data.get("http_trace"),
                    "refund": {
                        "endpoint": "refund",
                        "url": SRT_API_ENDPOINTS["refund"],
                        "status_code": response.status_code,
                        "request": refund_request,
                        "response": payload,
                    },
                },
            },
        )

    async def cancel(
        self,
        *,
        artifact_data: dict[str, Any],
        user_id: str,
    ) -> ProviderOutcome:
        if user_id not in self._logged_in_user_ids:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="not_logged_in",
                error_message_safe="SRT login is required before cancel",
            )

        reservation_id = str(artifact_data.get("reservation_id") or artifact_data.get("pnrNo") or "")
        if not reservation_id:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reservation_id_missing",
                error_message_safe="Reservation id is required to cancel SRT ticket",
            )

        ticket_paid = bool(artifact_data.get("paid")) or str(artifact_data.get("status") or "") == "paid"
        if ticket_paid:
            return await self._refund_paid_reservation(reservation_id=reservation_id, user_id=user_id)

        response = await self._transport.request(
            method="POST",
            url=SRT_API_ENDPOINTS["cancel"],
            headers=SRT_DEFAULT_HEADERS,
            data={"pnrNo": reservation_id, "jrnyCnt": "1", "rsvChgTno": "0"},
            timeout=20.0,
        )
        if response.status_code >= 500:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="srt_server_error",
                error_message_safe="SRT server error while cancelling ticket",
            )

        try:
            payload = json.loads(response.text)
        except json.JSONDecodeError:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="invalid_json",
                error_message_safe="SRT cancel response was not valid JSON",
            )

        failure = _srt_status_failure(
            payload,
            error_prefix="srt_cancel_fail",
            default_message="SRT cancel failed",
        )
        if failure is not None:
            return failure

        return ProviderOutcome(
            ok=True,
            data={
                "reservation_id": reservation_id,
                "cancelled": True,
                "http_trace": {
                    "endpoint": "cancel",
                    "url": SRT_API_ENDPOINTS["cancel"],
                    "status_code": response.status_code,
                    "request": {"pnrNo": reservation_id, "jrnyCnt": "1", "rsvChgTno": "0"},
                    "response": payload,
                },
            },
        )
