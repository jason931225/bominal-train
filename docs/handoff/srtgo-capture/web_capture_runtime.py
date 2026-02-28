from __future__ import annotations

import argparse
import json
import re
import uuid
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path
from typing import Any
from urllib.parse import parse_qsl, urlparse

from vendor.srtgo_capture.capture_runtime import (
    _redact_headers,
    _redact_payload,
    _serialize_cookies,
)

REDACTED = "<redacted>"


def _iso_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def _normalize_method(method: Any) -> str:
    if not method:
        return "GET"
    return str(method).upper()


def _safe_endpoint(url: str) -> str:
    parsed = urlparse(url)
    path = (parsed.path or "").strip("/")
    if not path:
        return "root"
    endpoint = path.split("/")[-1]
    endpoint = re.sub(r"[^a-zA-Z0-9_.-]", "_", endpoint)
    return endpoint or "root"


def _component_from_url(url: str) -> str:
    host = (urlparse(url).hostname or "").lower()
    if "srail.kr" in host:
        return "srt_web"
    return "web_unknown"


def _operation_from_url(url: str) -> str:
    endpoint = _safe_endpoint(url).lower()
    mapping = {
        "selecttrainchargelist.do": "fare_charge",
        "selectschedulelist.do": "schedule",
        "selecttimetablelist.do": "schedule",
        "selectcalendarinfo.do": "calendar",
    }
    return mapping.get(endpoint, "unknown")


def _is_provider_relevant(event: dict[str, Any]) -> bool:
    request = event.get("request", {}) or {}
    url = str(request.get("url", "") or "")
    if not url:
        return False

    resource_type = str(event.get("resource_type", "") or "").lower()
    if ".do" in url.lower():
        return True
    return resource_type in {"xhr", "fetch"}


def _coerce_json(text: Any) -> Any:
    if not isinstance(text, str):
        return None
    stripped = text.strip()
    if not stripped:
        return None
    if (stripped.startswith("{") and stripped.endswith("}")) or (
        stripped.startswith("[") and stripped.endswith("]")
    ):
        try:
            return json.loads(stripped)
        except Exception:
            return None
    # x-www-form-urlencoded payload in response text is rare, but parse when present.
    if "=" in stripped and "&" in stripped:
        try:
            pairs = parse_qsl(stripped, keep_blank_values=True)
            if pairs:
                return dict(pairs)
        except Exception:
            return None
    return None


def _event_filename(seq: int, component: str, method: str, endpoint: str) -> str:
    safe_component = re.sub(r"[^a-zA-Z0-9_.-]", "_", component)[:40] or "unknown"
    safe_method = re.sub(r"[^a-zA-Z0-9_.-]", "_", method.upper())[:12] or "GET"
    safe_endpoint = re.sub(r"[^a-zA-Z0-9_.-]", "_", endpoint)[:80] or "root"
    return f"{seq:04d}_{safe_component}_{safe_method}_{safe_endpoint}.json"


def _json_write(path: Path, data: Any) -> None:
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")


def _build_redacted_event(event: dict[str, Any], sequence: int, run_id: str) -> dict[str, Any]:
    request = event.get("request", {}) or {}
    response = event.get("response", {}) or {}
    error = event.get("error")

    method = _normalize_method(request.get("method"))
    url = str(request.get("url", "") or "")
    endpoint = _safe_endpoint(url)
    component = _component_from_url(url)
    operation = _operation_from_url(url)

    response_text = response.get("body")
    response_json = _coerce_json(response_text)

    return {
        "metadata": {
            "sequence": sequence,
            "captured_at": event.get("captured_at") or _iso_now(),
            "run_id": run_id,
            "backend": "playwright",
            "component": component,
            "operation": operation,
            "endpoint": endpoint,
            "resource_type": event.get("resource_type"),
        },
        "request": {
            "method": method,
            "url": url,
            "headers": _redact_headers(request.get("headers")),
            "cookies": _serialize_cookies(request.get("cookies")),
            "query": _redact_payload(request.get("query")),
            "body": _redact_payload(request.get("body")),
        },
        "response": {
            "status_code": response.get("status"),
            "ok": response.get("ok"),
            "headers": _redact_headers(response.get("headers")),
            "cookies": _serialize_cookies(response.get("cookies")),
            "text": _redact_payload(response_text),
            "json": _redact_payload(response_json),
        },
        "error": _redact_payload(error) if error else None,
    }


def export_web_capture(raw_events: list[dict[str, Any]], output_root: Path) -> dict[str, Any]:
    output_root.mkdir(parents=True, exist_ok=True)

    run_id = f"{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}_{uuid.uuid4().hex[:8]}"
    run_dir = output_root / run_id
    run_dir.mkdir(parents=True, exist_ok=True)

    filtered = [event for event in raw_events if _is_provider_relevant(event)]

    index_events: list[dict[str, Any]] = []
    component_counts: Counter[str] = Counter()
    operation_counts: Counter[str] = Counter()
    status_code_counts: Counter[str] = Counter()
    error_count = 0

    for seq, raw in enumerate(filtered, start=1):
        redacted_event = _build_redacted_event(raw, sequence=seq, run_id=run_id)
        metadata = redacted_event["metadata"]

        method = redacted_event["request"]["method"]
        endpoint = str(metadata.get("endpoint") or "root")
        component = str(metadata.get("component") or "unknown")
        status_code = redacted_event["response"].get("status_code")
        has_error = bool(redacted_event.get("error"))

        filename = _event_filename(seq, component, method, endpoint)
        _json_write(run_dir / filename, redacted_event)

        component_counts[component] += 1
        operation_counts[str(metadata.get("operation") or "unknown")] += 1
        if status_code is not None:
            status_code_counts[str(status_code)] += 1
        if has_error:
            error_count += 1

        index_events.append(
            {
                "sequence": seq,
                "file": filename,
                "component": component,
                "operation": metadata.get("operation"),
                "method": method,
                "url": redacted_event["request"]["url"],
                "status_code": status_code,
                "error": "error" if has_error else None,
            }
        )

    step_boundaries: list[dict[str, Any]] = []
    current: dict[str, Any] | None = None
    for event in index_events:
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
        "run_id": run_id,
        "started_at": _iso_now(),
        "finished_at": _iso_now(),
        "event_count": len(index_events),
        "summary": {
            "component_counts": dict(component_counts),
            "operation_counts": dict(operation_counts),
            "status_code_counts": dict(status_code_counts),
            "error_count": error_count,
        },
        "step_boundaries": step_boundaries,
        "events": index_events,
    }
    _json_write(run_dir / "web_run_index.json", index)

    return {
        "run_id": run_id,
        "run_dir": str(run_dir),
        "event_count": len(index_events),
        "index_path": str(run_dir / "web_run_index.json"),
    }


def main() -> None:
    parser = argparse.ArgumentParser(description="Export redacted Playwright web capture events.")
    parser.add_argument("--input", required=True, help="Path to raw events JSON file.")
    parser.add_argument(
        "--output-root",
        default=str(Path(__file__).resolve().parent / "output" / "web_capture"),
        help="Output root for web capture runs.",
    )
    args = parser.parse_args()

    raw = json.loads(Path(args.input).read_text(encoding="utf-8"))
    if not isinstance(raw, list):
        raise SystemExit("--input must contain a JSON array of raw event objects")

    result = export_web_capture(raw_events=raw, output_root=Path(args.output_root))
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
