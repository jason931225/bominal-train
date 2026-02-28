#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import ssl
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Tuple
from urllib.parse import parse_qsl, urlencode, urlparse
from urllib.request import Request, urlopen

BASE_DIR = Path(__file__).resolve().parent
OUTPUT_PATH = BASE_DIR / "output" / "auth_scope_probe" / "latest_auth_scope_probe.json"

USER_AGENT = (
    "Mozilla/5.0 (iPhone; CPU iPhone OS 18_7 like Mac OS X) "
    "AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148 SRT-APP-iOS V.2.0.26"
)

LOGIN_MARKERS = [
    "login/login.do",
    "로그인 후 사용",
    "로그인 후 사용하십시요",
    "로그인이 필요",
    "세션",
    "session expired",
    "not logged",
    "unauthorized",
]


@dataclass
class ProbeSpec:
    probe_id: str
    method: str
    url: str
    data: Optional[Dict[str, str]] = None
    note: str = ""
    accept: str = "*/*"


def _iso_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def _read_response(req: Request, timeout: int = 25) -> Tuple[Optional[int], str, str, str]:
    context = ssl.create_default_context()
    status = None
    ctype = ""
    text = ""
    error = ""

    try:
        with urlopen(req, timeout=timeout, context=context) as response:
            status = response.status
            ctype = response.headers.get("Content-Type", "")
            text = response.read(50000).decode("utf-8", "ignore")
    except Exception as exc:  # pragma: no cover - network variance
        error = str(exc)

    return status, ctype, text, error


def _request(spec: ProbeSpec) -> Dict[str, Any]:
    headers = {
        "User-Agent": USER_AGENT,
        "Accept": spec.accept,
    }

    body = None
    if spec.data is not None:
        headers["Content-Type"] = "application/x-www-form-urlencoded; charset=UTF-8"
        body = urlencode(spec.data).encode("utf-8")

    req = Request(spec.url, data=body, headers=headers, method=spec.method)
    status, ctype, text, error = _read_response(req)

    parsed_url = urlparse(spec.url)
    host = parsed_url.hostname or ""
    path = parsed_url.path or "/"

    body_signals, has_login_marker, meaningful_payload = _analyze_body(path, ctype, text)
    auth_hint = _classify_hint(path, has_login_marker, meaningful_payload, body_signals)

    return {
        "probe_id": spec.probe_id,
        "captured_at": _iso_now(),
        "method": spec.method,
        "url": spec.url,
        "host": host,
        "path": path,
        "request_has_cookie": False,
        "status_code": status,
        "content_type": ctype,
        "error": error,
        "body_signals": sorted(set(body_signals)),
        "has_login_marker": has_login_marker,
        "meaningful_payload": meaningful_payload,
        "auth_classification_hint": auth_hint,
        "snippet": _sanitize_snippet(text),
        "note": spec.note,
    }


def _sanitize_snippet(text: str, max_len: int = 260) -> str:
    snippet = re.sub(r"\s+", " ", text).strip()
    if len(snippet) > max_len:
        snippet = snippet[:max_len]

    # Defensive redaction for any accidental secrets.
    snippet = re.sub(r"(?i)(authorization|cookie|token|passwd|password)\s*[:=]\s*[^\s\"']+", r"\1=<redacted>", snippet)
    return snippet


def _parse_json(text: str) -> Optional[Any]:
    probe = text.strip()
    if not probe or probe[0] not in "[{":
        return None
    try:
        return json.loads(probe)
    except Exception:
        return None


