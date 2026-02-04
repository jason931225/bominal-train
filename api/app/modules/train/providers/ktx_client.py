from __future__ import annotations

import base64
from collections import defaultdict
import hashlib
import json
import re
from datetime import date, datetime
from typing import Any

from cryptography.hazmat.primitives import padding
from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes

from app.modules.train.providers.base import ProviderOutcome, ProviderSchedule
from app.modules.train.providers.transport import AsyncTransport, HttpxTransport
from app.modules.train.timezone import KST

KORAIL_MOBILE = "https://smart.letskorail.com:443/classes/com.korail.mobile"
KTX_API_ENDPOINTS = {
    "code": f"{KORAIL_MOBILE}.common.code.do",
    "login": f"{KORAIL_MOBILE}.login.Login",
    "search_schedule": f"{KORAIL_MOBILE}.seatMovie.ScheduleView",
    "reserve": f"{KORAIL_MOBILE}.certification.TicketReservation",
    "myticketseat": f"{KORAIL_MOBILE}.refunds.SelTicketInfo",
    "myticketlist": f"{KORAIL_MOBILE}.myTicket.MyTicketList",
    "myreservationview": f"{KORAIL_MOBILE}.reservation.ReservationView",
    "myreservationlist": f"{KORAIL_MOBILE}.certification.ReservationList",
    "payment": f"{KORAIL_MOBILE}.payment.ReservationPayment",
    "cancel": f"{KORAIL_MOBILE}.reservationCancel.ReservationCancelChk",
    "refund": f"{KORAIL_MOBILE}.refunds.RefundsRequest",
}

EMAIL_REGEX = re.compile(r"[^@]+@[^@]+\.[^@]+")
# Korean mobile numbers: 01X-XXXX-XXXX (accepts with or without dashes)
PHONE_NUMBER_REGEX = re.compile(r"01[0-9]-?\d{3,4}-?\d{4}")

KTX_DEFAULT_HEADERS = {
    "Content-Type": "application/x-www-form-urlencoded; charset=UTF-8",
    "User-Agent": "Dalvik/2.1.0 (Linux; U; Android 14; SM-S912N Build/UP1A.231005.007)",
    "Host": "smart.letskorail.com",
    "Connection": "Keep-Alive",
    "Accept-Encoding": "gzip",
}


def _stable_schedule_id(provider: str, dep: str, arr: str, departure_at: datetime, train_no: str) -> str:
    raw = f"{provider}|{dep}|{arr}|{departure_at.isoformat()}|{train_no}"
    return hashlib.sha256(raw.encode("utf-8")).hexdigest()[:24]


def _parse_datetime_yyyymmdd_hhmmss(date_value: str, time_value: str) -> datetime:
    dt = datetime.strptime(f"{date_value}{time_value[:6]}", "%Y%m%d%H%M%S")
    return dt.replace(tzinfo=KST)


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


def _parse_ktx_datetime(date_value: str | None, time_value: str | None) -> datetime | None:
    if not date_value or not time_value:
        return None
    if len(str(date_value)) < 8 or len(str(time_value)) < 4:
        return None
    try:
        dt = datetime.strptime(f"{date_value}{str(time_value)[:6]}", "%Y%m%d%H%M%S")
    except ValueError:
        return None
    return dt.replace(tzinfo=KST)


def _ktx_failure(
    payload: dict[str, Any],
    *,
    error_prefix: str,
    default_message: str,
) -> ProviderOutcome | None:
    if payload.get("strResult") == "FAIL":
        raw_code = payload.get("h_msg_cd")
        return ProviderOutcome(
            ok=False,
            retryable=False,
            error_code=f"{error_prefix}_{raw_code}" if raw_code else error_prefix,
            error_message_safe=payload.get("h_msg_txt", default_message),
        )
    return None


