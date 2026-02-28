from __future__ import annotations

import atexit
import json
import os
import re
import threading
import traceback
import uuid
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable, Mapping
from urllib.parse import parse_qsl, urlencode, urlparse

REDACTED = "<redacted>"

# Sensitive payload keys (case-insensitive, normalized)
SENSITIVE_PAYLOAD_EXACT = {
    "password",
    "pass",
    "hmpgpwdcphd",
    "txtpwd",
    "cardnumber",
}

SENSITIVE_PAYLOAD_SUBSTR = {
    "stlcrcrdno",
    "hidstlcrcrdno",
}

SENSITIVE_PAYLOAD_REGEX = [
    re.compile(r"(?:hid)?vanpwd\d*$", re.IGNORECASE),
    re.compile(r"(?:hid)?athnval\d*$", re.IGNORECASE),
]

SENSITIVE_CARD_CONTEXT_TOKENS = (
    "card",
    "crd",
    "credit",
    "stlcrcrd",
    "hidstlcrcrd",
)

SENSITIVE_HEADER_EXACT = {
    "authorization",
    "proxy-authorization",
    "x-auth-token",
    "x-access-token",
    "x-api-key",
    "id-token",
    "cookie",
    "set-cookie",
}

_OPERATION_BY_ENDPOINT = {
    # SRT
    "selectlistapb01080_n.do": "login",
    "loginout.do": "logout",
    "selectlistara10007_n.do": "search_schedule",
    "selectlistarc05013_n.do": "reserve",
    "selectlistatc14016_n.do": "tickets",
    "selectlistard02019_n.do": "ticket_info",
    "selectlistard02017_n.do": "ticket_info",
    "selectlistard02045_n.do": "cancel",
    "selectlistata01135_n.do": "standby_option",
    "selectlistata09036_n.do": "payment",
    "getlistatc14087.do": "reserve_info",
    "selectlistatc02063_n.do": "refund",
    # KTX/Korail
    "login": "login",
    "logout": "logout",
    "scheduleview": "search_schedule",
    "ticketreservation": "reserve",
    "reservationcancelchk": "cancel",
    "selticketinfo": "myticketseat",
    "myticketlist": "myticketlist",
    "reservationview": "myreservationview",
    "reservationlist": "myreservationlist",
    "reservationpayment": "pay",
    "refundsrequest": "refund",
    "code.do": "code",
    # NetFunnel
    "ts.wseq": "netfunnel",
}


def _normalize_key(key: str) -> str:
    return re.sub(r"[^a-z0-9]", "", key.lower())


def _iso_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def _is_sensitive_payload_key(key: str) -> bool:
    norm = _normalize_key(key)
    if norm in SENSITIVE_PAYLOAD_EXACT:
        return True
    if any(substr in norm for substr in SENSITIVE_PAYLOAD_SUBSTR):
        return True
    return any(pattern.fullmatch(norm) for pattern in SENSITIVE_PAYLOAD_REGEX)


def _is_sensitive_payload_key_with_context(
    key: str,
    parent_path: tuple[str, ...],
    sibling_keys: tuple[str, ...],
) -> bool:
    if _is_sensitive_payload_key(key):
        return True

    norm = _normalize_key(key)
    if norm != "number":
        return False

    context = parent_path + sibling_keys
    return any(
        token and any(marker in token for marker in SENSITIVE_CARD_CONTEXT_TOKENS)
        for token in context
    )


def _is_sensitive_header_key(key: str) -> bool:
    lk = key.lower()
    return lk in SENSITIVE_HEADER_EXACT or "token" in lk


def _to_jsonable(value: Any) -> Any:
    if value is None or isinstance(value, (bool, int, float, str)):
        return value
    if isinstance(value, bytes):
        return value.decode("utf-8", errors="replace")
    if isinstance(value, Path):
        return str(value)
    if isinstance(value, Mapping):
        return {str(k): _to_jsonable(v) for k, v in value.items()}
    if isinstance(value, (list, tuple, set)):
        return [_to_jsonable(v) for v in value]
    return repr(value)