def _analyze_body(path: str, content_type: str, text: str) -> Tuple[List[str], bool, bool]:
    signals: List[str] = []
    low = text.lower()

    has_login_marker = any(marker.lower() in low for marker in LOGIN_MARKERS)
    if has_login_marker:
        signals.append("login_marker_in_body")

    parsed_json = _parse_json(text)
    meaningful_payload = False

    if isinstance(parsed_json, dict):
        keys = set(parsed_json.keys())
        if {"resultMap", "trainListMap"}.issubset(keys) and isinstance(parsed_json.get("trainListMap"), list):
            train_count = len(parsed_json.get("trainListMap") or [])
            if train_count > 0:
                meaningful_payload = True
                signals.extend(["business_payload_without_cookie", "search_result_json"])
        if "outDataSets" in keys and isinstance(parsed_json.get("outDataSets"), dict):
            signals.append("business_envelope_json")
        if "ERROR_CODE" in keys or "ErrorCode" in keys:
            signals.append("error_code_envelope")

    if path.endswith(".js") and "javascript" in content_type.lower():
        if len(text) > 80:
            meaningful_payload = True
            signals.extend(["static_js_asset", "business_payload_without_cookie"])

    if "text/html" in content_type.lower() or "<html" in low:
        signals.append("html_response")

    if path == "/ara/selectListAra12009_n.do":
        time_count = len(re.findall(r"\b\d{2}:\d{2}\b", text))
        if ("역명" in text and time_count >= 4) or "지연" in text:
            meaningful_payload = True
            signals.extend(["business_payload_without_cookie", "schedule_popup_html"])

    if path == "/ara/selectListAra13010_n.do":
        won_count = len(re.findall(r"\d{1,3}(?:,\d{3})*원", text))
        if won_count >= 2 and ("특실" in text or "일반실" in text):
            meaningful_payload = True
            signals.extend(["business_payload_without_cookie", "fare_popup_html"])

    if path == "/atd/selectListAtd02039_n.do":
        if "승차권구매 이력상세" in text and len(text) < 2000:
            signals.append("history_template_without_rows")

    if path == "/ara/selectListAra10007_n.do" and "서비스가 접속이 원활하지 않습니다" in text:
        signals.append("context_token_missing_or_invalid")

    if path in ("/hpg/hra/01/selectTrainScheduleList.do", "/hpg/hra/01/selectTrainChargeList.do"):
        signals.append("etk_popup_html")

    return signals, has_login_marker, meaningful_payload


def _classify_hint(path: str, has_login_marker: bool, meaningful_payload: bool, body_signals: Iterable[str]) -> str:
    signals = set(body_signals)
    if has_login_marker:
        return "auth_required"

    if path == "/ara/selectListAra10007_n.do":
        if meaningful_payload:
            return "public_context_required"
        if "context_token_missing_or_invalid" in signals:
            return "public_context_required"

    if meaningful_payload:
        return "public_stateless"

    return "unknown"


def _parse_netfunnel_result(text: str) -> Optional[Dict[str, str]]:
    match = re.search(r"NetFunnel\.gControl\.result='([^']+)'", text)
    if not match:
        return None

    parts = match.group(1).split(":", 2)
    if len(parts) != 3:
        return None

    code, status, params_part = parts
    params = dict(parse_qsl(params_part, keep_blank_values=True))
    params["code"] = code
    params["status"] = status
    return params


def _netfunnel_request(opcode: str, key: Optional[str] = None, ip: Optional[str] = None) -> Dict[str, str]:
    host = ip or "nf.letskorail.com"
    base_url = f"https://{host}/ts.wseq"

    params: List[Tuple[str, str]] = [
        ("opcode", opcode),
        ("nfid", "0"),
        ("prefix", f"NetFunnel.gRtype={opcode};"),
        ("js", "true"),
        (str(int(time.time() * 1000)), ""),
    ]

    if opcode in ("5101", "5002"):
        params.extend([("sid", "service_1"), ("aid", "act_10")])
        if opcode == "5002" and key:
            params.extend([("key", key), ("ttl", "1")])
    elif opcode == "5004" and key:
        params.append(("key", key))

    query = urlencode(params)
    req = Request(
        f"{base_url}?{query}",
        headers={"User-Agent": USER_AGENT, "Accept": "*/*"},
        method="GET",
    )
    _, _, text, error = _read_response(req)
    if error:
        raise RuntimeError(error)

    parsed = _parse_netfunnel_result(text)
    if not parsed:
        raise RuntimeError("netfunnel_parse_failed")
    return parsed