def parse_ktx_search_response(response_text: str, dep: str, arr: str) -> ProviderOutcome:
    try:
        payload = json.loads(response_text)
    except json.JSONDecodeError:
        return ProviderOutcome(
            ok=False,
            retryable=True,
            error_code="invalid_json",
            error_message_safe="KTX response was not valid JSON",
        )

    if payload.get("strResult") == "FAIL":
        raw_code = payload.get("h_msg_cd")
        return ProviderOutcome(
            ok=False,
            retryable=False,
            error_code=f"ktx_api_fail_{raw_code}" if raw_code else "ktx_api_fail",
            error_message_safe=payload.get("h_msg_txt", "KTX API returned failure"),
        )

    rows = payload.get("trn_infos", {}).get("trn_info", [])
    if isinstance(rows, dict):
        rows = [rows]
    if not isinstance(rows, list):
        rows = []
    schedules: list[ProviderSchedule] = []

    for row in rows:
        train_no = row.get("h_trn_no", "")
        dep_date = row.get("h_dpt_dt")
        dep_time = row.get("h_dpt_tm")
        arr_date = row.get("h_arv_dt") or dep_date
        arr_time = row.get("h_arv_tm")
        if not dep_date or not dep_time or not arr_time:
            continue

        departure_at = _parse_datetime_yyyymmdd_hhmmss(dep_date, dep_time)
        arrival_at = _parse_datetime_yyyymmdd_hhmmss(arr_date, arr_time)
        schedule_id = _stable_schedule_id("KTX", dep, arr, departure_at, train_no)

        schedules.append(
            ProviderSchedule(
                schedule_id=schedule_id,
                provider="KTX",
                dep=dep,
                arr=arr,
                departure_at=departure_at,
                arrival_at=arrival_at,
                train_no=train_no,
                availability={
                    "general": row.get("h_gen_rsv_cd") == "11",
                    "special": row.get("h_spe_rsv_cd") == "11",
                },
                metadata={
                    "reserve_possible_name": row.get("h_rsv_psb_nm"),
                    "wait_reserve_flag": row.get("h_wait_rsv_flg"),
                    "train_type_name": row.get("h_trn_clsf_nm"),
                    "train_type_code": row.get("h_trn_clsf_cd"),
                    "train_group_code": row.get("h_trn_gp_cd"),
                    "dep_station_code": row.get("h_dpt_rs_stn_cd"),
                    "arr_station_code": row.get("h_arv_rs_stn_cd"),
                    "dep_date": row.get("h_dpt_dt"),
                    "dep_time": row.get("h_dpt_tm"),
                    "arr_date": row.get("h_arv_dt"),
                    "arr_time": row.get("h_arv_tm"),
                    "run_date": row.get("h_run_dt"),
                },
            )
        )

    return ProviderOutcome(ok=True, data={"schedules": schedules})


def parse_ktx_login_response(response_text: str) -> ProviderOutcome:
    try:
        payload = json.loads(response_text)
    except json.JSONDecodeError:
        return ProviderOutcome(
            ok=False,
            retryable=True,
            error_code="invalid_json",
            error_message_safe="KTX login response was not valid JSON",
        )

    if payload.get("strResult") != "SUCC":
        raw_code = payload.get("h_msg_cd")
        return ProviderOutcome(
            ok=False,
            retryable=False,
            error_code=f"ktx_login_fail_{raw_code}" if raw_code else "ktx_login_fail",
            error_message_safe=payload.get("h_msg_txt", "KTX login failed"),
        )

    membership_number = payload.get("strMbCrdNo")
    if not membership_number:
        return ProviderOutcome(
            ok=False,
            retryable=False,
            error_code="ktx_login_no_membership",
            error_message_safe=payload.get("h_msg_txt", "KTX login failed"),
        )

    return ProviderOutcome(
        ok=True,
        data={
            "membership_number": membership_number,
            "name": payload.get("strCustNm"),
            "phone_number": payload.get("strCpNo"),
            "email": payload.get("strEmailAdr"),
        },
    )


def _build_ktx_passenger_payload(*, adults: int, children: int) -> dict[str, str]:
    rows: list[tuple[str, str, int]] = []
    if adults > 0:
        rows.append(("1", "000", adults))
    if children > 0:
        rows.append(("3", "000", children))

    payload: dict[str, str] = {}
    for idx, (passenger_type, discount_code, count) in enumerate(rows, start=1):
        payload[f"txtPsgTpCd{idx}"] = passenger_type
        payload[f"txtDiscKndCd{idx}"] = discount_code
        payload[f"txtCompaCnt{idx}"] = str(count)
        payload[f"txtCardCode_{idx}"] = ""
        payload[f"txtCardNo_{idx}"] = ""
        payload[f"txtCardPw_{idx}"] = ""
    return payload


def _seat_class_is_special(seat_class: str) -> bool:
    return seat_class in {"special", "special_preferred"}


def _build_ktx_ticket_dict(raw: dict[str, Any]) -> dict[str, Any]:
    seat_no = raw.get("h_seat_no")
    return {
        "car_no": raw.get("h_srcar_no"),
        "seat_no": seat_no,
        "seat_no_end": raw.get("h_seat_no_end"),
        "seat_count": _to_int(raw.get("h_seat_cnt"), 1),
        "seat_class_name": raw.get("h_psrm_cl_nm"),
        "passenger_type_name": raw.get("h_psg_tp_dv_nm"),
        "price": _to_int(raw.get("h_rcvd_amt"), 0),
        "original_price": _to_int(raw.get("h_seat_prc"), 0),
        "discount_amount": _to_int(raw.get("h_dcnt_amt"), 0),
        "waiting": seat_no in {"", None},
    }