def _redact_string_payload(text: str, parent_path: tuple[str, ...]) -> str:
    stripped = text.strip()
    if not stripped:
        return text

    if (stripped.startswith("{") and stripped.endswith("}")) or (
        stripped.startswith("[") and stripped.endswith("]")
    ):
        try:
            parsed = json.loads(stripped)
            redacted = _redact_payload(parsed, parent_path=parent_path)
            return json.dumps(redacted, ensure_ascii=False)
        except Exception:
            return text

    if "=" in text:
        try:
            pairs = parse_qsl(text, keep_blank_values=True)
        except Exception:
            return text
        if pairs:
            sibling_keys = tuple(_normalize_key(k) for k, _ in pairs)
            redacted_pairs: list[tuple[str, str]] = []
            for key, value in pairs:
                if _is_sensitive_payload_key_with_context(
                    key,
                    parent_path=parent_path,
                    sibling_keys=sibling_keys,
                ):
                    redacted_pairs.append((key, REDACTED))
                else:
                    redacted_pairs.append((key, value))
            return urlencode(redacted_pairs, doseq=True)

    return text


def _redact_payload(value: Any, parent_path: tuple[str, ...] = ()) -> Any:
    jsonable = _to_jsonable(value)

    if isinstance(jsonable, Mapping):
        sibling_keys = tuple(_normalize_key(str(k)) for k in jsonable.keys())
        redacted: dict[str, Any] = {}
        for key, raw_val in jsonable.items():
            key_s = str(key)
            key_norm = _normalize_key(key_s)
            if _is_sensitive_payload_key_with_context(
                key_s,
                parent_path=parent_path,
                sibling_keys=sibling_keys,
            ):
                redacted[key_s] = REDACTED
            else:
                redacted[key_s] = _redact_payload(
                    raw_val,
                    parent_path=parent_path + (key_norm,),
                )
        return redacted

    if isinstance(jsonable, list):
        return [_redact_payload(v, parent_path=parent_path) for v in jsonable]

    if isinstance(jsonable, str):
        return _redact_string_payload(jsonable, parent_path=parent_path)

    return jsonable


def _redact_headers(headers: Any) -> dict[str, Any]:
    if not headers:
        return {}

    try:
        header_map = dict(headers)
    except Exception:
        return {"_unparsed_headers": _to_jsonable(headers)}

    redacted: dict[str, Any] = {}
    for key, value in header_map.items():
        key_s = str(key)
        if _is_sensitive_header_key(key_s):
            redacted[key_s] = REDACTED
        else:
            redacted[key_s] = _to_jsonable(value)
    return redacted


def _serialize_cookies(cookie_jar: Any) -> list[dict[str, Any]]:
    if cookie_jar is None:
        return []

    cookies: list[dict[str, Any]] = []
    try:
        for cookie in cookie_jar:
            cookies.append(
                {
                    "name": getattr(cookie, "name", None),
                    "value": REDACTED,
                    "domain": getattr(cookie, "domain", None),
                    "path": getattr(cookie, "path", None),
                    "secure": getattr(cookie, "secure", None),
                    "expires": getattr(cookie, "expires", None),
                }
            )
        return cookies
    except Exception:
        pass

    try:
        as_dict = dict(cookie_jar)
    except Exception:
        return [{"raw": _to_jsonable(cookie_jar), "value": REDACTED}]

    for name in as_dict.keys():
        cookies.append({"name": str(name), "value": REDACTED})
    return cookies


def _component_from_url(url: str) -> str:
    parsed = urlparse(url)
    host = (parsed.hostname or "").lower()
    path = parsed.path.lower()

    if "srail.or.kr" in host:
        return "srt"
    if host.startswith("nf") and "letskorail.com" in host:
        return "netfunnel"
    if "letskorail.com" in host and path.endswith("/ts.wseq"):
        return "netfunnel"
    if "smart.letskorail.com" in host:
        return "ktx"
    if "letskorail.com" in host:
        return "ktx"
    return "unknown"


def _endpoint_from_url(url: str) -> str:
    parsed = urlparse(url)
    path = parsed.path.strip("/")
    if not path:
        return "root"
    return path.split("/")[-1]


def _operation_from_url(url: str) -> str:
    endpoint = _endpoint_from_url(url).lower()

    if endpoint in _OPERATION_BY_ENDPOINT:
        return _OPERATION_BY_ENDPOINT[endpoint]

    # KTX endpoint format often ends with ClassName suffixes
    for suffix, op in _OPERATION_BY_ENDPOINT.items():
        if endpoint.endswith(suffix):
            return op
    return "unknown"


def _safe_response_text(response: Any) -> str | None:
    try:
        return response.text
    except Exception:
        return None