def acquire_netfunnel_key() -> Optional[str]:
    try:
        start = _netfunnel_request("5101")
        status = start.get("status")
        key = start.get("key")
        ip = start.get("ip")

        max_wait = 8
        while status == "201" and max_wait > 0:
            time.sleep(1)
            check = _netfunnel_request("5002", key=key, ip=ip)
            status = check.get("status")
            key = check.get("key") or key
            ip = check.get("ip") or ip
            max_wait -= 1

        _netfunnel_request("5004", key=key, ip=ip)
        if key:
            return key
    except Exception:
        return None
    return None


def build_probe_specs(netfunnel_key: Optional[str]) -> List[ProbeSpec]:
    specs = [
        ProbeSpec(
            probe_id="srt_search_without_netfunnel",
            method="POST",
            url="https://app.srail.or.kr:443/ara/selectListAra10007_n.do",
            data={
                "chtnDvCd": "1",
                "dptDt": "20260306",
                "dptTm": "120000",
                "dptDt1": "20260306",
                "dptTm1": "120000",
                "dptRsStnCd": "0551",
                "arvRsStnCd": "0015",
                "stlbTrnClsfCd": "05",
                "trnGpCd": "109",
                "psgNum": "1",
                "seatAttCd": "015",
                "arriveTime": "N",
                "dlayTnumAplFlg": "Y",
                "netfunnelKey": "dummy",
            },
            note="Search probe without valid netfunnel key.",
            accept="application/json",
        ),
        ProbeSpec(
            probe_id="srt_reserve",
            method="POST",
            url="https://app.srail.or.kr:443/arc/selectListArc05013_n.do",
            data={"jobId": "1101"},
        ),
        ProbeSpec(
            probe_id="srt_tickets",
            method="POST",
            url="https://app.srail.or.kr:443/atc/selectListAtc14016_n.do",
            data={"pageNo": "0"},
        ),
        ProbeSpec(
            probe_id="srt_ticket_detail",
            method="POST",
            url="https://app.srail.or.kr:443/ard/selectListArd02019_n.do",
            data={"pnrNo": "0000000000", "jrnySqno": "1"},
        ),
        ProbeSpec(
            probe_id="srt_cancel",
            method="POST",
            url="https://app.srail.or.kr:443/ard/selectListArd02045_n.do",
            data={"pnrNo": "0000000000", "jrnyCnt": "1", "rsvChgTno": "0"},
        ),
        ProbeSpec(
            probe_id="srt_standby_option",
            method="POST",
            url="https://app.srail.or.kr:443/ata/selectListAta01135_n.do",
            data={"pnrNo": "0000000000"},
        ),
        ProbeSpec(
            probe_id="srt_payment",
            method="POST",
            url="https://app.srail.or.kr:443/ata/selectListAta09036_n.do",
            data={"pnrNo": "0000000000"},
        ),
        ProbeSpec(
            probe_id="srt_reserve_info",
            method="POST",
            url="https://app.srail.or.kr:443/atc/getListAtc14087.do",
            data={},
        ),
        ProbeSpec(
            probe_id="srt_refund",
            method="POST",
            url="https://app.srail.or.kr:443/atc/selectListAtc02063_n.do",
            data={"pnr_no": "0000000000"},
        ),
        ProbeSpec(
            probe_id="srt_refund_precheck",
            method="POST",
            url="https://app.srail.or.kr:443/atc/selectListAtc02085_n.do",
            data={"pnrNo": "0000000000"},
        ),
        ProbeSpec(
            probe_id="srt_history",
            method="POST",
            url="https://app.srail.or.kr:443/atd/selectListAtd02039_n.do",
            data={"qryNumNext": "0", "dptDtFrom": "20260101", "dptDtTo": "20260228"},
        ),
        ProbeSpec(
            probe_id="srt_schedule_detail",
            method="POST",
            url="https://app.srail.or.kr:443/ara/selectListAra12009_n.do",
            data={"stnCourseNm": "수서-동대구", "trnSort": "SRT", "runDt": "20260228", "trnNo": "00369"},
        ),
        ProbeSpec(
            probe_id="srt_fare_detail",
            method="POST",
            url="https://app.srail.or.kr:443/ara/selectListAra13010_n.do",
            data={
                "stnCourseNm": "수서-동대구",
                "trnSort": "SRT",
                "runDt": "20260228",
                "trnNo": "00369",
                "chtnDvCd": "1",
                "dptRsStnCd1": "0551",
                "arvRsStnCd1": "0015",
                "runDt1": "20260228",
                "trnNo1": "00369",
                "psgTpCd1": "1",
                "psgInfoPerPrnb1": "1",
            },
        ),
        ProbeSpec(
            probe_id="srt_ticket_page",
            method="GET",
            url="https://app.srail.or.kr:443/atc/selectListAtc14017_n.do?pageNo=0",
        ),
        ProbeSpec(
            probe_id="srt_station_info_js",
            method="GET",
            url="https://app.srail.or.kr:443/js/stationInfo.js",
        ),
        ProbeSpec(
            probe_id="srt_bridge_js",
            method="GET",
            url="https://app.srail.or.kr:443/js/common/bridge_new.js",
        ),
        ProbeSpec(
            probe_id="srt_storage_js",
            method="GET",
            url="https://app.srail.or.kr:443/js/common/storage_new.js",
        ),
        ProbeSpec(
            probe_id="srt_messages_js",
            method="GET",
            url="https://app.srail.or.kr:443/js/common/messages.js",
        ),
        ProbeSpec(
            probe_id="srt_util_js",
            method="GET",
            url="https://app.srail.or.kr:443/js/common/util.js",
        ),
        ProbeSpec(
            probe_id="srt_push_js",
            method="GET",
            url="https://app.srail.or.kr:443/js/common/push.js",
        ),
        ProbeSpec(
            probe_id="srt_holidays_js",
            method="GET",
            url="https://app.srail.or.kr:443/js/holidays.js",
        ),
        ProbeSpec(
            probe_id="etk_schedule_popup_get",
            method="GET",
            url=(
                "https://etk.srail.kr/hpg/hra/01/selectTrainScheduleList.do?pageId=TK0101011000"
                "&chtnDvCd=1&dptRsStnCd1=0551&arvRsStnCd1=0015&runDt1=20260228&trnNo1=00369"
                "&stlbTrnClsfCd1=17&psgInfoPerPrnb1=1"
            ),
        ),
        ProbeSpec(
            probe_id="etk_schedule_popup_post",
            method="POST",
            url="https://etk.srail.kr/hpg/hra/01/selectTrainScheduleList.do",
            data={
                "pageId": "TK0101011000",
                "chtnDvCd": "1",
                "dptRsStnCd1": "0551",
                "arvRsStnCd1": "0015",
                "runDt1": "20260228",
                "trnNo1": "00369",
                "stlbTrnClsfCd1": "17",
                "psgInfoPerPrnb1": "1",
            },
        ),
        ProbeSpec(
            probe_id="etk_fare_popup_get",
            method="GET",
            url=(
                "https://etk.srail.kr/hpg/hra/01/selectTrainChargeList.do?pageId=TK0101011000"
                "&chtnDvCd=1&dptRsStnCd1=0551&arvRsStnCd1=0015&runDt1=20260228&trnNo1=00369"
                "&stlbTrnClsfCd1=17&psgInfoPerPrnb1=1"
            ),
        ),
        ProbeSpec(
            probe_id="etk_fare_popup_post",
            method="POST",
            url="https://etk.srail.kr/hpg/hra/01/selectTrainChargeList.do",
            data={
                "pageId": "TK0101011000",
                "chtnDvCd": "1",
                "dptRsStnCd1": "0551",
                "arvRsStnCd1": "0015",
                "runDt1": "20260228",
                "trnNo1": "00369",
                "stlbTrnClsfCd1": "17",
                "psgInfoPerPrnb1": "1",
            },
        ),
        ProbeSpec(
            probe_id="netfunnel_direct",
            method="GET",
            url="https://nf.letskorail.com/ts.wseq?opcode=5101&nfid=0&prefix=NetFunnel.gRtype=5101;&js=true",
        ),
    ]

    if netfunnel_key:
        specs.insert(
            1,
            ProbeSpec(
                probe_id="srt_search_with_netfunnel",
                method="POST",
                url="https://app.srail.or.kr:443/ara/selectListAra10007_n.do",
                data={
                    "chtnDvCd": "1",
                    "dptDt": "20260306",
                    "dptTm": "120000",
                    "dptDt1": "20260306",
                    "dptTm1": "120000",
                    "dptRsStnCd": "0551",
                    "arvRsStnCd": "0015",
                    "stlbTrnClsfCd": "05",
                    "trnGpCd": "109",
                    "psgNum": "1",
                    "seatAttCd": "015",
                    "arriveTime": "N",
                    "dlayTnumAplFlg": "Y",
                    "netfunnelKey": netfunnel_key,
                },
                note="Search probe with runtime netfunnel key.",
                accept="application/json",
            ),
        )

    return specs