class KTXClient:
    provider_name = "KTX"

    def __init__(self, transport: AsyncTransport | None = None):
        self._transport = transport or HttpxTransport()
        self._logged_in_user_ids: set[str] = set()
        self._membership_number_by_user_id: dict[str, str] = {}
        self._schedule_cache: dict[str, dict[str, ProviderSchedule]] = defaultdict(dict)

    async def login(self, *, user_id: str, credentials: dict[str, str] | None = None) -> ProviderOutcome:
        username = (credentials or {}).get("username", "").strip()
        password = (credentials or {}).get("password", "")
        if not username or not password:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="not_configured",
                error_message_safe="KTX credentials are required",
            )

        code_response = await self._transport.request(
            method="POST",
            url=KTX_API_ENDPOINTS["code"],
            headers=KTX_DEFAULT_HEADERS,
            data={"code": "app.login.cphd"},
            timeout=20.0,
        )
        if code_response.status_code >= 500:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="ktx_server_error",
                error_message_safe="KTX server error during login bootstrap",
            )
        try:
            code_payload = json.loads(code_response.text)
            # Check for SUCC status first (aligned with third_party/srtgo/ktx.py)
            if code_payload.get("strResult") != "SUCC":
                return ProviderOutcome(
                    ok=False,
                    retryable=True,
                    error_code="ktx_login_bootstrap_failed",
                    error_message_safe=code_payload.get("h_msg_txt", "KTX login bootstrap failed"),
                )
            code_data = code_payload.get("app.login.cphd", {})
            idx = code_data.get("idx")
            key = code_data.get("key")
        except json.JSONDecodeError:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="ktx_login_bootstrap_invalid_json",
                error_message_safe="KTX login bootstrap returned invalid JSON",
            )

        if not idx or not key:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="ktx_login_bootstrap_missing_keys",
                error_message_safe="KTX login bootstrap missing idx or key",
            )

        encrypted_password = self._encrypt_password(password, key)
        input_flag = "5" if EMAIL_REGEX.match(username) else ("4" if PHONE_NUMBER_REGEX.match(username) else "2")

        request_data = {
            "Device": "AD",
            "Version": "240531001",
            "Key": "korail1234567890",
            "txtMemberNo": username,
            "txtPwd": encrypted_password,
            "txtInputFlg": input_flag,
            "idx": idx,
        }

        login_response = await self._transport.request(
            method="POST",
            url=KTX_API_ENDPOINTS["login"],
            headers=KTX_DEFAULT_HEADERS,
            data=request_data,
            timeout=20.0,
        )
        if login_response.status_code >= 500:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="ktx_server_error",
                error_message_safe="KTX server error during login",
            )

        outcome = parse_ktx_login_response(login_response.text)
        if outcome.ok:
            outcome.data["user_id"] = user_id
            outcome.data["username"] = username
            self._logged_in_user_ids.add(user_id)
            membership_number = str(outcome.data.get("membership_number") or "").strip()
            if membership_number:
                self._membership_number_by_user_id[user_id] = membership_number
        return outcome

    def _encrypt_password(self, password: str, key: str) -> str:
        encrypt_key = key.encode("utf-8")
        iv = key[:16].encode("utf-8")
        padder = padding.PKCS7(algorithms.AES.block_size).padder()
        padded = padder.update(password.encode("utf-8")) + padder.finalize()
        cipher = Cipher(algorithms.AES(encrypt_key), modes.CBC(iv))
        encryptor = cipher.encryptor()
        encrypted = encryptor.update(padded) + encryptor.finalize()
        return base64.b64encode(base64.b64encode(encrypted)).decode("utf-8")

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
        date_yyyymmdd = date_value.strftime("%Y%m%d")
        time_hhmmss = time_window_start.replace(":", "") + "00"
        membership_number = self._membership_number_by_user_id.get(user_id, "")

        request_data = {
            "Device": "AD",
            "Version": "240531001",
            "Sid": "",
            "Key": "korail1234567890",
            "txtMenuId": "11",
            "radJobId": "1",
            # srtgo uses TrainType.ALL ("109") for generic KTX-family schedule lookup.
            "selGoTrain": "109",
            "txtTrnGpCd": "109",
            "txtGoStart": dep,
            "txtGoEnd": arr,
            "txtGoAbrdDt": date_yyyymmdd,
            "txtGoHour": time_hhmmss,
            "txtPsgFlg_1": 1,
            "txtPsgFlg_2": 0,
            "txtPsgFlg_3": 0,
            "txtPsgFlg_4": 0,
            "txtPsgFlg_5": 0,
            "txtSeatAttCd_2": "000",
            "txtSeatAttCd_3": "000",
            "txtSeatAttCd_4": "015",
            "ebizCrossCheck": "N",
            "srtCheckYn": "N",
            "rtYn": "N",
            "adjStnScdlOfrFlg": "N",
            # srtgo sends membership number from a successful login.
            "mbCrdNo": membership_number,
        }

        response = await self._transport.request(
            method="GET",
            url=KTX_API_ENDPOINTS["search_schedule"],
            headers=KTX_DEFAULT_HEADERS,
            params=request_data,
            timeout=20.0,
        )

        if response.status_code >= 500:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="ktx_server_error",
                error_message_safe="KTX server error during search",
            )

        parsed = parse_ktx_search_response(response.text, dep=dep, arr=arr)
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
        dep_date = str(metadata.get("dep_date") or "")
        dep_station_code = str(metadata.get("dep_station_code") or "")
        arr_station_code = str(metadata.get("arr_station_code") or "")
        dep_time = str(metadata.get("dep_time") or "")
        run_date = str(metadata.get("run_date") or dep_date)
        train_type_code = str(metadata.get("train_type_code") or "100")
        train_group_code = str(metadata.get("train_group_code") or "109")
        if not dep_date or not dep_station_code or not arr_station_code or not dep_time:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="invalid_schedule_context",
                error_message_safe="Schedule details are incomplete for reserve",
                data={"schedule_id": schedule.schedule_id},
            )

        adults = max(0, int(passengers.get("adults", 0)))
        children = max(0, int(passengers.get("children", 0)))
        total_passengers = adults + children
        if total_passengers <= 0:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="invalid_passengers",
                error_message_safe="At least one passenger is required for reserve",
            )

        passenger_payload = _build_ktx_passenger_payload(adults=adults, children=children)
        request_data = {
            "Device": "AD",
            "Version": "240531001",
            "Key": "korail1234567890",
            "txtMenuId": "11",
            "txtJobId": "1102" if standby else "1101",
            "txtGdNo": "",
            "hidFreeFlg": "N",
            "txtTotPsgCnt": str(total_passengers),
            "txtSeatAttCd1": "000",
            "txtSeatAttCd2": "000",
            "txtSeatAttCd3": "000",
            "txtSeatAttCd4": "015",
            "txtSeatAttCd5": "000",
            "txtStndFlg": "N",
            "txtSrcarCnt": "0",
            "txtJrnyCnt": "1",
            "txtJrnySqno1": "001",
            "txtJrnyTpCd1": "11",
            "txtDptDt1": dep_date,
            "txtDptRsStnCd1": dep_station_code,
            "txtDptTm1": dep_time,
            "txtArvRsStnCd1": arr_station_code,
            "txtTrnNo1": str(schedule.train_no),
            "txtRunDt1": run_date,
            "txtTrnClsfCd1": train_type_code,
            "txtTrnGpCd1": train_group_code,
            "txtPsrmClCd1": "2" if _seat_class_is_special(seat_class) else "1",
            "txtChgFlg1": "",
            "txtJrnySqno2": "",
            "txtJrnyTpCd2": "",
            "txtDptDt2": "",
            "txtDptRsStnCd2": "",
            "txtDptTm2": "",
            "txtArvRsStnCd2": "",
            "txtTrnNo2": "",
            "txtRunDt2": "",
            "txtTrnClsfCd2": "",
            "txtPsrmClCd2": "",
            "txtChgFlg2": "",
        }
        request_data.update(passenger_payload)

        response = await self._transport.request(
            method="GET",
            url=KTX_API_ENDPOINTS["reserve"],
            headers=KTX_DEFAULT_HEADERS,
            params=request_data,
            timeout=20.0,
        )
        if response.status_code >= 500:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="ktx_server_error",
                error_message_safe="KTX server error during reserve",
            )

        try:
            payload = json.loads(response.text)
        except json.JSONDecodeError:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="invalid_json",
                error_message_safe="KTX reserve response was not valid JSON",
            )

        failure = _ktx_failure(payload, error_prefix="ktx_reserve_fail", default_message="KTX reserve failed")
        if failure is not None:
            return failure

        reservation_id = payload.get("h_pnr_no")
        if not reservation_id:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="reservation_id_missing",
                error_message_safe="KTX reserve succeeded but reservation id was missing",
            )

        extra_metadata: dict[str, Any] = {}
        reservation_lookup = await self.get_reservations(
            user_id=user_id,
            reservation_id=str(reservation_id),
        )
        if reservation_lookup.ok:
            rows = reservation_lookup.data.get("reservations", [])
            if rows:
                metadata_row = rows[0]
                extra_metadata = {
                    "journey_no": metadata_row.get("journey_no"),
                    "journey_cnt": metadata_row.get("journey_cnt"),
                    "rsv_chg_no": metadata_row.get("rsv_chg_no"),
                    "wct_no": metadata_row.get("wct_no"),
                }

        return ProviderOutcome(
            ok=True,
            data={
                "reservation_id": str(reservation_id),
                "schedule_id": schedule.schedule_id,
                "seat_class": seat_class,
                "provider": "KTX",
                "standby": standby,
                "http_trace": {
                    "endpoint": "reserve",
                    "url": KTX_API_ENDPOINTS["reserve"],
                    "status_code": response.status_code,
                    "request": request_data,
                    "response": payload,
                },
                **extra_metadata,
            },
        )

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
                error_message_safe="KTX login is required before reserve",
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
        wait_flag = _to_int((schedule.metadata or {}).get("wait_reserve_flag"), default=-1)

        has_any_seat = has_general or has_special
        standby = not has_any_seat and wait_flag >= 0
        if not has_any_seat and not standby:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="sold_out",
                error_message_safe="No reservable seats are available for this schedule.",
            )

        chosen_seat_class = seat_class
        if seat_class == "special" and not has_special:
            if standby:
                chosen_seat_class = "special"
            else:
                return ProviderOutcome(
                    ok=False,
                    retryable=False,
                    error_code="special_seat_unavailable",
                    error_message_safe="Special seat is unavailable for this schedule.",
                )
        if seat_class == "general" and not has_general:
            if standby:
                chosen_seat_class = "general"
            else:
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
            standby=standby,
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
                error_message_safe="KTX login is required before reserve standby",
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
        return await self._reserve_request(
            schedule=schedule,
            seat_class=forced_seat_class,
            passengers=passengers,
            user_id=user_id,
            standby=True,
        )

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
                error_message_safe="KTX login is required before payment",
                data={"reservation_id": reservation_id},
            )
        if not payment_card:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="payment_card_missing",
                error_message_safe="Payment card settings are required for KTX payment",
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
                error_message_safe="KTX reservation not found for payment",
                data={"reservation_id": reservation_id},
            )
        if bool(reservation_row.get("paid")):
            return ProviderOutcome(
                ok=True,
                data={
                    "reservation_id": reservation_id,
                    "payment_id": f"ktx-paid-{reservation_id}",
                    "already_paid": True,
                },
            )

        wct_no = str(reservation_row.get("wct_no") or "")
        if not wct_no:
            ticket_outcome = await self.ticket_info(reservation_id=reservation_id, user_id=user_id)
            if ticket_outcome.ok:
                wct_no = str(ticket_outcome.data.get("wct_no") or "")

        card_number = str(payment_card.get("card_number") or "")
        card_password = str(payment_card.get("card_password") or "")
        validation_number = str(payment_card.get("validation_number") or "")
        card_expire = str(payment_card.get("card_expire") or "")
        card_type = str(payment_card.get("card_type") or "J")
        installment = int(payment_card.get("installment") or 0)
        if not card_number or not card_password or not validation_number or not card_expire:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="payment_card_incomplete",
                error_message_safe="Payment card details are incomplete",
                data={"reservation_id": reservation_id},
            )
        if not wct_no:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="ktx_wct_no_missing",
                error_message_safe="KTX payment requires ticket issue number. Try again.",
                data={"reservation_id": reservation_id},
            )

        request_data = {
            "Device": "AD",
            "Version": "240531001",
            "Key": "korail1234567890",
            "hidPnrNo": reservation_id,
            "hidWctNo": wct_no,
            "hidTmpJobSqno1": "000000",
            "hidTmpJobSqno2": "000000",
            "hidRsvChgNo": str(reservation_row.get("rsv_chg_no") or "00000"),
            "hidInrecmnsGridcnt": "1",
            "hidStlMnsSqno1": "1",
            "hidStlMnsCd1": "02",
            "hidMnsStlAmt1": str(reservation_row.get("total_cost") or 0),
            "hidCrdInpWayCd1": "@",
            "hidStlCrCrdNo1": card_number,
            "hidVanPwd1": card_password,
            "hidCrdVlidTrm1": card_expire,
            "hidIsmtMnthNum1": str(installment),
            "hidAthnDvCd1": card_type,
            "hidAthnVal1": validation_number,
            "hiduserYn": "Y",
        }

        response = await self._transport.request(
            method="POST",
            url=KTX_API_ENDPOINTS["payment"],
            headers=KTX_DEFAULT_HEADERS,
            data=request_data,
            timeout=20.0,
        )
        if response.status_code >= 500:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="ktx_server_error",
                error_message_safe="KTX server error during payment",
                data={"reservation_id": reservation_id},
            )

        try:
            payload = json.loads(response.text)
        except json.JSONDecodeError:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="invalid_json",
                error_message_safe="KTX payment response was not valid JSON",
                data={"reservation_id": reservation_id},
            )

        failure = _ktx_failure(payload, error_prefix="ktx_pay_fail", default_message="KTX payment failed")
        if failure is not None:
            return failure

        return ProviderOutcome(
            ok=True,
            data={
                "reservation_id": reservation_id,
                "payment_id": f"ktx-{reservation_id}",
                "paid": True,
                "http_trace": {
                    "endpoint": "payment",
                    "url": KTX_API_ENDPOINTS["payment"],
                    "status_code": response.status_code,
                    "request": {
                        "hidPnrNo": reservation_id,
                        "hidWctNo": wct_no,
                        "hidMnsStlAmt1": request_data["hidMnsStlAmt1"],
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
                error_message_safe="KTX login is required before reading reservations",
            )

        response = await self._transport.request(
            method="GET",
            url=KTX_API_ENDPOINTS["myreservationview"],
            headers=KTX_DEFAULT_HEADERS,
            params={
                "Device": "AD",
                "Version": "240531001",
                "Key": "korail1234567890",
            },
            timeout=20.0,
        )
        if response.status_code >= 500:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="ktx_server_error",
                error_message_safe="KTX server error while reading reservations",
            )

        try:
            payload = json.loads(response.text)
        except json.JSONDecodeError:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="invalid_json",
                error_message_safe="KTX reservation response was not valid JSON",
            )

        failure = _ktx_failure(
            payload,
            error_prefix="ktx_reservations_fail",
            default_message="KTX failed to return reservations",
        )
        if failure is not None:
            return failure

        reservation_rows: list[dict[str, Any]] = []
        for journey in _as_list(payload.get("jrny_infos", {}).get("jrny_info")):
            if not isinstance(journey, dict):
                continue
            for train_row in _as_list(journey.get("train_infos", {}).get("train_info")):
                if not isinstance(train_row, dict):
                    continue

                current_reservation_id = str(train_row.get("h_pnr_no") or "")
                if not current_reservation_id:
                    continue
                if reservation_id and current_reservation_id != reservation_id:
                    continue

                dep_date = str(train_row.get("h_run_dt") or train_row.get("h_dpt_dt") or "")
                dep_time = str(train_row.get("h_dpt_tm") or "")
                arr_date = str(train_row.get("h_run_dt") or train_row.get("h_arv_dt") or dep_date)
                arr_time = str(train_row.get("h_arv_tm") or "")
                departure_at = _parse_ktx_datetime(dep_date, dep_time)
                arrival_at = _parse_ktx_datetime(arr_date, arr_time)

                buy_limit_date = str(train_row.get("h_ntisu_lmt_dt") or "")
                buy_limit_time = str(train_row.get("h_ntisu_lmt_tm") or "")
                payment_deadline_at = _parse_ktx_datetime(buy_limit_date, buy_limit_time)
                is_waiting = buy_limit_date == "00000000" or buy_limit_time == "235959"
                pay_state_raw = str(
                    train_row.get("h_stl_flg")
                    or train_row.get("h_pay_flg")
                    or train_row.get("h_ntisu_yn")
                    or train_row.get("h_ntisu_flg")
                    or ""
                ).strip()
                is_paid = pay_state_raw.upper() in {"Y", "1", "T", "PAID"}
                if paid_only and not is_paid:
                    continue

                ticket_outcome = await self.ticket_info(
                    reservation_id=current_reservation_id,
                    user_id=user_id,
                )
                ticket_rows = ticket_outcome.data.get("tickets", []) if ticket_outcome.ok else []
                wct_no = ticket_outcome.data.get("wct_no") if ticket_outcome.ok else None

                reservation_rows.append(
                    {
                        "reservation_id": current_reservation_id,
                        "provider": "KTX",
                        "paid": is_paid,
                        "waiting": is_waiting,
                        "train_no": train_row.get("h_trn_no"),
                        "train_type_code": train_row.get("h_trn_clsf_cd"),
                        "train_type_name": train_row.get("h_trn_clsf_nm"),
                        "dep": train_row.get("h_dpt_rs_stn_nm"),
                        "arr": train_row.get("h_arv_rs_stn_nm"),
                        "departure_at": departure_at.isoformat() if departure_at else None,
                        "arrival_at": arrival_at.isoformat() if arrival_at else None,
                        "payment_deadline_at": payment_deadline_at.isoformat() if payment_deadline_at else None,
                        "payment_state_raw": pay_state_raw,
                        "seat_count": _to_int(train_row.get("h_tot_seat_cnt"), 0),
                        "total_cost": _to_int(train_row.get("h_rsv_amt"), 0),
                        "journey_no": str(train_row.get("txtJrnySqno") or "001"),
                        "journey_cnt": str(train_row.get("txtJrnyCnt") or "01"),
                        "rsv_chg_no": str(train_row.get("hidRsvChgNo") or "00000"),
                        "wct_no": wct_no,
                        "tickets": ticket_rows,
                    }
                )
        return ProviderOutcome(
            ok=True,
            data={
                "reservations": reservation_rows,
                "http_trace": {
                    "endpoint": "get_reservations",
                    "url": KTX_API_ENDPOINTS["myreservationview"],
                    "status_code": response.status_code,
                    "request": {
                        "Device": "AD",
                        "Version": "240531001",
                        "Key": "korail1234567890",
                    },
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
                error_message_safe="KTX login is required before reading ticket info",
            )

        response = await self._transport.request(
            method="GET",
            url=KTX_API_ENDPOINTS["myreservationlist"],
            headers=KTX_DEFAULT_HEADERS,
            params={
                "Device": "AD",
                "Version": "240531001",
                "Key": "korail1234567890",
                "hidPnrNo": reservation_id,
            },
            timeout=20.0,
        )
        if response.status_code >= 500:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="ktx_server_error",
                error_message_safe="KTX server error while reading ticket info",
            )

        try:
            payload = json.loads(response.text)
        except json.JSONDecodeError:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="invalid_json",
                error_message_safe="KTX ticket info response was not valid JSON",
            )

        failure = _ktx_failure(
            payload,
            error_prefix="ktx_ticket_info_fail",
            default_message="KTX failed to return ticket info",
        )
        if failure is not None:
            return failure

        journey_rows = _as_list(payload.get("jrny_infos", {}).get("jrny_info"))
        seat_rows: list[dict[str, Any]] = []
        if journey_rows and isinstance(journey_rows[0], dict):
            seat_rows = [
                _build_ktx_ticket_dict(seat_row)
                for seat_row in _as_list(journey_rows[0].get("seat_infos", {}).get("seat_info"))
                if isinstance(seat_row, dict)
            ]

        return ProviderOutcome(
            ok=True,
            data={
                "reservation_id": reservation_id,
                "wct_no": payload.get("h_wct_no"),
                "tickets": seat_rows,
                "http_trace": {
                    "endpoint": "ticket_info",
                    "url": KTX_API_ENDPOINTS["myreservationlist"],
                    "status_code": response.status_code,
                    "request": {
                        "Device": "AD",
                        "Version": "240531001",
                        "Key": "korail1234567890",
                        "hidPnrNo": reservation_id,
                    },
                    "response": payload,
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
                error_message_safe="KTX login is required before cancel",
            )

        reservation_id = str(artifact_data.get("reservation_id") or artifact_data.get("pnrNo") or "")
        if not reservation_id:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reservation_id_missing",
                error_message_safe="Reservation id is required to cancel KTX ticket",
            )

        ticket_paid = bool(artifact_data.get("paid")) or str(artifact_data.get("status") or "") == "paid"
        if ticket_paid:
            return await self._refund_paid_reservation(
                reservation_id=reservation_id,
                user_id=user_id,
            )

        journey_no = str(artifact_data.get("journey_no") or "")
        journey_cnt = str(artifact_data.get("journey_cnt") or "")
        rsv_chg_no = str(artifact_data.get("rsv_chg_no") or "")

        if not (journey_no and journey_cnt and rsv_chg_no):
            reservation_lookup = await self.get_reservations(
                user_id=user_id,
                reservation_id=reservation_id,
            )
            if not reservation_lookup.ok:
                return reservation_lookup
            rows = reservation_lookup.data.get("reservations", [])
            if not rows:
                return ProviderOutcome(
                    ok=False,
                    retryable=False,
                    error_code="reservation_not_found",
                    error_message_safe="KTX reservation was not found",
                )
            row = rows[0]
            journey_no = str(row.get("journey_no") or "001")
            journey_cnt = str(row.get("journey_cnt") or "01")
            rsv_chg_no = str(row.get("rsv_chg_no") or "00000")

        response = await self._transport.request(
            method="POST",
            url=KTX_API_ENDPOINTS["cancel"],
            headers=KTX_DEFAULT_HEADERS,
            data={
                "Device": "AD",
                "Version": "240531001",
                "Key": "korail1234567890",
                "txtPnrNo": reservation_id,
                "txtJrnySqno": journey_no,
                "txtJrnyCnt": journey_cnt,
                "hidRsvChgNo": rsv_chg_no,
            },
            timeout=20.0,
        )
        if response.status_code >= 500:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="ktx_server_error",
                error_message_safe="KTX server error while cancelling ticket",
            )

        try:
            payload = json.loads(response.text)
        except json.JSONDecodeError:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="invalid_json",
                error_message_safe="KTX cancel response was not valid JSON",
            )

        failure = _ktx_failure(payload, error_prefix="ktx_cancel_fail", default_message="KTX cancel failed")
        if failure is not None:
            return failure

        return ProviderOutcome(
            ok=True,
            data={
                "reservation_id": reservation_id,
                "cancelled": True,
                "http_trace": {
                    "endpoint": "cancel",
                    "url": KTX_API_ENDPOINTS["cancel"],
                    "status_code": response.status_code,
                    "request": {
                        "Device": "AD",
                        "Version": "240531001",
                        "Key": "korail1234567890",
                        "txtPnrNo": reservation_id,
                        "txtJrnySqno": journey_no,
                        "txtJrnyCnt": journey_cnt,
                        "hidRsvChgNo": rsv_chg_no,
                    },
                    "response": payload,
                },
            },
        )

    async def _refund_paid_reservation(
        self,
        *,
        reservation_id: str,
        user_id: str,
    ) -> ProviderOutcome:
        ticket_list_response = await self._transport.request(
            method="GET",
            url=KTX_API_ENDPOINTS["myticketlist"],
            headers=KTX_DEFAULT_HEADERS,
            params={
                "Device": "AD",
                "Version": "240531001",
                "Key": "korail1234567890",
                "txtDeviceId": "",
                "txtIndex": "1",
                "h_page_no": "1",
                "h_abrd_dt_from": "",
                "h_abrd_dt_to": "",
                "hiduserYn": "Y",
            },
            timeout=20.0,
        )
        if ticket_list_response.status_code >= 500:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="ktx_server_error",
                error_message_safe="KTX server error while loading refund ticket list",
            )

        try:
            ticket_list_payload = json.loads(ticket_list_response.text)
        except json.JSONDecodeError:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="invalid_json",
                error_message_safe="KTX ticket list response was not valid JSON",
            )

        failure = _ktx_failure(
            ticket_list_payload,
            error_prefix="ktx_ticket_list_fail",
            default_message="KTX failed to return ticket list for refund",
        )
        if failure is not None:
            return failure

        refund_context: dict[str, Any] | None = None
        for entry in _as_list(ticket_list_payload.get("reservation_list")):
            if not isinstance(entry, dict):
                continue
            ticket_list = _as_list(entry.get("ticket_list"))
            if not ticket_list or not isinstance(ticket_list[0], dict):
                continue
            train_info_rows = _as_list(ticket_list[0].get("train_info"))
            if not train_info_rows or not isinstance(train_info_rows[0], dict):
                continue
            row = train_info_rows[0]
            if str(row.get("h_pnr_no") or "") != reservation_id:
                continue
            refund_context = row
            break

        if refund_context is None:
            return ProviderOutcome(
                ok=False,
                retryable=False,
                error_code="reservation_not_found",
                error_message_safe="KTX paid reservation was not found in ticket list",
            )

        refund_request = {
            "Device": "AD",
            "Version": "240531001",
            "Key": "korail1234567890",
            "txtPrnNo": reservation_id,
            "h_orgtk_sale_dt": refund_context.get("h_orgtk_ret_sale_dt"),
            "h_orgtk_sale_wct_no": refund_context.get("h_orgtk_wct_no"),
            "h_orgtk_sale_sqno": refund_context.get("h_orgtk_sale_sqno"),
            "h_orgtk_ret_pwd": refund_context.get("h_orgtk_ret_pwd"),
            "h_mlg_stl": "N",
            "tk_ret_tms_dv_cd": "21",
            "trnNo": refund_context.get("h_trn_no"),
            "pbpAcepTgtFlg": "N",
            "latitude": "",
            "longitude": "",
        }

        refund_response = await self._transport.request(
            method="POST",
            url=KTX_API_ENDPOINTS["refund"],
            headers=KTX_DEFAULT_HEADERS,
            data=refund_request,
            timeout=20.0,
        )
        if refund_response.status_code >= 500:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="ktx_server_error",
                error_message_safe="KTX server error while refunding paid ticket",
            )

        try:
            refund_payload = json.loads(refund_response.text)
        except json.JSONDecodeError:
            return ProviderOutcome(
                ok=False,
                retryable=True,
                error_code="invalid_json",
                error_message_safe="KTX refund response was not valid JSON",
            )

        refund_failure = _ktx_failure(
            refund_payload,
            error_prefix="ktx_refund_fail",
            default_message="KTX refund failed",
        )
        if refund_failure is not None:
            return refund_failure

        return ProviderOutcome(
            ok=True,
            data={
                "reservation_id": reservation_id,
                "cancelled": True,
                "refunded": True,
                "http_trace": {
                    "ticket_list": {
                        "endpoint": "myticketlist",
                        "url": KTX_API_ENDPOINTS["myticketlist"],
                        "status_code": ticket_list_response.status_code,
                        "request": {
                            "Device": "AD",
                            "Version": "240531001",
                            "Key": "korail1234567890",
                            "h_page_no": "1",
                        },
                        "response": ticket_list_payload,
                    },
                    "refund": {
                        "endpoint": "refund",
                        "url": KTX_API_ENDPOINTS["refund"],
                        "status_code": refund_response.status_code,
                        "request": {
                            "txtPrnNo": reservation_id,
                            "h_orgtk_sale_dt": refund_request.get("h_orgtk_sale_dt"),
                            "h_orgtk_sale_wct_no": refund_request.get("h_orgtk_sale_wct_no"),
                            "h_orgtk_sale_sqno": refund_request.get("h_orgtk_sale_sqno"),
                            "trnNo": refund_request.get("trnNo"),
                        },
                        "response": refund_payload,
                    },
                },
            },
        )