class CaptureRecorder:
    def __init__(self, output_root: str | Path | None = None) -> None:
        root = Path(output_root or os.environ.get("SRTGO_CAPTURE_OUTPUT_DIR") or "")
        if not root:
            root = Path(__file__).resolve().parents[2] / "output"

        self.output_root = root
        self.output_root.mkdir(parents=True, exist_ok=True)

        ts = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
        self.run_id = f"{ts}_{uuid.uuid4().hex[:8]}"
        self.run_dir = self.output_root / self.run_id
        self.run_dir.mkdir(parents=True, exist_ok=True)

        self._sequence = 0
        self._lock = threading.Lock()
        self._events: list[dict[str, Any]] = []
        self.started_at = _iso_now()
        self.finished_at: str | None = None

        self._write_index()

    def finalize(self) -> None:
        self.finished_at = _iso_now()
        self._write_index()

    def _next_sequence(self) -> int:
        with self._lock:
            self._sequence += 1
            return self._sequence

    def _event_filename(self, seq: int, component: str, method: str, endpoint: str) -> str:
        safe_endpoint = re.sub(r"[^a-zA-Z0-9_.-]", "_", endpoint)[:80] or "root"
        safe_component = re.sub(r"[^a-zA-Z0-9_.-]", "_", component)[:40] or "unknown"
        safe_method = re.sub(r"[^a-zA-Z0-9_.-]", "_", method.upper())
        return f"{seq:04d}_{safe_component}_{safe_method}_{safe_endpoint}.json"

    def _write_json(self, path: Path, data: Any) -> None:
        path.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")

    def _write_index(self) -> None:
        component_counts: dict[str, int] = dict(
            Counter((event.get("component") or "unknown") for event in self._events)
        )
        operation_counts: dict[str, int] = dict(
            Counter((event.get("operation") or "unknown") for event in self._events)
        )
        status_code_counts: dict[str, int] = dict(
            Counter(
                str(event.get("status_code"))
                for event in self._events
                if event.get("status_code") is not None
            )
        )
        error_count = sum(1 for event in self._events if event.get("error"))

        step_boundaries: list[dict[str, Any]] = []
        current: dict[str, Any] | None = None
        for event in self._events:
            op = event.get("operation") or "unknown"
            seq = int(event.get("sequence") or 0)
            component = event.get("component") or "unknown"

            if current is None:
                current = {
                    "operation": op,
                    "component": component,
                    "start_sequence": seq,
                    "end_sequence": seq,
                    "event_count": 1,
                }
                continue

            if op == current["operation"]:
                current["end_sequence"] = seq
                current["event_count"] += 1
            else:
                step_boundaries.append(current)
                current = {
                    "operation": op,
                    "component": component,
                    "start_sequence": seq,
                    "end_sequence": seq,
                    "event_count": 1,
                }

        if current is not None:
            step_boundaries.append(current)

        index = {
            "run_id": self.run_id,
            "started_at": self.started_at,
            "finished_at": self.finished_at,
            "event_count": len(self._events),
            "summary": {
                "component_counts": component_counts,
                "operation_counts": operation_counts,
                "status_code_counts": status_code_counts,
                "error_count": error_count,
            },
            "step_boundaries": step_boundaries,
            "events": self._events,
        }
        self._write_json(self.run_dir / "run_index.json", index)

    def _request_snapshot(
        self,
        session: Any,
        method: str,
        url: str,
        kwargs: dict[str, Any],
    ) -> dict[str, Any]:
        request = {
            "method": method.upper(),
            "url": url,
            "params": _redact_payload(kwargs.get("params")),
            "data": _redact_payload(kwargs.get("data")),
            "json": _redact_payload(kwargs.get("json")),
            "headers": _redact_headers(kwargs.get("headers")),
            "cookies": _serialize_cookies(kwargs.get("cookies")),
            "session_headers": _redact_headers(getattr(session, "headers", {})),
            "session_cookies": _serialize_cookies(getattr(session, "cookies", None)),
            "extra": _redact_payload(
                {
                    k: v
                    for k, v in kwargs.items()
                    if k
                    not in {
                        "params",
                        "data",
                        "json",
                        "headers",
                        "cookies",
                    }
                }
            ),
        }
        return request

    def _effective_request_snapshot(self, response: Any) -> dict[str, Any] | None:
        req = getattr(response, "request", None)
        if req is None:
            return None

        body = getattr(req, "body", None)
        if isinstance(body, bytes):
            body = body.decode("utf-8", errors="replace")

        return {
            "method": _to_jsonable(getattr(req, "method", None)),
            "url": _to_jsonable(getattr(req, "url", None)),
            "headers": _redact_headers(getattr(req, "headers", {})),
            "body": _redact_payload(body),
        }

    def _response_snapshot(self, response: Any) -> dict[str, Any]:
        text = _safe_response_text(response)

        parsed_json = None
        if text is not None:
            try:
                parsed_json = _redact_payload(json.loads(text))
            except Exception:
                parsed_json = None

        return {
            "status_code": _to_jsonable(getattr(response, "status_code", None)),
            "ok": _to_jsonable(getattr(response, "ok", None)),
            "url": _to_jsonable(getattr(response, "url", None)),
            "headers": _redact_headers(getattr(response, "headers", {})),
            "cookies": _serialize_cookies(getattr(response, "cookies", None)),
            "text": _redact_payload(text),
            "json": parsed_json,
        }

    def record_call(
        self,
        backend: str,
        session: Any,
        method: str,
        url: str,
        kwargs: dict[str, Any],
        call: Callable[[], Any],
    ) -> Any:
        seq = self._next_sequence()
        component = _component_from_url(url)
        endpoint = _endpoint_from_url(url)
        operation = _operation_from_url(url)

        event = {
            "metadata": {
                "sequence": seq,
                "captured_at": _iso_now(),
                "run_id": self.run_id,
                "backend": backend,
                "component": component,
                "operation": operation,
                "endpoint": endpoint,
            },
            "request": self._request_snapshot(session, method, url, kwargs),
        }

        try:
            response = call()
        except Exception as exc:
            event["error"] = {
                "type": exc.__class__.__name__,
                "message": str(exc),
                "traceback": traceback.format_exc(limit=6),
            }

            filename = self._event_filename(seq, component, method, endpoint)
            self._write_json(self.run_dir / filename, event)
            self._events.append(
                {
                    "sequence": seq,
                    "file": filename,
                    "component": component,
                    "operation": operation,
                    "method": method.upper(),
                    "url": url,
                    "status_code": None,
                    "error": exc.__class__.__name__,
                }
            )
            self._write_index()
            raise

        event["effective_request"] = self._effective_request_snapshot(response)
        event["response"] = self._response_snapshot(response)

        filename = self._event_filename(seq, component, method, endpoint)
        self._write_json(self.run_dir / filename, event)

        self._events.append(
            {
                "sequence": seq,
                "file": filename,
                "component": component,
                "operation": operation,
                "method": method.upper(),
                "url": url,
                "status_code": event["response"].get("status_code"),
                "error": None,
            }
        )
        self._write_index()
        return response


