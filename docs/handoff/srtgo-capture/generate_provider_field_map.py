#!/usr/bin/env python3
from __future__ import annotations

import ast
import json
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Tuple
from urllib.parse import parse_qsl, urlparse

BASE_DIR = Path(__file__).resolve().parent
REPO_ROOT = BASE_DIR.parent.parent
OUTPUT_DIR = BASE_DIR / "output"
CONTRACT_PATH = BASE_DIR / "PROVIDER_CONTRACT.md"
FIELD_MAP_JSON_PATH = BASE_DIR / "PROVIDER_FIELD_MAP.json"
FIELD_MAP_MD_PATH = BASE_DIR / "PROVIDER_FIELD_MAP.md"
AUTH_SCOPE_PROBE_PATH = OUTPUT_DIR / "auth_scope_probe" / "latest_auth_scope_probe.json"
FLOW_PATH = Path("/Users/jasonlee/Downloads/flows")

SOURCE_FILES = [
    REPO_ROOT / "third_party/srtgo/srtgo/srt.py",
    REPO_ROOT / "third_party/srt/SRT/srt.py",
    REPO_ROOT / "third_party/srtgo/srtgo/ktx.py",
    REPO_ROOT / "third_party/korail2/korail2/korail2.py",
]

USER_CURL_SOURCES = [
    "user-curl-2026-02-28:/js/stationInfo.js",
    "user-curl-2026-02-28:/ara/selectListAra12009_n.do",
    "user-curl-2026-02-28:/ara/selectListAra13010_n.do",
    "user-curl-2026-02-28:/atd/selectListAtd02039_n.do",
    "user-curl-2026-02-28:/atc/selectListAtc14017_n.do",
    "user-curl-2026-02-28:/js/common/bridge_new.js",
    "user-curl-2026-02-28:/js/common/storage_new.js",
    "user-curl-2026-02-28:/js/common/messages.js",
    "user-curl-2026-02-28:/js/common/util.js",
    "user-curl-2026-02-28:/js/common/push.js",
    "user-curl-2026-02-28:/js/holidays.js",
    "user-curl-2026-02-28:/atc/selectListAtc02A01_n.do",
    "user-curl-2026-02-28:/ata/selectListAta04A01_n.do",
    "user-curl-2026-02-28:/push/surveyPush.do",
]

TIER_PRIORITY = {"unknown": 0, "mapped": 1, "high_signal": 2, "critical": 3}
CONFIDENCE_PRIORITY = {"unresolved": 0, "inferred": 1, "confirmed": 2}
AUTH_SCOPE_LABELS = {"public_stateless", "public_context_required", "auth_required", "unknown"}

PUBLIC_CONTEXT_REQUIRED_SEEDS = {
    ("nf.letskorail.com", "/ts.wseq"),
    ("app.srail.or.kr", "/ara/selectListAra10007_n.do"),
}

PUBLIC_STATELESS_SEEDS = {
    ("app.srail.or.kr", "/ara/selectListAra12009_n.do"),
    ("app.srail.or.kr", "/ara/selectListAra13010_n.do"),
    ("app.srail.or.kr", "/js/stationInfo.js"),
    ("app.srail.or.kr", "/js/common/bridge_new.js"),
    ("app.srail.or.kr", "/js/common/storage_new.js"),
    ("app.srail.or.kr", "/js/common/messages.js"),
    ("app.srail.or.kr", "/js/common/util.js"),
    ("app.srail.or.kr", "/js/common/push.js"),
    ("app.srail.or.kr", "/js/holidays.js"),
}

AUTH_REQUIRED_SEEDS = {
    ("app.srail.or.kr", "/arc/selectListArc05013_n.do"),
    ("app.srail.or.kr", "/atc/selectListAtc14016_n.do"),
    ("app.srail.or.kr", "/atc/selectListAtc14017_n.do"),
    ("app.srail.or.kr", "/ard/selectListArd02019_n.do"),
    ("app.srail.or.kr", "/ard/selectListArd02045_n.do"),
    ("app.srail.or.kr", "/ata/selectListAta01135_n.do"),
    ("app.srail.or.kr", "/ata/selectListAta09036_n.do"),
    ("app.srail.or.kr", "/atc/getListAtc14087.do"),
    ("app.srail.or.kr", "/atc/selectListAtc02063_n.do"),
    ("app.srail.or.kr", "/atc/selectListAtc02085_n.do"),
    ("app.srail.or.kr", "/atd/selectListAtd02039_n.do"),
    ("app.srail.or.kr", "/login/loginOut.do"),
}

FORCE_UNKNOWN = {
    ("etk.srail.kr", "/hpg/hra/01/selectTrainScheduleList.do"),
}