def summarize_endpoint(records: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    grouped: Dict[Tuple[str, str], List[Dict[str, Any]]] = {}
    for record in records:
        key = (record["host"], record["path"])
        grouped.setdefault(key, []).append(record)

    summaries: List[Dict[str, Any]] = []

    for (host, path), items in sorted(grouped.items()):
        has_login_marker = any(item.get("has_login_marker") for item in items)
        meaningful_items = [item for item in items if item.get("meaningful_payload") and not item.get("has_login_marker")]
        has_context_signal = any("context_token_missing_or_invalid" in item.get("body_signals", []) for item in items)
        has_netfunnel_probe = any(item.get("probe_id") == "srt_search_with_netfunnel" for item in items)

        auth_label = "unknown"
        evidence: List[Dict[str, str]] = []

        if has_login_marker:
            auth_label = "auth_required"
            source_item = next(item for item in items if item.get("has_login_marker"))
            evidence.append(
                {
                    "source": "probe_response",
                    "signal": "login_marker_in_body",
                    "reference": f"probe_id:{source_item['probe_id']}",
                }
            )
        elif meaningful_items:
            if path == "/ara/selectListAra10007_n.do" and (has_context_signal or has_netfunnel_probe):
                auth_label = "public_context_required"
                source_item = next((item for item in items if item.get("probe_id") == "srt_search_with_netfunnel"), meaningful_items[0])
                evidence.append(
                    {
                        "source": "probe_response",
                        "signal": "netfunnel_key_required",
                        "reference": f"probe_id:{source_item['probe_id']}",
                    }
                )
            else:
                auth_label = "public_stateless"
                source_item = meaningful_items[0]
                evidence.append(
                    {
                        "source": "probe_response",
                        "signal": "business_payload_without_cookie",
                        "reference": f"probe_id:{source_item['probe_id']}",
                    }
                )

        summaries.append(
            {
                "host": host,
                "path": path,
                "auth_scope_label": auth_label,
                "auth_scope_evidence": evidence,
            }
        )

    return summaries


def main() -> None:
    netfunnel_key = acquire_netfunnel_key()
    probe_specs = build_probe_specs(netfunnel_key)
    records = [_request(spec) for spec in probe_specs]
    endpoint_summary = summarize_endpoint(records)

    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_PATH.write_text(
        json.dumps(
            {
                "generated_at": _iso_now(),
                "methodology": {
                    "status_code_not_sufficient": True,
                    "classification_basis": [
                        "response_body_markers",
                        "response_envelope_shape",
                        "payload_meaningfulness",
                    ],
                    "cookie_auth_used": False,
                },
                "netfunnel_key_used": bool(netfunnel_key),
                "records": records,
                "endpoint_summary": endpoint_summary,
            },
            ensure_ascii=False,
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )


if __name__ == "__main__":
    main()