_recorder: CaptureRecorder | None = None
_installed = False


def install_capture(output_root: str | Path | None = None) -> CaptureRecorder:
    global _recorder, _installed

    if _installed and _recorder is not None:
        return _recorder

    recorder = CaptureRecorder(output_root=output_root)
    installed_any = False

    try:
        import requests

        original_requests_request = requests.sessions.Session.request

        def wrapped_requests_request(self: Any, method: str, url: str, **kwargs: Any) -> Any:
            return recorder.record_call(
                backend="requests",
                session=self,
                method=method,
                url=url,
                kwargs=dict(kwargs),
                call=lambda: original_requests_request(self, method, url, **kwargs),
            )

        wrapped_requests_request.__srtgo_capture_wrapped__ = True  # type: ignore[attr-defined]
        requests.sessions.Session.request = wrapped_requests_request
        requests.Session.request = wrapped_requests_request
        installed_any = True
    except Exception:
        pass

    try:
        import curl_cffi

        original_curl_request = curl_cffi.Session.request

        def wrapped_curl_request(self: Any, method: str, url: str, **kwargs: Any) -> Any:
            return recorder.record_call(
                backend="curl_cffi",
                session=self,
                method=method,
                url=url,
                kwargs=dict(kwargs),
                call=lambda: original_curl_request(self, method, url, **kwargs),
            )

        wrapped_curl_request.__srtgo_capture_wrapped__ = True  # type: ignore[attr-defined]
        curl_cffi.Session.request = wrapped_curl_request
        installed_any = True
    except Exception:
        pass

    if not installed_any:  # pragma: no cover
        raise RuntimeError("Either requests or curl_cffi is required for capture runtime")

    _recorder = recorder
    _installed = True

    def _finalize() -> None:
        if _recorder is not None:
            _recorder.finalize()

    atexit.register(_finalize)
    return recorder