@dataclass
class FieldRecord:
    path: str
    name: str
    endpoint_tier_label: str
    field_confidence_label: str
    observed_in: set[str] = field(default_factory=set)
    source_of_truth: set[str] = field(default_factory=set)

    def merge(self, confidence: str, observed: Iterable[str], sources: Iterable[str], endpoint_tier: str) -> None:
        if CONFIDENCE_PRIORITY.get(confidence, 0) > CONFIDENCE_PRIORITY.get(self.field_confidence_label, 0):
            self.field_confidence_label = confidence
        if TIER_PRIORITY.get(endpoint_tier, 0) > TIER_PRIORITY.get(self.endpoint_tier_label, 0):
            self.endpoint_tier_label = endpoint_tier
        self.observed_in.update(observed)
        self.source_of_truth.update(sources)


@dataclass
class EndpointRecord:
    host: str
    path: str
    endpoint_tier_label: str
    methods: set[str] = field(default_factory=set)
    request_fields: Dict[str, FieldRecord] = field(default_factory=dict)
    response_fields: Dict[str, FieldRecord] = field(default_factory=dict)
    source_only_fields: Dict[str, FieldRecord] = field(default_factory=dict)
    auth_scope_label: str = "unknown"
    auth_scope_evidence: List[Dict[str, str]] = field(default_factory=list)


def repo_rel(path: Path) -> str:
    try:
        return str(path.relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


def normalize_path(path: str) -> str:
    if not path:
        return "/"
    path = path.strip()
    if path.startswith("http://") or path.startswith("https://"):
        parsed = urlparse(path)
        return parsed.path or "/"
    return path


def infer_host(path: str) -> str:
    if path.startswith(".") or path.startswith("/classes/com.korail.mobile"):
        return "smart.letskorail.com"
    if path == "/ts.wseq":
        return "nf.letskorail.com"
    if path.startswith("/hpg/"):
        return "etk.srail.kr"
    return "app.srail.or.kr"


def parse_contract_tiers(contract_text: str) -> Dict[str, str]:
    endpoint_tiers: Dict[str, str] = {}
    current_tier: Optional[str] = None

    for line in contract_text.splitlines():
        stripped = line.strip()
        if stripped.startswith("### 10.1"):
            current_tier = "critical"
        elif stripped.startswith("### 10.2"):
            current_tier = "high_signal"
        elif stripped.startswith("### 10.3"):
            current_tier = "high_signal"
        elif stripped.startswith("### 10.4"):
            current_tier = "mapped"
        elif stripped.startswith("### 10.5"):
            current_tier = "unknown"
        elif stripped.startswith("## 11"):
            current_tier = None

        if current_tier and stripped.startswith("-"):
            for token in re.findall(r"`([^`]+)`", line):
                if token.startswith("/") or token.startswith("."):
                    endpoint_tiers[token] = current_tier

    return endpoint_tiers


def flatten_fields(value: Any, prefix: str) -> Iterable[Tuple[str, str]]:
    if isinstance(value, dict):
        for key, nested in value.items():
            key_str = str(key)
            path = f"{prefix}.{key_str}" if prefix else key_str
            yield path, key_str
            yield from flatten_fields(nested, path)
    elif isinstance(value, list):
        wildcard = f"{prefix}[*]" if prefix else "[*]"
        yield wildcard, "*"
        for item in value:
            yield from flatten_fields(item, wildcard)


def ensure_endpoint(
    endpoints: Dict[Tuple[str, str], EndpointRecord],
    host: str,
    path: str,
    endpoint_tiers: Dict[str, str],
    default_tier: str = "unknown",
) -> EndpointRecord:
    path = normalize_path(path)
    key = (host, path)
    tier = endpoint_tiers.get(path, default_tier)

    if key not in endpoints:
        endpoints[key] = EndpointRecord(host=host, path=path, endpoint_tier_label=tier)
    else:
        existing = endpoints[key]
        if TIER_PRIORITY.get(tier, 0) > TIER_PRIORITY.get(existing.endpoint_tier_label, 0):
            existing.endpoint_tier_label = tier

    return endpoints[key]


def add_field(
    endpoint: EndpointRecord,
    section: str,
    field_path: str,
    field_name: str,
    confidence: str,
    observed: Iterable[str],
    evidence: Iterable[str],
) -> None:
    target: Dict[str, FieldRecord] = getattr(endpoint, section)
    if field_path not in target:
        target[field_path] = FieldRecord(
            path=field_path,
            name=field_name,
            endpoint_tier_label=endpoint.endpoint_tier_label,
            field_confidence_label=confidence,
            observed_in=set(observed),
            source_of_truth=set(evidence),
        )
    else:
        target[field_path].merge(
            confidence=confidence,
            observed=observed,
            sources=evidence,
            endpoint_tier=endpoint.endpoint_tier_label,
        )


def safe_json_loads(text: str) -> Optional[Any]:
    if not isinstance(text, str):
        return None
    probe = text.strip()
    if not probe:
        return None
    if probe[0] not in "[{":
        return None
    try:
        return json.loads(probe)
    except Exception:
        return None


def parse_capture_files(
    endpoints: Dict[Tuple[str, str], EndpointRecord],
    endpoint_tiers: Dict[str, str],
) -> Tuple[List[str], List[str]]:
    capture_sources: List[str] = []
    captured_timestamps: List[str] = []

    capture_files = sorted(p for p in OUTPUT_DIR.glob("*/*.json") if p.name != "run_index.json")
    for cap in capture_files:
        try:
            payload = json.loads(cap.read_text(encoding="utf-8"))
        except Exception:
            continue

        capture_sources.append(repo_rel(cap))
        captured_at = payload.get("metadata", {}).get("captured_at")
        if isinstance(captured_at, str) and captured_at:
            captured_timestamps.append(captured_at)

        request = payload.get("request", {}) or {}
        effective_request = payload.get("effective_request", {}) or {}
        response = payload.get("response", {}) or {}

        request_url = request.get("url") or effective_request.get("url") or response.get("url")
        if not isinstance(request_url, str) or not request_url:
            continue

        parsed = urlparse(request_url)
        host = parsed.hostname or infer_host(parsed.path)
        path = parsed.path or "/"
        endpoint = ensure_endpoint(endpoints, host, path, endpoint_tiers)

        method = request.get("method") or effective_request.get("method")
        if isinstance(method, str) and method:
            endpoint.methods.add(method.upper())

        evidence_ref = [repo_rel(cap)]
        observed_ref = ["capture"]

        for qk, _ in parse_qsl(parsed.query, keep_blank_values=True):
            add_field(
                endpoint,
                "request_fields",
                f"request.url_query.{qk}",
                qk,
                "confirmed",
                observed_ref,
                evidence_ref,
            )

        for req_block, prefix in (("params", "request.params"), ("data", "request.data"), ("json", "request.json")):
            block_value = request.get(req_block)
            if block_value is None:
                continue
            for field_path, field_name in flatten_fields(block_value, prefix):
                add_field(endpoint, "request_fields", field_path, field_name, "confirmed", observed_ref, evidence_ref)

        for req_header_block, prefix in (("headers", "request.headers"), ("session_headers", "request.session_headers")):
            block = request.get(req_header_block)
            if not isinstance(block, dict):
                continue
            for key in sorted(block):
                normalized = str(key).lower()
                add_field(
                    endpoint,
                    "request_fields",
                    f"{prefix}.{normalized}",
                    normalized,
                    "confirmed",
                    observed_ref,
                    evidence_ref,
                )

        response_headers = response.get("headers")
        if isinstance(response_headers, dict):
            for key in sorted(response_headers):
                normalized = str(key).lower()
                add_field(
                    endpoint,
                    "response_fields",
                    f"response.headers.{normalized}",
                    normalized,
                    "confirmed",
                    observed_ref,
                    evidence_ref,
                )

        response_json = response.get("json")
        if response_json is None:
            response_json = safe_json_loads(response.get("text", ""))

        if response_json is not None:
            for field_path, field_name in flatten_fields(response_json, "response.json"):
                add_field(endpoint, "response_fields", field_path, field_name, "confirmed", observed_ref, evidence_ref)
        else:
            text_body = response.get("text", "")
            if isinstance(text_body, str) and "NetFunnel.gControl.result" in text_body:
                add_field(
                    endpoint,
                    "response_fields",
                    "response.text.netfunnel.result",
                    "result",
                    "confirmed",
                    observed_ref,
                    evidence_ref,
                )
                result_match = re.search(r"result='([^']+)'", text_body)
                if result_match:
                    result_payload = result_match.group(1)
                    parts = result_payload.split(":", 2)
                    if len(parts) >= 2:
                        add_field(endpoint, "response_fields", "response.text.netfunnel.code", "code", "confirmed", observed_ref, evidence_ref)
                        add_field(endpoint, "response_fields", "response.text.netfunnel.status", "status", "confirmed", observed_ref, evidence_ref)
                    if len(parts) == 3:
                        for qk, _ in parse_qsl(parts[2], keep_blank_values=True):
                            add_field(
                                endpoint,
                                "response_fields",
                                f"response.text.netfunnel.{qk}",
                                qk,
                                "confirmed",
                                observed_ref,
                                evidence_ref,
                            )

    return sorted(set(capture_sources)), sorted(set(captured_timestamps))


def extract_api_endpoints_from_text(source_text: str) -> Dict[str, str]:
    constants: Dict[str, str] = {}
    for name, value in re.findall(r"^([A-Z_]+)\s*=\s*[\"']([^\"']+)[\"']", source_text, flags=re.MULTILINE):
        constants[name] = value

    match = re.search(r"API_ENDPOINTS\s*=\s*\{", source_text)
    if not match:
        return {}

    start = match.end() - 1
    depth = 0
    end = start
    for idx in range(start, len(source_text)):
        char = source_text[idx]
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                end = idx
                break

    body = source_text[start + 1 : end]
    endpoints: Dict[str, str] = {}
    for key, raw_value in re.findall(r"[\"']([^\"']+)[\"']\s*:\s*f?[\"']([^\"']+)[\"']", body):
        value = raw_value
        for const_name, const_value in constants.items():
            value = value.replace("{" + const_name + "}", const_value)
        endpoints[key] = value

    return endpoints


def dict_literal_keys(expr: ast.AST) -> List[str]:
    if not isinstance(expr, ast.Dict):
        return []
    keys: List[str] = []
    for key in expr.keys:
        if isinstance(key, ast.Constant) and isinstance(key.value, str):
            keys.append(key.value)
    return keys


def extract_api_key(expr: ast.AST) -> Optional[str]:
    if not isinstance(expr, ast.Subscript):
        return None

    val = expr.value
    if isinstance(val, ast.Name) and val.id == "API_ENDPOINTS":
        pass
    elif isinstance(val, ast.Attribute) and val.attr == "API_ENDPOINTS":
        pass
    else:
        return None

    slc = expr.slice
    if isinstance(slc, ast.Constant) and isinstance(slc.value, str):
        return slc.value
    if isinstance(slc, ast.Index) and isinstance(slc.value, ast.Constant) and isinstance(slc.value.value, str):  # pragma: no cover
        return slc.value.value
    return None


def parse_source_code_fields(
    endpoints: Dict[Tuple[str, str], EndpointRecord],
    endpoint_tiers: Dict[str, str],
) -> List[str]:
    source_refs: List[str] = []

    for source_file in SOURCE_FILES:
        if not source_file.exists():
            continue
        rel = repo_rel(source_file)
        source_refs.append(rel)

        source_text = source_file.read_text(encoding="utf-8")
        api_endpoints = extract_api_endpoints_from_text(source_text)

        try:
            tree = ast.parse(source_text, filename=str(source_file))
        except SyntaxError:
            continue

        for node in ast.walk(tree):
            if not isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                continue

            assignments: Dict[str, List[Tuple[int, List[str]]]] = {}
            for inner in ast.walk(node):
                if not isinstance(inner, ast.Assign):
                    continue
                if len(inner.targets) != 1 or not isinstance(inner.targets[0], ast.Name):
                    continue
                name = inner.targets[0].id
                keys = dict_literal_keys(inner.value)
                if not keys:
                    continue
                assignments.setdefault(name, []).append((inner.lineno, keys))

            for calls in ast.walk(node):
                if not isinstance(calls, ast.Call):
                    continue
                if not isinstance(calls.func, ast.Attribute):
                    continue
                method = calls.func.attr.lower()
                if method not in {"get", "post"}:
                    continue

                url_expr = None
                for kw in calls.keywords:
                    if kw.arg == "url":
                        url_expr = kw.value
                        break
                if url_expr is None and calls.args:
                    url_expr = calls.args[0]

                if url_expr is None:
                    continue

                endpoint_url: Optional[str] = None
                endpoint_key = extract_api_key(url_expr)
                if endpoint_key and endpoint_key in api_endpoints:
                    endpoint_url = api_endpoints[endpoint_key]
                elif isinstance(url_expr, ast.Constant) and isinstance(url_expr.value, str):
                    endpoint_url = url_expr.value

                if not endpoint_url:
                    continue

                if endpoint_url.startswith("http://") or endpoint_url.startswith("https://"):
                    parsed = urlparse(endpoint_url)
                    host = parsed.hostname or infer_host(parsed.path)
                    path = parsed.path or endpoint_url
                elif endpoint_url.startswith("/"):
                    host = infer_host(endpoint_url)
                    path = endpoint_url
                else:
                    host = "smart.letskorail.com"
                    path = endpoint_url if endpoint_url.startswith(".") else f"/{endpoint_url}"

                default_tier = "mapped" if "third_party/srtgo" not in str(source_file) else "unknown"
                endpoint = ensure_endpoint(endpoints, host, path, endpoint_tiers, default_tier=default_tier)
                endpoint.methods.add(method.upper())

                for kw in calls.keywords:
                    if kw.arg not in {"data", "params", "json"}:
                        continue

                    keys: List[str] = []
                    if isinstance(kw.value, ast.Dict):
                        keys = dict_literal_keys(kw.value)
                    elif isinstance(kw.value, ast.Name):
                        var_name = kw.value.id
                        candidates = assignments.get(var_name, [])
                        valid = [entry for entry in candidates if entry[0] < calls.lineno]
                        if valid:
                            keys = valid[-1][1]

                    for key in sorted(set(keys)):
                        add_field(
                            endpoint,
                            "source_only_fields",
                            f"request.{kw.arg}.{key}",
                            key,
                            "confirmed",
                            ["source_code"],
                            [f"{rel}:{calls.lineno}"],
                        )

    return sorted(set(source_refs))


def parse_flows_file(
    endpoints: Dict[Tuple[str, str], EndpointRecord],
    endpoint_tiers: Dict[str, str],
) -> List[str]:
    if not FLOW_PATH.exists():
        return []

    flow_refs = [str(FLOW_PATH)]
    text = FLOW_PATH.read_text(encoding="utf-8", errors="ignore")

    def add_endpoint_from_url(url: str, confidence: str = "inferred") -> None:
        if not url:
            return

        if url.startswith("http://") or url.startswith("https://"):
            parsed = urlparse(url)
            host = parsed.hostname or "app.srail.or.kr"
            path = parsed.path or "/"
            query = parsed.query
        else:
            cleaned = url.rstrip("\\")
            if "?" in cleaned:
                path, query = cleaned.split("?", 1)
            else:
                path, query = cleaned, ""
            host = infer_host(path)

        if not (path.endswith(".do") or path.endswith(".js")):
            return

        endpoint = ensure_endpoint(endpoints, host, path, endpoint_tiers)
        for qk, _ in parse_qsl(query, keep_blank_values=True):
            add_field(
                endpoint,
                "request_fields",
                f"request.url_query.{qk}",
                qk,
                confidence,
                ["flow"],
                [str(FLOW_PATH)],
            )

    for absolute in re.finditer(r"https?://[A-Za-z0-9._:-]+/[A-Za-z0-9._~/%?=&;:+,-]*", text):
        add_endpoint_from_url(absolute.group(0))

    for relative in re.finditer(r"/[A-Za-z0-9/_-]+\.(?:do|js)(?:\?[^\"'\s<,]*)?", text):
        add_endpoint_from_url(relative.group(0))

    url_pattern = re.compile(r"url\s*:\s*[\"'](?P<url>/[^\"']+\.(?:do|js)(?:\?[^\"']*)?)[\"']", re.IGNORECASE)
    for match in url_pattern.finditer(text):
        url = match.group("url")
        add_endpoint_from_url(url, confidence="confirmed")

        endpoint_path = url.split("?", 1)[0]
        endpoint = ensure_endpoint(endpoints, infer_host(endpoint_path), endpoint_path, endpoint_tiers)
        window = text[match.end() : match.end() + 2500]

        method_match = re.search(r"type\s*:\s*[\"'](GET|POST)[\"']", window, flags=re.IGNORECASE)
        if method_match:
            endpoint.methods.add(method_match.group(1).upper())

        data_match = re.search(r"data\s*:\s*\{(?P<body>.*?)\}", window, flags=re.DOTALL)
        if data_match:
            body = data_match.group("body")
            keys = re.findall(r"([A-Za-z_][A-Za-z0-9_]*)\s*:", body)
            for key in sorted(set(keys)):
                add_field(
                    endpoint,
                    "request_fields",
                    f"request.data.{key}",
                    key,
                    "inferred",
                    ["flow"],
                    [f"{FLOW_PATH}#offset:{match.start()}"],
                )

    return flow_refs


def add_manual_curl_supplements(
    endpoints: Dict[Tuple[str, str], EndpointRecord],
    endpoint_tiers: Dict[str, str],
) -> None:
    supplements = [
        {
            "host": "app.srail.or.kr",
            "path": "/js/stationInfo.js",
            "method": "GET",
            "request_fields": ["_"],
            "response_fields": ["stations[*].stnCd", "stations[*].stnNm"],
            "response_confidence": "inferred",
        },
        {
            "host": "app.srail.or.kr",
            "path": "/ara/selectListAra12009_n.do",
            "method": "POST",
            "request_fields": ["stnCourseNm", "trnSort", "runDt", "trnNo"],
            "response_fields": ["schedule.rows[*].stnNm", "schedule.rows[*].arvTm", "schedule.rows[*].dptTm"],
            "response_confidence": "inferred",
        },
        {
            "host": "app.srail.or.kr",
            "path": "/ara/selectListAra13010_n.do",
            "method": "POST",
            "request_fields": [
                "stnCourseNm", "trnSort", "runDt", "trnNo", "chtnDvCd",
                "dptRsStnCd1", "arvRsStnCd1", "runDt1", "trnNo1",
                "psgTpCd1", "psgInfoPerPrnb1", "psgTpCd2", "psgInfoPerPrnb2",
                "psgTpCd3", "psgInfoPerPrnb3", "psgTpCd4", "psgInfoPerPrnb4",
                "psgTpCd5", "psgInfoPerPrnb5", "psgTpCd6", "psgInfoPerPrnb6",
                "dptRsStnCd2", "arvRsStnCd2", "runDt2", "trnNo2",
            ],
            "response_fields": ["fare.rows[*].psrmClCd", "fare.rows[*].rcvdFare", "fare.rows[*].rcvdAmt"],
            "response_confidence": "inferred",
        },
        {
            "host": "app.srail.or.kr",
            "path": "/atd/selectListAtd02039_n.do",
            "method": "POST",
            "request_fields": ["qryNumNext", "dptDtFrom", "dptDtTo"],
            "response_fields": ["history.rows[*].pnrNo", "history.rows[*].dptDt", "history.rows[*].trnNo"],
            "response_confidence": "inferred",
        },
        {
            "host": "app.srail.or.kr",
            "path": "/atc/selectListAtc14017_n.do",
            "method": "GET",
            "request_query_fields": ["pageNo"],
            "response_fields": ["ticket_list.rows[*].pnrNo"],
            "response_confidence": "inferred",
        },
        {
            "host": "app.srail.or.kr",
            "path": "/ata/selectListAta04A01_n.do",
            "method": "POST",
            "request_fields": [
                "pnr_no", "mb_crd_no", "dpt_dt", "run_dt", "trn_no", "dpt_stn_cd", "arv_stn_cd",
                "dpt_snd_dttm", "arv_snd_dttm", "PushContentDpt", "PushContentArv",
            ],
        },
        {
            "host": "app.srail.or.kr",
            "path": "/atc/selectListAtc02A01_n.do",
            "method": "POST",
            "request_fields": ["pnrNo", "cnc_dmn_cont"],
        },
        {
            "host": "app.srail.or.kr",
            "path": "/push/surveyPush.do",
            "method": "POST",
            "request_fields": ["SIUFlg"],
        },
    ]

    for item in supplements:
        endpoint = ensure_endpoint(endpoints, item["host"], item["path"], endpoint_tiers)
        endpoint.methods.add(item["method"])

        for key in item.get("request_fields", []):
            add_field(
                endpoint,
                "request_fields",
                f"request.data.{key}",
                key,
                "confirmed",
                ["curl"],
                [f"user-curl-2026-02-28:{item['path']}"],
            )

        for key in item.get("request_query_fields", []):
            add_field(
                endpoint,
                "request_fields",
                f"request.url_query.{key}",
                key,
                "confirmed",
                ["curl"],
                [f"user-curl-2026-02-28:{item['path']}"],
            )

        for key in item.get("response_fields", []):
            add_field(
                endpoint,
                "response_fields",
                f"response.{key}",
                key.split(".")[-1],
                item.get("response_confidence", "inferred"),
                ["curl"],
                [f"user-curl-2026-02-28:{item['path']}"],
            )


def load_auth_scope_probe() -> Tuple[List[str], Dict[Tuple[str, str], List[Dict[str, Any]]], Dict[Tuple[str, str], Dict[str, Any]]]:
    if not AUTH_SCOPE_PROBE_PATH.exists():
        return [], {}, {}

    try:
        payload = json.loads(AUTH_SCOPE_PROBE_PATH.read_text(encoding="utf-8"))
    except Exception:
        return [], {}, {}

    records_idx: Dict[Tuple[str, str], List[Dict[str, Any]]] = {}
    for record in payload.get("records", []):
        host = str(record.get("host") or "")
        path = normalize_path(str(record.get("path") or ""))
        if not host or not path:
            continue
        records_idx.setdefault((host, path), []).append(record)

    summary_idx: Dict[Tuple[str, str], Dict[str, Any]] = {}
    for summary in payload.get("endpoint_summary", []):
        host = str(summary.get("host") or "")
        path = normalize_path(str(summary.get("path") or ""))
        if not host or not path:
            continue
        summary_idx[(host, path)] = summary

    return [repo_rel(AUTH_SCOPE_PROBE_PATH)], records_idx, summary_idx


def _dedupe_auth_evidence(items: List[Dict[str, str]]) -> List[Dict[str, str]]:
    normalized = []
    seen = set()
    for item in items:
        source = item.get("source", "")
        signal = item.get("signal", "")
        reference = item.get("reference", "")
        if not source or not signal or not reference:
            continue
        key = (source, signal, reference)
        if key in seen:
            continue
        seen.add(key)
        normalized.append({"source": source, "signal": signal, "reference": reference})

    normalized.sort(key=lambda x: (x["source"], x["signal"], x["reference"]))
    return normalized


def apply_auth_scope_labels(
    endpoints: Dict[Tuple[str, str], EndpointRecord],
    probe_records_idx: Dict[Tuple[str, str], List[Dict[str, Any]]],
    probe_summary_idx: Dict[Tuple[str, str], Dict[str, Any]],
) -> None:
    for key, endpoint in endpoints.items():
        host, path = key
        label = "unknown"
        evidence: List[Dict[str, str]] = []

        summary = probe_summary_idx.get(key)
        if summary:
            summary_label = summary.get("auth_scope_label")
            if summary_label in AUTH_SCOPE_LABELS and summary_label != "unknown":
                label = summary_label
                for item in summary.get("auth_scope_evidence", []):
                    evidence.append(
                        {
                            "source": str(item.get("source") or "probe_response"),
                            "signal": str(item.get("signal") or "probe_summary"),
                            "reference": str(item.get("reference") or f"probe_summary:{host}{path}"),
                        }
                    )

        records = probe_records_idx.get(key, [])
        login_record = next((r for r in records if bool(r.get("has_login_marker"))), None)
        meaningful_record = next(
            (r for r in records if bool(r.get("meaningful_payload")) and not bool(r.get("has_login_marker"))),
            None,
        )
        context_record = next(
            (
                r
                for r in records
                if "context_token_missing_or_invalid" in (r.get("body_signals") or [])
                or "netfunnel_key_required" in (r.get("body_signals") or [])
            ),
            None,
        )

        if login_record:
            label = "auth_required"
            evidence.append(
                {
                    "source": "probe_response",
                    "signal": "login_marker_in_body",
                    "reference": f"probe_id:{login_record.get('probe_id', 'unknown')}",
                }
            )
        elif meaningful_record and label == "unknown":
            if key in PUBLIC_CONTEXT_REQUIRED_SEEDS or context_record:
                label = "public_context_required"
                evidence.append(
                    {
                        "source": "probe_response",
                        "signal": "netfunnel_key_required" if context_record else "context_gate_required",
                        "reference": f"probe_id:{(context_record or meaningful_record).get('probe_id', 'unknown')}",
                    }
                )
            else:
                label = "public_stateless"
                evidence.append(
                    {
                        "source": "probe_response",
                        "signal": "business_payload_without_cookie",
                        "reference": f"probe_id:{meaningful_record.get('probe_id', 'unknown')}",
                    }
                )

        if key in FORCE_UNKNOWN:
            label = "unknown"
            evidence = []

        if label == "unknown" and key in PUBLIC_CONTEXT_REQUIRED_SEEDS:
            label = "public_context_required"
            evidence.append(
                {
                    "source": "source_code",
                    "signal": "netfunnel_key_required",
                    "reference": "third_party/srtgo/srtgo/srt.py:NetFunnelHelper",
                }
            )

        if label == "unknown" and key in PUBLIC_STATELESS_SEEDS:
            label = "public_stateless"
            evidence.append(
                {
                    "source": "probe_response",
                    "signal": "business_payload_without_cookie",
                    "reference": f"seed:{host}{path}",
                }
            )

        if label == "unknown" and key in AUTH_REQUIRED_SEEDS:
            label = "auth_required"
            evidence.append(
                {
                    "source": "source_code",
                    "signal": "session_bound_endpoint",
                    "reference": "third_party/srtgo/srtgo/srt.py:is_login_guard",
                }
            )

        if label != "unknown" and not evidence:
            evidence.append(
                {
                    "source": "source_code",
                    "signal": "auth_scope_seed",
                    "reference": f"seed:{host}{path}",
                }
            )

        endpoint.auth_scope_label = label
        endpoint.auth_scope_evidence = _dedupe_auth_evidence(evidence)


def sync_field_tiers(endpoints: Dict[Tuple[str, str], EndpointRecord]) -> None:
    for endpoint in endpoints.values():
        for section in ("request_fields", "response_fields", "source_only_fields"):
            for rec in getattr(endpoint, section).values():
                rec.endpoint_tier_label = endpoint.endpoint_tier_label


def build_output_json(
    endpoints: Dict[Tuple[str, str], EndpointRecord],
    capture_sources: List[str],
    captured_timestamps: List[str],
    flow_sources: List[str],
    source_code_sources: List[str],
    auth_probe_sources: List[str],
) -> Dict[str, Any]:
    generated_at = captured_timestamps[-1] if captured_timestamps else "2026-02-28T00:00:00+00:00"

    def serialize_fields(field_map: Dict[str, FieldRecord]) -> List[Dict[str, Any]]:
        result: List[Dict[str, Any]] = []
        for key in sorted(field_map):
            rec = field_map[key]
            result.append(
                {
                    "path": rec.path,
                    "name": rec.name,
                    "endpoint_tier_label": rec.endpoint_tier_label,
                    "field_confidence_label": rec.field_confidence_label,
                    "observed_in": sorted(rec.observed_in),
                    "source_of_truth": sorted(rec.source_of_truth),
                }
            )
        return result

    endpoint_rows: List[Dict[str, Any]] = []
    sorted_endpoints = sorted(
        endpoints.values(),
        key=lambda ep: (-(TIER_PRIORITY.get(ep.endpoint_tier_label, 0)), ep.host, ep.path),
    )

    for ep in sorted_endpoints:
        endpoint_rows.append(
            {
                "host": ep.host,
                "path": ep.path,
                "methods": sorted(ep.methods),
                "endpoint_tier_label": ep.endpoint_tier_label,
                "auth_scope_label": ep.auth_scope_label,
                "auth_scope_evidence": ep.auth_scope_evidence,
                "request_fields": serialize_fields(ep.request_fields),
                "response_fields": serialize_fields(ep.response_fields),
                "source_only_fields": serialize_fields(ep.source_only_fields),
            }
        )

    return {
        "generated_at": generated_at,
        "sources": {
            "captures": sorted(set(capture_sources)),
            "curls": sorted(set(USER_CURL_SOURCES)),
            "flows": sorted(set(flow_sources)),
            "source_code": sorted(set(source_code_sources)),
            "auth_scope_probe": sorted(set(auth_probe_sources)),
        },
        "endpoints": endpoint_rows,
    }


def render_markdown(field_map: Dict[str, Any]) -> str:
    lines: List[str] = []
    lines.append("# Provider Field Map")
    lines.append("")
    lines.append("Generated from captures, source code, flow artifact, user-supplied curl contracts, and auth-scope probes.")
    lines.append("")
    lines.append("## Label Taxonomy")
    lines.append("")
    lines.append("- Endpoint tier label: `critical`, `high_signal`, `mapped`, `unknown`")
    lines.append("- Field confidence label: `confirmed`, `inferred`, `unresolved`")
    lines.append("- Auth scope label: `public_stateless`, `public_context_required`, `auth_required`, `unknown`")
    lines.append("")
    lines.append(f"Generated at: `{field_map['generated_at']}`")
    lines.append("")

    lines.append("## Sources")
    lines.append("")
    for source_type in ("captures", "curls", "flows", "source_code", "auth_scope_probe"):
        entries = field_map["sources"].get(source_type, [])
        lines.append(f"### {source_type}")
        if entries:
            for entry in entries:
                lines.append(f"- `{entry}`")
        else:
            lines.append("- _none_")
        lines.append("")

    lines.append("## Endpoint Matrix")
    lines.append("")
    for endpoint in field_map["endpoints"]:
        title = f"{endpoint['host']} {endpoint['path']}"
        lines.append(f"### {title}")
        lines.append("")
        methods = ", ".join(endpoint["methods"]) if endpoint["methods"] else "_unknown_"
        lines.append(f"- `endpoint_tier_label`: `{endpoint['endpoint_tier_label']}`")
        lines.append(f"- `auth_scope_label`: `{endpoint['auth_scope_label']}`")
        lines.append(f"- `methods`: `{methods}`")
        lines.append(f"- `request_fields`: `{len(endpoint['request_fields'])}`")
        lines.append(f"- `response_fields`: `{len(endpoint['response_fields'])}`")
        lines.append(f"- `source_only_fields`: `{len(endpoint['source_only_fields'])}`")
        lines.append("")

        lines.append("#### Auth Scope Evidence")
        auth_rows = endpoint.get("auth_scope_evidence", [])
        if auth_rows:
            lines.append("| source | signal | reference |")
            lines.append("|---|---|---|")
            for row in auth_rows:
                lines.append(f"| `{row['source']}` | `{row['signal']}` | `{row['reference']}` |")
        else:
            lines.append("- _none_")
        lines.append("")

        for section_key, section_title in (
            ("request_fields", "Request Fields"),
            ("response_fields", "Response Fields"),
            ("source_only_fields", "Source-Only Fields"),
        ):
            lines.append(f"#### {section_title}")
            rows = endpoint[section_key]
            if not rows:
                lines.append("- _none_")
                lines.append("")
                continue
            lines.append("| path | name | endpoint_tier_label | field_confidence_label | evidence |")
            lines.append("|---|---|---|---|---|")
            for row in rows:
                evidence = "; ".join(row["source_of_truth"])
                lines.append(
                    "| "
                    + f"`{row['path']}` | `{row['name']}` | `{row['endpoint_tier_label']}` | "
                    + f"`{row['field_confidence_label']}` | `{evidence}` |"
                )
            lines.append("")

    return "\n".join(lines) + "\n"


def main() -> None:
    contract_text = CONTRACT_PATH.read_text(encoding="utf-8")
    endpoint_tiers = parse_contract_tiers(contract_text)

    endpoints: Dict[Tuple[str, str], EndpointRecord] = {}

    for endpoint_path, tier in sorted(endpoint_tiers.items()):
        host = infer_host(endpoint_path)
        ensure_endpoint(endpoints, host, endpoint_path, endpoint_tiers, default_tier=tier)

    capture_sources, capture_timestamps = parse_capture_files(endpoints, endpoint_tiers)
    flow_sources = parse_flows_file(endpoints, endpoint_tiers)
    add_manual_curl_supplements(endpoints, endpoint_tiers)
    source_code_sources = parse_source_code_fields(endpoints, endpoint_tiers)
    auth_probe_sources, probe_records_idx, probe_summary_idx = load_auth_scope_probe()

    sync_field_tiers(endpoints)
    apply_auth_scope_labels(endpoints, probe_records_idx, probe_summary_idx)

    output_json = build_output_json(
        endpoints=endpoints,
        capture_sources=capture_sources,
        captured_timestamps=capture_timestamps,
        flow_sources=flow_sources,
        source_code_sources=source_code_sources,
        auth_probe_sources=auth_probe_sources,
    )

    FIELD_MAP_JSON_PATH.write_text(
        json.dumps(output_json, ensure_ascii=False, indent=2, sort_keys=False) + "\n",
        encoding="utf-8",
    )
    FIELD_MAP_MD_PATH.write_text(render_markdown(output_json), encoding="utf-8")


if __name__ == "__main__":
    main()
